pub(super) use crate::prompt_optimizer::Proposal;
use eframe::egui;

pub(super) enum Event {
    Request(String),
    Use(String),
    Keep,
}

#[derive(Default)]
pub(in crate::native_ui_probe) struct State {
    proposal: Option<Proposal>,
    goal: Option<String>,
    job: Option<String>,
    requesting: bool,
    open: bool,
    error: Option<String>,
}

impl State {
    pub(in crate::native_ui_probe) fn awaiting(&self) -> bool {
        self.requesting || self.job.is_some()
    }

    #[cfg(test)]
    pub(super) fn set_proposal(&mut self, proposal: Proposal) {
        self.proposal = Some(proposal);
    }

    #[cfg(test)]
    pub(super) fn take_proposal(&mut self) -> Option<Proposal> {
        self.proposal.take()
    }

    fn start(&mut self, goal: String) {
        self.goal = Some(goal);
        self.proposal = None;
        self.job = None;
        self.requesting = true;
        self.error = None;
        self.open = false;
    }

    fn started(&mut self, job: String) {
        self.requesting = false;
        self.job = Some(job);
    }

    pub(in crate::native_ui_probe) fn failed(&mut self, error: String) {
        self.requesting = false;
        self.job = None;
        self.error = Some(error);
    }

    fn completed(&mut self, job: &str, value: &serde_json::Value) {
        if self.job.as_deref() != Some(job) {
            self.failed("The goal optimizer job no longer matches the request".into());
            return;
        }
        match serde_json::from_value(value["proposal"].clone()) {
            Ok(proposal) => {
                self.goal = value["goal"].as_str().map(str::to_owned);
                self.proposal = Some(proposal);
                self.job = None;
                self.open = true;
            }
            Err(_) => self.failed("The coordinator returned an invalid proposal".into()),
        }
    }

    pub(in crate::native_ui_probe) fn restore(&mut self, snapshot: &serde_json::Value) {
        if let (Some(goal), Ok(proposal)) = (
            snapshot["optimize"]["goal"].as_str(),
            serde_json::from_value(snapshot["optimize"]["proposal"].clone()),
        ) {
            self.goal = Some(goal.to_owned());
            self.proposal = Some(proposal);
            self.open = true;
        }
    }

    pub(super) fn show(
        &mut self,
        language: &str,
        goal: &str,
        enabled: bool,
        ui: &mut egui::Ui,
    ) -> Option<Event> {
        if self.goal.as_deref().is_some_and(|source| source != goal) && self.proposal.is_some() {
            self.proposal = None;
            self.open = false;
        }
        let mut event = None;
        ui.horizontal(|ui| {
            let label = if self.proposal.is_some() {
                text(language, "startup.review_optimized_goal")
            } else {
                text(language, "startup.optimize_goal")
            };
            if ui
                .add_enabled(enabled && !self.awaiting(), egui::Button::new(label))
                .clicked()
            {
                if self.proposal.is_some() {
                    self.open = true;
                } else {
                    event = Some(Event::Request(goal.trim().to_owned()));
                }
            }
            if self.awaiting() {
                ui.spinner();
                ui.label(text(language, "startup.optimizing_goal"));
            }
        });
        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        if self.open {
            let mut open = self.open;
            egui::Window::new(text(language, "startup.optimized_goal_title"))
                .open(&mut open)
                .collapsible(false)
                .resizable(true)
                .show(ui.ctx(), |ui| {
                    if let Some(proposal) = &self.proposal {
                        ui.label(text(language, "startup.proposed_goal"));
                        egui::ScrollArea::vertical()
                            .max_height(180.0)
                            .show(ui, |ui| ui.label(&proposal.optimized));
                        list(
                            ui,
                            text(language, "startup.optimization_changes"),
                            &proposal.changes,
                        );
                        list(
                            ui,
                            text(language, "startup.optimization_questions"),
                            &proposal.questions,
                        );
                        ui.label(
                            egui::RichText::new(text(language, "startup.optimization_credit"))
                                .small(),
                        );
                        ui.horizontal(|ui| {
                            if ui
                                .button(text(language, "startup.use_optimized_goal"))
                                .clicked()
                            {
                                event = Some(Event::Use(proposal.optimized.clone()));
                            }
                            if ui
                                .button(text(language, "startup.keep_original_goal"))
                                .clicked()
                            {
                                event = Some(Event::Keep);
                            }
                        });
                    }
                });
            self.open = open && self.proposal.is_some();
        }
        if matches!(&event, Some(Event::Use(_) | Event::Keep)) {
            self.proposal = None;
            self.goal = None;
            self.open = false;
        }
        event
    }
}

