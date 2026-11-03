#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::workspace::profile_tools::{self, Editor};
use eframe::egui;
use serde_json::json;

#[test]
fn requesting_agent_removal_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    let mut editor = Editor::default();
    let mut profiles = vec![json!({"id":"local","parser":"custom"})];
    let snapshot = json!({"config":{"profiles":profiles}});
    let mut coordinator = "local".to_owned();
    let mut roles = Default::default();
    let mut render = |editor: &mut Editor| {
        let mut below = egui::Rect::NOTHING;
        let mut output = context.run_ui(Default::default(), |ui| {
            profile_tools::show(
                &mut profiles,
                &mut coordinator,
                &mut roles,
                &snapshot,
                true,
                editor,
                ui,
            );
            below = ui.label("Workspace content below agent controls").rect;
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        below
    };
    render(&mut editor);
    let before = render(&mut editor);
    editor.request_remove("local");
    render(&mut editor);
    let after = render(&mut editor);
    assert_eq!(
        before, after,
        "removal confirmation must overlay the workspace"
    );
    assert_eq!(
        profiles.len(),
        1,
        "opening confirmation must not remove the profile"
    );
}

#[test]
fn modal_confirmation_and_cancel_keep_their_removal_semantics() {
    for confirm in [false, true] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut editor = Editor::default();
        editor.request_remove("local");
        let mut profiles = vec![json!({"id":"local","parser":"custom"})];
        let snapshot = json!({"config":{"profiles":profiles}});
        let mut coordinator = "local".to_owned();
        let mut roles = Default::default();
        let label = if confirm {
            "Confirm remove agent local"
        } else {
            "Cancel remove agent local"
        };
        let mut event = None;
        let mut frame = |events: Vec<egui::Event>| {
            let mut output = context.run_ui(
                egui::RawInput {
                    events,
                    ..Default::default()
                },
                |ui| {
                    event = profile_tools::show(
                        &mut profiles,
                        &mut coordinator,
                        &mut roles,
                        &snapshot,
                        true,
                        &mut editor,
                        ui,
                    );
                },
            );
            let bounds = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .find(|(_, node)| node.label() == Some(label))
                .and_then(|(_, node)| node.bounds());
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            bounds
        };
        frame(vec![]);
        let bounds = frame(vec![]).expect("confirmation control must be accessible");
        let position = egui::pos2(
            ((bounds.x0 + bounds.x1) / 2.0) as f32,
            ((bounds.y0 + bounds.y1) / 2.0) as f32,
        );
        frame(vec![
            egui::Event::PointerMoved(position),
            egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
        ]);
        frame(vec![egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }]);
        assert_eq!(profiles.is_empty(), confirm);
        assert_eq!(
            matches!(event, Some(profile_tools::Event::SaveProfiles)),
            confirm
        );
    }
}

#[test]
fn profile_removal_requires_a_confirmation_screen() {
    let mut profiles = vec![serde_json::json!({"id":"claude","parser":"claude"})];
    let snapshot = serde_json::json!({"config":{"profiles":[{"id":"claude"}]}});
    let mut editor = agentconductor::native_ui_probe::workspace::profile_tools::Editor::default();
    editor.request_remove("claude");
    let mut coordinator = "claude".to_owned();
    let mut roles = agentconductor::roles::RoleRules::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut warm_up = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::profile_tools::show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            true,
            &mut editor,
            ui,
        );
    });
    warm_up.textures_delta.set.clear();
    warm_up.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::workspace::profile_tools::show(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &snapshot,
            true,
            &mut editor,
            ui,
        );
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(labels.contains(&"Confirm remove agent claude"));
    assert!(labels.contains(&"Cancel remove agent claude"));
    assert_eq!(profiles.len(), 1);
}
