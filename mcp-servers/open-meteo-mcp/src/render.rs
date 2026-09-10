//! Turning a forecast into one short paragraph a speech model can say.

use crate::config::Units;
use crate::open_meteo::Forecast;
use chrono::{Datelike, NaiveDate};

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

/// `Weekday::to_string()` renders "Sat", which a speech model reads aloud
/// as the abbreviation. Spelled out on purpose.
fn weekday_name(day: chrono::Weekday) -> &'static str {
    use chrono::Weekday::*;
    match day {
        Mon => "Monday",
        Tue => "Tuesday",
        Wed => "Wednesday",
        Thu => "Thursday",
        Fri => "Friday",
        Sat => "Saturday",
        Sun => "Sunday",
    }
}

/// Falls back to the ISO date rather than failing: an unparseable date is
/// still worth speaking, and this is a forecast, not a parser.
fn day_label(iso: &str, index: usize) -> String {
    match index {
        0 => "Today".to_string(),
        1 => "Tomorrow".to_string(),
        _ => NaiveDate::parse_from_str(iso, "%Y-%m-%d")
            .map(|d| weekday_name(d.weekday()).to_string())
            .unwrap_or_else(|_| iso.to_string()),
    }
}

fn temp_unit(units: &Units) -> &'static str {
    match units {
        Units::Metric => "°C",
        Units::Imperial => "°F",
    }
}

/// `mph`, not the API's own `"mp/h"` — this string is spoken, not echoed.
fn wind_unit(units: &Units) -> &'static str {
    match units {
        Units::Metric => "km/h",
        Units::Imperial => "mph",
    }
}

fn precip_unit(units: &Units) -> &'static str {
    match units {
        Units::Metric => "mm",
        Units::Imperial => "in",
    }
}

pub fn render(place: &str, forecast: &Forecast, units: &Units, clamp_note: Option<&str>) -> String {
    let t = temp_unit(units);
    let mut out = format!(
        "{place} — currently {:.0}{t}, {}, wind {:.0} {}.",
        forecast.current.temperature_2m,
        describe_code(forecast.current.weather_code),
        forecast.current.wind_speed_10m,
        wind_unit(units),
    );

    let days = &forecast.daily;
    for i in 0..days.time.len() {
        let precip = days.precipitation_sum.get(i).copied().unwrap_or(0.0);
        // The amount is only worth saying when there is any.
        let weather = if precip > 0.0 {
            format!(
                ", {} {precip:.0}{}",
                describe_code(days.weather_code[i]),
                precip_unit(units)
            )
        } else {
            format!(", {}", describe_code(days.weather_code[i]))
        };
        out.push_str(&format!(
            " {} {:.0}-{:.0}{t}{weather}.",
            day_label(&days.time[i], i),
            days.temperature_2m_min[i],
            days.temperature_2m_max[i],
        ));
    }

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
        let out = render(
            "Melbourne, Victoria, Australia",
            &forecast(),
            &Units::Metric,
            None,
        );
        assert!(
            out.starts_with(
                "Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h."
            ),
            "{out}"
        );
    }

    #[test]
    fn every_successful_result_carries_the_attribution_line() {
        assert!(render("X", &forecast(), &Units::Metric, None).ends_with(ATTRIBUTION));
    }

    #[test]
    fn a_clamp_note_is_spoken_before_the_attribution() {
        let out = render(
            "X",
            &forecast(),
            &Units::Metric,
            Some("Asked for 99 days; 7 are shown."),
        );
        assert!(
            out.contains("Asked for 99 days; 7 are shown. Data by Open-Meteo.com"),
            "{out}"
        );
    }

    #[test]
    fn an_unknown_wmo_code_still_produces_a_word() {
        assert_eq!(describe_code(4242), "unsettled");
    }

    #[test]
    fn a_rendered_forecast_reads_as_one_speakable_paragraph() {
        let out = render(
            "Melbourne, Victoria, Australia",
            &forecast(),
            &Units::Metric,
            None,
        );
        assert_eq!(
            out,
            "Melbourne, Victoria, Australia — currently 14°C, overcast, wind 19 km/h. \
Today 11-17°C, rain 4mm. Tomorrow 9-16°C, showers. Saturday 10-19°C, clear. \
Data by Open-Meteo.com, CC-BY 4.0."
        );
    }

    #[test]
    fn no_hourly_rows_appear_in_the_output() {
        let out = render("X", &forecast(), &Units::Metric, None);
        // Three daily sentences, one current sentence, one attribution.
        assert_eq!(out.matches("°C").count(), 4);
    }

    #[test]
    fn imperial_units_change_the_rendered_unit_strings() {
        let out = render("Melbourne", &forecast(), &Units::Imperial, None);
        assert!(out.contains("°F"), "{out}");
        assert!(out.contains("mph"), "{out}");
        assert!(out.contains("4in"), "{out}");
        assert!(!out.contains("°C"), "{out}");
    }
}
