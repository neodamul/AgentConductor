use super::{Probe, card, permissions::editable};
use eframe::egui::{self, RichText};

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let can_edit = editable(probe);
    let (
        section,
        name,
        name_hint,
        project,
        project_hint,
        browse,
        goal,
        goal_hint,
        isolation,
        shared,
        worktree,
        save,
        coordinator_label,
        coordinator_none,
    ) = {
        let locale = &probe.navigation.appearance;
        (
            locale.text("startup.what_should_the_team_work_on"),
            locale.text("startup.workspace_name"),
            locale.text("startup.workspace_name_hint"),
            locale.text("startup.project_folder"),
            locale.text("startup.project_folder_hint"),
            locale.text("startup.choose_your_project_folder"),
            locale.text("startup.goal"),
            locale.text("startup.goal_hint"),
            locale.text("startup.execution_isolation"),
            locale.text("startup.shared_folder"),
            locale.text("startup.git_worktree"),
            locale.text("startup.save_workspace"),
            locale.text("startup.coordinator_analysis_of_the_whole_goal"),
            locale.text("startup.coordinator_none"),
        )
    };
    card(ui).show(ui, |ui| {
        ui.label(
            RichText::new(section)
                .size(22.0)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.add_space(8.0);
        let label = ui.label(name);
        ui.add_enabled(
            can_edit,
            egui::TextEdit::singleline(&mut probe.name)
                .desired_width(ui.available_width())
                .margin(egui::vec2(8.0, 6.0))
                .hint_text(name_hint),
        )
        .labelled_by(label.id)
        .accessible_name(name);
        let selected = probe
            .profiles
            .iter()
            .find(|profile| profile["id"] == probe.coordinator)
            .and_then(|profile| profile["id"].as_str())
            .unwrap_or(coordinator_none);
        ui.add_enabled_ui(can_edit && !probe.profiles.is_empty(), |ui| {
            egui::ComboBox::from_label(coordinator_label)
                .selected_text(selected)
                .width(ui.available_width().min(320.0))
                .show_ui(ui, |ui| {
                    for profile in &probe.profiles {
                        if let Some(id) = profile["id"].as_str() {
                            ui.selectable_value(&mut probe.coordinator, id.into(), id);
                        }
                    }
                });
        });
        ui.horizontal(|ui| {
            let label = ui.label(project);
            ui.add_enabled(
                can_edit,
                egui::TextEdit::singleline(&mut probe.repository)
                    .desired_width(ui.available_width().min(420.0))
                    .margin(egui::vec2(8.0, 6.0))
                    .hint_text(project_hint),
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
            egui::TextEdit::multiline(&mut probe.goal)
                .desired_width(ui.available_width())
                .desired_rows(4)
                .margin(egui::vec2(8.0, 8.0))
                .hint_text(goal_hint),
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
