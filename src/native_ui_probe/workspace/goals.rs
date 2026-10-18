use super::{Probe, card};
use eframe::egui::{self, RichText};
use serde_json::Value;

mod action;
pub use action::{next_phase_request, select_goal_request};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    PlanNextPhase,
    SelectGoal(String),
}

pub(super) fn request_next_phase(probe: &mut Probe) {
    action::request_next_phase(probe);
}

pub(super) fn request_select_goal(probe: &mut Probe, slug: &str) {
    action::request_select_goal(probe, slug);
}

pub(super) fn apply(probe: &mut Probe, event: Option<Event>) {
    if let Some(event) = event {
        match event {
            Event::PlanNextPhase => request_next_phase(probe),
            Event::SelectGoal(slug) => request_select_goal(probe, &slug),
        }
    }
}

pub fn show(snapshot: &Value, language: &str, pending: bool, ui: &mut egui::Ui) -> Option<Event> {
    let slugs = snapshot["slugs"].as_array()?;
    let text = |key| crate::ui::messages::in_language(language, key);
    let mut event = None;
    if slugs.len() > 1 {
        let can_select = !pending && snapshot["job"]["state"] != "running";
        ui.horizontal_wrapped(|ui| {
            for slug in slugs {
                let name = slug["goal"]
                    .as_str()
                    .unwrap_or_else(|| slug["slug"].as_str().unwrap_or_default());
                if slug["active"] == true {
                    ui.strong(name)
                        .accessible_name(crate::ui::messages::interpolate(
                            text("startup.current_goal_named"),
                            &[("value1", name)],
                        ));
                } else if ui
                    .add_enabled(can_select, egui::Button::new(name))
                    .accessible_name(crate::ui::messages::interpolate(
                        text("startup.select_goal_named"),
                        &[("value1", name)],
                    ))
                    .clicked()
                {
                    event = slug["slug"]
                        .as_str()
                        .map(|slug| Event::SelectGoal(slug.into()));
                }
            }
        });
    }
    let Some(active) = slugs.iter().find(|slug| slug["active"] == true) else {
        return event;
    };
    let progress = &active["progress"];
    let Some(phases) = progress["phases"].as_array() else {
        return event;
    };
    if phases.is_empty() {
        return event;
    }
    let current = progress["current_phase"]
        .as_str()
        .filter(|id| phases.iter().any(|phase| phase["id"].as_str() == Some(id)));
    let current_number = current
        .and_then(|id| phases.iter().position(|phase| phase["id"] == id))
        .map_or(0, |index| index + 1);
    let proposal_phase = snapshot["proposal"]["slug_phase"].as_str();
    let busy = pending || snapshot["job"]["state"] == "running";
    let can_plan = current.is_some() && proposal_phase != current && !busy;
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
