use eframe::egui::{self, Color32, RichText};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    Verify,
    Merge,
    Workspace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Progress {
    pub stage: Stage,
    pub merge_running: bool,
}

pub fn progress(snapshot: &Value) -> Progress {
    let state = snapshot["report"]["state"].as_str().unwrap_or_default();
    let merge_running = snapshot["job"]["kind"] == "merge" && snapshot["job"]["state"] == "running";
    let stage = if state == "completed" {
        Stage::Workspace
    } else if state == "approved" || merge_running {
        Stage::Merge
    } else {
        Stage::Verify
    };
    Progress {
        stage,
        merge_running,
    }
}

pub fn can_start_merge(snapshot: &Value, current_evidence: bool, pending: bool) -> bool {
    !pending
        && current_evidence
        && snapshot["report"]["state"] == "approved"
        && !progress(snapshot).merge_running
}

pub fn show(snapshot: &Value, ui: &mut egui::Ui) {
    let progress = progress(snapshot);
    ui.horizontal_wrapped(|ui| {
        stage_label(ui, "room.03_verify", Stage::Verify, progress.stage);
        separator(ui);
        stage_label(ui, "room.04_merge", Stage::Merge, progress.stage);
        separator(ui);
        stage_label(ui, "room.05_workspace", Stage::Workspace, progress.stage);
    });
    if progress.merge_running {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(crate::ui::messages::text("room.merge_in_progress"));
        });
        ui.label(crate::ui::messages::text(
            "room.applying_approved_candidate_to_workspace",
        ));
    }
}

fn separator(ui: &mut egui::Ui) {
    ui.label(RichText::new("›").color(Color32::GRAY));
}

fn stage_label(ui: &mut egui::Ui, id: &'static str, stage: Stage, current: Stage) {
    let label = crate::ui::messages::text(id);
    let visuals = ui.visuals();
    let text = if stage == current {
        RichText::new(label)
            .strong()
            .color(visuals.strong_text_color())
    } else {
        RichText::new(label).color(visuals.weak_text_color())
    };
    ui.label(text);
}
