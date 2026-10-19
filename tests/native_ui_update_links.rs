#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::updates::State;
use eframe::egui::{self, accesskit::Role};

#[test]
fn clicking_the_release_notes_link_opens_its_page() {
    let page = "https://github.com/neodamul/AgentConductor/releases/tag/v0.1.25";
    let mut state = State::default();
    state.open = true;
    state.apply_status(serde_json::json!({"phase":"idle","page":page}));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show("en", false, ui);
        },
    );
    let point = first
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::Link)
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            egui::Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .expect("release notes link should be visible and accessible");
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();

    let mut output = None;
    for pressed in [true, false] {
        output = Some(context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 600.0),
                )),
                events: vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| {
                state.show("en", false, ui);
            },
        ));
    }
    let commands = output.unwrap().platform_output.commands;
    assert!(commands.iter().any(|command| matches!(
        command,
        egui::OutputCommand::OpenUrl(open) if open.url == page
    )));
    assert!(!commands.iter().any(|command| matches!(
        command,
        egui::OutputCommand::CopyText(text) if text == page
    )));
}

#[test]
fn untrusted_release_urls_do_not_create_browser_links() {
    let mut state = State::default();
    state.open = true;
    state.apply_status(serde_json::json!({
        "phase":"idle",
        "page":"https://example.com/collect?data=release-notes"
    }));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        state.show("en", false, ui);
    });
    let has_link = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.role() == Role::Link);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(!has_link, "untrusted release URLs must not open a browser");
}
