use super::{Probe, card, header, permissions::editable};
use eframe::egui::{self, RichText};

pub(super) fn show(probe: &mut Probe, ui: &mut egui::Ui) {
    let can_edit = editable(probe);
    let (
        settings_title,
        settings_description,
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
            locale.text("startup.workspace_settings_5ebc59"),
            locale.text("startup.saves_the_workspace_name_goal_coordinator_repository_and"),
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
        header::show_card_eyebrow(0, probe.navigation.appearance.language(), ui);
        ui.heading(
            RichText::new(settings_title)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.label(
            RichText::new(settings_description)
                .small()
                .color(ui.visuals().weak_text_color()),
        );
        ui.add_space(8.0);
        let selected = probe
            .profiles
            .iter()
            .find(|profile| profile["id"] == probe.coordinator)
            .and_then(|profile| profile["id"].as_str())
            .unwrap_or(coordinator_none);
        let name_width = (ui.available_width() * 0.52).min(380.0);
        ui.horizontal_wrapped(|ui| {
            ui.vertical(|ui| {
                ui.label(name);
                ui.add_enabled(
                    can_edit,
                    egui::TextEdit::singleline(&mut probe.name)
                        .desired_width(name_width)
                        .margin(egui::vec2(8.0, 6.0))
                        .hint_text(name_hint),
                )
                .accessible_name(name);
            });
            ui.vertical(|ui| {
                ui.label(coordinator_label);
                ui.add_enabled_ui(can_edit && !probe.profiles.is_empty(), |ui| {
                    egui::ComboBox::from_id_salt("native-workspace-coordinator")
                        .selected_text(selected)
                        .width(ui.available_width().min(280.0))
                        .show_ui(ui, |ui| {
                            for profile in &probe.profiles {
                                if let Some(id) = profile["id"].as_str() {
                                    ui.selectable_value(&mut probe.coordinator, id.into(), id);
                                }
                            }
                        })
                        .response
                        .accessible_name(coordinator_label);
                });
            });
        });
        ui.horizontal_wrapped(|ui| {
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
        let language = probe.navigation.appearance.language().to_owned();
        let config = probe.snapshot.as_ref().map(|snapshot| &snapshot["config"]);
        let optimization_enabled = can_edit
            && probe.pending.is_none()
            && !probe.goal.trim().is_empty()
            && std::path::Path::new(&probe.repository).is_dir()
            && config.is_some_and(|config| {
                config["repository"] == probe.repository
                    && config["coordinator"] == probe.coordinator
                    && probe
                        .snapshot
                        .as_ref()
                        .is_some_and(|snapshot| snapshot["job"]["state"] != "running")
            });
        if let Some(event) =
            probe
                .goal_optimizer
                .show(&language, &probe.goal, optimization_enabled, ui)
        {
            match event {
                super::optimizer::Event::Request(goal) => probe.optimize_goal(goal),
                super::optimizer::Event::Use(goal) => probe.goal = goal,
                super::optimizer::Event::Keep => {}
            }
        }
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
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if super::super::appearance::primary_button(
                ui,
                can_edit && probe.snapshot.is_some(),
                save,
            )
            .clicked()
            {
                probe.send_workspace();
            }
        });
    });
}
