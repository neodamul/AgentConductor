use crate::quota::Budget;
use serde::{Deserialize, Serialize};

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
                .filter(|restriction| !old && restriction.available_at.is_none_or(|at| at > now)),
            budgets,
            plan_type: self.plan_type.clone(),
            source: self.source.clone(),
            observed_at: self.observed_at,
            stale,
            windows,
        }
    }
}
