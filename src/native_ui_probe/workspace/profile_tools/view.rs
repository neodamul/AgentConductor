use super::{Editor, Event, metadata_active, remove_profile};
use eframe::egui;
use serde_json::Value;

mod form;

pub struct RenderContext<'a> {
    pub snapshot: &'a Value,
    pub editable: bool,
    pub language: &'a str,
    pub show_registry: bool,
}

pub fn show_in_language(
    profiles: &mut Vec<Value>,
    coordinator: &mut String,
    roles: &mut crate::roles::RoleRules,
    editor: &mut Editor,
    context: RenderContext<'_>,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let RenderContext {
        snapshot,
        editable,
        language,
        show_registry,
    } = context;
    let mut event = None;
    if snapshot["custom_registry_invalid"] == true {
        ui.colored_label(
            ui.visuals().error_fg_color,
            text(language, "startup.custom_cli_registry_unreadable"),
        );
    }
    if show_registry {
        ui.separator();
        ui.label(text(language, "startup.registered_agents"))
            .accessible_name(text(language, "startup.registered_agents"));
        for profile in profiles.iter_mut() {
            let id = profile["id"].as_str().unwrap_or_default();
            let parser = profile["parser"].as_str().unwrap_or("custom");
            if id.is_empty() {
                continue;
            }
            let saved = snapshot["config"]["profiles"]
                .as_array()
                .is_some_and(|saved| saved.iter().any(|item| item["id"] == id));
            let models = profile["catalog"]["models"].as_array().map_or(0, Vec::len);
            let mut open = false;
            let removal_busy =
                metadata_active(snapshot, "quota", id) || metadata_active(snapshot, "catalog", id);
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(id)
                        .strong()
                        .color(ui.visuals().widgets.noninteractive.fg_stroke.color),
                );
                if ui
                    .add_enabled(editable, egui::Button::new(text(language, "startup.edit")))
                    .accessible_name(message(
                        language,
                        "startup.edit_agent_named",
                        &[("value1", id)],
                    ))
                    .clicked()
                {
                    open = true;
                }
                if ui
                    .add_enabled(
                        editable && editor.id.is_none() && !removal_busy,
                        egui::Button::new("🗑"),
                    )
                    .accessible_name(message(
                        language,
                        "startup.remove_agent_named",
                        &[("value1", id)],
                    ))
                    .clicked()
                {
                    editor.request_remove(id);
                }
                let model_count = if models == 0 {
                    text(language, "startup.models_not_queried").to_owned()
                } else {
                    message(
                        language,
                        "startup.models_dee84b",
                        &[("value1", &models.to_string())],
                    )
                };
                ui.label(&model_count).accessible_name(model_count);
                let catalog_busy = metadata_active(snapshot, "catalog", id);
                if parser != "custom"
                    && ui
                        .add_enabled(
                            editable && saved && !catalog_busy,
                            egui::Button::new(format!("{} ↻", text(language, "startup.models"))),
                        )
                        .accessible_name(message(
                            language,
                            "startup.refresh_models_for_agent",
                            &[("value1", id)],
                        ))
                        .clicked()
                {
                    event = Some(Event::Discover(id.into()));
                }
                let quota_refresh = matches!(parser, "claude" | "codex");
                let badge = &snapshot["quota_badges"][id];
                if quota_refresh || !badge["text"].is_null() {
                    let busy = metadata_active(snapshot, "quota", id);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        quota_badge(badge, language, ui);
                        if quota_refresh {
                            if busy {
                                ui.spinner();
                            } else if ui
                                .add_enabled(editable && saved, egui::Button::new("↻").small())
                                .accessible_name(message(
                                    language,
                                    "startup.refresh_quota_for_agent",
                                    &[("value1", id)],
                                ))
                                .on_hover_text(message(
                                    language,
                                    "startup.refresh_quota_for_agent",
                                    &[("value1", id)],
                                ))
                                .clicked()
                            {
                                event = Some(Event::Quota(id.into()));
                            }
                        }
                    });
                }
                if catalog_busy {
                    ui.spinner();
                }
            });
            if open {
                editor.open(profile);
            }
        }
    }
    if let Some(id) = editor.remove_confirmation.clone() {
        ui.separator();
        ui.label(message(
            language,
            "startup.confirm_remove_agent_message",
            &[("value1", &id)],
        ));
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    editable,
                    egui::Button::new(text(language, "startup.confirm_remove_agent")),
                )
                .accessible_name(message(
                    language,
                    "startup.confirm_remove_agent_named",
                    &[("value1", &id)],
                ))
                .clicked()
                && remove_profile(profiles, coordinator, roles, editor, &id)
            {
                event = Some(Event::SaveProfiles);
            }
            if ui
                .button(text(language, "startup.cancel_remove_agent"))
                .accessible_name(message(
                    language,
                    "startup.cancel_remove_agent_named",
                    &[("value1", &id)],
                ))
                .clicked()
            {
                editor.remove_confirmation = None;
            }
        });
    }
    if let Some(catalog_event) = super::catalog_view::show(editor, editable, language, ui) {
        event = Some(catalog_event);
    }
    if editor.id.is_some() {
        let title = if editor.is_registering() {
            text(language, "startup.register_cli_agent").to_owned()
        } else {
            message(
                language,
                "startup.edit_agent_named",
                &[("value1", editor.id.as_deref().unwrap_or_default())],
            )
        };
        let mut open = true;
        let window = egui::Window::new(title)
            .id(egui::Id::NULL.with("native-agent-profile-editor"))
            .open(&mut open)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .default_width(780.0)
            .collapsible(false)
            .resizable(true)
            .show(ui.ctx(), |ui| {
                ui.set_min_width(740.0);
                ui.set_max_width(780.0);
                form::show(profiles, snapshot, editable, editor, language, ui)
            });
        if editable && !open {
            editor.cancel();
        }
        if let Some(window) = window {
            event = window.inner.flatten().or(event);
        }
    }
    event
}

fn quota_badge(badge: &Value, language: &str, ui: &mut egui::Ui) {
    if let Some(value) = badge["text"].as_str() {
        let title = badge["title"].as_str().unwrap_or_default();
        ui.label(value)
            .on_hover_text(title)
            .accessible_name(if title.is_empty() {
                value.to_owned()
            } else {
                format!("{value}: {title}")
            });
    } else {
        let unavailable = text(language, "room.quota_unavailable");
        ui.label("—")
            .on_hover_text(unavailable)
            .accessible_name(format!("—: {unavailable}"));
    }
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn message(language: &str, id: &'static str, params: &[(&str, &str)]) -> String {
    crate::ui::messages::interpolate(text(language, id), params)
}
