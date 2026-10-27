use serde_json::Value;

pub(super) fn msg(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

pub(super) fn local_time(timestamp: i64, now: i64) -> String {
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let Ok(value) = time::OffsetDateTime::from_unix_timestamp(timestamp) else {
        return timestamp.to_string();
    };
    let value = value.to_offset(offset);
    if let Ok(today) = time::OffsetDateTime::from_unix_timestamp(now) {
        let today = today.to_offset(offset);
        if value.year() == today.year()
            && value.month() == today.month()
            && value.day() == today.day()
        {
            return format!("{:02}:{:02}", value.hour(), value.minute());
        }
    }
    format!(
        "{}/{:02} {:02}:{:02}",
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute()
    )
}

pub(super) fn local_datetime(timestamp: i64) -> String {
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    time::OffsetDateTime::from_unix_timestamp(timestamp)
        .map(|value| {
            let value = value.to_offset(offset);
            format!(
                "{}-{:02}-{:02} {:02}:{:02}:{:02}",
                value.year(),
                u8::from(value.month()),
                value.day(),
                value.hour(),
                value.minute(),
                value.second()
            )
        })
        .unwrap_or_else(|_| timestamp.to_string())
}

pub(super) fn room_reset_label(timestamp: i64, language: &str) -> String {
    format!(
        "{}{}",
        msg(language, "room.resets"),
        local_datetime(timestamp)
    )
}

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

pub(super) fn percent(value: &Value) -> Option<f64> {
    value.as_f64().filter(|n| n.is_finite())
}

pub(super) fn amount(value: &Value) -> String {
    let text = value.as_str().unwrap_or("—");
    match text.trim().parse::<f64>() {
        Ok(number) if number.is_finite() && number.abs() < 1e15 => {
            format!("{}", number.round())
        }
        _ => text.to_owned(),
    }
}

pub(super) fn budget_share(budget: &Value) -> Option<f64> {
    percent(&budget["remaining_percent"]).or_else(|| {
        let allocated = budget["allocated"].as_str()?.parse::<f64>().ok()?;
        let remaining = budget["remaining"].as_str()?.parse::<f64>().ok()?;
        (allocated > 0.0 && allocated.is_finite() && remaining.is_finite())
            .then_some(remaining / allocated * 100.0)
    })
}
