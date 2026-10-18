use eframe::egui;
use serde_json::Value;

mod suggested;
pub use suggested::suggested;

pub fn show(
    rules: &mut crate::roles::RoleRules,
    profiles: &[Value],
    editable: bool,
    ui: &mut egui::Ui,
) {
    show_in_language(rules, profiles, editable, "en", ui);
}

pub fn show_in_language(
    rules: &mut crate::roles::RoleRules,
    profiles: &[Value],
    editable: bool,
    language: &str,
    ui: &mut egui::Ui,
) {
    let words = [
        "startup.role",
        "startup.agent",
        "startup.model",
        "startup.alternate_1b9971",
        "startup.planning",
        "startup.planning_coordinator_workspace_settings",
        "startup.default_model",
        "startup.cli_default_query_models_to_choose",
    ]
    .map(|key| text(language, key));
    let [
        role_header,
        agent_header,
        model_header,
        alternate,
        planning,
        coordinator,
        default,
        query,
    ] = words;
    egui::Grid::new("native-role-rules")
        .num_columns(5)
        .spacing(egui::vec2(10.0, 8.0))
        .show(ui, |ui| {
            for label in [
                role_header,
                agent_header,
                model_header,
                alternate,
                model_header,
            ] {
                ui.label(egui::RichText::new(label).strong())
                    .accessible_name(label);
            }
            ui.end_row();
            ui.label(planning).accessible_name(planning);
            ui.label(coordinator).accessible_name(coordinator);
            ui.label("");
            ui.label("");
            ui.label("");
            ui.end_row();
            for role in crate::roles::ROLES {
                let name = role_name(language, role);
                ui.label(name).accessible_name(name);
                let entries = rules.entry(role.into()).or_default();
                let primary = entries.first();
                let alternate_entry = entries.get(1);
                let mut primary_agent =
                    primary.map_or_else(String::new, |entry| entry.agent.clone());
                let mut primary_model = primary.and_then(|entry| entry.model.clone());
                let mut alternate_agent =
                    alternate_entry.map_or_else(String::new, |entry| entry.agent.clone());
                let mut alternate_model = alternate_entry.and_then(|entry| entry.model.clone());
                let saved_primary_agent = primary_agent.clone();
                agent_select(
                    ui,
                    role,
                    "primary",
                    name,
                    "",
                    &mut primary_agent,
                    &alternate_agent,
                    profiles,
                    editable,
                    language,
                );
                if saved_primary_agent != primary_agent {
                    primary_model = None;
                }
                model_select(
                    ui,
                    role,
                    "primary",
                    name,
                    model_header,
                    &primary_agent,
                    &mut primary_model,
                    profiles,
                    editable,
                    language,
                    default,
                    query,
                );
                let saved_alternate_agent = alternate_agent.clone();
                agent_select(
                    ui,
                    role,
                    "alternate",
                    name,
                    alternate,
                    &mut alternate_agent,
                    &primary_agent,
                    profiles,
                    editable,
                    language,
                );
                if saved_alternate_agent != alternate_agent {
                    alternate_model = None;
                }
                model_select(
                    ui,
                    role,
                    "alternate",
                    name,
                    model_header,
                    &alternate_agent,
                    &mut alternate_model,
                    profiles,
                    editable,
                    language,
                    default,
                    query,
                );
                ui.end_row();
                retain_slots(
                    entries,
                    [
                        (primary_agent, primary_model),
                        (alternate_agent, alternate_model),
                    ],
                );
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn agent_select(
    ui: &mut egui::Ui,
    role: &'static str,
    slot: &'static str,
    role_name: &str,
    slot_name: &str,
    selected: &mut String,
    other: &str,
    profiles: &[Value],
    editable: bool,
    language: &str,
) {
    ui.add_enabled_ui(editable, |ui| {
        egui::ComboBox::from_id_salt((role, slot, "agent"))
            .selected_text(if selected.is_empty() {
                "—"
            } else {
                selected.as_str()
            })
            .width(150.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(selected, String::new(), "—");
                for profile in profiles {
                    let id = profile["id"].as_str().unwrap_or_default();
                    if !id.is_empty() && (id == selected || id != other) {
                        ui.selectable_value(selected, id.to_owned(), id);
                    }
                }
                if !selected.is_empty() && !profiles.iter().any(|p| p["id"] == *selected) {
                    ui.selectable_value(selected, selected.clone(), format!("{} ⚠", selected));
                }
            })
            .response
            .accessible_name(if slot_name.is_empty() {
                format!("{role_name} {}", text(language, "startup.agent"))
            } else {
                format!(
                    "{role_name} {} {}",
                    alternate_word(language),
                    text(language, "startup.agent")
                )
            });
    });
}

#[allow(clippy::too_many_arguments)]
fn model_select(
    ui: &mut egui::Ui,
    role: &'static str,
    slot: &'static str,
    role_name: &str,
    model_name: &str,
    agent: &str,
    model: &mut Option<String>,
    profiles: &[Value],
    editable: bool,
    language: &str,
    default: &str,
    query: &str,
) {
    let profile = profiles.iter().find(|profile| profile["id"] == agent);
    let models = profile
        .and_then(|profile| profile["catalog"]["models"].as_array())
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    let label = format!(
        "{role_name} {}{model_name}",
        if slot == "alternate" {
            format!("{} ", alternate_word(language))
        } else {
            String::new()
        }
    );
    ui.add_enabled_ui(editable && !agent.is_empty(), |ui| {
        egui::ComboBox::from_id_salt((role, slot, "model"))
            .selected_text(model.as_deref().unwrap_or(if models.is_empty() {
                query
            } else {
                default
            }))
            .width(150.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(model, None, default);
                for value in &models {
                    ui.selectable_value(model, Some((*value).to_owned()), *value);
                }
            })
            .response
            .accessible_name(label);
    });
    if agent.is_empty() {
        ui.label("");
    }
}

fn retain_slots(entries: &mut Vec<crate::roles::RoleAgent>, slots: [(String, Option<String>); 2]) {
    let extra = entries.iter().skip(2).cloned().collect::<Vec<_>>();
    entries.clear();
    for (agent, model) in slots {
        if !agent.is_empty() && !entries.iter().any(|entry| entry.agent == agent) {
            entries.push(crate::roles::RoleAgent { agent, model });
        }
    }
    entries.extend(extra);
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn alternate_word(language: &str) -> &'static str {
    if language == "ko" {
        "대체"
    } else {
        "alternate"
    }
}

fn role_name(language: &str, role: &'static str) -> &'static str {
    if language != "ko" {
        return role;
    }
    crate::ui::messages::in_language(
        language,
        match role {
            "verification" => "startup.verification",
            "development" => "startup.development",
            "documentation" => "startup.documentation",
            "knowledge" => "startup.knowledge",
            "testing" => "startup.testing",
            _ => return role,
        },
    )
}
