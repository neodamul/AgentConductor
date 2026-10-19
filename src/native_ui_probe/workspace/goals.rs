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
    let reviewing_phase = proposal_phase.filter(|proposal| {
        !phases
            .iter()
            .any(|phase| phase["id"].as_str() == Some(proposal) && phase["stage"] == "completed")
    });
    let busy = pending || snapshot["job"]["state"] == "running";
    let can_plan = current.is_some() && proposal_phase != current && !busy;
    let planned_phases = active["plan"]["phases"].as_array();
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
        if let Some(analysis) = active["plan"]["analysis"]
            .as_str()
            .filter(|analysis| !analysis.is_empty())
        {
            egui::CollapsingHeader::new(text("startup.coordinator_analysis_of_the_whole_goal"))
                .id_salt("goal-plan-analysis")
                .show(ui, |ui| {
                    ui.label(analysis);
                });
        }
        for (index, phase) in phases.iter().enumerate() {
            let id = phase["id"].as_str().unwrap_or_default();
            let plan =
                planned_phases.and_then(|planned| planned.iter().find(|item| item["id"] == id));
            phase_row(ui, phase, plan, index, current, reviewing_phase, language);
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
    plan: Option<&Value>,
    index: usize,
    current: Option<&str>,
    reviewing: Option<&str>,
    language: &str,
) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let id = phase["id"].as_str().unwrap_or("");
    let name = phase["name"].as_str().unwrap_or(id);
    let stage = phase["stage"].as_str().unwrap_or("planned");
    let status = if stage == "completed" {
        "startup.completed_22a970"
    } else if reviewing == Some(id) && current == Some(id) {
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
    let title = format!("{}. {name} · {}{counts}", index + 1, text(status));
    egui::CollapsingHeader::new(title)
        .id_salt(id)
        .default_open(reviewing == Some(id) || reviewing.is_none() && current == Some(id))
        .show(ui, |ui| {
            if let Some(brief) = plan.and_then(|plan| plan["brief"].as_str())
                && !brief.is_empty()
            {
                ui.label(brief);
            }
            if reviewing == Some(id) {
                ui.label(text("startup.these_tasks_are_open_for_review_and_editing"));
            } else if let Some(specs) = plan.and_then(|plan| plan["specs"].as_array()) {
                for spec in specs {
                    task_row(ui, spec, phase, language);
                }
            }
        });
}

fn task_row(ui: &mut egui::Ui, spec: &Value, phase: &Value, language: &str) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let id = spec["id"].as_str().unwrap_or_default();
    let status = if phase["completed"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == id))
    {
        "startup.completed_22a970"
    } else if phase["verification"]
        .as_array()
        .is_some_and(|items| items.iter().any(|item| item == id))
    {
        "startup.in_verification"
    } else {
        "startup.planned"
    };
    ui.horizontal_wrapped(|ui| {
        ui.monospace(id);
        ui.label(
            RichText::new(task_line(spec["prompt"].as_str().unwrap_or_default()))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        ui.label(spec["agent"].as_str().unwrap_or("—"));
        ui.label(RichText::new(text(status)).small());
    });
}

fn task_line(prompt: &str) -> String {
    let line = prompt
        .lines()
        .map(str::trim)
        .map(|line| {
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .unwrap_or(line)
        })
        .find(|line| !line.is_empty() && !line.ends_with(':'))
        .unwrap_or_default();
    let mut chars = line.chars();
    let summary: String = chars.by_ref().take(160).collect();
    if chars.next().is_some() {
        summary + "…"
    } else {
        summary
    }
}

fn length(value: &Value, key: &str) -> String {
    value[key].as_array().map_or(0, Vec::len).to_string()
}
