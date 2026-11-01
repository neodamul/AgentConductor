use eframe::egui::{RichText, Ui};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(test)]
#[path = "../../../../tests/support/native_ui_coordinator_board.rs"]
mod tests;

#[derive(Debug, Default, Eq, PartialEq)]
struct Counts {
    total: usize,
    completed: usize,
    running: usize,
    failed: usize,
    pending: usize,
}

struct TaskRow {
    id: String,
    agent: String,
    status: String,
    observed_at_ms: Option<u64>,
    quiet_ms: Option<u64>,
}

struct Board {
    run_state: String,
    counts: Counts,
    rows: Vec<TaskRow>,
}

fn model(snapshot: &Value) -> Board {
    let report = &snapshot["report"];
    let reports = report["tasks"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let plan = report["plan"]["tasks"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut counts = Counts {
        total: plan.len(),
        ..Counts::default()
    };
    for task in reports {
        match task["status"].as_str().unwrap_or_default() {
            "completed" => counts.completed += 1,
            "running" => counts.running += 1,
            "failed" => counts.failed += 1,
            "pending" => counts.pending += 1,
            _ => {}
        }
    }
    let rows = plan
        .iter()
        .map(|planned| {
            let id = planned["id"].as_str().unwrap_or("unknown");
            let status = reports
                .iter()
                .find(|report| report["id"] == id)
                .and_then(|report| report["status"].as_str())
                .unwrap_or_default()
                .to_owned();
            TaskRow {
                id: id.to_owned(),
                agent: planned["agent"].as_str().unwrap_or_default().to_owned(),
                status,
                observed_at_ms: snapshot["activity"][id]["observed_at_ms"].as_u64(),
                quiet_ms: snapshot["activity"][id]["quiet_ms"].as_u64(),
            }
        })
        .collect();
    Board {
        run_state: report["state"].as_str().unwrap_or_default().to_owned(),
        counts,
        rows,
    }
}

fn output_age_ms(task: &TaskRow, now_ms: u64) -> Option<u64> {
    task.observed_at_ms
        .and_then(|observed| now_ms.checked_sub(observed))
        .or(task.quiet_ms)
}

pub(super) fn show(snapshot: &Value, language: &str, ui: &mut Ui) {
    let board = model(snapshot);
    if board
        .rows
        .iter()
        .any(|row| row.status == "running" && row.observed_at_ms.is_some())
    {
        ui.ctx().request_repaint_after(Duration::from_secs(1));
    }
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();
    let text = |key| crate::ui::messages::in_language(language, key);
    let state = if board.run_state.is_empty() {
        text("room.not_reported").to_owned()
    } else {
        crate::native_ui_probe::render::status_label(&board.run_state, language).into_owned()
    };
    let summary = crate::ui::messages::interpolate(
        text("room.board_summary"),
        &[
            ("value1", state.as_ref()),
            ("value2", &board.counts.completed.to_string()),
            ("value3", &board.counts.total.to_string()),
            ("value4", &board.counts.running.to_string()),
            ("value5", &board.counts.failed.to_string()),
            ("value6", &board.counts.pending.to_string()),
        ],
    );
    ui.group(|ui| {
        ui.label(RichText::new(summary.clone()).strong())
            .accessible_name(summary);
        for task in board.rows {
            let status = if task.status.is_empty() {
                text("room.not_reported").to_owned()
            } else {
                crate::native_ui_probe::render::status_label(&task.status, language).into_owned()
            };
            let output_age = output_age_ms(&task, now_ms).map_or_else(
                || text("room.not_reported").to_owned(),
                |millis| {
                    let age = if language == "ko" {
                        format!("{}초", millis / 1000)
                    } else {
                        format!("{} s", millis / 1000)
                    };
                    crate::ui::messages::interpolate(
                        text("room.last_output_age"),
                        &[("value1", &age)],
                    )
                },
            );
            let agent = if task.agent.is_empty() {
                text("room.not_reported").to_owned()
            } else {
                task.agent
            };
            let agent_accessible =
                crate::ui::messages::interpolate(text("room.agent_8739d3"), &[("value1", &agent)]);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&task.id).strong())
                    .accessible_name(&task.id);
                ui.label(agent).accessible_name(agent_accessible);
                let color = crate::native_ui_probe::render::status_color(
                    &task.status,
                    ui.input(|input| input.time),
                );
                ui.colored_label(color, &status).accessible_name(&status);
                ui.label(&output_age).accessible_name(&output_age);
            });
        }
    });
}
