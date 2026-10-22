use eframe::egui;
use serde_json::Value;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};
mod candidate;
pub mod inspector;
pub mod judgments;
mod scope_recovery;
mod status_banner;
pub mod task_activity;
pub mod task_controls;
mod view;
pub use candidate::candidate_decision_request;
pub use view::show;

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Refresh,
    Stop(String),
    Evidence,
    ScopeFailure,
    ScopeRecoveryPreview {
        task: String,
        paths: Vec<String>,
        test: Option<Vec<String>>,
    },
    ScopeRecoveryApply {
        task: String,
        paths: Vec<String>,
        test: Option<Vec<String>>,
        digest: String,
    },
    Decision(CandidateDecision, String),
    Resume,
    Recover,
    ToolDecision(super::room_actions::ToolAction, String),
    Briefing,
    Coordinator,
    ReturnToWorkspace,
    TaskDetails(usize),
    TaskAction(task_controls::Action),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateDecision {
    Approve,
    Reject,
    Merge,
}

pub struct State {
    pub next_refresh: Option<Instant>,
    pub confirming_stop: bool,
    pub confirming_decision: Option<CandidateDecision>,
    pub confirming_recover: bool,
    pub confirming_tool: Option<(String, super::room_actions::ToolAction, String)>,
    pub tool_dialog: Option<String>,
    pub deferred_tools: HashSet<String>,
    pub briefing: Option<Value>,
    pub coordinator_judgments: Option<Value>,
    pub coordinator_selected: bool,
    pub inspector: inspector::Inspector,
    reject_reason: String,
    stop_reason: String,
    pub notice: Option<String>,
    pub evidence: Option<Value>,
    pub diff: String,
    pub task_controls: task_controls::State,
    pub activity_tab: task_activity::Tab,
    pub task_details: Option<Value>,
    pub details_task: Option<String>,
    pub workspace_available: bool,
    pub scope_failure: Option<Value>,
    pub scope_preview: Option<Value>,
    pub scope_recovery_open: bool,
    pub scope_task: String,
    pub scope_paths: String,
    pub scope_test_command: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            next_refresh: Some(Instant::now() + Duration::from_secs(5)),
            confirming_stop: false,
            confirming_decision: None,
            confirming_recover: false,
            confirming_tool: None,
            tool_dialog: None,
            deferred_tools: HashSet::new(),
            briefing: None,
            coordinator_judgments: None,
            coordinator_selected: false,
            inspector: inspector::Inspector::default(),
            reject_reason: String::new(),
            stop_reason: String::new(),
            notice: None,
            evidence: None,
            diff: String::new(),
            task_controls: task_controls::State::default(),
            activity_tab: task_activity::Tab::default(),
            task_details: None,
            details_task: None,
            workspace_available: false,
            scope_failure: None,
            scope_preview: None,
            scope_recovery_open: false,
            scope_task: String::new(),
            scope_paths: String::new(),
            scope_test_command: "[]".into(),
        }
    }
}

pub fn evidence_request() -> &'static [u8] {
    br#"{"kind":"evidence"}"#
}

pub fn evidence_reply(
    state: &mut State,
    current: Option<&Value>,
    evidence: Value,
) -> Result<(), String> {
    let expected = current
        .and_then(|snapshot| snapshot["revision"].as_str())
        .ok_or("current run revision is unavailable")?;
    if evidence["revision"].as_str() != Some(expected) {
        return Err("run changed while loading review evidence; refresh and try again".into());
    }
    state.diff = evidence["diff"].as_str().unwrap_or_default().to_owned();
    state.evidence = Some(evidence);
    Ok(())
}

pub fn stop_request(snapshot: &Value, reason: &str) -> Result<Vec<u8>, String> {
    if reason.trim().is_empty() || reason.len() > 16 * 1024 {
        return Err("stop needs a nonempty reason up to 16 KiB".into());
    }
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    serde_json::to_vec(&serde_json::json!({
        "kind":"action",
        "revision":revision,
        "action":{"kind":"stop","reason":reason}
    }))
    .map_err(|error| error.to_string())
}

pub(super) fn action_reply(
    worker: &super::RequestWorker,
    response: &Value,
) -> Result<Option<String>, String> {
    if response["accepted"] != true {
        return Err(response["message"]
            .as_str()
            .unwrap_or("stop request was not accepted")
            .to_owned());
    }
    worker.send(br#"{"kind":"snapshot","icons":true}"#)?;
    Ok(response["message"].as_str().map(str::to_owned))
}

impl super::Probe {
    pub(super) fn decide_room(&mut self, decision: CandidateDecision, reason: String) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| candidate_decision_request(snapshot, decision, &reason))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::Pending::Action);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn load_evidence(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(evidence_request()));
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::Pending::Evidence);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn refresh_room(&mut self) {
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(br#"{"kind":"snapshot","icons":true}"#));
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::Pending::Refresh);
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn stop_room(&mut self, reason: String) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Agent Room snapshot is unavailable".to_owned())
            .and_then(|snapshot| stop_request(snapshot, &reason))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Agent Room controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(super::Pending::Action);
            }
            Err(error) => self.error = Some(error),
        }
    }
}

pub fn room_refresh_interval(snapshot: &Value) -> Duration {
    let state = snapshot["report"]["state"]
        .as_str()
        .or_else(|| snapshot["state"].as_str());
    if snapshot["job"]["state"] == "running"
        || super::workspace::profile_tools::metadata_running(snapshot)
        || matches!(state, Some("running" | "ready" | "waiting_availability"))
    {
        Duration::from_secs(1)
    } else {
        Duration::from_secs(5)
    }
}

pub fn refresh_due(
    context: &egui::Context,
    snapshot: Option<&Value>,
    pending: bool,
    next: &mut Option<Instant>,
) -> bool {
    if pending {
        return false;
    }
    let interval = room_refresh_interval(snapshot.unwrap_or(&Value::Null));
    let now = Instant::now();
    let deadline = next.get_or_insert(now + interval);
    *deadline = (*deadline).min(now + interval);
    if now >= *deadline {
        *deadline = now + interval;
        context.request_repaint_after(interval);
        true
    } else {
        context.request_repaint_after(*deadline - now);
        false
    }
}
