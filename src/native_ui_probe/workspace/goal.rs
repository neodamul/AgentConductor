use super::{Probe, card, header, permissions::editable};
use eframe::egui::{self, RichText};

mod warning;

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
            locale.text("startup.browse"),
            locale.text("startup.goal"),
            locale.text("startup.goal_hint"),
            locale.text("startup.save_workspace"),
            locale.text("startup.planning_coordinator"),
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
            .unwrap_or(coordinator_none)
            .to_owned();
        let language = probe.navigation.appearance.language().to_owned();
        let coordinator = probe.coordinator.clone();
        ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = 40.0;
            ui.columns(3, |columns| {
                columns[0].vertical(|ui| {
                    ui.label(name);
                    ui.add_enabled(
                        can_edit,
                        egui::TextEdit::singleline(&mut probe.name)
                            .desired_width(ui.available_width())
                            .margin(egui::vec2(8.0, 6.0))
                            .hint_text(name_hint),
                    )
                    .accessible_name(name);
                });
                columns[1].vertical(|ui| {
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
                if let Some(profile) = probe
                    .profiles
                    .iter_mut()
                    .find(|profile| profile["id"] == coordinator)
                {
                    coordinator_model_selector(profile, &language, can_edit, &mut columns[2]);
                }
            });
        });
        if let Some(reason) = super::chat::coordinator_issue(&probe.profiles, &probe.coordinator) {
            let template =
                crate::ui::messages::in_language(&language, "startup.coordinator_cannot_chat");
            let warning = crate::ui::messages::interpolate(template, &[("value1", &reason)]);
            warning::show(&warning, ui);
        }
        let mut use_worktrees = probe.workspace_mode == crate::plan::WorkspaceMode::Worktree;
        ui.label(project);
        ui.add_enabled_ui(can_edit, |ui| {
            ui.scope(|ui| {
                ui.spacing_mut().interact_size.y = 40.0;
                ui.horizontal_wrapped(|ui| {
                    let input_width = (ui.available_width() - 290.0).clamp(180.0, 380.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut probe.repository)
                            .desired_width(input_width)
                            .margin(egui::vec2(8.0, 6.0))
                            .hint_text(project_hint),
                    )
                    .accessible_name(project);
                    if ui
                        .add(egui::Button::new(browse))
                        .accessible_name(browse)
                        .clicked()
                    {
                        probe.pick_folder();
                    }
                    ui.checkbox(
                        &mut use_worktrees,
                        probe
                            .navigation
                            .appearance
                            .text("startup.isolate_tasks_in_git_worktrees"),
                    );
                });
            });
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
        if let Some(event) = probe.goal_optimizer.show(
            &language,
            &probe.goal,
            optimization_enabled,
            probe
                .active_work
                .label("optimize", &language, std::time::Instant::now())
                .as_deref(),
            ui,
        ) {
            match event {
                super::optimizer::Event::Request(goal) => probe.optimize_goal(goal),
                super::optimizer::Event::Use(goal) => probe.goal = goal,
                super::optimizer::Event::Keep => {}
            }
        }
        match probe.mcp_sources.show_with_progress(
            &mut probe.goal,
            probe.snapshot.as_ref(),
            &language,
            can_edit,
            probe
                .active_work
                .label("mcp", &language, std::time::Instant::now())
                .as_deref(),
            ui,
        ) {
            Some(super::mcp::SourceEvent::Preview) => probe.begin_mcp_preview(),
            Some(super::mcp::SourceEvent::Apply(selected)) => probe.apply_mcp_sources(&selected),
            None => {}
        }
        probe.workspace_mode = if use_worktrees {
            crate::plan::WorkspaceMode::Worktree
        } else {
            crate::plan::WorkspaceMode::Shared
        };
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

fn coordinator_model_selector(
    profile: &mut serde_json::Value,
    language: &str,
    can_edit: bool,
    ui: &mut egui::Ui,
) {
    let model_label = crate::ui::messages::in_language(language, "startup.coordinator_model");
    let cli_default = crate::ui::messages::in_language(language, "startup.cli_default");
    let models = profile["catalog"]["models"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut selected = profile["coordinator_model"]
        .as_str()
        .filter(|selected| models.iter().any(|model| model.as_str() == Some(selected)))
        .unwrap_or_default()
        .to_owned();
    let previous = selected.clone();
    ui.vertical(|ui| {
        let label = ui.label(model_label);
        ui.add_enabled_ui(can_edit && !models.is_empty(), |ui| {
            egui::ComboBox::from_id_salt("native-coordinator-model-choice")
                .selected_text(if selected.is_empty() {
                    cli_default
                } else {
                    &selected
                })
                .width(190.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, String::new(), cli_default);
                    for model in models {
                        if let Some(model) = model.as_str() {
                            ui.selectable_value(&mut selected, model.to_owned(), model);
                        }
                    }
                })
                .response
                .labelled_by(label.id)
                .accessible_name(model_label);
        });
    });
    if selected != previous
        && let Some(profile) = profile.as_object_mut()
    {
        if selected.is_empty() {
            profile.remove("coordinator_model");
        } else {
            profile.insert("coordinator_model".into(), selected.into());
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/native_ui_probe/workspace_model.rs"]
mod model_tests;
