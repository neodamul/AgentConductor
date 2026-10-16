use super::{Probe, card, permissions::editable};
use eframe::egui::{self, RichText};

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let can_edit = editable(probe);
    let (
        step,
        section,
        description,
        name,
        project,
        browse,
        goal,
        isolation,
        shared,
        worktree,
        save,
    ) = {
        let locale = &probe.navigation.appearance;
        (
            locale.text("startup.step_1_workspace"),
            locale.text("startup.workspace_settings_5ebc59"),
            locale.text("startup.saves_the_workspace_name_goal_coordinator_repository_and"),
            locale.text("startup.workspace_name"),
            locale.text("startup.project_folder"),
            locale.text("startup.choose_your_project_folder"),
            locale.text("startup.goal"),
            locale.text("startup.execution_isolation"),
            locale.text("startup.shared_folder"),
            locale.text("startup.git_worktree"),
            locale.text("startup.save_workspace"),
        )
    };
    card(ui).show(ui, |ui| {
        ui.label(
            RichText::new(step)
                .small()
                .strong()
                .color(ui.visuals().hyperlink_color),
        );
        ui.label(
            RichText::new(section)
                .size(22.0)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.label(RichText::new(description).color(ui.visuals().weak_text_color()));
        ui.add_space(8.0);
        let label = ui.label(name);
        ui.add_enabled(
            can_edit,
            egui::TextEdit::singleline(&mut probe.name).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name(name);
        ui.horizontal(|ui| {
            let label = ui.label(project);
            ui.add_enabled(
                can_edit,
                egui::TextEdit::singleline(&mut probe.repository)
                    .desired_width(ui.available_width().min(420.0)),
            )
            .labelled_by(label.id)
            .accessible_name(project);
            if ui
                .add_enabled(can_edit, egui::Button::new(browse))
                .accessible_name(browse)
                .clicked()
            {
                probe.pick_folder();
            }
        });
        let label = ui.label(goal);
        ui.add_enabled(
            can_edit,
            egui::TextEdit::singleline(&mut probe.goal).desired_width(ui.available_width()),
        )
        .labelled_by(label.id)
        .accessible_name(goal);
        match probe.mcp_sources.show(
            &mut probe.goal,
            probe.snapshot.as_ref(),
            probe.navigation.appearance.language(),
            can_edit,
            ui,
        ) {
            Some(super::mcp::SourceEvent::Preview) => probe.preview_mcp_sources(),
            Some(super::mcp::SourceEvent::Apply(selected)) => probe.apply_mcp_sources(&selected),
            None => {}
        }
        ui.label(isolation).accessible_name(isolation);
        ui.add_enabled_ui(can_edit, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut probe.workspace_mode,
                    crate::plan::WorkspaceMode::Shared,
                    shared,
                );
                ui.radio_value(
                    &mut probe.workspace_mode,
                    crate::plan::WorkspaceMode::Worktree,
                    worktree,
                );
            })
        });
        if ui
            .add_enabled(
                can_edit && probe.snapshot.is_some(),
                egui::Button::new(save),
            )
            .accessible_name(save)
            .clicked()
        {
            probe.send_workspace();
        }
    });
}
