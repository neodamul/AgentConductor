use super::{Probe, execution, header, planning_activity_color, planning_status, tasks};
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
    let plan_already_started = snapshot["proposal"]["phase"] == "starting";
    let content_width = ui.available_width();
    super::card(ui).show(ui, |ui| {
        ui.set_min_width(content_width - 48.0);
        header::show_card_eyebrow(2, &language, ui);
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
        if let Some(base) = snapshot["proposal"]["repository_base"].as_str() {
            let base_prefix: String = base.chars().take(12).collect();
            let note = crate::ui::messages::interpolate(
                text(
                    &language,
                    "startup.snapshot_base_review_every_executable_permission_mode_scope",
                ),
                &[("value1", base_prefix.as_str())],
            );
            ui.label(
                RichText::new(&note)
                    .small()
                    .color(ui.visuals().weak_text_color()),
            )
            .accessible_name(note);
        }
        if matches!(
            probe.pending,
            Some(super::super::Pending::Prepare | super::super::Pending::PrepareRefresh)
        ) {
            ui.label(text(&language, "startup.review_plan_preparing"));
        }
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
            if let Some(color) =
                planning_activity_color(&snapshot, now, ui.style().animation_time > 0.0)
            {
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
        if plan_already_started {
            ui.label(text(&language, "startup.review_plan_already_started"));
            let run_available = snapshot["run_available"] == true;
            if ui
                .add_enabled(
                    run_available && probe.pending.is_none(),
                    egui::Button::new(text(&language, "startup.open_agent_room")),
                )
                .clicked()
            {
                let result = probe
                    .worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())
                    .and_then(|worker| worker.send(br#"{"kind":"open_run"}"#));
                match result {
                    Ok(()) => {
                        probe.pending = Some(super::super::Pending::OpenRun);
                        probe.error = None;
                    }
                    Err(error) => probe.error = Some(error),
                }
            }
        } else if let Some(proposal) = snapshot
            .get("proposal")
            .filter(|value| value["plan"]["tasks"].is_array())
        {
            Probe::render_snapshot_in_language(proposal, &language, ui);
        }
        if has_plan && !plan_already_started {
            let digest = snapshot["proposal"]["digest"].as_str().unwrap_or_default();
            ui.monospace(if digest.is_empty() {
                "No reviewed plan"
            } else {
                digest
            });
        } else if !plan_already_started {
            ui.label(text(&language, "startup.no_reviewed_plan"));
            ui.add_enabled(
                false,
                egui::Button::new(text(&language, "startup.approve_plan_run")),
            );
        }
        if plan_failure_is_current(probe, &snapshot) {
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
            probe.task_saved_notice && current,
            &language,
            ui,
        );
        execution::controls_in_language(
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
        if can_retry_plan(probe, &snapshot, current, task_dirty)
            && ui
                .button(text(&language, "startup.retry_plan_generation"))
                .clicked()
        {
            probe.prepare_plan();
        }
        if !current {
            ui.label(text(&language, "startup.review_save_before_generate"));
        }
    });
}

fn plan_failure_is_current(probe: &Probe, snapshot: &Value) -> bool {
    snapshot["job"]["kind"] == "plan"
        && snapshot["job"]["state"] == "failed"
        && probe.error.is_none()
        && !matches!(
            probe.pending,
            Some(super::super::Pending::Prepare | super::super::Pending::PrepareRefresh)
        )
}

fn can_retry_plan(probe: &Probe, snapshot: &Value, current: bool, task_dirty: bool) -> bool {
    snapshot["job"]["kind"] == "plan"
        && snapshot["job"]["state"] == "failed"
        && probe.pending.is_none()
        && current
        && !task_dirty
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
