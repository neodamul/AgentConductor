#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::{
    room,
    workspace::navigation::{NavigationContext, State, show_in_language},
};
use eframe::egui;

fn visible(context: &egui::Context, render: impl FnMut(&mut egui::Ui)) -> String {
    let mut output = context.run_ui(Default::default(), render);
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    text
}

#[test]
fn workspace_and_agent_room_show_the_executing_build_version() {
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let context = egui::Context::default();
    let mut state = State::default();
    let workspace = visible(&context, |ui| {
        show_in_language(
            &mut state,
            NavigationContext {
                current_name: "Demo",
                current_path: "/tmp/demo",
                can_change: true,
                dirty: false,
                show_cache: false,
                language: "en",
            },
            ui,
        );
    });
    assert!(workspace.contains(&version), "{workspace}");

    let room_context = egui::Context::default();
    let mut room_state = room::State::default();
    let agent_room = visible(&room_context, |ui| {
        room::show(&mut room_state, None, false, None, "en", ui);
    });
    assert!(agent_room.contains(&version), "{agent_room}");
}