fn request(snapshot: &serde_json::Value, goal: &str) -> Result<Vec<u8>, String> {
    crate::prompt_optimizer::build_prompt(goal, "English")?;
    let revision = snapshot["revision"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())?;
    serde_json::to_vec(&serde_json::json!({
        "kind":"optimize_goal", "revision":revision, "goal":goal.trim()
    }))
    .map_err(|error| error.to_string())
}

impl super::super::Probe {
    pub(super) fn optimize_goal(&mut self, goal: String) {
        let packet = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace revision is unavailable".to_owned())
            .and_then(|snapshot| request(snapshot, &goal))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match packet {
            Ok(()) => {
                self.goal_optimizer.start(goal);
                self.pending = Some(super::super::Pending::OptimizeStart);
                self.error = None;
            }
            Err(error) => self.goal_optimizer.failed(error),
        }
    }

    pub(in crate::native_ui_probe) fn refresh_goal_optimizer(&mut self) {
        match self.send_poll(
            br#"{"kind":"snapshot","optimize_result":true}"#,
            super::super::Pending::OptimizeRefresh,
        ) {
            Ok(_) => {}
            Err(error) => {
                self.goal_optimizer.failed(error);
                self.workspace_next_refresh = None;
            }
        }
    }

    pub(in crate::native_ui_probe) fn optimize_started(&mut self, snapshot: serde_json::Value) {
        if snapshot["accepted"] == true
            && let Some(job) = snapshot["id"].as_str()
        {
            self.goal_optimizer.started(job.to_owned());
            self.workspace_next_refresh = None;
        } else {
            self.goal_optimizer
                .failed("Goal optimizer job was not accepted".into());
        }
        self.pending = None;
        self.error = None;
    }

    pub(in crate::native_ui_probe) fn optimize_refreshed(&mut self, snapshot: serde_json::Value) {
        let Some(job) = self.goal_optimizer.job.clone() else {
            self.pending = None;
            return;
        };
        if snapshot["job"]["id"] != job || snapshot["job"]["kind"] != "optimize" {
            self.goal_optimizer
                .failed("The goal optimizer job no longer matches the request".into());
            self.workspace_next_refresh = None;
        } else if snapshot["job"]["state"] == "running" {
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        } else if snapshot["job"]["state"] == "completed" {
            self.goal_optimizer.completed(&job, &snapshot["optimize"]);
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        } else {
            let error = snapshot["job"]["error"]
                .as_str()
                .unwrap_or("Coordinator could not rewrite the goal");
            self.goal_optimizer.failed(error.to_owned());
            self.snapshot = Some(snapshot);
            self.workspace_next_refresh = None;
        }
        self.pending = None;
        self.error = None;
    }
}

#[cfg(test)]
pub(super) fn request_for_test(
    snapshot: &serde_json::Value,
    goal: &str,
) -> Result<Vec<u8>, String> {
    request(snapshot, goal)
}

fn list(ui: &mut egui::Ui, heading: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    ui.label(egui::RichText::new(heading).strong());
    for value in values {
        ui.label(format!("• {value}"));
    }
}

fn text(language: &str, key: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, key)
}
