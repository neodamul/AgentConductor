use super::review;
use crate::native_ui_probe::Probe;
use eframe::egui::{self, Ui};
use serde_json::Value;

pub(super) fn show_role_actions(
    probe: &mut Probe,
    snapshot: Option<&Value>,
    editable: bool,
    ui: &mut Ui,
) {
    let save_roles = probe.navigation.appearance.text("startup.save_agent_roles");
    let generate_plan = probe.navigation.appearance.text("startup.generate_plan");
    let current = snapshot.is_some_and(|snapshot| review::settings_are_current(probe, snapshot));
    let planning = snapshot.is_some_and(|snapshot| {
        snapshot["job"]["kind"] == "plan" && snapshot["job"]["state"] == "running"
    });
    let snapshot = snapshot.unwrap_or(&Value::Null);
    let can_generate = editable
        && current
        && !planning
        && probe.pending.is_none()
        && super::plan_configuration_available(snapshot);

    ui.horizontal(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if super::super::appearance::primary_button(ui, can_generate, generate_plan).clicked() {
                probe.workspace_step = 2;
                probe.prepare_plan();
            }
            if ui
                .add_enabled(editable, egui::Button::new(save_roles))
                .accessible_name(save_roles)
                .clicked()
            {
                probe.send_workspace();
            }
        });
        if !current {
            ui.label(
                probe
                    .navigation
                    .appearance
                    .text("startup.review_save_before_generate"),
            );
        } else if probe.goal.trim().is_empty() {
            ui.label(
                probe
                    .navigation
                    .appearance
                    .text("native.enter_a_nonempty_goal"),
            );
        } else if !super::plan_configuration_available(snapshot) {
            ui.label(
                probe
                    .navigation
                    .appearance
                    .text("native.coordinator_not_registered"),
            );
        }
    });
}
