use crate::plan::{Plan, Task, WorkspaceMode};
use eframe::egui::{self, Color32, RichText, Ui};
use serde_json::Value;

mod fields;
use fields::{agent_picker, execution_option_pickers, model_picker, text_field};

pub fn edit_request(
    snapshot: &Value,
    tasks: &[Value],
    workspace_mode: WorkspaceMode,
    confirmed_digest: &str,
) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or("workspace revision is unavailable")?;
    let proposal = &snapshot["proposal"];
    if proposal["phase"] != "ready"
        || confirmed_digest.is_empty()
        || proposal["digest"].as_str() != Some(confirmed_digest)
    {
        return Err("the reviewed plan changed or is no longer ready".into());
    }
    let tasks: Vec<Task> = serde_json::from_value(Value::Array(tasks.to_vec()))
        .map_err(|error| format!("invalid task edit: {error}"))?;
    let plan = Plan {
        goal: proposal["plan"]["goal"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        agents: serde_json::from_value(proposal["plan"]["agents"].clone())
            .map_err(|error| format!("invalid plan agents: {error}"))?,
        tasks: tasks.clone(),
        workspace_mode,
    };
    plan.validate()?;
    serde_json::to_vec(&serde_json::json!({
        "kind":"edit", "revision":revision, "digest":confirmed_digest,
        "tasks":tasks, "workspace_mode":workspace_mode
    }))
    .map_err(|error| error.to_string())
}

pub struct Draft {
    digest: String,
    pub tasks: Vec<Value>,
    pub workspace_mode: WorkspaceMode,
}

impl Draft {
    pub fn from_snapshot(snapshot: &Value) -> Option<Self> {
        let proposal = &snapshot["proposal"];
        Some(Self {
            digest: proposal["digest"].as_str()?.to_owned(),
            tasks: proposal["plan"]["tasks"].as_array()?.clone(),
            workspace_mode: serde_json::from_value(proposal["plan"]["workspace_mode"].clone())
                .unwrap_or_default(),
        })
    }

    pub(super) fn changed(&self, snapshot: &Value) -> bool {
        snapshot["proposal"]["plan"]["tasks"].as_array() != Some(&self.tasks)
            || serde_json::to_value(self.workspace_mode).ok()
                != Some(snapshot["proposal"]["plan"]["workspace_mode"].clone())
    }
}

pub enum Event {
    Save(Vec<Value>, WorkspaceMode, String),
}

pub fn controls(
    snapshot: &Value,
    draft: &mut Option<Draft>,
    pending: bool,
    saved: bool,
    ui: &mut Ui,
) -> (Option<Event>, bool) {
    if snapshot["proposal"]["phase"] != "ready" {
        *draft = None;
        return (None, false);
    }
    if draft
        .as_ref()
        .is_none_or(|value| value.digest != snapshot["proposal"]["digest"])
    {
        *draft = Draft::from_snapshot(snapshot);
    }
    let Some(draft) = draft.as_mut() else {
        return (None, false);
    };
    ui.separator();
    ui.heading("Edit tasks");
    let agents = snapshot["proposal"]["plan"]["agents"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mut remove = None;
    let can_remove = draft.tasks.len() > 1;
    for (index, task) in draft.tasks.iter_mut().enumerate() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(task["id"].as_str().unwrap_or("Task"));
                if can_remove && ui.button("Remove").clicked() {
                    remove = Some(index);
                }
            });
            text_field(ui, "Task ID", task, "id");
            agent_picker(ui, task, &agents);
            model_picker(ui, task, &agents);
            execution_option_pickers(ui, task, &agents);
            text_field(ui, "Instructions", task, "prompt");
            text_field(ui, "Owned paths (one per line)", task, "paths");
            text_field(ui, "Dependencies (comma separated)", task, "depends_on");
            text_field(ui, "Test command (JSON argv)", task, "test_command");
        });
    }
    if let Some(index) = remove {
        draft.tasks.remove(index);
    }
    if draft.tasks.len() < 12 && ui.button("Add task").clicked() {
        let id = format!("new-task-{}", draft.tasks.len() + 1);
        draft.tasks.push(serde_json::json!({
            "id":id, "agent":agents.first().and_then(|a| a["id"].as_str()).unwrap_or_default(),
            "model":null, "prompt":"", "paths":[], "depends_on":[], "test_command":[]
        }));
    }
    let changed = draft.changed(snapshot);
    if saved && !changed {
        ui.label(
            RichText::new(
                "Plan saved. Next: review the updated tasks, then select ‘Review execution and approve’.",
            )
            .color(Color32::from_rgb(16, 106, 78)),
        )
        .accessible_name(
            "Plan saved. Next: review the updated tasks, then select ‘Review execution and approve’.",
        );
    }
    let mut save = false;
    if changed {
        ui.label("Plan changes need review")
            .accessible_name("Plan changes need review");
    }
    let save_button = if changed {
        egui::Button::new(RichText::new("Save changed plan and review again").color(Color32::WHITE))
            .fill(Color32::from_rgb(16, 106, 78))
    } else {
        egui::Button::new("Save edited plan for review")
    };
    if ui.add_enabled(changed && !pending, save_button).clicked() {
        save = true;
    }
    let event = save.then(|| {
        Event::Save(
            draft.tasks.clone(),
            draft.workspace_mode,
            draft.digest.clone(),
        )
    });
    (event, changed)
}
