use super::{execution, now_ms, planning_activity_color, planning_status, tasks};
use eframe::egui::{self, Color32, RichText};
use serde_json::Value;

type Probe = super::super::Probe;
type Pending = super::super::Pending;

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    ui.heading("Review the plan");
    let Some(snapshot) = probe.snapshot.clone() else {
        ui.label("Connecting to the Rust workspace controller…");
        return;
    };
    let running = snapshot["job"]["kind"] == "plan" && snapshot["job"]["state"] == "running";
    let preparing = matches!(
        probe.pending,
        Some(Pending::Prepare | Pending::PrepareRefresh)
    );
    if preparing {
        ui.label(crate::ui::messages::text("startup.review_plan_preparing"));
    }
    let now = now_ms();
    if let Some((coordinator, elapsed, output_bytes)) = planning_status(&snapshot, now) {
        if let Some(color) = planning_activity_color(&snapshot, now) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("●").strong().size(18.0).color(color))
                    .accessible_name("Coordinator activity: running");
                ui.label(format!("Planning in progress · coordinator {coordinator}"));
            });
        }
        ui.label(format!("Elapsed · {elapsed}s"));
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(500));
        ui.label(format!("Coordinator output · {output_bytes} bytes"));
    }
    if let Some(proposal) = snapshot
        .get("proposal")
        .filter(|value| value["plan"]["tasks"].is_array())
    {
        Probe::render_snapshot(proposal, ui);
    }
    if plan_failure_is_current(&snapshot, preparing, probe.error.is_some()) {
        ui.colored_label(
            Color32::DARK_RED,
            snapshot["job"]["error"]
                .as_str()
                .unwrap_or("Plan generation failed"),
        );
        ui.label(format!(
            "Logs · {}",
            snapshot["job"]["log"].as_str().unwrap_or("unavailable")
        ));
    }
    let current = snapshot["config"]["name"] == probe.name
        && snapshot["config"]["repository"].as_str() == Some(probe.repository.as_str())
        && snapshot["config"]["goal"] == probe.goal
        && snapshot["config"]["coordinator"] == probe.coordinator
        && snapshot["config"]["workspace_mode"]
            == serde_json::to_value(probe.workspace_mode).unwrap_or_default()
        && snapshot["config"]["profiles"]
            .as_array()
            .is_some_and(|profiles| profiles == &probe.profiles);
    let (edit, task_dirty) = tasks::controls(
        &snapshot,
        &mut probe.task_draft,
        probe.pending.is_some(),
        probe.task_saved_notice && current,
        ui,
    );
    if task_dirty {
        probe.task_saved_notice = false;
    }
    let mut prepare = false;
    ui.add_enabled_ui(
        current && !running && !task_dirty && probe.pending.is_none(),
        |ui| {
            if ui
                .button("Generate plan")
                .accessible_name("Generate plan")
                .clicked()
            {
                prepare = true;
            }
        },
    );
    let retry = can_retry_plan(&snapshot, current, task_dirty, probe.pending.is_some())
        && ui
            .button(crate::ui::messages::text("startup.retry_plan_generation"))
            .clicked();
    let start = execution::controls(
        &snapshot,
        current && !task_dirty,
        running,
        probe.pending.is_some(),
        &mut probe.confirming_start,
        ui,
    );
    if prepare || retry {
        probe.prepare_plan();
    }
    if let Some(tasks::Event::Save(tasks, mode, digest)) = edit {
        probe.save_task_edits(tasks, mode, &digest);
    }
    if let Some(execution::Event::Start(revision, digest)) = start {
        probe.start_plan(&revision, &digest);
    }
    if !current {
        ui.label("Save Workspace settings and Agent roles before generating the plan.");
    }
}

pub fn plan_failure_is_current(snapshot: &Value, preparing: bool, has_error: bool) -> bool {
    snapshot["job"]["kind"] == "plan"
        && snapshot["job"]["state"] == "failed"
        && !preparing
        && !has_error
}

pub fn can_retry_plan(snapshot: &Value, current: bool, task_dirty: bool, pending: bool) -> bool {
    snapshot["job"]["kind"] == "plan"
        && snapshot["job"]["state"] == "failed"
        && current
        && !task_dirty
        && !pending
}
