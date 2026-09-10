//! Environment configuration. Parsed through an injected lookup so tests
//! never touch process env: `std::env::set_var` is `unsafe` in edition 2024
//! and Rust runs tests in parallel threads within one process.

pub const DEFAULT_FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
pub const DEFAULT_GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Units {
    Metric,
    Imperial,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub default_location: Option<String>,
    pub forecast_url: String,
    pub geocoding_url: String,
    pub units: Units,
    pub api_key: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        Self::from_vars(|k| std::env::var(k).ok())
    }

    /// The lookup is a parameter, not `std::env`, so tests can vary it
    /// without mutating shared process state.
    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> Self {
        // Exact match only: a wrong value falls back to metric rather than
        // being guessed at, because the wrong units are a silent wrong answer.
        let units = match get("OPEN_METEO_UNITS").as_deref() {
            Some("imperial") => Units::Imperial,
            _ => Units::Metric,
        };
        Config {
            default_location: get("OPEN_METEO_DEFAULT_LOCATION").filter(|v| !v.trim().is_empty()),
            forecast_url: get("OPEN_METEO_BASE_URL")
                .unwrap_or_else(|| DEFAULT_FORECAST_URL.to_string()),
            geocoding_url: get("OPEN_METEO_GEOCODING_URL")
                .unwrap_or_else(|| DEFAULT_GEOCODING_URL.to_string()),
            units,
            api_key: get("OPEN_METEO_API_KEY").filter(|v| !v.trim().is_empty()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn defaults_are_the_public_endpoints_and_no_default_location() {
        let c = Config::from_vars(empty);
        assert_eq!(c.forecast_url, DEFAULT_FORECAST_URL);
        assert_eq!(c.geocoding_url, DEFAULT_GEOCODING_URL);
        assert!(c.default_location.is_none());
    }

    #[test]
    fn the_urls_are_overridable_because_that_is_the_commercial_tier_seam() {
        let c = Config::from_vars(|k| {
            (k == "OPEN_METEO_BASE_URL")
                .then(|| "https://customer-api.example/v1/forecast".to_string())
        });
        assert_eq!(c.forecast_url, "https://customer-api.example/v1/forecast");
        assert_eq!(c.geocoding_url, DEFAULT_GEOCODING_URL);
    }

    #[test]
    fn a_blank_default_location_is_treated_as_unset() {
        let c =
            Config::from_vars(|k| (k == "OPEN_METEO_DEFAULT_LOCATION").then(|| "   ".to_string()));
        assert!(c.default_location.is_none());
    }

    #[test]
    fn units_default_to_metric_and_only_the_exact_word_imperial_switches_them() {
        assert_eq!(Config::from_vars(empty).units, Units::Metric);
        assert_eq!(
            Config::from_vars(|k| (k == "OPEN_METEO_UNITS").then(|| "imperial".to_string())).units,
            Units::Imperial
        );
        // A plausible-but-wrong value must not silently become imperial.
        assert_eq!(
            Config::from_vars(|k| (k == "OPEN_METEO_UNITS").then(|| "celsius".to_string())).units,
            Units::Metric
        );
    }

    #[test]
    fn an_empty_api_key_is_treated_as_unset() {
        let c = Config::from_vars(|k| (k == "OPEN_METEO_API_KEY").then(|| "  ".to_string()));
        assert!(c.api_key.is_none());
    }
}
