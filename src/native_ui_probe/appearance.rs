use eframe::egui::{self, Ui};

pub struct State {
    theme: String,
    error: Option<String>,
}

impl State {
    pub fn load() -> Self {
        Self {
            theme: crate::ui::language::load_theme(),
            error: None,
        }
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

pub fn show(state: &mut State, ui: &mut Ui) {
    ui.menu_button("Appearance", |ui| {
        let mut changed = false;
        changed |= ui
            .selectable_value(&mut state.theme, "light".into(), "Light")
            .clicked();
        changed |= ui
            .selectable_value(&mut state.theme, "dark".into(), "Dark")
            .clicked();
        if changed {
            match crate::ui::language::save_theme(&state.theme) {
                Ok(()) => state.error = None,
                Err(error) => state.error = Some(error),
            }
            apply(&state.theme, ui.ctx());
        }
        if let Some(error) = &state.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
    });
}
