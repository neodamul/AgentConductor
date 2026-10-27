use super::{Event, State};
use crate::cooperative::route::Transport;
use eframe::egui::{self, TextEdit};

pub(super) fn show(
    state: &mut State,
    language: &str,
    can_edit: bool,
    pending: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    if !state.open {
        return None;
    }
    let mut open = state.open;
    let mut close_requested = false;
    let mut event = None;
    egui::Window::new(text(language, "startup.signal_transport"))
        .open(&mut open)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .default_width(520.0)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            if state.loading {
                ui.spinner();
            }
            ui.add_enabled_ui(
                can_edit && state.loaded && !pending && !state.loading,
                |ui| {
                    ui.vertical(|ui| {
                        ui.label(text(language, "startup.signal_transport"));
                        ui.add_space(8.0);
                        ui.scope(|ui| {
                            ui.spacing_mut().interact_size.y = 36.0;
                            ui.horizontal_wrapped(|ui| {
                                ui.selectable_value(
                                    &mut state.preference.transport,
                                    Transport::Local,
                                    text(language, "startup.local_rust"),
                                );
                                ui.selectable_value(
                                    &mut state.preference.transport,
                                    Transport::Agentmemory,
                                    text(language, "startup.agentmemory_preferred"),
                                );
                            });
                        });
                    });
                    if state.preference.transport == Transport::Agentmemory {
                        ui.separator();
                        ui.vertical(|ui| {
                            let label = ui.label(text(language, "startup.agentmemory_port"));
                            ui.add_space(8.0);
                            ui.add(
                                TextEdit::singleline(&mut state.port)
                                    .desired_width(180.0)
                                    .min_size(egui::vec2(180.0, 36.0))
                                    .char_limit(5),
                            )
                            .labelled_by(label.id)
                            .accessible_name(text(language, "startup.agentmemory_port"));
                        });
                    }
                    ui.add_space(8.0);
                    if ui
                        .add_enabled(
                            can_save(state) && !pending,
                            egui::Button::new(text(language, "startup.save")),
                        )
                        .clicked()
                    {
                        event = match state.save_request() {
                            Ok(packet) => Some(Event::Save(packet)),
                            Err(error) => {
                                state.error = Some(error);
                                None
                            }
                        };
                    }
                },
            );
            if let Some(error) = &state.error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            if ui.button(text(language, "startup.close")).clicked() {
                close_requested = true;
            }
        });
    state.open = open && !close_requested;
    event
}

fn can_save(state: &State) -> bool {
    state.preference.transport == Transport::Local
        || state.port.parse::<u16>().is_ok_and(|port| port != 0)
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
