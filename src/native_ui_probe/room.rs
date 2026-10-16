use eframe::egui::{self, Color32};
use serde_json::Value;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Refresh,
    Stop(String),
    Evidence,
    Decision(CandidateDecision, String),
    Resume,
    Recover,
    ToolDecision(super::room_actions::ToolAction, String),
    Briefing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CandidateDecision {
    Approve,
    Reject,
    Merge,
}

pub fn candidate_decision_request(
    snapshot: &Value,
    decision: CandidateDecision,
    reason: &str,
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .ok_or("current run revision is unavailable")?;
    let state = snapshot["report"]["state"]
        .as_str()
        .ok_or("current run state is unavailable")?;
    let candidate = snapshot["report"]["candidate"]["commit"]
        .as_str()
        .filter(|commit| !commit.is_empty())
        .ok_or("no candidate is available")?;
    let action = match decision {
        CandidateDecision::Approve if state == "waiting_approval" => {
            serde_json::json!({"kind":"approve","sha":candidate})
        }
        CandidateDecision::Approve => return Err("candidate is not waiting for approval".into()),
        CandidateDecision::Merge if state == "approved" => {
            serde_json::json!({"kind":"merge","sha":candidate})
        }
        CandidateDecision::Merge => return Err("candidate must be approved before merge".into()),
        CandidateDecision::Reject if !reason.trim().is_empty() && reason.len() <= 16 * 1024 => {
            serde_json::json!({"kind":"reject","reason":reason})
        }
        CandidateDecision::Reject => {
            return Err("rejection needs a nonempty reason up to 16 KiB".into());
        }
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":action
    }))
    .map_err(|error| error.to_string())
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
    reject_reason: String,
    stop_reason: String,
    pub notice: Option<String>,
    pub evidence: Option<Value>,
    pub diff: String,
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
            reject_reason: String::new(),
            stop_reason: String::new(),
            notice: None,
            evidence: None,
            diff: String::new(),
        }
    }
}

pub fn show(
    state: &mut State,
    snapshot: Option<&Value>,
    pending: bool,
    error: Option<&str>,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    ui.heading(text("room.agent_room"));
    let mut event = ui
        .add_enabled(
            !pending,
            egui::Button::new(format!("↻ {}", text("room.refresh"))),
        )
        .accessible_name(text("room.refresh_run"))
        .clicked()
        .then_some(Event::Refresh);
    if ui
        .add_enabled(!pending, egui::Button::new(text("room.team_briefing")))
        .accessible_name(text("room.load_read_only_team_briefing"))
        .clicked()
    {
        event = Some(Event::Briefing);
    }
    if let Some(briefing) = &state.briefing {
        super::room_briefing::show(briefing, language, ui);
    }
    let run_state = snapshot
        .and_then(|value| value["report"]["state"].as_str())
        .unwrap_or_default();
    let stoppable = matches!(
        run_state,
        "running" | "ready" | "waiting_availability" | "waiting_tool_approval" | "stopped"
    );
    if !stoppable {
        state.confirming_stop = false;
    }
    if !state.confirming_stop
        && stoppable
        && ui
            .add_enabled(!pending, egui::Button::new(text("room.stop_run")))
            .accessible_name(text("room.stop_run"))
            .clicked()
    {
        state.confirming_stop = true;
        if state.stop_reason.is_empty() {
            state.stop_reason = text("room.operator_requested_stop").into();
        }
    }
    if state.confirming_stop && stoppable {
        ui.separator();
        ui.label(text("room.stop_all_active_tasks_and_tests_retain_their"));
        let label = ui.label(text("room.stop_reason"));
        ui.add_enabled(
            !pending,
            egui::TextEdit::singleline(&mut state.stop_reason).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name(text("room.stop_reason"));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!pending, egui::Button::new(text("room.cancel_stop")))
                .accessible_name(text("room.cancel_stop"))
                .clicked()
            {
                state.confirming_stop = false;
            }
            if ui
                .add_enabled(
                    !pending
                        && !state.stop_reason.trim().is_empty()
                        && state.stop_reason.len() <= 16 * 1024,
                    egui::Button::new(text("room.confirm_stop")),
                )
                .accessible_name(text("room.confirm_stop"))
                .clicked()
            {
                state.confirming_stop = false;
                event = Some(Event::Stop(state.stop_reason.clone()));
            }
        });
    }
    if let Some(action) = super::room_actions::controls(state, snapshot, pending, language, ui) {
        event = Some(action);
    }
    if let Some(action) = super::room_actions::tool_controls(state, snapshot, pending, language, ui)
    {
        event = Some(action);
    }
    if let Some(snapshot) = snapshot {
        super::render::snapshot(snapshot, language, ui);
    } else {
        ui.label(text("room.connecting_to_local_run"));
    }
    if state.evidence.is_none()
        && snapshot.is_some_and(|snapshot| snapshot["report"]["candidate"].is_object())
    {
        ui.add_enabled_ui(!pending, |ui| {
            if ui
                .button(text("room.load_diff_and_tests"))
                .accessible_name(text("room.load_review_evidence"))
                .clicked()
            {
                event = Some(Event::Evidence);
            }
        });
    }
    if let Some(evidence) = &state.evidence {
        super::render::evidence_in_language(evidence, &mut state.diff, language, ui);
    }
    let current_evidence = state.evidence.as_ref().is_some_and(|evidence| {
        snapshot.is_some_and(|snapshot| evidence["revision"] == snapshot["revision"])
    });
    if let Some(snapshot) = snapshot
        && let Some(decision) =
            decision_controls(state, snapshot, current_evidence, pending, language, ui)
    {
        event = Some(decision);
    }
    if pending {
        ui.spinner();
    }
    if let Some(notice) = state.notice.as_deref() {
        ui.label(notice);
    }
    if let Some(error) = error {
        ui.colored_label(Color32::DARK_RED, error);
    }
    event
}

