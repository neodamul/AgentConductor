use super::{
    Confirmation, Event, NavigationContext, State, display_path, message,
    needs_discard_confirmation, text, tools_guide,
};
use eframe::egui::{self, RichText};

pub fn show_in_language(
    state: &mut State,
    context: NavigationContext<'_>,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let NavigationContext {
        current_name,
        current_path,
        can_change,
        dirty,
        show_cache,
        language,
    } = context;
    let mut event = None;
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("AC")
                .strong()
                .color(egui::Color32::WHITE)
                .background_color(egui::Color32::from_rgb(10, 102, 216)),
        );
        if show_cache
            && ui
                .add_enabled(
                    can_change,
                    egui::Button::new(text(language, "startup.cache_cleanup")),
                )
                .accessible_name(text(language, "startup.cache_cleanup_accessible"))
                .clicked()
        {
            event = Some(Event::Cache);
        }
        ui.heading("AgentConductor");
        if ui
            .button(format!(
                "{} ▾ {current_name}",
                text(language, "startup.workspace_373010")
            ))
            .accessible_name(text(language, "startup.open_workspace_menu"))
            .clicked()
        {
            state.open = !state.open;
            if state.open {
                event = Some(Event::List);
            }
        }
        match crate::native_ui_probe::appearance::show(&mut state.appearance, ui) {
            Some(crate::native_ui_probe::appearance::SettingsAction::Mcp) => {
                event = Some(Event::Mcp);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::Messenger) => {
                event = Some(Event::Messenger);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::Updates) => {
                event = Some(Event::Updates);
            }
            None => {}
        }
    });
    if state.open {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.strong(text(language, "startup.saved_workspaces"))
                .accessible_name(text(language, "startup.saved_workspaces"));
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut state.name)
                        .hint_text(text(language, "startup.new_workspace_name")),
                )
                .accessible_name(text(language, "startup.new_workspace_name"));
                if ui
                    .add_enabled(
                        can_change && !state.loading && !state.name.trim().is_empty(),
                        egui::Button::new(text(language, "startup.create_workspace_action")),
                    )
                    .accessible_name(text(language, "startup.create_workspace_action"))
                    .clicked()
                {
                    let create = Event::Create(state.name.trim().to_owned());
                    if needs_discard_confirmation(&create, dirty) {
                        state.confirmation = Some(Confirmation::Create(state.name.trim().into()));
                    } else {
                        event = Some(create);
                    }
                }
            });
            if state.loading {
                ui.spinner();
            } else if state.workspaces.is_empty() {
                ui.label(text(language, "startup.no_saved_workspaces"));
            }
            if ui
                .button(text(language, "startup.agent_tools_guide"))
                .clicked()
            {
                state.tools_guide = true;
                state.open = false;
            }
            if ui.button(text(language, "startup.user_manual")).clicked() {
                state.manual_blocks = crate::native_ui_probe::manual_viewer::blocks(language);
                state.manual_language = language.to_owned();
                state.manual_open = true;
                state.open = false;
            }
            let mut confirmation = None;
            for workspace in &state.workspaces {
                let name = workspace["name"]
                    .as_str()
                    .unwrap_or(text(language, "startup.workspace_fallback"));
                let path = workspace["path"].as_str().unwrap_or_default();
                let current = path == current_path;
                ui.horizontal(|ui| {
                    ui.label(format!("{name} · {}", display_path(path)));
                    if current {
                        ui.label(text(language, "startup.current_workspace"))
                            .accessible_name(text(language, "startup.current_workspace"));
                    } else if ui
                        .add_enabled(
                            can_change && !state.loading,
                            egui::Button::new(text(
                                language,
                                "startup.open_existing_workspace_87a48e",
                            )),
                        )
                        .accessible_name(message(
                            language,
                            "startup.open_workspace_named",
                            &[("value1", name)],
                        ))
                        .clicked()
                    {
                        let open = Event::Open(path.into());
                        if needs_discard_confirmation(&open, dirty) {
                            confirmation = Some(Confirmation::Open {
                                path: path.into(),
                                name: name.into(),
                            });
                        } else {
                            event = Some(open);
                        }
                    }
                    if !current
                        && workspace["registered"] == true
                        && ui
                            .add_enabled(
                                !state.loading,
                                egui::Button::new(text(language, "startup.remove_from_list")),
                            )
                            .accessible_name(message(
                                language,
                                "startup.remove_workspace_named_from_list",
                                &[("value1", name)],
                            ))
                            .clicked()
                    {
                        event = Some(Event::Forget(path.into()));
                    }
                    if ui
                        .add_enabled(
                            can_change && !state.loading,
                            egui::Button::new(text(language, "startup.archive")),
                        )
                        .accessible_name(message(
                            language,
                            "startup.archive_workspace_named",
                            &[("value1", name)],
                        ))
                        .clicked()
                    {
                        confirmation = Some(Confirmation::Delete {
                            path: path.into(),
                            name: name.into(),
                        });
                    }
                });
            }
            if confirmation.is_some() {
                state.confirmation = confirmation;
            }
        });
    }
    tools_guide::show(&mut state.tools_guide, language, ui);
    if state.manual_open && state.manual_language != language {
        state.manual_blocks = crate::native_ui_probe::manual_viewer::blocks(language);
        state.manual_language = language.to_owned();
    }
    crate::native_ui_probe::manual_viewer::show(
        &mut state.manual_open,
        language,
        &state.manual_blocks,
        ui,
    );
    if let Some(confirm) = state.confirmation.clone()
        && let Some(confirmed) = super::confirmation::show(state, confirm, dirty, language, ui)
    {
        event = Some(confirmed);
    }
    event
}
