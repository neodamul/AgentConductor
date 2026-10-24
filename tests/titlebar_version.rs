#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::workspace::navigation::{
    NavigationContext, State, show_in_language,
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
fn the_shared_titlebar_shows_the_executing_build_version() {
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
                planning_running: false,
                dirty: false,
                show_cache: false,
                language: "en",
            },
            ui,
        );
    });
    assert!(workspace.contains(&version), "{workspace}");
    // The Agent Room window uses the same titlebar, so the room view itself must not repeat the
    // version (see `native_ui_room_header`).
}
