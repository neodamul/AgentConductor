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

pub fn window_labels(quotas: &Value, language: &str) -> Value {
    let Some(values) = quotas.as_object() else {
        return Value::Object(Default::default());
    };
    Value::Object(
        values
            .iter()
            .map(|(id, quota)| {
                let labels = quota["windows"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|window| Value::String(room_window_label(window, language)))
                    .collect();
                (id.clone(), Value::Array(labels))
            })
            .collect(),
    )
}

pub fn window_reset_labels(quotas: &Value, language: &str) -> Value {
    let Some(values) = quotas.as_object() else {
        return Value::Object(Default::default());
    };
    Value::Object(
        values
            .iter()
            .map(|(id, quota)| {
                let labels = quota["windows"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|window| {
                        window["resets_at"]
                            .as_i64()
                            .map(|reset| Value::String(room_reset_label(reset, language)))
                            .unwrap_or(Value::Null)
                    })
                    .collect();
                (id.clone(), Value::Array(labels))
            })
            .collect(),
    )
}

pub fn budget_rows(quotas: &Value, language: &str) -> Value {
    let Some(values) = quotas.as_object() else {
        return Value::Object(Default::default());
    };
    Value::Object(
        values
            .iter()
            .map(|(id, quota)| {
                let stale = quota["status"] == "stale";
                let rows = quota["budgets"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|budget| {
                        let share = budget_share(budget);
                        let unlimited = budget["unlimited"] == true;
                        let value = if stale {
                            msg(language, "startup.stale").to_owned()
                        } else if unlimited {
                            msg(language, "room.unlimited").to_owned()
                        } else if let Some(value) = share {
                            crate::ui::messages::interpolate(
                                msg(language, "room.remaining"),
                                &[("value1", &value.round().to_string())],
                            )
                        } else {
                            format!(
                                "{}/{}",
                                amount(&budget["remaining"]),
                                amount(&budget["allocated"])
                            )
                        };
                        let scope = msg(
                            language,
                            if budget["scope"] == "individual" {
                                "startup.individual_allocation"
                            } else {
                                "startup.credit_balance"
                            },
                        );
                        let headline = format!("{value} · {scope}");
                        let amounts = (!unlimited
                            && share.is_some()
                            && (!budget["remaining"].is_null() || !budget["allocated"].is_null()))
                        .then(|| {
                            format!(
                                "{}/{}",
                                amount(&budget["remaining"]),
                                amount(&budget["allocated"])
                            )
                        });
                        serde_json::json!({
                            "headline": headline,
                            "amounts": amounts,
                            "reset_label": budget["resets_at"].as_i64().map(|reset| room_reset_label(reset, language))
                        })
                    })
                    .collect();
                (id.clone(), Value::Array(rows))
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

fn room_reset_label(timestamp: i64, language: &str) -> String {
    format!(
        "{}{}",
        msg(language, "room.resets"),
        local_datetime(timestamp)
    )
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

fn room_window_label(window: &Value, language: &str) -> String {
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

fn percent(value: &Value) -> Option<f64> {
    value.as_f64().filter(|n| n.is_finite())
}

/// #162: balances are shown as whole numbers; text that is not a finite number stays as is.
fn amount(value: &Value) -> String {
    let text = value.as_str().unwrap_or("—");
    match text.trim().parse::<f64>() {
        Ok(n) if n.is_finite() && n.abs() < 1e15 => format!("{}", n.round()),
        _ => text.to_owned(),
    }
}

fn budget_share(budget: &Value) -> Option<f64> {
    percent(&budget["remaining_percent"]).or_else(|| {
        let allocated = budget["allocated"].as_str()?.parse::<f64>().ok()?;
        let remaining = budget["remaining"].as_str()?.parse::<f64>().ok()?;
        (allocated > 0.0 && allocated.is_finite() && remaining.is_finite())
            .then_some(remaining / allocated * 100.0)
    })
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
        let share = budget_share(chosen);
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
