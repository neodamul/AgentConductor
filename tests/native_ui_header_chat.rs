#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{
    chat,
    navigation::{self, NavigationContext},
};
use eframe::egui::{self, accesskit::Role};

#[test]
fn application_actions_match_the_legacy_header_order() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut navigation = navigation::State::default();
    let mut chat = chat::State::default();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1280.0, 120.0),
            )),
            ..Default::default()
        },
        |ui| {
            navigation::show_in_language_with_chat(
                &mut navigation,
                NavigationContext {
                    current_name: "Workspace",
                    current_path: "/repo",
                    can_change: true,
                    planning_running: false,
                    dirty: true,
                    show_cache: false,
                    language: "en",
                },
                Some(&mut chat),
                ui,
            );
        },
    );
    let tree = output.platform_output.accesskit_update.as_ref().unwrap();
    let bounds = |label: &str| {
        tree.nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button && node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("missing app-bar button {label:?}"))
    };
    let chat = bounds("Ask coordinator");
    let settings = tree
        .nodes
        .iter()
        .find(|(_, node)| {
            node.role() == Role::Button
                && node
                    .label()
                    .is_some_and(|label| ["Settings", "설정"].contains(&label))
        })
        .and_then(|(_, node)| node.bounds())
        .expect("Settings button bounds");
    let discard = bounds("Discard changes");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        (chat.y0 - settings.y0).abs() < 2.0,
        "chat must share the app bar row"
    );
    assert!(
        chat.x0 < settings.x0,
        "chat must appear immediately before Settings"
    );
    assert!(
        settings.x0 < discard.x0,
        "Discard changes must follow the chat and Settings actions"
    );
    assert!(
        chat.y1 <= 50.0,
        "chat must not be pushed into page content: {chat:?}"
    );
}
