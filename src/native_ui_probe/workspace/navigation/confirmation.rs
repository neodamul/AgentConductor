use super::helpers::{display_path, message, text};
use super::{Confirmation, Event, State};
use eframe::egui;

pub(super) fn show(
    state: &mut State,
    confirm: Confirmation,
    dirty: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    let mut open = true;
    let mut close_requested = false;
    egui::Window::new(text(language, "startup.workspace_confirmation_title"))
        .id(egui::Id::NULL.with("native-workspace-confirmation"))
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .collapsible(false)
        .default_open(true)
        .resizable(false)
        .default_width(560.0)
        .max_width(720.0)
        .max_height(360.0)
        .vscroll(true)
        .show(ui.ctx(), |ui| {
            if ui.button(text(language, "startup.close")).clicked() {
                close_requested = true;
            }
            match confirm {
                Confirmation::Create(name) => {
                    ui.label(message(
                        language,
                        "startup.confirm_create_discard_changes",
                        &[("value1", &name)],
                    ));
                    if ui
                        .button(text(language, "startup.confirm_and_switch_workspace"))
                        .clicked()
                    {
                        event = Some(Event::ConfirmCreate(name));
                        state.confirmation = None;
                    }
                    if ui
                        .button(text(language, "startup.cancel_workspace_creation"))
                        .clicked()
                    {
                        state.confirmation = None;
                    }
                }
                Confirmation::Open { path, name } => {
                    ui.label(message(
                        language,
                        "startup.confirm_open_discard_changes",
                        &[("value1", &name), ("value2", &path)],
                    ));
                    if ui
                        .button(message(
                            language,
                            "startup.confirm_open_workspace_named",
                            &[("value1", &name)],
                        ))
                        .accessible_name(message(
                            language,
                            "startup.confirm_open_workspace_named",
                            &[("value1", &name)],
                        ))
                        .clicked()
                    {
                        event = Some(Event::ConfirmOpen(path));
                        state.confirmation = None;
                    }
                    if ui
                        .button(message(
                            language,
                            "startup.cancel_open_workspace_named",
                            &[("value1", &name)],
                        ))
                        .clicked()
                    {
                        state.confirmation = None;
                    }
                }
                Confirmation::Delete { path, name } => {
                    let discard = if dirty {
                        text(language, "startup.unsaved_workspace_edits_suffix")
                    } else {
                        ""
                    };
                    ui.label(message(
                        language,
                        "startup.archive_workspace_confirmation",
                        &[
                            ("value1", &name),
                            ("value2", &display_path(&path)),
                            ("value3", discard),
                        ],
                    ))
                    .accessible_name(message(
                        language,
                        "startup.archive_workspace_accessible",
                        &[("value1", &name), ("value2", &path), ("value3", discard)],
                    ));
                    if ui
                        .button(message(
                            language,
                            "startup.confirm_archive_workspace_named",
                            &[("value1", &name)],
                        ))
                        .accessible_name(message(
                            language,
                            "startup.confirm_archive_workspace_named",
                            &[("value1", &name)],
                        ))
                        .clicked()
                    {
                        event = Some(Event::Delete {
                            path: path.clone(),
                            name: name.clone(),
                        });
                        state.confirmation = None;
                    }
                    if ui
                        .button(message(
                            language,
                            "startup.cancel_archive_workspace_named",
                            &[("value1", &name)],
                        ))
                        .accessible_name(message(
                            language,
                            "startup.cancel_archive_workspace_named",
                            &[("value1", &name)],
                        ))
                        .clicked()
                    {
                        state.confirmation = None;
                    }
                }
                Confirmation::StopAndDelete { path, name } => {
                    ui.label(message(
                        language,
                        "startup.archive_workspace_confirmation",
                        &[
                            ("value1", &name),
                            ("value2", &display_path(&path)),
                            ("value3", ""),
                        ],
                    ))
                    .accessible_name(message(
                        language,
                        "startup.archive_workspace_accessible",
                        &[("value1", &name), ("value2", &path), ("value3", "")],
                    ));
                    if ui
                        .button(text(language, "startup.stop_planning_and_delete"))
                        .accessible_name(message(language, "startup.stop_planning_and_delete", &[]))
                        .clicked()
                    {
                        event = Some(Event::Delete {
                            path: path.clone(),
                            name: name.clone(),
                        });
                        state.confirmation = None;
                    }
                    if ui
                        .button(message(
                            language,
                            "startup.cancel_archive_workspace_named",
                            &[("value1", &name)],
                        ))
                        .accessible_name(message(
                            language,
                            "startup.cancel_archive_workspace_named",
                            &[("value1", &name)],
                        ))
                        .clicked()
                    {
                        state.confirmation = None;
                    }
                }
                Confirmation::Discard {
                    areas,
                    return_to_run,
                } => {
                    let areas = areas.join(", ");
                    let warning = message(
                        language,
                        "startup.confirm_discard_changes",
                        &[("value1", &areas)],
                    );
                    ui.label(&warning).accessible_name(&warning);
                    if ui
                        .button(text(language, "startup.discard"))
                        .accessible_name(text(language, "startup.discard_unsaved_changes"))
                        .clicked()
                    {
                        event = Some(Event::DiscardChanges { return_to_run });
                        state.confirmation = None;
                    }
                    if ui.button(text(language, "startup.keep_editing")).clicked() {
                        state.confirmation = None;
                    }
                }
            }
        });
    if !open || close_requested {
        state.confirmation = None;
    }
    event
}
