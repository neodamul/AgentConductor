use std::collections::BTreeMap;

/// The browser reports only its last attempt; Rust owns the refresh interval policy.
pub fn refresh_due(last: Option<u64>, now: u64, minutes: u16) -> bool {
    let minutes = if (1..=60).contains(&minutes) {
        u64::from(minutes)
    } else {
        5
    };
    last.is_none_or(|last| now < last || now - last >= minutes * 60_000)
}

pub fn due_by_profile(
    profiles: &[crate::plan::AgentProfile],
    last: &BTreeMap<String, u64>,
    now: u64,
) -> serde_json::Value {
    serde_json::Value::Object(
        profiles
            .iter()
            .filter(|profile| {
                matches!(
                    profile.parser,
                    crate::protocol::Provider::Claude | crate::protocol::Provider::Codex
                )
            })
            .map(|profile| {
                (
                    profile.id.clone(),
                    refresh_due(
                        last.get(&profile.id).copied(),
                        now,
                        profile.quota_refresh_minutes,
                    )
                    .into(),
                )
            })
            .collect(),
    )
}
