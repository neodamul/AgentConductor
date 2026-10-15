use eframe::egui::{self, Color32};
use serde_json::Value;
use std::time::{Duration, Instant};

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Refresh,
    Stop(String),
    Evidence,
}

pub struct State {
    pub next_refresh: Option<Instant>,
    pub confirming_stop: bool,
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
