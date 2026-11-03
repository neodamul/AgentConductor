use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(snapshot: &Value, language: &str, ui: &mut egui::Ui) {
    let state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let current = match state {
        "prepared" => 0,
        "waiting_approval" => 2,
        "approved" => 3,
        "completed" => 4,
        "failed" => {
            let total = snapshot["report"]["plan"]["tasks"]
                .as_array()
                .map_or(0, Vec::len);
            let completed = snapshot["report"]["tasks"].as_array().map_or(0, |tasks| {
                tasks
                    .iter()
                    .filter(|task| task["status"] == "completed")
                    .count()
            });
            match (total, completed == total) {
                (0, _) => 0,
                (_, true) => 2,
                _ => 1,
            }
        }
        _ => 1,
    };
    let text = |key| crate::ui::messages::in_language(language, key);
    ui.horizontal_wrapped(|ui| {
        for (index, key) in [
            "room.01_plan",
            "room.02_tasks",
            "room.03_verify",
            "room.04_merge",
            "room.05_workspace",
        ]
        .into_iter()
        .enumerate()
        {
            let label = text(key);
            let active = index == current;
            egui::Frame::new()
                .fill(if active {
                    ui.visuals().selection.bg_fill
                } else {
                    ui.visuals().extreme_bg_color
                })
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(10, 5))
                .show(ui, |ui| {
                    let response = ui.label(if active {
                        RichText::new(label).strong()
                    } else {
                        RichText::new(label).weak()
                    });
                    response.accessible_name(if active {
                        format!("{}: {}", text("room.current_stage"), text(key))
                    } else {
                        text(key).to_owned()
                    });
                });
        }
    });
}
