use eframe::egui::{self, Color32};
use serde_json::Value;
use std::{
    collections::HashSet,
    time::{Duration, Instant},
};

mod decisions;
pub mod finalization;
mod lifecycle;
mod refresh;

pub use decisions::{CandidateDecision, candidate_decision_request};
pub(super) use lifecycle::action_reply;
pub use lifecycle::{evidence_reply, evidence_request, stop_request};
pub use refresh::{refresh_due, room_refresh_interval};

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

pub struct State {
    pub next_refresh: Option<Instant>,
    pub confirming_stop: bool,
    pub confirming_decision: Option<CandidateDecision>,
    pub confirming_recover: bool,
    pub confirming_tool: Option<(String, super::room_actions::ToolAction, String)>,
    pub tool_dialog: Option<String>,
    pub deferred_tools: HashSet<String>,
    pub briefing: Option<Value>,
    pub(super) reject_reason: String,
    pub(super) stop_reason: String,
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
            stop_reason: "Operator requested stop".into(),
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
    ui: &mut egui::Ui,
) -> Option<Event> {
    ui.heading("Agent Room");
    let mut event = ui
        .add_enabled(!pending, egui::Button::new("↻ Refresh"))
        .accessible_name("Refresh Agent Room")
        .clicked()
        .then_some(Event::Refresh);
    if ui
        .add_enabled(!pending, egui::Button::new("Team briefing"))
        .accessible_name("Load read-only team briefing")
        .clicked()
    {
        event = Some(Event::Briefing);
    }
    if let Some(briefing) = &state.briefing {
        super::room_briefing::show(briefing, ui);
    }
    if let Some(snapshot) = snapshot {
        finalization::show(snapshot, ui);
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
            .add_enabled(!pending, egui::Button::new("Stop run"))
            .accessible_name("Stop run")
            .clicked()
    {
        state.confirming_stop = true;
        if state.stop_reason.is_empty() {
            state.stop_reason = "Operator requested stop".into();
        }
    }
    if state.confirming_stop && stoppable {
        ui.separator();
        ui.label("Stop the active run? The request will be recorded for the coordinator.");
        let label = ui.label("Stop reason");
        ui.add_enabled(
            !pending,
            egui::TextEdit::singleline(&mut state.stop_reason).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name("Stop reason");
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!pending, egui::Button::new("Cancel stop"))
                .accessible_name("Cancel stop")
                .clicked()
            {
                state.confirming_stop = false;
            }
            if ui
                .add_enabled(
                    !pending
                        && !state.stop_reason.trim().is_empty()
                        && state.stop_reason.len() <= 16 * 1024,
                    egui::Button::new("Confirm stop"),
                )
                .accessible_name("Confirm stop")
                .clicked()
            {
                state.confirming_stop = false;
                event = Some(Event::Stop(state.stop_reason.clone()));
            }
        });
    }
    if let Some(action) = super::room_actions::controls(state, snapshot, pending, ui) {
        event = Some(action);
    }
    if let Some(action) = super::room_actions::tool_controls(state, snapshot, pending, ui) {
        event = Some(action);
    }
    if let Some(snapshot) = snapshot {
        super::render::snapshot(snapshot, ui);
    } else {
        ui.label("Connecting to the Rust run controller…");
    }
    if state.evidence.is_none()
        && snapshot.is_some_and(|snapshot| snapshot["report"]["candidate"].is_object())
    {
        ui.add_enabled_ui(!pending, |ui| {
            if ui
                .button("Load diff & tests")
                .accessible_name("Load review evidence")
                .clicked()
            {
                event = Some(Event::Evidence);
            }
        });
    }
    if let Some(evidence) = &state.evidence {
        super::render::evidence(evidence, &mut state.diff, ui);
    }
    let current_evidence = state.evidence.as_ref().is_some_and(|evidence| {
        snapshot.is_some_and(|snapshot| evidence["revision"] == snapshot["revision"])
    });
    if let Some(snapshot) = snapshot
        && let Some(decision) = decisions::controls(state, snapshot, current_evidence, pending, ui)
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
