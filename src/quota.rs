mod custom;
pub use custom::{parse_custom_output, query_custom};
mod restriction;
pub use restriction::restriction;
mod display;
pub use display::{Badge, badge, badges, budget_rows, window_labels, window_reset_labels};
mod refresh;
pub use refresh::{due_by_profile, refresh_due};
mod budget;
mod reset_time;
pub use budget::Budget;
mod workspace;
pub use workspace::stored_for_run;
pub(crate) use workspace::workspace_cache;
mod terminal;
pub use terminal::{
    claude_blocker, claude_trust_answer, claude_trust_prompt, claude_usage_ready,
    parse_claude_usage, query_claude,
};
mod claude;
pub(crate) use claude::observe_events;
pub use claude::parse_claude_event;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Window {
    pub bucket: String,
    pub label: String,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<i64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restriction: Option<crate::availability::Restriction>,
    #[serde(default)]
    pub budgets: Vec<Budget>,
    #[serde(default)]
    pub plan_type: Option<String>,
    pub source: String,
    pub observed_at: i64,
    pub windows: Vec<Window>,
}
#[derive(Clone, Debug, Serialize)]
pub struct View {
    pub restriction: Option<crate::availability::Restriction>,
    pub budgets: Vec<Budget>,
    pub plan_type: Option<String>,
    pub source: String,
    pub observed_at: i64,
    pub stale: bool,
    pub windows: Vec<Window>,
}
impl Snapshot {
    /// Never extrapolate capacity from old data or an expired reset window.
    pub fn view(&self, now: i64) -> View {
        let age = now.checked_sub(self.observed_at);
        let old = age.is_none_or(|age| !(0..=300).contains(&age));
        let mut stale = old;
        let windows = self
            .windows
            .iter()
            .cloned()
            .map(|mut window| {
                if old || window.resets_at.is_some_and(|reset| reset <= now) {
                    window.remaining_percent = None;
                    stale = true;
                }
                window
            })
            .collect();
        let budgets = self
            .budgets
            .iter()
            .cloned()
            .map(|mut budget| {
                if old || budget.resets_at.is_some_and(|reset| reset <= now) {
                    budget.remaining = None;
                    budget.remaining_percent = None;
                    budget.unlimited = false;
                    stale = true;
                }
                budget
            })
            .collect();
        View {
            restriction: self
                .restriction
                .clone()
                .filter(|r| !old && r.available_at.is_none_or(|at| at > now)),
            budgets,
            plan_type: self.plan_type.clone(),
            source: self.source.clone(),
            observed_at: self.observed_at,
            stale,
            windows,
        }
    }
}
mod parse;
pub use parse::{parse_claude, parse_codex};
pub(super) use parse::{snapshot, window};

mod query;
pub use query::query;

fn profile_digest(profile: &crate::plan::AgentProfile) -> Result<String, String> {
    let mut profile = profile.clone();
    profile.quota_refresh_minutes = 5;
    profile.catalog = None;
    Ok(crate::workflow::digest(
        &serde_json::to_vec(&profile).map_err(|e| e.to_string())?,
    ))
}
pub fn persist(
    store: &mut crate::state::Store,
    profile: &crate::plan::AgentProfile,
    result: Result<Snapshot, String>,
) -> Result<(), String> {
    let (snapshot, error) = match result {
        Ok(snapshot) => (Some(snapshot), None),
        Err(error) => (None, Some(error)),
    };
    store.record(&format!("quota/{}",profile.id),&serde_json::json!({"profile_digest":profile_digest(profile)?,"snapshot":snapshot,"error":error}),&format!("Account quota metadata refreshed for agent {}; no generation or execution permission granted",profile.id))
}
pub fn stored(
    store: &crate::state::Store,
    profile: &crate::plan::AgentProfile,
    now: i64,
) -> Result<Value, String> {
    let unavailable = |error: &str| serde_json::json!({"status":"unavailable","windows":[],"error":error,"refresh_minutes":profile.quota_refresh_minutes});
    let Some(value) = store.optional_document(&format!("quota/{}", profile.id))? else {
        return Ok(unavailable("Account quota not queried"));
    };
    let mut legacy = profile.clone();
    legacy.quota_refresh_minutes = 5;
    let legacy_digest =
        crate::workflow::digest(&serde_json::to_vec(&legacy).map_err(|e| e.to_string())?);
    if value["profile_digest"] != profile_digest(profile)?
        && value["profile_digest"] != legacy_digest
    {
        return Ok(unavailable("CLI profile changed; refresh quota"));
    }
    if value["snapshot"].is_null() {
        return Ok(unavailable(
            value["error"]
                .as_str()
                .unwrap_or("Quota metadata unavailable"),
        ));
    }
    let snapshot: Snapshot =
        serde_json::from_value(value["snapshot"].clone()).map_err(|e| e.to_string())?;
    let view = snapshot.view(now);
    let mut value = serde_json::to_value(&view).map_err(|e| e.to_string())?;
    value["status"] = if view.windows.iter().all(|w| w.used_percent.is_none())
        && !snapshot.budgets.iter().any(Budget::available)
    {
        "unavailable"
    } else if view.stale {
        "stale"
    } else {
        "available"
    }
    .into();
    value["refresh_minutes"] = profile.quota_refresh_minutes.into();
    value["restriction"] =
        serde_json::to_value(restriction(&value, now)).map_err(|e| e.to_string())?;
    Ok(value)
}
