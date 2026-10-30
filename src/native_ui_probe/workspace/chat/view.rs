use super::{Event, State};
use eframe::egui::{self, Key, Modifiers, RichText};
use egui::text::{CCursor, CCursorRange};

impl State {
    pub fn show(&mut self, language: &str, ui: &mut egui::Ui) -> Option<Event> {
        let mut new_conversation = false;
        ui.horizontal(|ui| {
            ui.heading(text(language, "chat.title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(text(language, "chat.close")).clicked() {
                    self.open = false;
                }
                if !self.turns.is_empty()
                    && !self.awaiting()
                    && ui.button(text(language, "chat.new")).clicked()
                {
                    new_conversation = true;
                }
            });
        });
        if new_conversation {
            return Some(Event::NewConversation);
        }
        ui.separator();
        egui::ScrollArea::vertical()
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for turn in &self.turns {
                    let marker = if turn.role == "operator" { "> " } else { "< " };
                    ui.label(RichText::new(format!("{marker}{}", turn.text)).monospace());
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
        let can_edit = !self.awaiting();
        let (send_key, newline, previous, next) = if can_edit && self.editor_focused {
            ui.input_mut(|input| {
                let newline = input.consume_key(Modifiers::SHIFT, Key::Enter);
                let send = !newline && input.consume_key(Modifiers::NONE, Key::Enter);
                (
                    send,
                    newline,
                    input.consume_key(Modifiers::NONE, Key::ArrowUp),
                    input.consume_key(Modifiers::NONE, Key::ArrowDown),
                )
            })
        } else {
            (false, false, false, false)
        };
        if previous {
            self.recall_question(true);
        }
        if next {
            self.recall_question(false);
        }
        let recalling = previous || next;
        let mut editor = None;
        ui.add_enabled_ui(can_edit, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("> ").monospace());
                let mut output = egui::TextEdit::multiline(&mut self.draft)
                    .desired_rows(3)
                    .font(egui::TextStyle::Monospace)
                    .hint_text(text(language, "chat.placeholder"))
                    .show(ui);
                output.response.response = output
                    .response
                    .response
                    .clone()
                    .accessible_name(text(language, "chat.placeholder"));
                editor = Some(output);
            });
        });
        let changed = editor
            .as_ref()
            .is_some_and(|output| output.response.changed());
        if let Some(mut output) = editor {
            self.editor_focused = output.response.has_focus();
            if newline {
                let character_index = self.editor_cursor.min(self.draft.chars().count());
                let byte_index = self
                    .draft
                    .char_indices()
                    .nth(character_index)
                    .map_or(self.draft.len(), |(index, _)| index);
                self.draft.insert(byte_index, '\n');
                self.editor_cursor = character_index + 1;
                output
                    .state
                    .cursor
                    .set_char_range(Some(CCursorRange::one(CCursor::new(self.editor_cursor))));
                output.state.store(ui.ctx(), output.response.id);
            } else if let Some(range) = output.cursor_range {
                self.editor_cursor = range.primary.index.into();
            }
        }
        if changed && !recalling {
            self.question_cursor = None;
            self.history_draft.clear();
        }
        let send_button = ui
            .add_enabled(
                can_edit && !self.draft.trim().is_empty(),
                egui::Button::new(text(language, "chat.send")),
            )
            .clicked();
        if !(send_key || send_button) || self.draft.trim().is_empty() {
            return None;
        }
        let turn = crate::chat::Turn {
            role: "operator".into(),
            text: std::mem::take(&mut self.draft),
        };
        self.pending_before = Some(self.turns.clone());
        self.turns.push(turn.clone());
        self.pending_turn = Some(turn);
        self.question_cursor = None;
        self.history_draft.clear();
        self.error = None;
        self.requesting = true;
        Some(Event::Send(self.turns.clone()))
    }

    fn recall_question(&mut self, previous: bool) {
        let questions = self
            .turns
            .iter()
            .filter(|turn| turn.role == "operator")
            .map(|turn| turn.text.clone())
            .collect::<Vec<_>>();
        if questions.is_empty() {
            return;
        }
        if previous {
            let current = self.question_cursor.unwrap_or(questions.len());
            if self.question_cursor.is_none() {
                self.history_draft.clone_from(&self.draft);
            }
            let index = current.saturating_sub(1);
            self.question_cursor = Some(index);
            self.draft.clone_from(&questions[index]);
        } else if let Some(current) = self.question_cursor {
            let next = current + 1;
            if next == questions.len() {
                self.question_cursor = None;
                self.draft = std::mem::take(&mut self.history_draft);
            } else {
                self.question_cursor = Some(next);
                self.draft.clone_from(&questions[next]);
            }
        }
    }
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
