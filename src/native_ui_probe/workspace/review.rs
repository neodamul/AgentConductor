use super::{Probe, execution, planning_activity_color, planning_status, tasks};
use eframe::egui::{self, Color32, RichText};

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    ui.heading("Review the plan");
    let Some(snapshot) = probe.snapshot.clone() else {
        ui.label("Connecting to the Rust workspace controller…");
        return;
    };
    let running = snapshot["job"]["kind"] == "plan" && snapshot["job"]["state"] == "running";
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
        Probe::render_snapshot_in_language(proposal, probe.navigation.appearance.language(), ui);
    }
    if snapshot["job"]["state"] == "failed" {
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
        ui,
    );
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
    let start = execution::controls(
        &snapshot,
        current && !task_dirty,
        running,
        probe.pending.is_some(),
        &mut probe.confirming_start,
        ui,
    );
    if prepare {
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

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
