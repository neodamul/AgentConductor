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
    let dark = theme == "dark";
    let (bg, surface, rail, fill, line, ink, soft, accent, accent_ink, accent_soft, accent_text) =
        if dark {
            (
                rgb(0x14, 0x1a, 0x19),
                rgb(0x1a, 0x22, 0x20),
                rgb(0x11, 0x17, 0x16),
                rgb(0x22, 0x2c, 0x29),
                rgb(0x2a, 0x34, 0x31),
                rgb(0xdd, 0xe6, 0xe1),
                rgb(0x8e, 0x9c, 0x96),
                rgb(0x8c, 0xc9, 0xa8),
                rgb(0x0f, 0x1a, 0x15),
                rgb(0x23, 0x38, 0x2e),
                rgb(0xa9, 0xdc, 0xc1),
            )
        } else {
            (
                rgb(0xf2, 0xf2, 0xf6),
                egui::Color32::WHITE,
                rgb(0xeb, 0xeb, 0xf0),
                rgb(0xe6, 0xe6, 0xec),
                rgb(0xdc, 0xdc, 0xe2),
                rgb(0x1c, 0x1c, 0x1e),
                rgb(0x60, 0x60, 0x67),
                rgb(0x0a, 0x66, 0xd8),
                egui::Color32::WHITE,
                rgb(0xe3, 0xed, 0xfb),
                rgb(0x0a, 0x56, 0xb8),
            )
        };
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    visuals.panel_fill = bg;
    visuals.window_fill = surface;
    visuals.extreme_bg_color = surface;
    visuals.faint_bg_color = rail;
    visuals.code_bg_color = rail;
    visuals.weak_text_color = Some(soft);
    visuals.hyperlink_color = accent_text;
    visuals.selection.bg_fill = accent_soft;
    visuals.warn_fg_color = if dark {
        rgb(0xe2, 0xc0, 0x6a)
    } else {
        rgb(0x9a, 0x5a, 0x00)
    };
    visuals.error_fg_color = if dark {
        rgb(0xe0, 0x7a, 0x6a)
    } else {
        rgb(0xc4, 0x31, 0x2a)
    };
    visuals.widgets.noninteractive.bg_fill = surface;
    visuals.widgets.noninteractive.fg_stroke.color = ink;
    visuals.widgets.inactive.bg_fill = fill;
    visuals.widgets.inactive.weak_bg_fill = fill;
    visuals.widgets.inactive.fg_stroke.color = ink;
    visuals.widgets.inactive.bg_stroke.color = line;
    visuals.widgets.hovered.bg_fill = accent_soft;
    visuals.widgets.hovered.weak_bg_fill = accent_soft;
    visuals.widgets.hovered.fg_stroke.color = ink;
    visuals.widgets.hovered.bg_stroke.color = accent;
    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.active.weak_bg_fill = accent;
    visuals.widgets.active.fg_stroke.color = accent_ink;
    visuals.widgets.active.bg_stroke.color = accent;
    visuals.widgets.open.bg_fill = accent_soft;
    visuals.widgets.open.weak_bg_fill = accent_soft;
    visuals.widgets.open.fg_stroke.color = ink;
    visuals.widgets.open.bg_stroke.color = accent;
    context.set_visuals(visuals);
}

fn rgb(red: u8, green: u8, blue: u8) -> egui::Color32 {
    egui::Color32::from_rgb(red, green, blue)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsAction {
    Mcp,
    Messenger,
}

pub fn show(state: &mut State, ui: &mut Ui) -> Option<SettingsAction> {
    let mut action = None;
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
            action = Some(SettingsAction::Mcp);
            ui.close();
        }
        if ui
            .button(state.text("startup.messenger_notifications"))
            .clicked()
        {
            action = Some(SettingsAction::Messenger);
            ui.close();
        }
        if let Some(error) = &state.error {
            ui.colored_label(egui::Color32::DARK_RED, error);
        }
    });
    action
}
