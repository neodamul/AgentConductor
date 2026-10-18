use super::{Probe, card};
use eframe::egui::{self, RichText};
use serde_json::Value;

mod action;
pub use action::next_phase_request;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event {
    PlanNextPhase,
}

pub(super) fn request_next_phase(probe: &mut Probe) {
    action::request_next_phase(probe);
}

pub fn show(snapshot: &Value, language: &str, pending: bool, ui: &mut egui::Ui) -> Option<Event> {
    let active = snapshot["slugs"]
        .as_array()?
        .iter()
        .find(|slug| slug["active"] == true)?;
    let progress = &active["progress"];
    let phases = progress["phases"].as_array()?;
    if phases.is_empty() {
        return None;
    }
    let text = |key| crate::ui::messages::in_language(language, key);
    let current = progress["current_phase"]
        .as_str()
        .filter(|id| phases.iter().any(|phase| phase["id"].as_str() == Some(id)));
    let current_number = current
        .and_then(|id| phases.iter().position(|phase| phase["id"] == id))
        .map_or(0, |index| index + 1);
    let proposal_phase = snapshot["proposal"]["slug_phase"].as_str();
    let busy = pending || snapshot["job"]["state"] == "running";
    let can_plan = current.is_some() && proposal_phase != current && !busy;
    let mut event = None;

    card(ui).show(ui, |ui| {
        ui.label(
            RichText::new(text("startup.goals_progress"))
                .small()
                .strong(),
        );
        ui.heading(RichText::new(
            active["goal"]
                .as_str()
                .unwrap_or_else(|| active["slug"].as_str().unwrap_or("")),
        ));
        ui.label(if let Some(id) = current {
            let name = phases[current_number - 1]["name"].as_str().unwrap_or(id);
            crate::ui::messages::interpolate(
                text("startup.next_phase"),
                &[
                    ("value1", &current_number.to_string()),
                    ("value2", &phases.len().to_string()),
                    ("value3", name),
                ],
            )
        } else {
            crate::ui::messages::interpolate(
                text("startup.all_phases_are_complete"),
                &[("value1", &phases.len().to_string())],
            )
        });
        for (index, phase) in phases.iter().enumerate() {
            phase_row(ui, phase, index, current, proposal_phase, language);
        }
        if can_plan
            && ui
                .add_enabled(true, egui::Button::new(text("startup.plan_next_phase")))
                .accessible_name(text("startup.plan_next_phase"))
                .clicked()
        {
            event = Some(Event::PlanNextPhase);
        }
    });
    event
}

fn phase_row(
    ui: &mut egui::Ui,
    phase: &Value,
    index: usize,
    current: Option<&str>,
    proposal: Option<&str>,
    language: &str,
) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let id = phase["id"].as_str().unwrap_or("");
    let name = phase["name"].as_str().unwrap_or(id);
    let stage = phase["stage"].as_str().unwrap_or("planned");
    let status = if stage == "completed" {
        "startup.completed_22a970"
    } else if proposal == Some(id) && current == Some(id) {
        "startup.under_review"
    } else if stage == "verification" {
        "startup.in_verification"
    } else if current == Some(id) {
        "startup.next"
    } else {
        "startup.planned"
    };
    let counts = crate::ui::messages::interpolate(
        text("startup.task_verification_completed"),
        &[
            ("value1", &length(phase, "task")),
            ("value2", &length(phase, "verification")),
            ("value3", &length(phase, "completed")),
        ],
    );
    ui.horizontal_wrapped(|ui| {
        ui.strong(format!("{}. {name}", index + 1));
        ui.label(
            RichText::new(text(status))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        ui.label(
            RichText::new(counts)
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    });
}

fn length(value: &Value, key: &str) -> String {
    value[key].as_array().map_or(0, Vec::len).to_string()
}
