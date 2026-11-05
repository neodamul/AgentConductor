//! Quota window labels for the start screen and the room.

use super::msg;
use serde_json::Value;

pub(super) fn window_label(window: &Value, language: &str) -> String {
    let label = window["label"].as_str().unwrap_or("");
    let minutes = window["window_minutes"].as_u64().or(match label {
        "five_hour" => Some(300),
        "seven_day" | "seven_day_opus" | "seven_day_sonnet" => Some(10_080),
        _ => None,
    });
    match label {
        "five_hour" => "5h".into(),
        "seven_day" | "seven_day_opus" | "seven_day_sonnet" => msg(
            language,
            match label {
                "seven_day_opus" => "startup.weekly_opus",
                "seven_day_sonnet" => "startup.weekly_sonnet",
                _ => "startup.weekly",
            },
        )
        .into(),
        _ if label.starts_with("seven_day_") => format!(
            "{}{}",
            msg(language, "startup.weekly_fb43b5"),
            label[10..].replace('_', " ")
        ),
        _ => match minutes {
            Some(1_440) => msg(language, "startup.daily").into(),
            Some(10_080) => msg(language, "startup.weekly").into(),
            Some(value) if value % 1_440 == 0 => format!("{}d", value / 1_440),
            Some(value) if value % 60 == 0 => format!("{}h", value / 60),
            Some(value) => format!("{value}m"),
            None if !label.is_empty() => label.into(),
            None => msg(language, "startup.window").into(),
        },
    }
}

pub(super) fn room_window_label(window: &Value, language: &str) -> String {
    let label = window["label"].as_str().unwrap_or("");
    match label {
        "five_hour" => "5h".into(),
        "seven_day" | "seven_day_opus" | "seven_day_sonnet" => msg(
            language,
            match label {
                "seven_day_opus" => "startup.weekly_opus",
                "seven_day_sonnet" => "startup.weekly_sonnet",
                _ => "startup.weekly",
            },
        )
        .into(),
        _ if label.starts_with("seven_day_") => format!(
            "{}{}",
            msg(language, "startup.weekly_fb43b5"),
            label[10..].replace('_', " ")
        ),
        _ => match window["window_minutes"].as_u64() {
            Some(300) => "5h".into(),
            Some(1_440) => msg(language, "startup.daily").into(),
            Some(10_080) => msg(language, "startup.weekly").into(),
            Some(minutes) => {
                let value = minutes.to_string();
                crate::ui::messages::interpolate(
                    msg(language, "room.minute"),
                    &[("value1", &value)],
                )
            }
            None => window_label(window, language),
        },
    }
}
