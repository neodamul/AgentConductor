use super::{Event, State};
use eframe::egui::{self, RichText};
impl State {
    pub fn show_launcher(&mut self, language: &str, ui: &mut egui::Ui) {
        if ui.button(text(language, "chat.open")).clicked() {
            self.open = true;
        }
    }

    pub fn show(&mut self, language: &str, ui: &mut egui::Ui) -> Option<Event> {
        let was_focused = self.editor_focused;
        let (enter_pressed, shift_pressed) = ui.input(|input| {
            input
                .events
                .iter()
                .find_map(|event| match event {
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        modifiers,
                        ..
                    } => Some((true, modifiers.shift || modifiers.mac_cmd)),
                    _ => None,
                })
                .unwrap_or((false, false))
        });
        let up_pressed = ui.input(|input| input.key_pressed(egui::Key::ArrowUp));
        ui.horizontal(|ui| {
            ui.heading(text(language, "chat.title"));
            if ui.button(text(language, "chat.new_conversation")).clicked() && !self.awaiting() {
                self.turns.clear();
                self.draft.clear();
                self.error = None;
                self.requesting = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(text(language, "chat.close")).clicked() {
                    self.open = false;
                }
            });
        });
        if self.requesting && self.draft.is_empty() && self.turns.is_empty() {
            return Some(Event::Reset);
        }
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for turn in &self.turns {
                    ui.label(
                        RichText::new(if turn.role == "operator" {
                            text(language, "chat.you")
                        } else {
                            text(language, "chat.title")
                        })
                        .strong(),
                    );
                    let prefix = if turn.role == "operator" { "> " } else { "" };
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("{prefix}{}", turn.text)).monospace(),
                        )
                        .selectable(true),
                    );
                    ui.add_space(8.0);
                }
                if self.awaiting() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(text(language, "chat.thinking"));
                    });
                }
                if let Some(error) = &self.error {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                }
            });
        ui.separator();
        let mut editor_focused = false;
        ui.add_enabled_ui(!self.awaiting(), |ui| {
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.draft)
                    .desired_rows(3)
                    .hint_text(text(language, "chat.placeholder")),
            );
            editor_focused = response.has_focus();
        });
        self.editor_focused = editor_focused;
        if (was_focused || editor_focused)
            && up_pressed
            && self.draft.is_empty()
            && let Some(previous) = self.turns.iter().rev().find(|turn| turn.role == "operator")
        {
            self.draft = previous.text.clone();
        }
        let send_key = enter_pressed && !shift_pressed && !self.draft.trim().is_empty();
        let send_by_key = send_key && !self.draft.trim().is_empty();
        let send = ui
            .add_enabled(
                !self.awaiting() && !self.draft.trim().is_empty(),
                egui::Button::new(text(language, "chat.send")),
            )
            .clicked()
            || send_by_key;
        if !send {
            return None;
        }
        let mut question = std::mem::take(&mut self.draft);
        if send_key && question.ends_with('\n') {
            question.pop();
        }
        self.turns.push(crate::chat::Turn {
            role: "operator".into(),
            text: question.clone(),
        });
        self.error = None;
        self.requesting = true;
        Some(Event::Send(question))
    }
}
fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
