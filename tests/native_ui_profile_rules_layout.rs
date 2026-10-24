#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{self, Editor};
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::json;

#[test]
fn opening_profile_rules_popup_does_not_push_editor_actions_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let profile = json!({
        "id":"fixture-agent",
        "parser":"custom",
        "executable":"aipro",
        "args":[],
        "input_mode":"stdin",
        "max_sessions":3,
        "quota_refresh_minutes":5
    });
    let mut profiles = vec![profile.clone()];
    let mut coordinator = String::new();
    let mut roles = Default::default();
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.set_extensions("{\"rule\":\"visible in popup\"}");
    let snapshot = json!({"config":{"profiles":[profile]}});
    let menu_label = agentconductor::ui::messages::in_language("en", "startup.profile_rules_json");
    let save_label = agentconductor::ui::messages::in_language("en", "startup.save_profile");
    clear(frame(
        &context,
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        &snapshot,
        Vec::new(),
    ));
    let before = frame(
        &context,
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        &snapshot,
        Vec::new(),
    );
    let nodes = before
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| Some((node.role(), node.label()?.to_owned(), node.bounds()?)))
        .collect::<Vec<_>>();
    let menu = nodes.iter().find_map(|(role, label, bounds)| {
        (*role == Role::Button && label == menu_label).then_some(*bounds)
    });
    let save_before = nodes.iter().find_map(|(role, label, bounds)| {
        (*role == Role::Button && label == save_label).then_some(*bounds)
    });
    clear(before);
    let menu = menu.unwrap_or_else(|| panic!("missing profile-rules menu: {nodes:?}"));
    let save_before = save_before.unwrap_or_else(|| panic!("missing save action: {nodes:?}"));
    let point = Pos2::new(
        ((menu.x0 + menu.x1) / 2.0) as f32,
        ((menu.y0 + menu.y1) / 2.0) as f32,
    );
    clear(frame(
        &context,
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        &snapshot,
        vec![Event::PointerMoved(point)],
    ));
    clear(frame(
        &context,
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        &snapshot,
        vec![Event::PointerMoved(point), pointer(point, true)],
    ));
    clear(frame(
        &context,
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        &snapshot,
        vec![pointer(point, false)],
    ));
    let opened = frame(
        &context,
        &mut profiles,
        &mut coordinator,
        &mut roles,
        &mut editor,
        &snapshot,
        Vec::new(),
    );
    let opened_nodes = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| Some((node.role(), node.label()?.to_owned(), node.bounds()?)))
        .collect::<Vec<_>>();
    let save_after = opened_nodes.iter().find_map(|(role, label, bounds)| {
        (*role == Role::Button && label == save_label).then_some(*bounds)
    });
    let painted = opened
        .shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            egui::Shape::Text(shape) => Some(shape.galley.job.text.as_str().to_owned()),
            _ => None,
        })
        .collect::<Vec<_>>();
    clear(opened);
    let save_after = save_after.unwrap_or_else(|| panic!("missing save action: {opened_nodes:?}"));

    assert!(
        painted.iter().any(|text| text.contains("visible in popup")),
        "the popup should paint the profile rules: {painted:?}"
    );
    assert!(
        (save_after.y0 - save_before.y0).abs() < 1.0,
        "opening profile rules must not move editor actions: before={save_before:?}, after={save_after:?}"
    );
}

fn frame(
    context: &egui::Context,
    profiles: &mut Vec<serde_json::Value>,
    coordinator: &mut String,
    roles: &mut agentconductor::roles::RoleRules,
    editor: &mut Editor,
    snapshot: &serde_json::Value,
    events: Vec<Event>,
) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1000.0, 720.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        profile_tools::show_in_language(
                            profiles,
                            coordinator,
                            roles,
                            editor,
                            profile_tools::RenderContext {
                                snapshot,
                                editable: true,
                                language: "en",
                                show_registry: false,
                            },
                            ui,
                        );
                    });
            });
        },
    )
}

fn pointer(point: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos: point,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
