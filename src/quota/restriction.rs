use super::*;
/// Only current, shared capacity windows can restrict an entire account.
pub fn restriction(value: &Value, now: i64) -> Option<crate::availability::Restriction> {
    if value["status"] != "available" {
        return None;
    }
    let empty: Vec<_> = value["windows"]
        .as_array()?
        .iter()
        .filter(|w| !w["label"].as_str().unwrap_or("").starts_with("seven_day_"))
        .filter(|w| w["remaining_percent"].as_f64().is_some_and(|n| n <= 0.0))
        .collect();
    if empty.is_empty() {
        return None;
    }
    let resets: Option<Vec<_>> = empty
        .iter()
        .map(|w| w["resets_at"].as_i64().filter(|at| *at > now))
        .collect();
    Some(crate::availability::Restriction {
        available_at: resets.and_then(|v| v.into_iter().max()),
        observed_at: value["observed_at"].as_i64()?,
        reason: "room.account_quota_exhausted".into(),
        source: "quota_observation".into(),
    })
}
