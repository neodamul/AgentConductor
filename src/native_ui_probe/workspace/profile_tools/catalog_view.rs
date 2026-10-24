use super::{Editor, Event, catalog::parse_manual_catalog};
use eframe::egui;

pub(super) fn show(
    editor: &mut Editor,
    editable: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let agent = editor.catalog_agent.clone()?;
    let modal =
        egui::Modal::new(egui::Id::unique("native-custom-model-catalog")).show(ui.ctx(), |ui| {
            ui.set_min_width(520.0);
            ui.label(
                egui::RichText::new(text(language, "startup.custom_cli_models"))
                    .small()
                    .strong(),
            );
            ui.heading(text(language, "startup.configure_cli_model_ids"));
            ui.label(text(
                language,
                "startup.this_cli_does_not_expose_model_discovery_enter",
            ));
            ui.label(text(language, "startup.model_ids_one_per_line"));
            let input = ui.add_enabled(
                editable,
                egui::TextEdit::multiline(&mut editor.catalog_text)
                    .desired_rows(9)
                    .desired_width(ui.available_width()),
            );
            input.accessible_name(text(language, "startup.model_ids_one_per_line"));
            ui.label(text(
                language,
                "startup.to_select_a_session_model_include_in_this",
            ));
            if let Some(error) = &editor.catalog_error {
                ui.colored_label(ui.visuals().error_fg_color, error);
            }
            let mut event = None;
            ui.horizontal(|ui| {
                if ui.button(text(language, "startup.cancel")).clicked() {
                    editor.close_catalog();
                    ui.close();
                }
                if ui
                    .add_enabled(
                        editable,
                        egui::Button::new(text(language, "startup.save_model_ids")),
                    )
                    .accessible_name(text(language, "startup.save_model_ids"))
                    .clicked()
                {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs() as i64);
                    match parse_manual_catalog(&editor.catalog_text, now, language) {
                        Ok(catalog) => {
                            event = Some(Event::SetCatalog {
                                agent: agent.clone(),
                                catalog,
                            })
                        }
                        Err(error) => editor.catalog_error = Some(error),
                    }
                }
            });
            event
        });
    if modal.should_close() {
        editor.close_catalog();
    }
    modal.inner
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
