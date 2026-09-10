//! The HTTP calls to Open-Meteo, and the shapes they answer with.
//!
//! Every field name here was checked against a live response on 2026-09-10.

use crate::config::{Config, Units};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Current {
    pub temperature_2m: f64,
    pub weather_code: i64,
    pub wind_speed_10m: f64,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Daily {
    pub time: Vec<String>,
    pub weather_code: Vec<i64>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Forecast {
    pub current: Current,
    pub daily: Daily,
}

/// One geocoding hit. `country`, `admin1` and `timezone` are optional
/// because the API genuinely omits them for some places — a city-state has
/// no admin region.
#[derive(Debug, Deserialize, Clone)]
pub struct Place {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub country: Option<String>,
    pub admin1: Option<String>,
    pub timezone: Option<String>,
}

impl Place {
    /// "Melbourne, Victoria, Australia" — enough for a listener to hear
    /// that it picked the wrong Melbourne.
    pub fn label(&self) -> String {
        [
            Some(self.name.clone()),
            self.admin1.clone(),
            self.country.clone(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(", ")
    }
}

/// `results` is `Option` rather than a defaulted `Vec` because the field is
/// genuinely absent on a no-match, and the type should say what the wire
/// actually does.
#[derive(Debug, Deserialize)]
struct GeocodeResponse {
    #[serde(default)]
    results: Option<Vec<Place>>,
}

#[derive(Debug)]
pub enum WeatherError {
    NoMatch(String),
    Upstream(String),
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::NoMatch(q) => write!(
                f,
                "I could not find a place called \"{q}\". Ask the user to say it another way, \
or to give a nearby larger town."
            ),
            WeatherError::Upstream(why) => write!(f, "The weather service did not answer: {why}."),
        }
    }
}

pub struct Client {
    http: reqwest::Client,
    config: Config,
}

/// A request that never returns would hang the tool for as long as the
/// caller waits. Ten seconds is well past a healthy response and well
/// inside a speech turn's patience.
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

impl Client {
    pub fn new(config: Config) -> Self {
        Client {
            http: reqwest::Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .expect("a client with only a timeout set always builds"),
            config,
        }
    }

    /// Sent as a query parameter on both endpoints, and only when set.
    /// The paid tier is API-identical to the free one, so this plus the two
    /// URL variables is the whole of what switching to it requires.
    fn key_param(&self) -> Vec<(&str, String)> {
        match &self.config.api_key {
            Some(key) => vec![("apikey", key.clone())],
            None => vec![],
        }
    }

    pub async fn forecast(
        &self,
        latitude: f64,
        longitude: f64,
        days: u8,
        timezone: Option<&str>,
    ) -> Result<Forecast, WeatherError> {
        // `hourly` is deliberately absent: 24 rows a speech model would read
        // aloud, for an answer two sentences already give.
        let mut query = vec![
            ("latitude", latitude.to_string()),
            ("longitude", longitude.to_string()),
            (
                "current",
                "temperature_2m,weather_code,wind_speed_10m".to_string(),
            ),
            (
                "daily",
                "weather_code,temperature_2m_max,temperature_2m_min,precipitation_sum".to_string(),
            ),
            ("forecast_days", days.to_string()),
            ("timezone", timezone.unwrap_or("auto").to_string()),
        ];
        if self.config.units == Units::Imperial {
            query.push(("temperature_unit", "fahrenheit".to_string()));
            query.push(("wind_speed_unit", "mph".to_string()));
            query.push(("precipitation_unit", "inch".to_string()));
        }
        query.extend(self.key_param());

        let response = self
            .http
            .get(&self.config.forecast_url)
            .query(&query)
            .send()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))?;
        // Checked before the body is touched: a 400 carries
        // `{"error":true,"reason":"..."}`, which would fail to parse as a
        // Forecast and report the wrong cause.
        if !response.status().is_success() {
            return Err(WeatherError::Upstream(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }
        response
            .json()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))
    }

    pub async fn geocode(&self, name: &str) -> Result<Place, WeatherError> {
        // count=1: the spec resolves a multi-match to the first hit and
        // names it, rather than asking the model to choose.
        let mut query = vec![
            ("name", name.to_string()),
            ("count", "1".to_string()),
            ("language", "en".to_string()),
            ("format", "json".to_string()),
        ];
        query.extend(self.key_param());

        let response = self
            .http
            .get(&self.config.geocoding_url)
            .query(&query)
            .send()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))?;
        if !response.status().is_success() {
            return Err(WeatherError::Upstream(format!(
                "HTTP {}",
                response.status().as_u16()
            )));
        }
        let body: GeocodeResponse = response
            .json()
            .await
            .map_err(|e| WeatherError::Upstream(e.to_string()))?;
        body.results
            .and_then(|r| r.into_iter().next())
            .ok_or_else(|| WeatherError::NoMatch(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::Duration;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Points the client at the mock server. No test in this crate is ever
    /// allowed to reach the real api.open-meteo.com.
    fn config(server: &MockServer, imperial: bool, key: Option<&str>) -> Config {
        let base = server.uri();
        Config::from_vars(move |k| match k {
            "OPEN_METEO_BASE_URL" => Some(format!("{base}/v1/forecast")),
            "OPEN_METEO_GEOCODING_URL" => Some(format!("{base}/v1/search")),
            "OPEN_METEO_UNITS" if imperial => Some("imperial".to_string()),
            "OPEN_METEO_API_KEY" => key.map(str::to_string),
            _ => None,
        })
    }

    fn melbourne_forecast() -> serde_json::Value {
        json!({
            "current": {"temperature_2m": 14.3, "weather_code": 3, "wind_speed_10m": 19.4},
            "daily": {
                "time": ["2026-09-10", "2026-09-11", "2026-09-12"],
                "weather_code": [61, 80, 0],
                "temperature_2m_max": [17.0, 16.0, 19.0],
                "temperature_2m_min": [11.0, 9.0, 10.0],
                "precipitation_sum": [4.0, 0.0, 0.0]
            }
        })
    }

    #[tokio::test]
    async fn imperial_units_reach_the_forecast_request() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .and(query_param("temperature_unit", "fahrenheit"))
            .and(query_param("wind_speed_unit", "mph"))
            .and(query_param("precipitation_unit", "inch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_forecast()))
            .mount(&server)
            .await;

        Client::new(config(&server, true, None))
            .forecast(28.08, -80.61, 1, None)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn the_api_key_is_sent_when_set_and_absent_when_not() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_place()))
            .mount(&server)
            .await;

        Client::new(config(&server, false, Some("k-123")))
            .geocode("Melbourne")
            .await
            .unwrap();
        Client::new(config(&server, false, None))
            .geocode("Melbourne")
            .await
            .unwrap();

        let requests = server.received_requests().await.unwrap();
        assert!(requests[0].url.as_str().contains("apikey=k-123"));
        assert!(!requests[1].url.as_str().contains("apikey"));
    }

    #[tokio::test]
    async fn a_forecast_is_parsed_and_no_hourly_block_is_ever_requested() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_forecast()))
            .mount(&server)
            .await;

        let forecast = Client::new(config(&server, false, None))
            .forecast(-37.814, 144.9633, 3, Some("Australia/Melbourne"))
            .await
            .unwrap();
        assert_eq!(forecast.current.temperature_2m, 14.3);
        assert_eq!(forecast.daily.time.len(), 3);

        let url = server.received_requests().await.unwrap()[0].url.to_string();
        assert!(!url.contains("hourly"), "{url}");
        assert!(url.contains("forecast_days=3"), "{url}");
        assert!(url.contains("Australia%2FMelbourne"), "{url}");
    }

    #[tokio::test]
    async fn a_non_200_is_an_upstream_error_not_a_panic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let err = Client::new(config(&server, false, None))
            .forecast(-37.8, 144.9, 3, None)
            .await
            .unwrap_err();
        assert!(matches!(err, WeatherError::Upstream(_)));
        assert!(err.to_string().contains("503"), "{err}");
    }

    /// The spec asks for a timeout to be an error result like any other.
    /// The client under test is built by hand with a short timeout: waiting
    /// out the real ten-second one would make the suite unusable.
    #[tokio::test]
    async fn a_request_that_never_answers_times_out_as_an_upstream_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(
                ResponseTemplate::new(200).set_delay(REQUEST_TIMEOUT + Duration::from_secs(1)),
            )
            .mount(&server)
            .await;

        let client = Client {
            http: reqwest::Client::builder()
                .timeout(Duration::from_millis(150))
                .build()
                .unwrap(),
            config: config(&server, false, None),
        };
        let err = client.forecast(-37.8, 144.9, 3, None).await.unwrap_err();
        assert!(matches!(err, WeatherError::Upstream(_)));
    }

    #[tokio::test]
    async fn malformed_json_is_an_upstream_error_not_a_panic() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_string("{not json"))
            .mount(&server)
            .await;

        let err = Client::new(config(&server, false, None))
            .forecast(-37.8, 144.9, 3, None)
            .await
            .unwrap_err();
        assert!(matches!(err, WeatherError::Upstream(_)));
    }

    fn melbourne_place() -> serde_json::Value {
        json!({"results": [{
            "name": "Melbourne", "latitude": -37.814, "longitude": 144.96332,
            "country": "Australia", "admin1": "Victoria", "timezone": "Australia/Melbourne"
        }]})
    }

    #[tokio::test]
    async fn a_matched_name_yields_a_place_labelled_with_region_and_country() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .and(query_param("name", "Melbourne"))
            .respond_with(ResponseTemplate::new(200).set_body_json(melbourne_place()))
            .mount(&server)
            .await;

        let place = Client::new(config(&server, false, None))
            .geocode("Melbourne")
            .await
            .unwrap();
        assert_eq!(place.label(), "Melbourne, Victoria, Australia");
        assert_eq!(place.timezone.as_deref(), Some("Australia/Melbourne"));
        assert_eq!(place.latitude, -37.814);
    }

    #[tokio::test]
    async fn a_name_that_matches_nothing_is_an_error_quoting_the_input() {
        let server = MockServer::start().await;
        // The live API omits `results` entirely; it does not return [].
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"generationtime_ms": 0.5})),
            )
            .mount(&server)
            .await;

        let err = Client::new(config(&server, false, None))
            .geocode("zzzqqq")
            .await
            .unwrap_err();
        assert!(matches!(err, WeatherError::NoMatch(ref q) if q == "zzzqqq"));
        assert!(err.to_string().contains("\"zzzqqq\""), "{err}");
    }

    #[tokio::test]
    async fn a_place_missing_its_admin_region_is_still_labelled() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"results": [{
                    "name": "Singapore", "latitude": 1.28967, "longitude": 103.85007,
                    "country": "Singapore", "timezone": "Asia/Singapore"
                }]})),
            )
            .mount(&server)
            .await;

        let place = Client::new(config(&server, false, None))
            .geocode("Singapore")
            .await
            .unwrap();
        assert_eq!(place.label(), "Singapore, Singapore");
    }
}
