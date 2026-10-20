use eframe::egui;
use serde_json::Value;

pub(super) fn show(activity: Option<&Value>, language: &str, ui: &mut egui::Ui) {
    let Some(activity) = activity else { return };
    let text = |key| crate::ui::messages::in_language(language, key);
    if let Some(tool) = activity["tool"]["name"].as_str() {
        let elapsed = seconds(activity["tool"]["running_ms"].as_u64());
        let label = crate::ui::messages::interpolate(
            text("room.live_tool_activity"),
            &[("value1", tool), ("value2", &elapsed)],
        );
        ui.label(&label).accessible_name(label);
    }
    if let Some(quiet) = activity["quiet_ms"].as_u64() {
        let age = seconds(Some(quiet));
        let label =
            crate::ui::messages::interpolate(text("room.last_output_age"), &[("value1", &age)]);
        ui.label(&label).accessible_name(label);
    }
    if let Some(process) = activity["process"].as_str() {
        let label =
            crate::ui::messages::interpolate(text("room.process_state"), &[("value1", process)]);
        ui.label(&label).accessible_name(label);
    }
}

fn seconds(milliseconds: Option<u64>) -> String {
    milliseconds.map_or_else(
        || crate::ui::messages::text("room.time_unavailable").into(),
        |value| format!("{} s", value / 1000),
    )
}
