use eframe::egui;
use serde_json::Value;

pub(super) fn show(activity: Option<&Value>, language: &str, ui: &mut egui::Ui) {
    let text = |key| crate::ui::messages::in_language(language, key);
    let mut parts = Vec::new();
    if let Some(activity) = activity {
        if let Some(tool) = activity["tool"]["name"].as_str() {
            let elapsed = seconds(activity["tool"]["running_ms"].as_u64());
            parts.push(crate::ui::messages::interpolate(
                text("room.live_tool_activity"),
                &[("value1", tool), ("value2", &elapsed)],
            ));
        }
        if let Some(quiet) = activity["quiet_ms"].as_u64() {
            let age = seconds(Some(quiet));
            parts.push(crate::ui::messages::interpolate(
                text("room.last_output_age"),
                &[("value1", &age)],
            ));
        }
        if let Some(process) = activity["process"].as_str() {
            parts.push(crate::ui::messages::interpolate(
                text("room.process_state"),
                &[("value1", process)],
            ));
        }
    }
    let line = parts.join(" · ");
    let row = ui.add_sized(
        egui::vec2(ui.available_width(), 18.0),
        egui::Label::new(egui::RichText::new(line.clone()).size(12.0)).truncate(),
    );
    if !line.is_empty() {
        row.accessible_name(line);
    }
}

fn seconds(milliseconds: Option<u64>) -> String {
    milliseconds.map_or_else(
        || crate::ui::messages::text("room.time_unavailable").into(),
        |value| format!("{} s", value / 1000),
    )
}
