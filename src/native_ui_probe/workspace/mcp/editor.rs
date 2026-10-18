use eframe::egui::{self, TextEdit};
use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event {
    Save(String),
}

#[derive(Default)]
pub struct State {
    open: bool,
    settings: String,
    show_examples: bool,
    server: String,
    discovery_action: Option<String>,
    discovering: Option<String>,
    discovery_error: Option<String>,
    catalog: Option<(String, Value, String)>,
    discovery_config: Value,
}

impl State {
    pub fn open(&mut self, snapshot: Option<&Value>) {
        let Some(snapshot) = snapshot else { return };
        self.show_examples = super::has_no_mcp_servers(snapshot);
        self.settings = super::settings_text(snapshot);
        let servers = super::server_names(snapshot);
        if !servers.contains(&self.server) {
            self.server = servers.first().cloned().unwrap_or_default();
        }
        self.discovery_error = None;
        self.open = true;
    }

    pub(in crate::native_ui_probe) fn take_discovery(&mut self) -> Option<String> {
        self.discovery_action.take()
    }

    pub(in crate::native_ui_probe) fn discovering(&self) -> bool {
        self.discovering.is_some()
    }

    pub(in crate::native_ui_probe) fn begin_discovery(&mut self, server: &str, snapshot: &Value) {
        self.server = server.to_owned();
        self.discovering = Some(server.to_owned());
        self.discovery_error = None;
        self.catalog = None;
        self.discovery_config = snapshot["config"]["mcp"][server].clone();
    }

    pub(in crate::native_ui_probe) fn discovery_result(&mut self, result: &Value) {
        let server = result["server"].as_str().unwrap_or_default();
        let rendered =
            serde_json::to_string_pretty(&result["tools"]).unwrap_or_else(|_| "[]".to_owned());
        self.catalog = Some((server.to_owned(), self.discovery_config.clone(), rendered));
        self.discovering = None;
        self.discovery_error = None;
    }

    pub(in crate::native_ui_probe) fn discovery_failed(&mut self, error: &str) {
        self.discovering = None;
        self.discovery_error = Some(error.to_owned());
    }

    pub(in crate::native_ui_probe) fn saved(&mut self) {
        self.open = false;
    }

    pub fn show(
        &mut self,
        snapshot: Option<&Value>,
        language: &str,
        can_save: bool,
        ui: &mut egui::Ui,
    ) -> Option<Event> {
        if !self.open {
            return None;
        }
        let text = |id| crate::ui::messages::in_language(language, id);
        let mut open = self.open;
        let mut event = None;
        egui::Window::new(text("mcp.mcp_server_registration"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(780.0)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.heading(text("mcp.register_mcp_servers"));
                ui.label(text("mcp.paste_opencode_json_jsonc_with_an_mcp_map"));
                ui.label(text("mcp.server_definitions_and_optional_source_bindings"));
                if self.show_examples {
                    ui.label(text(
                        "mcp.examples_are_commented_replace_placeholders_and_remove_comments",
                    ));
                }
                ui.add_sized(
                    [ui.available_width(), 260.0],
                    TextEdit::multiline(&mut self.settings)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(14),
                );
                ui.label(text(
                    "mcp.local_commands_run_only_when_you_request_discovery",
                ));
                self.show_discovery(snapshot, language, can_save, ui);
                if ui
                    .add_enabled(
                        can_save && snapshot.is_some() && self.settings.len() < 64 * 1024,
                        egui::Button::new(text("mcp.save_mcp_settings")),
                    )
                    .accessible_name(text("mcp.save_mcp_settings"))
                    .clicked()
                {
                    event = Some(Event::Save(self.settings.clone()));
                }
            });
        self.open = open;
        event
    }

    fn show_discovery(
        &mut self,
        snapshot: Option<&Value>,
        language: &str,
        can_save: bool,
        ui: &mut egui::Ui,
    ) {
        let Some(snapshot) = snapshot else { return };
        let names = super::server_names(snapshot);
        if names.is_empty() {
            ui.label(crate::ui::messages::in_language(
                language,
                "mcp.catalog_empty",
            ));
            return;
        }
        let text = |id| crate::ui::messages::in_language(language, id);
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("native-mcp-discovery-server")
                .selected_text(&self.server)
                .show_ui(ui, |ui| {
                    for name in &names {
                        ui.selectable_value(&mut self.server, name.clone(), name);
                    }
                });
            let enabled = can_save
                && !self.discovering()
                && snapshot["config"]["mcp"][&self.server]["enabled"] != false;
            if ui
                .add_enabled(enabled, egui::Button::new(text("mcp.discover_tools")))
                .accessible_name(text("mcp.discover_tools"))
                .clicked()
            {
                let server = self.server.clone();
                self.discovery_action = Some(server.clone());
                self.begin_discovery(&server, snapshot);
            }
        });
        if self.discovering() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(text("mcp.loading_tools"));
            });
        }
        if let Some(error) = &self.discovery_error {
            ui.colored_label(
                egui::Color32::DARK_RED,
                crate::ui::messages::render("mcp.lookup_failed", &[("value1", error)]),
            );
        }
        if let Some((server, config, catalog)) = &self.catalog
            && self.server == *server
            && snapshot["config"]["mcp"][server] == *config
        {
            egui::ScrollArea::vertical()
                .max_height(180.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(catalog).monospace()).selectable(true),
                    );
                });
        }
    }
}
