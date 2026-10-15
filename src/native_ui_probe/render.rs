use eframe::egui::{self, Color32, RichText};
use serde_json::Value;

pub fn status_color(status: &str, time: f64) -> Color32 {
    match status {
        "waiting_approval" | "blocked" => Color32::from_rgb(218, 148, 34),
        "failed" => Color32::from_rgb(205, 55, 55),
        "running" => {
            let pulse = if (time * 2.0).floor().rem_euclid(2.0) == 0.0 {
                0.65
            } else {
                1.0
            };
            Color32::from_rgb(
                (45.0 * pulse) as u8,
                (165.0 * pulse) as u8,
                (95.0 * pulse) as u8,
            )
        }
        "completed" => Color32::from_rgb(45, 165, 95),
        _ => Color32::from_rgb(125, 130, 140),
    }
}

pub fn snapshot(snapshot: &Value, ui: &mut egui::Ui) {
    if let Some(root) = snapshot["root"].as_str() {
        ui.label(root);
    }
    let goal = snapshot["config"]["goal"]
        .as_str()
        .filter(|goal| !goal.is_empty())
        .or_else(|| snapshot["plan"]["goal"].as_str());
    if let Some(goal) = goal {
        ui.label(RichText::new(goal).strong());
    }
    if let Some(state) = snapshot["state"].as_str() {
        ui.label(format!("Run · {state}"));
    }
    if let Some(tasks) = snapshot["plan"]["tasks"].as_array() {
        let now = ui.input(|input| input.time);
        let running = snapshot["tasks"]
            .as_array()
            .is_some_and(|reports| reports.iter().any(|report| report["status"] == "running"));
        if running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for task in tasks {
                let id = task["id"].as_str().unwrap_or("unknown");
                let status = snapshot["tasks"]
                    .as_array()
                    .and_then(|reports| reports.iter().find(|report| report["id"] == id))
                    .and_then(|report| report["status"].as_str())
                    .unwrap_or("pending");
                let agent = task["agent"].as_str().unwrap_or("unassigned");
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(status)
                            .strong()
                            .color(status_color(status, now)),
                    );
                    ui.label(id);
                    ui.label(format!("· {agent}"));
                });
            }
        });
    }
}
