use super::{
    Probe, execution, now_ms, planning_activity_color, planning_status, profile_from_template,
    profile_tools, show_role_rules, tasks,
};
use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke};
use serde_json::Value;

pub(super) fn show_team(
    probe: &mut Probe,
    editable: bool,
    snapshot: &Value,
    ui: &mut egui::Ui,
) -> Option<profile_tools::Event> {
    let mut event = None;
    Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(RichText::new("TEAM").small().strong());
            let mut selected = None;
            ui.add_enabled_ui(editable && probe.profiles.len() < 16, |ui| {
                ui.menu_button("Add agent", |ui| {
                    if let Some(templates) = snapshot["templates"].as_array() {
                        for template in templates {
                            let id = template["id"].as_str().unwrap_or("agent");
                            let parser = template["parser"].as_str().unwrap_or("custom");
                            if ui.button(format!("{id} · {parser}")).clicked() {
                                selected = Some(template.clone());
                                ui.close();
                            }
                        }
                    }
                });
            });
            if let Some(template) = selected
                && let Some(profile) = profile_from_template(&template, &probe.profiles)
            {
                let id = profile["id"].as_str().unwrap_or_default().to_owned();
                if probe.coordinator.is_empty() {
                    probe.coordinator = id;
                }
                probe.profiles.push(profile);
            }
            for profile in &probe.profiles {
                let id = profile["id"].as_str().unwrap_or("unknown").to_owned();
                let parser = profile["parser"].as_str().unwrap_or("custom");
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{id} · {parser}")).strong());
                    ui.add_enabled_ui(editable, |ui| {
                        ui.radio_value(
                            &mut probe.coordinator,
                            id.clone(),
                            format!("Coordinator: {id}"),
                        );
                    });
                });
            }
            show_role_rules(&mut probe.roles, &probe.profiles, editable, ui);
            event = profile_tools::show(
                &mut probe.profiles,
                &mut probe.coordinator,
                &mut probe.roles,
                snapshot,
                editable,
                &mut probe.profile_editor,
                ui,
            );
        });
    event
}

pub(super) fn show_workspace(probe: &mut Probe, ui: &mut egui::Ui) {
    let editable = editable(probe);
    Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(RichText::new("WORKSPACE").small().strong());
            let label = ui.label("Workspace name");
            ui.add_enabled(
                editable,
                egui::TextEdit::singleline(&mut probe.name).desired_width(ui.available_width()),
            )
            .labelled_by(label.id)
            .accessible_name("Workspace name");
            ui.horizontal(|ui| {
                let label = ui.label("Repository path");
                ui.add_enabled(
                    editable,
                    egui::TextEdit::singleline(&mut probe.repository)
                        .desired_width(ui.available_width().min(420.0)),
                )
                .labelled_by(label.id)
                .accessible_name("Repository path");
                if ui
                    .add_enabled(editable, egui::Button::new("Browse…"))
                    .accessible_name("Browse for project folder")
                    .clicked()
                {
                    probe.pick_folder();
                }
            });
            let label = ui.label("Goal");
            ui.add_enabled(
                editable,
                egui::TextEdit::singleline(&mut probe.goal).desired_width(ui.available_width()),
            )
            .labelled_by(label.id)
            .accessible_name("Goal");
            ui.label("Execution isolation");
            ui.add_enabled_ui(editable, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut probe.workspace_mode,
                        crate::plan::WorkspaceMode::Shared,
                        "Shared folder",
                    );
                    ui.radio_value(
                        &mut probe.workspace_mode,
                        crate::plan::WorkspaceMode::Worktree,
                        "Git worktree",
                    );
                })
            });
            if ui
                .add_enabled(
                    editable && probe.snapshot.is_some(),
                    egui::Button::new("Save workspace"),
                )
                .clicked()
            {
                probe.send_workspace();
            }
        });
}

pub(super) fn editable(probe: &Probe) -> bool {
    probe.pending.is_none()
        && probe
            .snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot["job"]["state"] != "running")
}

pub(super) fn show_review(probe: &mut Probe, ui: &mut egui::Ui) {
    ui.heading("Review the plan");
    let Some(snapshot) = probe.snapshot.clone() else {
        ui.label("Connecting to the Rust workspace controller…");
        return;
    };
    let running = snapshot["job"]["kind"] == "plan" && snapshot["job"]["state"] == "running";
    let now = now_ms();
    if let Some((coordinator, elapsed, output_bytes)) = planning_status(&snapshot, now) {
        if let Some(color) =
            planning_activity_color(&snapshot, now, ui.style().animation_time > 0.0)
        {
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
