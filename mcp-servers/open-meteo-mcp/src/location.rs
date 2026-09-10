//! Argument resolution: which place a `get_weather` call is actually about.
//! Pure — no I/O, no clock, no network — so every rule here is cheap to test.

use crate::config::Config;

pub const MIN_DAYS: u8 = 1;
pub const MAX_DAYS: u8 = 7;
pub const DEFAULT_DAYS: u8 = 3;

#[derive(Debug, Clone, PartialEq)]
pub enum Resolution {
    Coords { latitude: f64, longitude: f64 },
    Name(String),
    Ask,
    MissingHalf(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Clamped {
    pub days: u8,
    /// Set only when the request was actually changed. The result says so
    /// out loud: a model that asked for 10 days should hear why it got 7,
    /// rather than silently believing it got what it asked for.
    pub note: Option<String>,
}

pub fn clamp_days(days: Option<i64>) -> Clamped {
    let Some(asked) = days else {
        return Clamped {
            days: DEFAULT_DAYS,
            note: None,
        };
    };
    let clamped = asked.clamp(MIN_DAYS as i64, MAX_DAYS as i64) as u8;
    let note = (i64::from(clamped) != asked)
        .then(|| format!("Asked for {asked} days; this server returns at most {MAX_DAYS} and at least {MIN_DAYS}, so {clamped} are shown."));
    Clamped {
        days: clamped,
        note,
    }
}

/// The spec's resolution order, in one place: coordinates, then a name,
/// then the configured default, then ask.
pub fn resolve(
    location: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    config: &Config,
) -> Resolution {
    match (latitude, longitude) {
        (Some(latitude), Some(longitude)) => Resolution::Coords {
            latitude,
            longitude,
        },
        (Some(_), None) => Resolution::MissingHalf("longitude"),
        (None, Some(_)) => Resolution::MissingHalf("latitude"),
        (None, None) => match location.map(str::trim).filter(|s| !s.is_empty()) {
            Some(name) => Resolution::Name(name.to_string()),
            None => match config.default_location.as_deref() {
                Some(default) => parse_default(default),
                None => Resolution::Ask,
            },
        },
    }
}

/// The spec lets the default be "a place name or `lat,lon`". Both halves
/// must parse as numbers, so "Melbourne, Australia" stays a name.
fn parse_default(value: &str) -> Resolution {
    if let Some((lat, lon)) = value.split_once(',')
        && let (Ok(latitude), Ok(longitude)) =
            (lat.trim().parse::<f64>(), lon.trim().parse::<f64>())
    {
        return Resolution::Coords {
            latitude,
            longitude,
        };
    }
    Resolution::Name(value.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(default_location: Option<&str>) -> Config {
        Config::from_vars(|k| match k {
            "OPEN_METEO_DEFAULT_LOCATION" => default_location.map(str::to_string),
            _ => None,
        })
    }

    #[test]
    fn coordinates_win_and_a_location_given_alongside_them_is_ignored() {
        let r = resolve(
            Some("Melbourne"),
            Some(-37.814),
            Some(144.9633),
            &config(None),
        );
        assert_eq!(
            r,
            Resolution::Coords {
                latitude: -37.814,
                longitude: 144.9633
            }
        );
    }

    #[test]
    fn a_name_alone_is_geocoded() {
        assert_eq!(
            resolve(Some("Melbourne"), None, None, &config(None)),
            Resolution::Name("Melbourne".into())
        );
    }

    #[test]
    fn nothing_with_a_default_configured_uses_the_default() {
        assert_eq!(
            resolve(None, None, None, &config(Some("Hobart"))),
            Resolution::Name("Hobart".into())
        );
    }

    #[test]
    fn nothing_with_no_default_asks_rather_than_guessing() {
        assert_eq!(resolve(None, None, None, &config(None)), Resolution::Ask);
    }

    #[test]
    fn half_a_coordinate_pair_names_the_half_that_is_missing() {
        assert_eq!(
            resolve(None, Some(-37.8), None, &config(None)),
            Resolution::MissingHalf("longitude")
        );
        assert_eq!(
            resolve(None, None, Some(144.9), &config(None)),
            Resolution::MissingHalf("latitude")
        );
    }

    #[test]
    fn a_default_written_as_lat_lon_is_used_as_coordinates() {
        assert_eq!(
            resolve(None, None, None, &config(Some("-37.814, 144.9633"))),
            Resolution::Coords {
                latitude: -37.814,
                longitude: 144.9633
            }
        );
    }

    #[test]
    fn a_default_that_merely_contains_a_comma_is_still_a_place_name() {
        assert_eq!(
            resolve(None, None, None, &config(Some("Melbourne, Australia"))),
            Resolution::Name("Melbourne, Australia".into())
        );
    }

    #[test]
    fn days_clamp_at_both_ends_and_say_so() {
        assert_eq!(
            clamp_days(None),
            Clamped {
                days: 3,
                note: None
            }
        );
        let low = clamp_days(Some(0));
        assert_eq!(low.days, 1);
        assert!(low.note.unwrap().contains("at least 1"));
        let high = clamp_days(Some(99));
        assert_eq!(high.days, 7);
        assert!(high.note.unwrap().contains("at most 7"));
        assert_eq!(
            clamp_days(Some(5)),
            Clamped {
                days: 5,
                note: None
            }
        );
    }
}
