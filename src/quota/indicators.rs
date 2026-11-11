//! R-212 / DD-224: a cached projection shared by quota rings and their detail popup.
use crate::plan::AgentProfile;
use serde_json::{Value, json};

fn percentage(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .filter(|n| n.is_finite() && (0.0..=100.0).contains(n))
}
fn minutes(row: &Value) -> Option<u64> {
    row["window_minutes"]
        .as_u64()
        .or(match row["label"].as_str() {
            Some("five_hour") => Some(300),
            Some(v) if v.starts_with("seven_day") => Some(10_080),
            _ => None,
        })
}
fn balance(row: &Value) -> Option<f64> {
    percentage(&row["remaining_percent"]).or_else(|| {
        let amount = |key: &str| {
            row[key]
                .as_str()?
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
        };
        let allocated = amount("allocated")?;
        let remaining = amount("remaining")?;
        if allocated <= 0.0 {
            return None;
        }
        let share = remaining / allocated * 100.0;
        (0.0..=100.0).contains(&share).then_some(share)
    })
}

pub fn indicators(quotas: &Value, profiles: &[AgentProfile], language: &str, now: i64) -> Value {
    let labels = super::window_labels(quotas, language);
    let resets = super::window_reset_labels(quotas, language);
    let budget_rows = super::budget_rows(quotas, language);
    Value::Object(profiles.iter().map(|profile| {
        let q=&quotas[&profile.id];
        let capable=matches!(profile.parser,crate::protocol::Provider::Claude|crate::protocol::Provider::Codex)||profile.quota_command.is_some();
        let status = if !capable && q["observed_at"].is_null() && q["windows"].as_array().is_none_or(Vec::is_empty) && q["budgets"].as_array().is_none_or(Vec::is_empty) {"unsupported"} else {q["status"].as_str().unwrap_or("unavailable")};
        let age=q["observed_at"].as_i64().and_then(|at| now.checked_sub(at));
        let old=age.is_none_or(|age| !(0..=super::fresh_limit(profile.quota_refresh_minutes)).contains(&age));
        let stale=status=="stale" || q["stale"]==true || (status=="available" && old);
        let fresh=status=="available" && !stale;
        let valid=|row:&Value| fresh && row["resets_at"].as_i64().is_none_or(|at|at>now);
        let mut windows:Vec<Value>=q["windows"].as_array().into_iter().flatten().enumerate().map(|(index,row)| {
            let remaining=valid(row).then(|| percentage(&row["remaining_percent"]).or_else(||percentage(&row["used_percent"]).map(|v|100.0-v))).flatten();
            json!({"kind":"window","index":index,"bucket":row["bucket"],"label":labels[&profile.id][index],"window_minutes":minutes(row),"remaining_percent":remaining,"used_percent":remaining.map(|v|100.0-v),"resets_at":row["resets_at"],"reset_label":resets[&profile.id][index]})
        }).collect();
        windows.sort_by_key(|row|row["window_minutes"].as_u64().unwrap_or(u64::MAX));
        let mut budgets:Vec<Value>=q["budgets"].as_array().into_iter().flatten().enumerate().map(|(index,row)| {
            let scope=if row["scope"]=="individual" {"startup.individual_allocation"}else{"startup.credit_balance"};
            let remaining=(valid(row)&&row["unlimited"]!=true).then(||balance(row)).flatten();
            json!({"kind":"budget","index":index,"scope":row["scope"],"label":crate::ui::messages::in_language(language,scope),"remaining_percent":remaining,"used_percent":remaining.map(|v|100.0-v),"remaining":row["remaining"],"allocated":row["allocated"],"unit":row["unit"],"unlimited":valid(row)&&row["unlimited"]==true,"resets_at":row["resets_at"],"reset_label":budget_rows[&profile.id][index]["reset_label"]})
        }).collect();
        budgets.sort_by_key(|row|usize::from(row["scope"]!="individual"));
        let enterprise=q["plan_type"].as_str().is_some_and(|p|p.eq_ignore_ascii_case("enterprise"));
        let selected=if !budgets.is_empty() && (enterprise||windows.is_empty()) {&budgets}else{&windows};
        let rings:Vec<_>=selected.iter().take(2).cloned().collect();
        let status=if stale {"stale"}else{status};
        (profile.id.clone(),json!({"status":status,"rings":rings,"windows":windows,"budgets":budgets,"source":q["source"],"observed_at":q["observed_at"],"plan_type":q["plan_type"],"error":q["error"],"error_code":(q["error"]==super::NO_PROJECT_FOLDER).then_some("no_project_folder"),"restriction":q["restriction"],"refresh_minutes":profile.quota_refresh_minutes}))
    }).collect())
}
