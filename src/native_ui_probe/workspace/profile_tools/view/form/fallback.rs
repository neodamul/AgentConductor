use crate::native_ui_probe::workspace::profile_tools::{Editor, Event, metadata_active};
use eframe::egui;
use serde_json::Value;

pub(super) fn show(
    profiles: &[Value],
    snapshot: &Value,
    editable: bool,
    editor: &mut Editor,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let current = editor.fallback_agent.clone();
    let previous_model = editor.fallback_model.clone();
    ui.columns(2, |columns| {
        let agent_label = text(language, "startup.fallback_agent");
        columns[0].label(agent_label);
        columns[0].add_enabled_ui(editable, |ui| {
            egui::ComboBox::from_id_salt("native-fallback-agent")
                .selected_text(
                    current
                        .as_deref()
                        .unwrap_or(text(language, "startup.no_fallback")),
                )
                .width(ui.available_width())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut editor.fallback_agent,
                        None,
                        text(language, "startup.no_fallback"),
                    );
                    for profile in profiles {
                        let id = profile["id"].as_str().unwrap_or_default();
                        if id.is_empty()
                            || id == editor.agent_id
                            || id == editor.origin["id"]
                            || quota_key(profile) == quota_key(&editor.base_profile)
                        {
                            continue;
                        }
                        ui.selectable_value(&mut editor.fallback_agent, Some(id.to_owned()), id);
                    }
                })
                .response
                .accessible_name(agent_label);
        });
        if editor.fallback_agent != current {
            let next = editor.fallback_agent.clone();
            editor.set_fallback(next.as_deref(), None);
        }
        if let Some(target) = editor.fallback_agent.as_deref().and_then(|id| {
            profiles
                .iter()
                .find(|profile| profile["id"].as_str() == Some(id))
        }) {
            let label = text(language, "startup.fallback_model");
            let models = target["catalog"]["models"]
                .as_array()
                .map_or(&[][..], Vec::as_slice);
            columns[1].label(label);
            columns[1].add_enabled_ui(editable, |ui| {
                egui::ComboBox::from_id_salt("native-fallback-model")
                    .selected_text(
                        editor
                            .fallback_model
                            .as_deref()
                            .unwrap_or(text(language, "startup.automatic_model")),
                    )
                    .width(ui.available_width())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut editor.fallback_model,
                            None,
                            text(language, "startup.automatic_model"),
                        );
                        for model in models.iter().filter_map(Value::as_str) {
                            ui.selectable_value(
                                &mut editor.fallback_model,
                                Some(model.to_owned()),
                                model,
                            );
                        }
                    })
                    .response
                    .accessible_name(label);
            });
        }
    });
    if editor.fallback_model != previous_model {
        editor.error = None;
    }
    let target = editor.fallback_agent.as_ref().and_then(|id| {
        profiles
            .iter()
            .find(|profile| profile["id"].as_str() == Some(id.as_str()))
    });
    let mut event: Option<Event> = None;
    if editable
        && let Some(agent) = editor.fallback_agent.clone()
        && !metadata_active(snapshot, "catalog", &agent)
        && let Some(request) = editor.fallback_catalog_event(
            profiles,
            snapshot["config"]["profiles"]
                .as_array()
                .map_or(&[], Vec::as_slice),
        )
    {
        event = Some(request);
    }
    if let Some(target) = target {
        let models = target["catalog"]["models"]
            .as_array()
            .map_or(&[][..], Vec::as_slice);
        let agent = editor.fallback_agent.as_deref().unwrap_or_default();
        if metadata_active(snapshot, "catalog", agent) {
            ui.spinner();
            ui.label(text(language, "startup.models_loading"));
        } else if models.is_empty() {
            ui.label(text(language, "startup.models_not_queried"));
        }
        ui.label(text(language, "startup.fallback_model_explanation"));
    }
    event
}

pub(super) fn validation_error(
    profiles: &[Value],
    editor: &Editor,
    language: &str,
) -> Option<String> {
    let id = editor.fallback_agent.as_ref()?;
    if id == &editor.agent_id || id == &editor.origin["id"] {
        return Some(text(language, "startup.fallback_agent_invalid").into());
    }
    let Some(target) = profiles
        .iter()
        .find(|profile| profile["id"].as_str() == Some(id.as_str()))
    else {
        return Some(text(language, "startup.fallback_agent_missing").into());
    };
    if quota_key(target) == quota_key(&editor.base_profile) {
        return Some(text(language, "startup.fallback_agent_invalid").into());
    }
    if let Some(model) = &editor.fallback_model
        && !target["catalog"]["models"]
            .as_array()
            .is_some_and(|models| models.iter().any(|id| id == model))
    {
        return Some(text(language, "startup.fallback_model_missing").into());
    }
    None
}

fn quota_key(profile: &Value) -> &str {
    profile["quota_group"]
        .as_str()
        .filter(|group| !group.is_empty())
        .or_else(|| profile["id"].as_str())
        .unwrap_or_default()
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}
