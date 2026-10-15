use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct Badge {
    pub text: String,
    pub title: String,
}

pub fn badges(quotas: &Value, language: &str, now: i64) -> Value {
    let Some(values) = quotas.as_object() else {
        return Value::Object(Default::default());
    };
    Value::Object(
        values
            .iter()
            .map(|(id, quota)| {
                (
                    id.clone(),
                    serde_json::to_value(badge(quota, language, now))
                        .expect("quota badge is serializable"),
                )
            })
            .collect(),
    )
}

fn msg(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn local_time(timestamp: i64, now: i64) -> String {
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

fn local_datetime(timestamp: i64) -> String {
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

fn window_label(window: &Value, language: &str) -> String {
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

fn percent(value: &Value) -> Option<f64> {
    value.as_f64().filter(|n| n.is_finite())
}

fn amount(value: &Value) -> String {
    value.as_str().unwrap_or("—").to_owned()
}

pub fn badge(quota: &Value, language: &str, now: i64) -> Badge {
    let mut windows: Vec<_> = quota["windows"].as_array().into_iter().flatten().collect();
    windows.sort_by_key(|window| {
        window["window_minutes"]
            .as_u64()
            .or(match window["label"].as_str() {
                Some("five_hour") => Some(300),
                Some("seven_day") => Some(10_080),
                _ => None,
            })
            .unwrap_or(u64::MAX)
    });
    let stale = quota["status"] == "stale";
    let valid = windows
        .iter()
        .any(|w| percent(&w["remaining_percent"]).is_some());
    let mut text = if stale {
        msg(language, "startup.stale").to_owned()
    } else if valid {
        windows
            .iter()
            .map(|w| {
                percent(&w["remaining_percent"]).map_or("—".into(), |n| format!("{}%", n.round()))
            })
            .collect::<Vec<_>>()
            .join("/")
    } else {
        "—".into()
    };
    let mut title = if stale {
        msg(language, "startup.quota_observation_is_stale")
    } else {
        msg(language, "startup.remaining_account_capacity")
    }
    .to_owned();
    for window in &windows {
        title.push_str(&window_label(window, language));
        title.push_str(": ");
        if let Some(value) = percent(&window["remaining_percent"]) {
            title.push_str(&format!(
                "{}{}",
                value.round(),
                msg(language, "startup.remaining")
            ));
        } else {
            title.push_str(msg(language, "startup.unavailable"));
        }
        if let Some(reset) = window["resets_at"].as_i64() {
            title.push_str(msg(language, "startup.resets"));
            title.push_str(&local_time(reset, now));
        } else if percent(&window["remaining_percent"]).is_some_and(|n| n <= 0.0) {
            title.push_str(" · ");
            title.push_str(msg(language, "common.reset_unknown"));
        }
        title.push_str(" / ");
    }
    if !windows.is_empty() {
        title.truncate(title.trim_end_matches(" / ").len());
    }
    if let Some(plan) = quota["plan_type"].as_str() {
        title.push_str(&format!(" · {plan}"));
    }
    if let Some(error) = quota["error"].as_str() {
        title.push_str(&format!(" · {error}"));
    }
    if let Some(source) = quota["source"].as_str() {
        title.push_str(&format!(" · {source}"));
    }
    if let Some(observed) = quota["observed_at"].as_i64() {
        title.push_str(&format!(
            "{}{}",
            msg(language, "startup.observed"),
            local_datetime(observed)
        ));
    }

    let budgets: Vec<_> = quota["budgets"].as_array().into_iter().flatten().collect();
    let chosen = budgets
        .iter()
        .find(|b| {
            b["scope"] == "individual"
                && (b["remaining"].is_string() || percent(&b["remaining_percent"]).is_some())
        })
        .or_else(|| {
            budgets
                .iter()
                .find(|b| b["remaining"].is_string() || b["unlimited"] == true)
        });
    if let Some(chosen) = chosen.filter(|_| {
        quota["plan_type"]
            .as_str()
            .is_some_and(|p| p.eq_ignore_ascii_case("enterprise"))
            || !valid
    }) {
        let share = percent(&chosen["remaining_percent"]).or_else(|| {
            let allocated = chosen["allocated"].as_str()?.parse::<f64>().ok()?;
            let remaining = chosen["remaining"].as_str()?.parse::<f64>().ok()?;
            (allocated > 0.0 && allocated.is_finite() && remaining.is_finite())
                .then_some(remaining / allocated * 100.0)
        });
        text = if stale {
            msg(language, "startup.stale").into()
        } else if chosen["unlimited"] == true {
            "∞".into()
        } else {
            share.map_or("—".into(), |n| format!("{}%/100%", n.round()))
        };
        let detail = budgets
            .iter()
            .map(|b| {
                let scope = if b["scope"] == "individual" {
                    "startup.individual_allocation"
                } else {
                    "startup.credit_balance"
                };
                let value = if b["unlimited"] == true {
                    msg(language, "startup.unlimited").into()
                } else {
                    format!("{}/{}", amount(&b["remaining"]), amount(&b["allocated"]))
                };
                let reset = b["resets_at"]
                    .as_i64()
                    .map(|r| format!("{}{}", msg(language, "startup.resets"), local_time(r, now)))
                    .unwrap_or_default();
                format!("{}: {}{}", msg(language, scope), value, reset)
            })
            .collect::<Vec<_>>()
            .join(" / ");
        title.push_str(" · ");
        title.push_str(&detail);
    }
    Badge { text, title }
}
