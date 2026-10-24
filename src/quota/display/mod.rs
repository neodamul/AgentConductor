mod badge;
mod labels;

pub use badge::{Badge, badge, badges};
use labels::{amount, budget_share, msg, room_reset_label, room_window_label};
use serde_json::Value;

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
