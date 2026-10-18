use super::{Probe, execution, planning_activity_color, planning_status, tasks};
use eframe::egui::{self, Color32, RichText};
use serde_json::Value;

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/review_width.rs"]
mod tests;

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let language = probe.navigation.appearance.language().to_owned();
    let Some(snapshot) = probe.snapshot.clone() else {
        ui.label(text(&language, "startup.review_connecting"));
        return;
    };
    let has_plan = snapshot["proposal"]["plan"]["tasks"].is_array();
    let content_width = ui.available_width();
    super::card(ui).show(ui, |ui| {
        ui.set_min_width(content_width - 48.0);
        ui.label(
            RichText::new(text(&language, "startup.step_3_your_execution_decision"))
                .small()
                .strong()
                .color(ui.visuals().hyperlink_color),
        );
        ui.heading(
            RichText::new(text(&language, "startup.review_the_plan"))
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.label(
            RichText::new(text(
                &language,
                "startup.generate_a_plan_then_review_each_role_scope",
            ))
            .small()
            .color(ui.visuals().weak_text_color()),
        );
        if !has_plan {
            ui.horizontal(|ui| {
                ui.add_enabled(
                    false,
                    egui::Button::new(text(&language, "startup.save_plan_edits")),
                );
            });
        }
        let running = snapshot["job"]["kind"] == "plan" && snapshot["job"]["state"] == "running";
        let now = now_ms();
        if let Some((coordinator, elapsed, output_bytes)) = planning_status(&snapshot, now) {
            if let Some(color) = planning_activity_color(&snapshot, now) {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").strong().size(18.0).color(color))
                        .accessible_name(text(&language, "startup.review_coordinator_activity"));
                    ui.label(crate::ui::messages::interpolate(
                        text(&language, "startup.review_planning_in_progress"),
                        &[("value1", &coordinator)],
                    ));
                });
            }
            ui.label(crate::ui::messages::interpolate(
                text(&language, "startup.review_elapsed"),
                &[("value1", &elapsed.to_string())],
            ));
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
            ui.label(crate::ui::messages::interpolate(
                text(&language, "startup.review_output_bytes"),
                &[("value1", &output_bytes.to_string())],
            ));
        }
        if let Some(proposal) = snapshot
            .get("proposal")
            .filter(|value| value["plan"]["tasks"].is_array())
        {
            Probe::render_snapshot_in_language(proposal, &language, ui);
        }
        if has_plan {
            let digest = snapshot["proposal"]["digest"].as_str().unwrap_or_default();
            ui.monospace(if digest.is_empty() {
                "No reviewed plan"
            } else {
                digest
            });
        } else {
            ui.label(text(&language, "startup.no_reviewed_plan"));
            ui.add_enabled(
                false,
                egui::Button::new(text(&language, "startup.approve_plan_run")),
            );
        }
        if snapshot["job"]["state"] == "failed" {
            ui.colored_label(
                Color32::DARK_RED,
                snapshot["job"]["error"]
                    .as_str()
                    .unwrap_or(text(&language, "startup.review_plan_generation_failed")),
            );
            ui.label(crate::ui::messages::interpolate(
                text(&language, "startup.review_logs"),
                &[(
                    "value1",
                    snapshot["job"]["log"]
                        .as_str()
                        .unwrap_or(text(&language, "startup.unavailable")),
                )],
            ));
        }
        let current = settings_are_current(probe, &snapshot);
        let (edit, task_dirty) = tasks::controls_in_language(
            &snapshot,
            &mut probe.task_draft,
            probe.pending.is_some(),
            &language,
            ui,
        );
        let start = execution::controls_in_language(
            &snapshot,
            current && !task_dirty,
            running,
            probe.pending.is_some(),
            &mut probe.confirming_start,
            &language,
            ui,
        );
        if let Some(tasks::Event::Save(tasks, mode, digest)) = edit {
            probe.save_task_edits(tasks, mode, &digest);
        }
        if let Some(execution::Event::Start(revision, digest)) = start {
            probe.start_plan(&revision, &digest);
        }
        if !current {
            ui.label(text(&language, "startup.review_save_before_generate"));
        }
    });
}

pub(super) fn settings_are_current(probe: &Probe, snapshot: &Value) -> bool {
    snapshot["config"]["name"] == probe.name
        && snapshot["config"]["repository"].as_str() == Some(probe.repository.as_str())
        && snapshot["config"]["goal"] == probe.goal
        && snapshot["config"]["coordinator"] == probe.coordinator
        && snapshot["config"]["workspace_mode"]
            == serde_json::to_value(probe.workspace_mode).unwrap_or_default()
        && snapshot["config"]["profiles"]
            .as_array()
            .is_some_and(|profiles| profiles == &probe.profiles)
        && snapshot["config"]["roles"] == serde_json::to_value(&probe.roles).unwrap_or_default()
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
