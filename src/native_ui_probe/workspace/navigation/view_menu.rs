use super::{Confirmation, Event, State, display_path, message, needs_discard_confirmation, text};
use eframe::egui::{self, Button, TextEdit};

pub(super) fn show(
    state: &mut State,
    current_path: &str,
    can_change: bool,
    planning_running: bool,
    dirty: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    let max_height = ui.ctx().content_rect().height() * 0.7;
    egui::ScrollArea::vertical()
        .max_height(max_height)
        .show(ui, |ui| {
            ui.strong(text(language, "startup.saved_workspaces"))
                .accessible_name(text(language, "startup.saved_workspaces"));
            ui.horizontal(|ui| {
                let name_input = ui
                    .add(
                        TextEdit::singleline(&mut state.name)
                            .hint_text(text(language, "startup.new_workspace_name")),
                    )
                    .accessible_name(text(language, "startup.new_workspace_name"));
                let submit_enter = name_input.lost_focus()
                    && ui.input(|input| input.key_pressed(egui::Key::Enter));
                if ui
                    .add_enabled(
                        can_change && !state.loading && !state.name.trim().is_empty(),
                        Button::new(text(language, "startup.create_workspace_action")),
                    )
                    .accessible_name(text(language, "startup.create_workspace_action"))
                    .clicked()
                    || (submit_enter
                        && can_change
                        && !state.loading
                        && !state.name.trim().is_empty())
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
            }
            if ui.button(text(language, "startup.user_manual")).clicked() {
                state.manual_blocks = crate::native_ui_probe::manual_viewer::blocks(language);
                state.manual_language = language.to_owned();
                state.manual_open = true;
            }
            let mut confirmation = None;
            for workspace in &state.workspaces {
                let name = workspace["name"]
                    .as_str()
                    .unwrap_or(text(language, "startup.workspace_fallback"));
                let path = workspace["path"].as_str().unwrap_or_default();
                let current = path == current_path;
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{name} · {}", display_path(path)));
                    if current {
                        ui.label(text(language, "startup.current_workspace"))
                            .accessible_name(text(language, "startup.current_workspace"));
                        if planning_running
                            && workspace["registered"] == true
                            && !state.loading
                            && ui
                                .button(text(language, "startup.stop_planning_and_delete"))
                                .accessible_name(message(
                                    language,
                                    "startup.stop_planning_and_delete",
                                    &[],
                                ))
                                .clicked()
                        {
                            confirmation = Some(Confirmation::StopAndDelete {
                                path: path.into(),
                                name: name.into(),
                            });
                        }
                    } else if ui
                        .add_enabled(
                            can_change && !state.loading,
                            Button::new(text(language, "startup.open_existing_workspace_87a48e")),
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
                                Button::new(text(language, "startup.remove_from_list")),
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
                    if !current
                        && ui
                            .add_enabled(
                                can_change && !state.loading,
                                Button::new(text(language, "startup.archive")),
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
    event
}
