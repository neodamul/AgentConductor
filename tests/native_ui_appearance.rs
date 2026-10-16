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
    assert_eq!(
        context.global_style().visuals.panel_fill,
        egui::Color32::from_rgb(20, 26, 25)
    );
    assert_eq!(
        context.global_style().visuals.window_fill,
        egui::Color32::from_rgb(26, 34, 32)
    );
    assert_eq!(
        context.global_style().visuals.selection.bg_fill,
        egui::Color32::from_rgb(35, 56, 46)
    );
    assert_eq!(
        context.global_style().visuals.widgets.active.bg_fill,
        egui::Color32::from_rgb(140, 201, 168)
    );
    assert_eq!(
        context
            .global_style()
            .visuals
            .widgets
            .active
            .fg_stroke
            .color,
        egui::Color32::from_rgb(15, 26, 21)
    );
    apply("light", &context);
    assert!(!context.global_style().visuals.dark_mode);
    assert_eq!(
        context.global_style().visuals.panel_fill,
        egui::Color32::from_rgb(242, 242, 246)
    );
    assert_eq!(
        context.global_style().visuals.window_fill,
        egui::Color32::WHITE
    );
    assert_eq!(
        context.global_style().visuals.widgets.active.bg_fill,
        egui::Color32::from_rgb(10, 102, 216)
    );
    assert_eq!(
        context
            .global_style()
            .visuals
            .widgets
            .active
            .fg_stroke
            .color,
        egui::Color32::WHITE
    );
}
