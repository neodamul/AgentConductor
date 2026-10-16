use agentconductor::native_ui_probe::appearance::{State, apply, show};
use eframe::egui::{self, accesskit::Role};

#[test]
fn appearance_menu_has_an_accessible_name() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        show(&mut state, ui);
    });
    let buttons = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(buttons.contains(&state.text("startup.settings")));
}

#[test]
fn saved_appearance_selects_matching_egui_visuals() {
    let context = egui::Context::default();
    apply("dark", &context);
    assert!(context.global_style().visuals.dark_mode);
    apply("light", &context);
    assert!(!context.global_style().visuals.dark_mode);
}
