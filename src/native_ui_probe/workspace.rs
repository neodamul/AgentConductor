use super::{Pending, Probe, room};
use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke, vec2};
use serde_json::Value;

pub mod profile_tools;
mod role_rules;
pub use role_rules::show as show_role_rules;

pub fn prepare_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"kind":"prepare","revision":revision}))
        .expect("prepare request is serializable")
}

pub fn planning_status(snapshot: &Value, now_ms: u64) -> Option<(String, u64, u64)> {
    let job = &snapshot["job"];
    if job["kind"] != "plan" || job["state"] != "running" {
        return None;
    }
    let started = job["started_ms"].as_u64()?;
    Some((
        snapshot["config"]["coordinator"]
            .as_str()
            .unwrap_or("unknown")
            .to_owned(),
        now_ms.saturating_sub(started) / 1000,
        job["output"]["bytes"].as_u64().unwrap_or_default(),
    ))
}

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let background = Color32::from_rgb(242, 242, 246);
    ui.painter().rect_filled(ui.max_rect(), 0.0, background);
    ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
    Frame::new()
        .fill(background)
        .inner_margin(24.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("AC")
                        .strong()
                        .color(Color32::WHITE)
                        .background_color(Color32::from_rgb(10, 102, 216)),
                );
                ui.heading("AgentConductor");
                ui.label(
                    RichText::new("NATIVE UI PROBE")
                        .small()
                        .color(Color32::from_rgb(96, 96, 103)),
                );
            });
            if let Some(room_view) = probe.room_view.as_mut() {
                if let Some(event) = room::show(
                    room_view,
                    probe.snapshot.as_ref(),
                    probe.pending.is_some(),
                    probe.error.as_deref(),
                    ui,
                ) {
                    match event {
                        room::Event::Refresh => probe.refresh_room(),
                        room::Event::Stop(reason) => probe.stop_room(reason),
                        room::Event::Evidence => probe.load_evidence(),
                        room::Event::Decision(decision, reason) => {
                            probe.decide_room(decision, reason)
                        }
                        room::Event::Resume => probe.resume_room(),
                        room::Event::Recover => probe.recover_room(),
                        room::Event::ToolDecision(action, id) => probe.decide_tool(action, id),
                    }
                }
                return;
            }
            ui.horizontal(|ui| {
                for (index, label) in [
                    "1. Workspace settings",
                    "2. Agent roles",
                    "3. Review the plan",
                ]
                .iter()
                .enumerate()
                {
                    ui.selectable_value(&mut probe.workspace_step, index, *label)
                        .accessible_name(*label);
                }
            });
            let editable = editable(probe);
            match probe.workspace_step {
                0 => show_workspace(probe, ui),
                1 => {
                    let snapshot = probe.snapshot.clone();
                    let event = snapshot
                        .as_ref()
                        .and_then(|snapshot| show_team(probe, editable, snapshot, ui));
                    if let Some(event) = event {
                        match event {
                            profile_tools::Event::Discover(_) | profile_tools::Event::Quota(_) => {
                                probe.request_metadata(event)
                            }
                            profile_tools::Event::SaveProfiles => probe.send_profile_save(),
                            profile_tools::Event::BrowseExecutable => probe.pick_executable(),
                            profile_tools::Event::ResolveExecutable(path) => {
                                probe.resolve_executable(path)
                            }
                        }
                    }
                    if ui
                        .add_enabled(editable, egui::Button::new("Save agent roles"))
                        .clicked()
                    {
                        probe.send_workspace();
                    }
                }
                _ => show_review(probe, ui),
            }
            if probe.pending.is_some() {
                ui.spinner();
                ui.label(match probe.pending {
                    Some(Pending::PickFolder) => "Choosing project folder…",
                    Some(Pending::Prepare) => "Starting plan generation…",
                    Some(Pending::Metadata) => "Queueing agent metadata refresh…",
                    Some(Pending::MetadataRefresh) => "Refreshing agent metadata…",
                    Some(Pending::Refresh)
                        if probe
                            .snapshot
                            .as_ref()
                            .is_some_and(|s| s["job"]["state"] == "running") =>
                    {
                        "Refreshing plan progress…"
                    }
                    _ => "Saving workspace…",
                });
            }
            if let Some(error) = &probe.error {
                ui.colored_label(Color32::DARK_RED, error);
            }
        });
}

fn show_team(
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
                snapshot,
                editable,
                &mut probe.profile_editor,
                ui,
            );
        });
    event
}

fn show_workspace(probe: &mut Probe, ui: &mut egui::Ui) {
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

fn editable(probe: &Probe) -> bool {
    probe.pending.is_none()
        && probe
            .snapshot
            .as_ref()
            .is_none_or(|snapshot| snapshot["job"]["state"] != "running")
}

fn show_review(probe: &mut Probe, ui: &mut egui::Ui) {
    ui.heading("Review the plan");
    let Some(snapshot) = probe.snapshot.as_ref() else {
        ui.label("Connecting to the Rust workspace controller…");
        return;
    };
    let running = snapshot["job"]["kind"] == "plan" && snapshot["job"]["state"] == "running";
    if let Some((coordinator, elapsed, output_bytes)) = planning_status(snapshot, now_ms()) {
        ui.label(format!("Planning in progress · coordinator {coordinator}"));
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
    ui.add_enabled_ui(current && !running && probe.pending.is_none(), |ui| {
        if ui
            .button("Generate plan")
            .accessible_name("Generate plan")
            .clicked()
        {
            probe.prepare_plan();
        }
    });
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

pub fn workspace_save_request(
    snapshot: &Value,
    name: &str,
    repository: &str,
    goal: &str,
    workspace_mode: crate::plan::WorkspaceMode,
    profiles: &[Value],
    coordinator: &str,
) -> Vec<u8> {
    let mut config = snapshot["config"].clone();
    config["name"] = name.into();
    config["repository"] = repository.into();
    config["goal"] = goal.into();
    config["workspace_mode"] = serde_json::to_value(workspace_mode).unwrap();
    config["profiles"] = serde_json::to_value(profiles).unwrap();
    config["coordinator"] = coordinator.into();
    serde_json::to_vec(&serde_json::json!({
        "kind": "save",
        "revision": snapshot["revision"],
        "config": config
    }))
    .expect("workspace save request is serializable")
}

pub fn profile_from_template(template: &Value, profiles: &[Value]) -> Option<Value> {
    if profiles.len() >= 16 {
        return None;
    }
    let base = template["id"].as_str()?;
    let mut id = base.to_owned();
    let mut suffix = 2;
    while profiles.iter().any(|profile| profile["id"] == id) {
        id = format!("{base}-{suffix}");
        suffix += 1;
    }
    let mut profile = template.clone();
    profile["id"] = id.into();
    Some(profile)
}

pub fn picked_folder(current: &str, response: &Value) -> String {
    response["picked_folder"]
        .as_str()
        .unwrap_or(current)
        .to_owned()
}
