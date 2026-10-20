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
    egui::Frame::group(ui.style()).show(ui, |ui| match confirm {
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
    });
    event
}
