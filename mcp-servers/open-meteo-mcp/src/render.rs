//! Turning a forecast into one short paragraph a speech model can say.

use crate::open_meteo::Forecast;

pub const ATTRIBUTION: &str = "Data by Open-Meteo.com, CC-BY 4.0.";

/// WMO 4677 present-weather codes, in the words a person would use.
/// The catch-all is a real answer rather than a number, because the
/// consumer has to say it out loud.
pub fn describe_code(code: i64) -> &'static str {
    match code {
        0 => "clear",
        1 => "mostly clear",
        2 => "partly cloudy",
        3 => "overcast",
        45 | 48 => "fog",
        51 | 53 | 55 => "drizzle",
        56 | 57 => "freezing drizzle",
        61 | 63 | 65 => "rain",
        66 | 67 => "freezing rain",
        71 | 73 | 75 => "snow",
        77 => "snow grains",
        80..=82 => "showers",
        85 | 86 => "snow showers",
        95 => "thunderstorms",
        96 | 99 => "thunderstorms with hail",
        _ => "unsettled",
    }
}

pub fn render(place: &str, forecast: &Forecast, clamp_note: Option<&str>) -> String {
    let mut out = format!(
        "{place} — currently {:.0}°C, {}, wind {:.0} km/h.",
        forecast.current.temperature_2m,
        describe_code(forecast.current.weather_code),
        forecast.current.wind_speed_10m,
    );

    // Task 3 appends the daily lines here.

    if let Some(note) = clamp_note {
        out.push(' ');
        out.push_str(note);
    }
    out.push(' ');
    out.push_str(ATTRIBUTION);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::open_meteo::{Current, Daily, Forecast};

    fn forecast() -> Forecast {
        Forecast {
            current: Current {
                temperature_2m: 14.3,
                weather_code: 3,
                wind_speed_10m: 19.4,
            },
            daily: Daily {
                time: vec![
                    "2026-09-10".into(),
                    "2026-09-11".into(),
                    "2026-09-12".into(),
                ],
                weather_code: vec![61, 80, 0],
                temperature_2m_max: vec![17.0, 16.0, 19.0],
                temperature_2m_min: vec![11.0, 9.0, 10.0],
                precipitation_sum: vec![4.0, 0.0, 0.0],
            },
        }
    }

    #[test]
    fn current_conditions_lead_and_read_as_spoken_words_not_codes() {
        let out = render("Melbourne, Victoria, Australia", &forecast(), None);
        assert!(
            out.starts_with(
                "Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h."
            ),
            "{out}"
        );
    }

    #[test]
    fn every_successful_result_carries_the_attribution_line() {
        assert!(render("X", &forecast(), None).ends_with(ATTRIBUTION));
    }

    #[test]
    fn a_clamp_note_is_spoken_before_the_attribution() {
        let out = render("X", &forecast(), Some("Asked for 99 days; 7 are shown."));
        assert!(
            out.contains("Asked for 99 days; 7 are shown. Data by Open-Meteo.com"),
            "{out}"
        );
    }

    #[test]
    fn an_unknown_wmo_code_still_produces_a_word() {
        assert_eq!(describe_code(4242), "unsettled");
    }
}
