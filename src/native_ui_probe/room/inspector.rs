use eframe::egui;
use serde_json::Value;

/// DD-179: a decision may open the inspector, but only the operator changes its preference.
pub struct Inspector {
    pub open: bool,
    pub preference_folded: bool,
    decision: Option<String>,
    error: Option<String>,
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new(crate::ui::language::stored_text("inspector_folded").as_deref() == Some("true"))
    }
}

impl Inspector {
    pub fn new(folded: bool) -> Self {
        Self {
            open: !folded,
            preference_folded: folded,
            decision: None,
            error: None,
        }
    }

    pub fn observe(&mut self, snapshot: &Value) -> bool {
        let key = decision_key(snapshot);
        let new_decision = key.is_some() && key != self.decision;
        if new_decision {
            self.open = true;
        } else if key.is_none() && self.decision.is_some() {
            self.open = !self.preference_folded;
        }
        self.decision = key;
        new_decision
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.preference_folded = !self.open;
    }

    pub fn attention(&self) -> bool {
        !self.open && self.decision.is_some()
    }

    pub fn warning_alpha(&self, reduced_motion: bool, seconds: f64) -> f32 {
        if reduced_motion || seconds.rem_euclid(1.1) < 0.55 {
            1.0
        } else {
            0.15
        }
    }

    pub(super) fn heading(&mut self, language: &str, ui: &mut egui::Ui) {
        let text = |id| crate::ui::messages::in_language(language, id);
        ui.horizontal(|ui| {
            let label = text(if self.open {
                "room.inspector_fold"
            } else {
                "room.inspector_unfold"
            });
            let response = ui
                .button(if self.open { "›" } else { "‹" })
                .accessible_name(label)
                .on_hover_text(label);
            ui.ctx()
                .accesskit_node_builder(response.id, |node| node.set_expanded(self.open));
            if response.clicked() {
                self.toggle();
                self.error = crate::ui::language::store(
                    "inspector_folded",
                    if self.preference_folded {
                        "true"
                    } else {
                        "false"
                    },
                )
                .err();
            }
            if self.attention() {
                let reduced = ui.style().animation_time == 0.0;
                let alpha = self.warning_alpha(reduced, ui.input(|input| input.time));
                if !reduced {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(250));
                }
                ui.label(
                    egui::RichText::new("●")
                        .color(ui.visuals().warn_fg_color.gamma_multiply(alpha)),
                )
                .accessible_name(text("room.inspector_decision"));
            }
        });
        if self.open
            && let Some(error) = &self.error
        {
            ui.colored_label(ui.visuals().warn_fg_color, error);
        }
    }
}

pub fn decision_key(snapshot: &Value) -> Option<String> {
    let report = &snapshot["report"];
    let state = report["state"].as_str().unwrap_or_default();
    let mut pending = snapshot["tool_requests"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|request| request["status"] == "pending")
        .filter_map(|request| request["id"].as_str())
        .collect::<Vec<_>>();
    pending.sort_unstable();
    let waiting = matches!(
        state,
        "waiting_approval" | "waiting_tool_approval" | "blocked"
    ) || (state == "failed"
        && report["error"]
            .as_str()
            .is_some_and(|error| error.starts_with("out-of-scope change:")));
    (waiting || !pending.is_empty())
        .then(|| serde_json::json!([state, report["candidate"]["commit"], pending]).to_string())
}
