use agentconductor::native_ui_probe::workspace::navigation::{
    NavigationContext, State, show_in_language,
};

pub fn show_navigation(state: &mut State, show_cache: bool, ui: &mut eframe::egui::Ui) {
    let _ = show_in_language(
        state,
        NavigationContext {
            current_name: "Current",
            current_path: "/work/current",
            can_change: true,
            dirty: false,
            show_cache,
            language: "en",
        },
        ui,
    );
}
