use eframe::egui::{self, Ui};

pub struct State {
    language: String,
    theme: String,
    error: Option<String>,
}

impl State {
    pub fn load() -> Self {
        Self {
            language: crate::ui::language::load(),
            theme: crate::ui::language::load_theme(),
            error: None,
        }
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn text(&self, id: &'static str) -> &'static str {
        crate::ui::messages::in_language(&self.language, id)
    }

    pub fn select_language(&mut self, language: &str) -> Result<(), String> {
        crate::ui::language::save(language)?;
        self.language = language.to_owned();
        self.error = None;
        Ok(())
    }

    pub fn apply(&self, context: &egui::Context) {
        apply(&self.theme, context);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::load()
    }
}

pub fn apply(theme: &str, context: &egui::Context) {
    context.set_theme(if theme == "dark" {
        egui::Theme::Dark
    } else {
        egui::Theme::Light
    });
}

pub fn show(state: &mut State, ui: &mut Ui) -> bool {
    let mut open_mcp = false;
    ui.menu_button(state.text("startup.settings"), |ui| {
        ui.label(state.text("startup.interface_language"));
        for (language, label) in [
            ("en", "startup.language_english"),
            ("ko", "startup.language_korean"),
        ] {
            if ui
                .selectable_label(state.language == language, state.text(label))
                .clicked()
                && let Err(error) = state.select_language(language)
            {
                state.error = Some(error);
            }
        }
        ui.separator();
        ui.label(state.text("startup.theme"));
        let light = state.text("startup.theme_light");
        let dark = state.text("startup.theme_dark");
        let mut changed = false;
        changed |= ui
            .selectable_value(&mut state.theme, "light".into(), light)
            .clicked();
        changed |= ui
            .selectable_value(&mut state.theme, "dark".into(), dark)
            .clicked();
        if changed {
            match crate::ui::language::save_theme(&state.theme) {
                Ok(()) => state.error = None,
                Err(error) => state.error = Some(error),
            }
            apply(&state.theme, ui.ctx());
        }
        if ui.button(state.text("mcp.settings_mcp_servers")).clicked() {
            open_mcp = true;
            ui.close();
        }
        if let Some(error) = &state.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
    });
    open_mcp
}
