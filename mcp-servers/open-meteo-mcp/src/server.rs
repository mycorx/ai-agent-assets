//! The MCP surface: one tool, `get_weather`.

use crate::config::Config;
use crate::location::{Resolution, clamp_days, resolve};
use crate::open_meteo::Client;
use crate::render::render;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData, ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

/// What the tool says when it has nothing to go on. Written at the model,
/// not the user: it is an instruction to ask, not a sentence to read aloud.
pub const ASK_USER: &str = "No default location is configured. Ask the user which city or place \
they want the weather for, then call get_weather again with it.";

/// Doc comments on these fields become the tool's JSON Schema descriptions.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetWeatherArgs {
    /// Place name, e.g. "Melbourne" or "Melbourne, Australia".
    pub location: Option<String>,
    /// Decimal degrees. Use with longitude instead of location.
    pub latitude: Option<f64>,
    /// Decimal degrees. Use with latitude instead of location.
    pub longitude: Option<f64>,
    /// Days of forecast, 1-7. Default 3.
    pub days: Option<i64>,
}

#[derive(Clone)]
pub struct WeatherServer {
    config: Config,
}

impl WeatherServer {
    pub fn new(config: Config) -> Self {
        WeatherServer { config }
    }
}

#[tool_router]
impl WeatherServer {
    #[tool(description = "Current conditions and a short forecast for a place. \
Give a place name, or a latitude/longitude pair.")]
    pub async fn get_weather(
        &self,
        Parameters(args): Parameters<GetWeatherArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let clamped = clamp_days(args.days);
        let client = Client::new(self.config.clone());

        let (place, latitude, longitude, timezone) = match resolve(
            args.location.as_deref(),
            args.latitude,
            args.longitude,
            &self.config,
        ) {
            Resolution::Ask => return Ok(error_text(ASK_USER)),
            Resolution::MissingHalf(missing) => {
                return Ok(error_text(format!(
                    "A coordinate pair needs both halves — {missing} is missing. \
Give both latitude and longitude, or a place name instead."
                )));
            }
            Resolution::Coords {
                latitude,
                longitude,
            } => (
                format!("{latitude:.4}, {longitude:.4}"),
                latitude,
                longitude,
                None,
            ),
            Resolution::Name(name) => match client.geocode(&name).await {
                Ok(found) => (
                    found.label(),
                    found.latitude,
                    found.longitude,
                    found.timezone.clone(),
                ),
                Err(e) => return Ok(error_text(e.to_string())),
            },
        };

        match client
            .forecast(latitude, longitude, clamped.days, timezone.as_deref())
            .await
        {
            Ok(forecast) => Ok(CallToolResult::success(vec![ContentBlock::text(render(
                &place,
                &forecast,
                &self.config.units,
                clamped.note.as_deref(),
            ))])),
            Err(e) => Ok(error_text(e.to_string())),
        }
    }
}

/// Tool-level error, never `Err(ErrorData)`: MCP clients render protocol
/// errors opaquely, so an `Err` here would reach the user as a spoken
/// failure with none of this text in it.
fn error_text(text: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(text)])
}

#[tool_handler(
    name = "open-meteo-mcp",
    instructions = "Answers questions about current weather and short forecasts. \
Weather is live data; do not answer from memory."
)]
impl ServerHandler for WeatherServer {}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ContentBlock;

    fn args() -> GetWeatherArgs {
        GetWeatherArgs {
            location: None,
            latitude: None,
            longitude: None,
            days: None,
        }
    }

    /// rmcp 3.x has no `Annotated`/`.raw` wrapper — `ContentBlock` is the
    /// enum itself.
    fn text_of(result: &CallToolResult) -> String {
        result
            .content
            .iter()
            .filter_map(|b| match b {
                ContentBlock::Text(t) => Some(t.text.clone()),
                _ => None,
            })
            .collect()
    }

    fn server(default_location: Option<&str>) -> WeatherServer {
        WeatherServer::new(Config::from_vars(|k| match k {
            "OPEN_METEO_DEFAULT_LOCATION" => default_location.map(str::to_string),
            _ => None,
        }))
    }

    #[test]
    fn the_router_lists_exactly_one_tool_named_get_weather() {
        let tools = WeatherServer::tool_router().list_all();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "get_weather");
    }

    #[tokio::test]
    async fn no_arguments_and_no_default_is_an_error_result_telling_the_model_to_ask() {
        let result = server(None).get_weather(Parameters(args())).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        assert_eq!(text_of(&result), ASK_USER);
    }

    #[tokio::test]
    async fn half_a_coordinate_pair_is_an_error_result_naming_the_missing_half() {
        let mut a = args();
        a.latitude = Some(-37.8);
        let result = server(None).get_weather(Parameters(a)).await.unwrap();
        assert_eq!(result.is_error, Some(true));
        assert!(
            text_of(&result).contains("longitude is missing"),
            "{}",
            text_of(&result)
        );
    }

    /// The one server test that needs a network seam, so it builds its own
    /// config rather than using `server()`.
    #[tokio::test]
    async fn a_named_place_is_geocoded_and_the_answer_says_which_place_it_picked() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{"name": "Melbourne", "latitude": -37.814, "longitude": 144.96332,
                             "country": "Australia", "admin1": "Victoria",
                             "timezone": "Australia/Melbourne"}]
            })))
            .mount(&mock)
            .await;
        Mock::given(method("GET"))
            .and(path("/v1/forecast"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "current": {"temperature_2m": 14.3, "weather_code": 3, "wind_speed_10m": 19.4},
                "daily": {"time": ["2026-09-10"], "weather_code": [3],
                          "temperature_2m_max": [17.0], "temperature_2m_min": [11.0],
                          "precipitation_sum": [0.0]}
            })))
            .mount(&mock)
            .await;

        let base = mock.uri();
        let server = WeatherServer::new(Config::from_vars(move |k| match k {
            "OPEN_METEO_BASE_URL" => Some(format!("{base}/v1/forecast")),
            "OPEN_METEO_GEOCODING_URL" => Some(format!("{base}/v1/search")),
            _ => None,
        }));

        let mut a = args();
        a.location = Some("Melbourne".into());
        let result = server.get_weather(Parameters(a)).await.unwrap();
        assert_eq!(result.is_error, Some(false));
        assert!(
            text_of(&result).starts_with("Melbourne, Victoria, Australia — "),
            "{}",
            text_of(&result)
        );

        // The geocoded IANA zone must reach the forecast call, or "Today"
        // means today here rather than today there.
        let forecast_url = mock.received_requests().await.unwrap()[1].url.to_string();
        assert!(
            forecast_url.contains("Australia%2FMelbourne"),
            "{forecast_url}"
        );
    }
}
