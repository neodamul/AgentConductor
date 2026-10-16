use super::{Pending, Probe, room};
use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, vec2};
use serde_json::Value;

mod execution;
pub mod mcp;
pub mod navigation;
mod planning;
pub mod profile_tools;
pub mod sharing;
pub(super) mod tasks;
pub use execution::execution_ready;
pub use execution::{Event as StartEvent, controls as start_controls, start_request};
pub use planning::{planning_activity_color, planning_status};
pub use tasks::{Draft as TaskDraft, Event as TaskEvent, controls as task_controls};
mod role_rules;
pub use role_rules::show as show_role_rules;
pub fn prepare_request(revision: &str) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"kind":"prepare","revision":revision}))
        .expect("prepare request is serializable")
}

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let background = ui.visuals().panel_fill;
    ui.painter().rect_filled(ui.max_rect(), 0.0, background);
    ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
    Frame::new()
        .fill(background)
        .inner_margin(24.0)
        .show(ui, |ui| {
            probe.show_navigation(ui);
            if let Some(mcp::Event::Save(settings)) = probe.mcp_editor.show(
                probe.snapshot.as_ref(),
                probe.navigation.appearance.language(),
                editable(probe) && probe.room_view.is_none(),
                ui,
            ) {
                probe.save_mcp_settings(&settings);
            }
            if let Some(room_view) = probe.room_view.as_mut() {
                if let Some(event) = room::show(
                    room_view,
                    probe.snapshot.as_ref(),
                    probe.pending.is_some(),
                    probe.error.as_deref(),
                    probe.navigation.appearance.language(),
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
                        room::Event::Briefing => probe.load_briefing(),
                    }
                }
                return;
            }
            ui.horizontal(|ui| {
                for (index, id) in [
                    "startup.workspace_settings_5ebc59",
                    "startup.agent_roles",
                    "startup.review_the_plan",
                ]
                .iter()
                .enumerate()
                {
                    let label = format!("{}. {}", index + 1, probe.navigation.appearance.text(id));
                    ui.selectable_value(&mut probe.workspace_step, index, label.as_str())
                        .accessible_name(label);
                }
            });
            if probe.room_view.is_none()
                && let Some(event) =
                    super::cache::show(&mut probe.cache, ui, probe.pending.is_some())
            {
                match event {
                    super::cache::Event::Preview => probe.preview_cache(),
                    super::cache::Event::Clear => probe.clear_cache(),
                }
            }
            let editable = editable(probe);
            match probe.workspace_step {
                0 => {
                    show_workspace(probe, ui);
                    probe.show_sharing(ui);
                }
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
            if probe.pending.is_some() || probe.starting_run.is_some() {
                ui.spinner();
                let activity = if probe.starting_run.is_some() {
                    "start"
                } else {
                    match probe.pending {
                        Some(Pending::PickFolder) => "folder",
                        Some(Pending::Prepare) => "prepare",
                        Some(Pending::Metadata) => "metadata",
                        Some(Pending::MetadataRefresh) => "metadata_refresh",
                        Some(Pending::Start) => "start",
                        Some(Pending::Refresh)
                            if probe
                                .snapshot
                                .as_ref()
                                .is_some_and(|s| s["job"]["state"] == "running") =>
                        {
                            "plan_refresh"
                        }
                        Some(Pending::Navigation | Pending::NavigationRefresh) => "workspace",
                        Some(Pending::McpSave) => "mcp_save",
                        _ => "save",
                    }
                };
                ui.label(if activity == "mcp_save" {
                    probe.navigation.appearance.text("mcp.saving_mcp_settings")
                } else {
                    pending_message(activity)
                });
            }
            if let Some(error) = &probe.error {
                ui.colored_label(Color32::DARK_RED, error);
            }
        });
}

pub fn pending_message(kind: &str) -> &'static str {
    match kind {
        "folder" => "Choosing project folder…",
        "prepare" => "Starting plan generation…",
        "metadata" => "Queueing agent metadata refresh…",
        "metadata_refresh" => "Refreshing agent metadata…",
        "start" => "Starting agent execution…",
        "plan_refresh" => "Refreshing plan progress…",
        "workspace" => "Updating workspaces…",
        _ => "Saving workspace…",
    }
}

fn show_team(
    probe: &mut Probe,
    editable: bool,
    snapshot: &Value,
    ui: &mut egui::Ui,
) -> Option<profile_tools::Event> {
    let mut event = None;
    card(ui).show(ui, |ui| {
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

fn show_workspace(probe: &mut Probe, ui: &mut egui::Ui) {
    let editable = editable(probe);
    let (section, name, project, browse, goal, isolation, shared, worktree, save) = {
        let locale = &probe.navigation.appearance;
        (
            locale.text("startup.workspace_settings_5ebc59"),
            locale.text("startup.workspace_name"),
            locale.text("startup.project_folder"),
            locale.text("startup.choose_your_project_folder"),
            locale.text("startup.goal"),
            locale.text("startup.execution_isolation"),
            locale.text("startup.shared_folder"),
            locale.text("startup.git_worktree"),
            locale.text("startup.save_workspace"),
        )
    };
    card(ui).show(ui, |ui| {
        ui.label(RichText::new(section).small().strong());
        let label = ui.label(name);
        ui.add_enabled(
            editable,
            egui::TextEdit::singleline(&mut probe.name).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name(name);
        ui.horizontal(|ui| {
            let label = ui.label(project);
            ui.add_enabled(
                editable,
                egui::TextEdit::singleline(&mut probe.repository)
                    .desired_width(ui.available_width().min(420.0)),
            )
            .labelled_by(label.id)
            .accessible_name(project);
            if ui
                .add_enabled(editable, egui::Button::new(browse))
                .accessible_name(browse)
                .clicked()
            {
                probe.pick_folder();
            }
        });
        let label = ui.label(goal);
        ui.add_enabled(
            editable,
            egui::TextEdit::singleline(&mut probe.goal).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name(goal);
        ui.label(isolation).accessible_name(isolation);
        ui.add_enabled_ui(editable, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut probe.workspace_mode,
                    crate::plan::WorkspaceMode::Shared,
                    shared,
                );
                ui.radio_value(
                    &mut probe.workspace_mode,
                    crate::plan::WorkspaceMode::Worktree,
                    worktree,
                );
            })
        });
        if ui
            .add_enabled(
                editable && probe.snapshot.is_some(),
                egui::Button::new(save),
            )
            .accessible_name(save)
            .clicked()
        {
            probe.send_workspace();
        }
    });
}

fn card(ui: &egui::Ui) -> Frame {
    Frame::new()
        .fill(ui.visuals().panel_fill)
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
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

pub fn task_edit_request(
    snapshot: &Value,
    tasks: &[Value],
    workspace_mode: crate::plan::WorkspaceMode,
    digest: &str,
) -> Result<Vec<u8>, String> {
    tasks::edit_request(snapshot, tasks, workspace_mode, digest)
}

impl Probe {
    pub(super) fn save_task_edits(
        &mut self,
        tasks: Vec<Value>,
        mode: crate::plan::WorkspaceMode,
        digest: &str,
    ) {
        let result = self
            .snapshot
            .as_ref()
            .ok_or_else(|| "Workspace proposal is unavailable".to_owned())
            .and_then(|snapshot| task_edit_request(snapshot, &tasks, mode, digest))
            .and_then(|packet| {
                self.worker
                    .as_ref()
                    .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                    .send(&packet)
            });
        match result {
            Ok(()) => {
                self.error = None;
                self.pending = Some(Pending::Edit);
            }
            Err(error) => self.error = Some(error),
        }
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
