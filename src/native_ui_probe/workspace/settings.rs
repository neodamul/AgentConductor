use super::{Probe, editable};
use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke};

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let editable = editable(probe);
    Frame::new()
        .fill(Color32::WHITE)
        .stroke(Stroke::new(1.0, Color32::from_rgb(220, 220, 226)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.label(RichText::new("WORKSPACE").small().strong());
            let label = ui.label("Workspace name");
            ui.add_enabled(
                editable,
                egui::TextEdit::singleline(&mut probe.name).desired_width(ui.available_width()),
            )
            .labelled_by(label.id)
            .accessible_name("Workspace name");
            ui.horizontal(|ui| {
                let label = ui.label("Repository path");
                ui.add_enabled(
                    editable,
                    egui::TextEdit::singleline(&mut probe.repository)
                        .desired_width(ui.available_width().min(420.0)),
                )
                .labelled_by(label.id)
                .accessible_name("Repository path");
                if ui
                    .add_enabled(editable, egui::Button::new("Browse…"))
                    .accessible_name("Browse for project folder")
                    .clicked()
                {
                    probe.pick_folder();
                }
            });
            let label = ui.label("Goal");
            ui.add_enabled(
                editable,
                egui::TextEdit::singleline(&mut probe.goal).desired_width(ui.available_width()),
            )
            .labelled_by(label.id)
            .accessible_name("Goal");
            ui.label("Execution isolation");
            ui.add_enabled_ui(editable, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut probe.workspace_mode,
                        crate::plan::WorkspaceMode::Shared,
                        "Shared folder",
                    );
                    ui.radio_value(
                        &mut probe.workspace_mode,
                        crate::plan::WorkspaceMode::Worktree,
                        "Git worktree",
                    );
                })
            });
            if ui
                .add_enabled(
                    editable && probe.snapshot.is_some(),
                    egui::Button::new("Save workspace"),
                )
                .clicked()
            {
                probe.send_workspace();
            }
        });
}