fn decision_controls(
    state: &mut State,
    snapshot: &Value,
    current_evidence: bool,
    pending: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    let run_state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let has_candidate = snapshot["report"]["candidate"]["commit"].is_string();
    let can_approve = current_evidence && run_state == "waiting_approval";
    let can_merge = current_evidence && run_state == "approved";
    if !has_candidate || matches!(run_state, "merged" | "completed" | "rejected") {
        state.confirming_decision = None;
        return None;
    }
    if matches!(
        state.confirming_decision,
        Some(CandidateDecision::Approve) if !can_approve
    ) || matches!(
        state.confirming_decision,
        Some(CandidateDecision::Merge) if !can_merge
    ) {
        state.confirming_decision = None;
    }
    if state.confirming_decision.is_none() {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !pending && can_approve,
                    egui::Button::new(text("room.approve_verified_candidate")),
                )
                .accessible_name(text("room.approve_verified_candidate"))
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Approve);
            }
            if ui
                .add_enabled(!pending, egui::Button::new(text("room.reject_candidate")))
                .accessible_name(text("room.reject_candidate"))
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Reject);
            }
            if ui
                .add_enabled(
                    !pending && can_merge,
                    egui::Button::new(text("room.merge_approved_candidate")),
                )
                .accessible_name(text("room.merge_approved_candidate"))
                .clicked()
            {
                state.confirming_decision = Some(CandidateDecision::Merge);
            }
        });
    }
    let decision = state.confirming_decision?;
    let label = match decision {
        CandidateDecision::Approve => text("room.confirm_approval_of_this_verified_diff_and_tests"),
        CandidateDecision::Reject => text("room.reject_this_candidate_and_send_feedback"),
        CandidateDecision::Merge => text("room.confirm_integrating_the_exact_approved_candidate"),
    };
    ui.separator();
    ui.label(label).accessible_name(label);
    if decision == CandidateDecision::Reject {
        let label = ui.label(text("room.rejection_reason"));
        ui.add_enabled(
            !pending,
            egui::TextEdit::singleline(&mut state.reject_reason),
        )
        .labelled_by(label.id)
        .accessible_name(text("room.rejection_reason"));
    }
    let mut event = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(
                !pending,
                egui::Button::new(text("room.cancel_candidate_decision")),
            )
            .accessible_name(text("room.cancel_candidate_decision"))
            .clicked()
        {
            state.confirming_decision = None;
        }
        let valid_reason = decision != CandidateDecision::Reject
            || !state.reject_reason.trim().is_empty() && state.reject_reason.len() <= 16 * 1024;
        if ui
            .add_enabled(
                !pending && valid_reason,
                egui::Button::new(text("room.confirm_candidate_decision")),
            )
            .accessible_name(text("room.confirm_candidate_decision"))
            .clicked()
        {
            event = Some(Event::Decision(decision, state.reject_reason.clone()));
            state.confirming_decision = None;
        }
    });
    event
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
    worker.send(br#"{"kind":"snapshot"}"#)?;
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
            .and_then(|worker| worker.send(br#"{"kind":"snapshot"}"#));
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
