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
    let report = snapshot["report"].as_object().map(|_| &snapshot["report"]);
    let proposal_plan = &snapshot["proposal"]["plan"];
    let plan = report
        .map(|report| &report["plan"])
        .filter(|plan| plan["tasks"].is_array())
        .or_else(|| snapshot.get("plan").filter(|plan| plan["tasks"].is_array()))
        .unwrap_or(proposal_plan);
    let goal = snapshot["config"]["goal"]
        .as_str()
        .filter(|goal| !goal.is_empty())
        .or_else(|| plan["goal"].as_str());
    if let Some(goal) = goal {
        ui.label(RichText::new(goal).strong());
    }
    if let Some(state) = report
        .and_then(|report| report["state"].as_str())
        .or_else(|| snapshot["state"].as_str())
        .or_else(|| snapshot["run_state"].as_str())
    {
        ui.label(format!("Run · {state}"));
    }
    if let Some(tasks) = plan["tasks"].as_array() {
        let now = ui.input(|input| input.time);
        let task_reports = report
            .map(|report| &report["tasks"])
            .unwrap_or(&snapshot["tasks"]);
        let running = task_reports
            .as_array()
            .is_some_and(|reports| reports.iter().any(|report| report["status"] == "running"));
        if running {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(500));
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for task in tasks {
                let id = task["id"].as_str().unwrap_or("unknown");
                let status = task_reports
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
