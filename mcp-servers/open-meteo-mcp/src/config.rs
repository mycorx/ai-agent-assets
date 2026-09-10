//! Environment configuration. Parsed through an injected lookup so tests
//! never touch process env: `std::env::set_var` is `unsafe` in edition 2024
//! and Rust runs tests in parallel threads within one process.

pub const DEFAULT_FORECAST_URL: &str = "https://api.open-meteo.com/v1/forecast";
pub const DEFAULT_GEOCODING_URL: &str = "https://geocoding-api.open-meteo.com/v1/search";

// Temporary: fields are only taken by reference in the Task-1 stub
// `get_weather`, never field-accessed, until Task 2 reads them.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    pub default_location: Option<String>,
    pub forecast_url: String,
    pub geocoding_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self::from_vars(|k| std::env::var(k).ok())
    }

    /// The lookup is a parameter, not `std::env`, so tests can vary it
    /// without mutating shared process state.
    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> Self {
        Config {
            default_location: get("OPEN_METEO_DEFAULT_LOCATION").filter(|v| !v.trim().is_empty()),
            forecast_url: get("OPEN_METEO_BASE_URL")
                .unwrap_or_else(|| DEFAULT_FORECAST_URL.to_string()),
            geocoding_url: get("OPEN_METEO_GEOCODING_URL")
                .unwrap_or_else(|| DEFAULT_GEOCODING_URL.to_string()),
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
}
