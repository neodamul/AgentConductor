use super::{Confirmation, Event, NavigationContext, State, text, tools_guide, view_menu};
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
        planning_running,
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
        ui.heading("AgentConductor");
        ui.label(
            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                .small()
                .weak(),
        );
        let response = ui
            .button(format!(
                "{} ▾ {current_name}",
                text(language, "startup.workspace_373010")
            ))
            .accessible_name(text(language, "startup.open_workspace_menu"));
        let popup_id = egui::Popup::default_response_id(&response);
        let was_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        let popup = egui::Popup::menu(&response)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside);
        let popup = if state.open && !was_open {
            popup.open_memory(egui::SetOpenCommand::Bool(true))
        } else {
            popup
        };
        let menu = popup.show(|ui| {
            view_menu::show(
                state,
                current_path,
                can_change,
                planning_running,
                dirty,
                language,
                ui,
            )
        });
        state.open = egui::Popup::is_id_open(ui.ctx(), popup_id);
        if state.open && !was_open {
            event = Some(Event::List);
        }
        let menu_event = menu.and_then(|inner| inner.inner);
        let close_menu = menu_event.is_some()
            || state.confirmation.is_some()
            || state.tools_guide
            || state.manual_open;
        if let Some(menu_event) = menu_event {
            event = Some(menu_event);
        }
        if close_menu {
            egui::Popup::close_id(ui.ctx(), popup_id);
            state.open = false;
        }
        if state.run_available
            && ui
                .add_enabled(
                    !state.loading,
                    egui::Button::new(text(language, "startup.back_to_run")),
                )
                .clicked()
        {
            if dirty {
                state.confirmation = Some(Confirmation::Discard {
                    areas: state.dirty_areas.clone(),
                    return_to_run: true,
                });
            } else {
                event = Some(Event::BackToRun);
            }
        }
        if ui
            .add_enabled(
                dirty && can_change,
                egui::Button::new(text(language, "startup.discard_changes")),
            )
            .clicked()
        {
            state.confirmation = Some(Confirmation::Discard {
                areas: state.dirty_areas.clone(),
                return_to_run: false,
            });
        }
        match crate::native_ui_probe::appearance::show_with_cache(
            &mut state.appearance,
            show_cache && can_change,
            ui,
        ) {
            Some(crate::native_ui_probe::appearance::SettingsAction::Cache) => {
                event = Some(Event::Cache);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::Mcp) => {
                event = Some(Event::Mcp);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::Messenger) => {
                event = Some(Event::Messenger);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::SignalTransport) => {
                event = Some(Event::SignalTransport);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::WorkspaceSharing) => {
                event = Some(Event::WorkspaceSharing);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::UpdateRepository) => {
                event = Some(Event::UpdateRepository);
            }
            Some(crate::native_ui_probe::appearance::SettingsAction::Updates) => {
                event = Some(Event::Updates);
            }
            None => {}
        }
    });
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
