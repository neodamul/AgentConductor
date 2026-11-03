#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::Editor;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};

struct Harness {
    context: egui::Context,
    input: egui::RawInput,
    profiles: Vec<serde_json::Value>,
    coordinator: String,
    roles: agentconductor::roles::RoleRules,
    editor: Editor,
    snapshot: serde_json::Value,
}

impl Harness {
    fn frame(&mut self, events: Vec<Event>) -> egui::FullOutput {
        let mut input = self.input.clone();
        input.events = events;
        self.context.run_ui(input, |ui| {
            agentconductor::native_ui_probe::workspace::profile_tools::show(
                &mut self.profiles,
                &mut self.coordinator,
                &mut self.roles,
                &self.snapshot,
                true,
                &mut self.editor,
                ui,
            );
        })
    }
}

#[test]
fn custom_profile_editor_exposes_coordinator_model_and_read_only_arguments() {
    let profile = serde_json::json!({
        "id":"aipro", "parser":"custom", "executable":"aipro",
        "args":["run", "--model", "{model}", "{task_file}"],
        "read_only_args":["chat", "--read-only", "{model}", "{task_file}"],
        "coordinator_model":"local/one", "input_mode":"stdin",
        "permission_mode":"operator_auto_approve", "max_sessions":1,
        "quota_refresh_minutes":5,
        "catalog":{"models":["local/one"],"source":"aipro models"}
    });
    let mut editor = Editor::default();
    editor.open(&profile);
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut harness = Harness {
        context,
        input: egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1000.0, 900.0),
            )),
            ..Default::default()
        },
        profiles: vec![profile.clone()],
        coordinator: String::new(),
        roles: Default::default(),
        editor,
        snapshot: serde_json::json!({"config":{"profiles":[profile]}}),
    };
    clear(&mut harness.frame(Vec::new()));
    let mut output = harness.frame(Vec::new());
    let menu = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button && node.label() == Some("Coordinator and agent settings")
        })
        .map(|(_, node)| node.bounds().unwrap())
        .unwrap();
    clear(&mut output);
    let point = Pos2::new(
        ((menu.x0 + menu.x1) / 2.0) as f32,
        ((menu.y0 + menu.y1) / 2.0) as f32,
    );
    clear(&mut harness.frame(vec![Event::PointerMoved(point)]));
    clear(&mut harness.frame(vec![Event::PointerMoved(point), pointer(point, true)]));
    clear(&mut harness.frame(vec![pointer(point, false)]));
    let mut output = harness.frame(Vec::new());
    let update = output.platform_output.accesskit_update.take().unwrap();
    let labels = update
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let model_visible = labels.contains(&"Coordinator model");
    let arguments_visible = labels.contains(&"Read-only arguments · JSON array");
    clear(&mut output);
    assert!(model_visible);
    assert!(arguments_visible);
}

fn pointer(point: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
