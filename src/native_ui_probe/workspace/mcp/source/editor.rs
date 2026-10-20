use super::append_reference;
use eframe::egui;
use serde_json::Value;
use std::collections::BTreeSet;

mod review;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Preview,
    Apply(Vec<String>),
}

#[derive(Default)]
pub struct State {
    reference_kind: Option<&'static str>,
    reference_id: String,
    preview: Option<Value>,
    preview_goal: String,
    preview_mcp: Value,
    preview_goal_sources: Value,
    previewing: bool,
    applying: bool,
    selected: BTreeSet<String>,
    error: Option<String>,
}

impl State {
    pub fn show(
        &mut self,
        goal: &mut String,
        snapshot: Option<&Value>,
        language: &str,
        can_edit: bool,
        ui: &mut egui::Ui,
    ) -> Option<Event> {
        let text = |id| crate::ui::messages::in_language(language, id);
        if snapshot.is_some_and(|snapshot| {
            self.preview.is_some()
                && (snapshot["config"]["goal"] != self.preview_goal
                    || snapshot["config"]["mcp"] != self.preview_mcp
                    || snapshot["config"]["goal_sources"] != self.preview_goal_sources)
        }) {
            self.clear_preview();
        }
        let mut event = None;
        let references = crate::mcp::references(goal).map_or(0, |items| items.len());
        let saved_goal = snapshot.is_some_and(|value| value["config"]["goal"] == *goal);
        let has_servers = snapshot.is_some_and(|value| {
            value["config"]["mcp"]
                .as_object()
                .is_some_and(|servers| !servers.is_empty())
        });
        let has_coordinator = snapshot.is_some_and(|value| {
            value["config"]["coordinator"]
                .as_str()
                .is_some_and(|id| !id.is_empty())
        });
        ui.horizontal(|ui| {
            for (kind, label) in [
                ("jira", "mcp.jira_story"),
                ("confluence", "mcp.confluence_page"),
            ] {
                if ui
                    .add_enabled(can_edit, egui::Button::new(text(label)))
                    .accessible_name(text(label))
                    .clicked()
                {
                    self.reference_kind = Some(kind);
                    self.reference_id.clear();
                    self.error = None;
                }
            }
            if ui
                .add_enabled(
                    can_edit
                        && saved_goal
                        && references > 0
                        && has_servers
                        && has_coordinator
                        && !self.previewing
                        && !self.applying,
                    egui::Button::new(text("mcp.preview_imports")),
                )
                .accessible_name(text("mcp.preview_imports"))
                .clicked()
            {
                self.preview = None;
                self.preview_goal = goal.clone();
                self.previewing = true;
                self.error = None;
                event = Some(Event::Preview);
            }
        });
        ui.label(text("mcp.explicit_references_are_kept_in_your_goal_adding"));
        if references > 0 {
            let count = references.to_string();
            ui.label(crate::ui::messages::render(
                "mcp.references_to_preview",
                &[("value1", &count)],
            ));
            if !saved_goal {
                ui.label(text("mcp.save_goal_before_preview"));
            } else if !has_servers {
                ui.label(text("mcp.register_server_before_preview"));
            } else if !has_coordinator {
                ui.label(text("mcp.coordinator_required"));
            }
        } else if goal.contains("jira:") || goal.contains("confluence:") {
            ui.colored_label(
                ui.visuals().error_fg_color,
                text("mcp.enter_a_jira_issue_key_or_numeric_confluence"),
            );
        }
        if let Some(snapshot) = snapshot {
            self.show_reference_dialog(goal, language, can_edit, ui);
            self.show_status(snapshot, language, ui);
        }
        if self.previewing {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(text("mcp.loading_sources"));
            });
        }
        if self.applying {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(text("mcp.applying_sources"));
            });
        }
        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
        if self.preview.is_some()
            && self.preview_goal == *goal
            && let Some(preview) = self.preview.clone()
        {
            event = review::show(
                &preview,
                &mut self.selected,
                self.applying,
                self.previewing,
                can_edit && saved_goal && has_servers && has_coordinator,
                language,
                ui,
            );
            self.applying |= matches!(event, Some(Event::Apply(_)));
        }
        event
    }

    fn show_reference_dialog(
        &mut self,
        goal: &mut String,
        language: &str,
        can_edit: bool,
        ui: &mut egui::Ui,
    ) {
        let Some(kind) = self.reference_kind else {
            return;
        };
        let text = |id| crate::ui::messages::in_language(language, id);
        let mut open = true;
        let mut close_requested = false;
        egui::Window::new(text("mcp.add_a_document_reference"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(520.0)
            .show(ui.ctx(), |ui| {
                ui.label(text("mcp.enter_a_jira_issue_key_or_numeric_confluence"));
                let input = ui.add(
                    egui::TextEdit::singleline(&mut self.reference_id)
                        .hint_text(text("mcp.emm_52599_or_53553565")),
                );
                input.accessible_name(text("mcp.add_reference"));
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            can_edit && !self.reference_id.trim().is_empty(),
                            egui::Button::new(text("mcp.add_reference")),
                        )
                        .accessible_name(text("mcp.add_reference"))
                        .clicked()
                    {
                        match append_reference(goal, kind, self.reference_id.trim()) {
                            Ok(()) => {
                                self.reference_kind = None;
                                self.error = None;
                            }
                            Err(error) => self.error = Some(error),
                        }
                    }
                    if ui
                        .button(text("startup.close"))
                        .accessible_name(text("startup.close"))
                        .clicked()
                    {
                        close_requested = true;
                    }
                });
            });
        if !open || close_requested {
            self.reference_kind = None;
        }
    }

    fn show_status(&self, snapshot: &Value, language: &str, ui: &mut egui::Ui) {
        if let Some(applied) = snapshot.get("imports").filter(|value| value.is_object()) {
            let count = applied["selected"].as_array().map_or(0, Vec::len);
            let count = count.to_string();
            ui.label(crate::ui::messages::render(
                "mcp.applied_sources",
                &[("value1", &count)],
            ));
        }
        if crate::mcp::references(snapshot["config"]["goal"].as_str().unwrap_or_default())
            .is_ok_and(|references| !references.is_empty())
            && snapshot["config"]["coordinator"]
                .as_str()
                .is_none_or(str::is_empty)
        {
            ui.label(crate::ui::messages::in_language(
                language,
                "mcp.coordinator_required",
            ));
        }
    }

    pub(in crate::native_ui_probe) fn preview_result(&mut self, preview: &Value) {
        self.preview = Some(preview.clone());
        self.selected = preview["records"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|record| review::reference_key(&record["reference"]))
            .collect();
        self.previewing = false;
        self.applying = false;
        self.error = None;
    }

    pub(in crate::native_ui_probe) fn preview(&self) -> Option<&Value> {
        self.preview.as_ref()
    }

    pub(in crate::native_ui_probe) fn previewing(&self) -> bool {
        self.previewing
    }

    pub(in crate::native_ui_probe) fn preview_started(&mut self, snapshot: &Value) {
        self.preview = None;
        self.preview_goal = snapshot["config"]["goal"]
            .as_str()
            .unwrap_or_default()
            .to_owned();
        self.preview_mcp = snapshot["config"]["mcp"].clone();
        self.preview_goal_sources = snapshot["config"]["goal_sources"].clone();
        self.previewing = true;
        self.error = None;
    }

    pub(in crate::native_ui_probe) fn preview_failed(&mut self, error: &str) {
        self.previewing = false;
        self.applying = false;
        self.error = Some(error.to_owned());
    }

    pub(in crate::native_ui_probe) fn apply_started(&mut self) {
        self.applying = true;
        self.error = None;
    }

    pub(in crate::native_ui_probe) fn applied(&mut self) {
        self.clear_preview();
        self.error = None;
    }

    pub(in crate::native_ui_probe) fn clear_preview(&mut self) {
        self.preview = None;
        self.preview_goal.clear();
        self.preview_mcp = Value::Null;
        self.preview_goal_sources = Value::Null;
        self.previewing = false;
        self.applying = false;
    }
}
