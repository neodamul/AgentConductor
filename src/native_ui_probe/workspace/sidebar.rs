use super::profile_tools;
use eframe::egui::{self, CornerRadius, Frame, RichText, Stroke, vec2};
use serde_json::Value;

pub fn default_panel_width(viewport_width: f32) -> f32 {
    if viewport_width <= 1000.0 {
        227.0
    } else {
        277.0
    }
}

pub fn panel_id(capture_path: Option<&std::ffi::OsStr>) -> egui::Id {
    capture_path
        .map(|path| egui::Id::unique(("native-agent-roster-capture", path)))
        .unwrap_or_else(|| egui::Id::unique("native-agent-roster"))
}

mod badges;
pub(in crate::native_ui_probe::workspace) mod controls;
mod header;
pub(crate) mod icons;
mod models;
mod quota;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    AddAgent,
    EditAgent(String),
    RemoveAgent(String),
    DiscoverModels(String),
    ConfigureCatalog(String),
    Quota(String),
    MoveAgent(String, usize),
}

pub fn show(
    profiles: &[Value],
    snapshot: &Value,
    language: &str,
    can_edit: bool,
    profile_editing: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let frame_time = ui.input(|input| input.time);
    ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
    if header::show(
        language,
        can_edit && !profile_editing && profiles.len() < 16,
        ui,
    ) {
        event = Some(Event::AddAgent);
    }
    ui.label(
        RichText::new(text(
            language,
            "startup.register_an_executable_authentication_stays_with_its_cli",
        ))
        .small()
        .color(ui.visuals().weak_text_color()),
    );
    ui.separator();
    egui::ScrollArea::vertical()
        .max_height((ui.available_height() - 112.0).max(0.0))
        .show(ui, |ui| {
            if profiles.is_empty() {
                ui.label(
                    egui::RichText::new(text(language, "startup.no_agents_registered_hint"))
                        .small()
                        .color(ui.visuals().weak_text_color()),
                );
            }
            for (index, profile) in profiles.iter().enumerate() {
                let id = profile["id"].as_str().unwrap_or_default();
                let parser = profile["parser"].as_str().unwrap_or("custom");
                if id.is_empty() {
                    continue;
                }
                let executable = profile["executable"].as_str().unwrap_or(parser);
                let permission = profile["permission_mode"].as_str().unwrap_or("default");
                let details = format!("{executable} · {permission}");
                let visible_details =
                    format!("{executable} · {}", permission_label(permission, language));
                let permission_badge = permission_label(permission, language);
                let coordinator = snapshot["config"]["coordinator"] == id;
                let status = status(id, parser, snapshot, now);
                let dot_status = if status == "ready" {
                    "completed"
                } else {
                    status
                };
                let animate = ui.style().animation_time > 0.0;
                let dot =
                    super::super::render::status_color_with_motion(dot_status, frame_time, animate);
                if status == "running" && animate {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_millis(500));
                }
                let card = Frame::new()
                    .fill(ui.visuals().window_fill)
                    .stroke(Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            icons::avatar(
                                ui,
                                id,
                                parser,
                                language,
                                snapshot["icons"][parser].as_str(),
                            );
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(id).strong().color(
                                        ui.visuals().widgets.noninteractive.fg_stroke.color,
                                    ))
                                    .on_hover_text(details.clone())
                                    .accessible_name(format!("{id} · {details}"));
                                    ui.label(RichText::new("●").size(15.0).color(dot))
                                        .accessible_name(format!(
                                            "{id}: {}",
                                            super::super::status_label(status, language)
                                        ));
                                });
                                let count =
                                    profile["max_sessions"].as_u64().unwrap_or(1).to_string();
                                let meta = crate::ui::messages::interpolate(
                                    text(language, "startup.sessions"),
                                    &[("value1", parser), ("value2", &count)],
                                );
                                ui.label(
                                    RichText::new(meta.clone())
                                        .small()
                                        .color(ui.visuals().weak_text_color()),
                                )
                                .accessible_name(meta);
                            });
                        });
                        let saved = snapshot["config"]["profiles"]
                            .as_array()
                            .is_some_and(|items| items.iter().any(|item| item["id"] == id));
                        ui.horizontal_wrapped(|ui| {
                            let model_count =
                                profile["catalog"]["models"].as_array().map_or(0, Vec::len);
                            let model_text = if model_count == 0 {
                                text(language, "startup.models_not_queried").to_owned()
                            } else {
                                crate::ui::messages::interpolate(
                                    text(language, "startup.models_dee84b"),
                                    &[("value1", &model_count.to_string())],
                                )
                            };
                            if coordinator {
                                badges::show(ui, text(language, "native.coordinator"));
                            }
                            badges::show(ui, &model_text);
                            badges::show(ui, &permission_badge);
                        });
                        ui.add(
                            egui::Label::new(
                                RichText::new(executable)
                                    .small()
                                    .color(ui.visuals().weak_text_color()),
                            )
                            .truncate(),
                        )
                        .on_hover_text(details.clone())
                        .accessible_name(visible_details);
                        ui.horizontal_wrapped(|ui| {
                            if let Some(action) = models::show(
                                profile,
                                snapshot,
                                language,
                                can_edit,
                                profile_editing,
                                saved,
                                ui,
                            ) {
                                event = Some(action);
                            }
                            if matches!(parser, "claude" | "codex")
                                && let Some(action) =
                                    quota::show(id, snapshot, language, can_edit && saved, ui)
                            {
                                event = Some(action);
                            }
                            if let Some(fallback) = profile["fallback"]["agent"].as_str() {
                                let fallback = crate::ui::messages::interpolate(
                                    text(language, "startup.fallback"),
                                    &[("value1", fallback)],
                                );
                                ui.label(RichText::new(fallback.clone()).small())
                                    .accessible_name(fallback);
                            }
                            if let Some(action) = controls::show(
                                id,
                                snapshot,
                                language,
                                can_edit,
                                profile_editing,
                                (index, profiles.len()),
                                ui,
                            ) {
                                event = Some(action);
                            }
                        });
                    });
                if can_edit
                    && !profile_editing
                    && let Some(source) = controls::dropped(&card.response)
                    && source != id
                {
                    event = Some(Event::MoveAgent(source, index));
                }
            }
        });
    event
}

fn status(id: &str, parser: &str, snapshot: &Value, now: i64) -> &'static str {
    let coordinator = snapshot["config"]["coordinator"] == id;
    let job = &snapshot["job"];
    if coordinator && job["kind"] == "plan" && job["state"] == "running" {
        return "running";
    }
    if coordinator && job["kind"] == "plan" && job["state"] == "failed" {
        return "failed";
    }
    match snapshot["agent_activity"][id].as_str() {
        Some("running") => return "running",
        Some("waiting_approval") => return "waiting_approval",
        Some("waiting_tool_approval") => return "waiting_tool_approval",
        Some("waiting_availability") => return "waiting_availability",
        Some("blocked") => return "blocked",
        Some("failed") => return "failed",
        _ => {}
    }
    let quota = &snapshot["quotas"][id];
    if matches!(parser, "claude" | "codex") && crate::quota::restriction(quota, now).is_some() {
        return "waiting_availability";
    }
    if matches!(
        quota["status"].as_str(),
        Some("stale" | "unavailable" | "error")
    ) {
        return "pending";
    }
    "ready"
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn permission_label(mode: &str, language: &str) -> String {
    match mode {
        "full_access" => text(language, "state.full_access").to_owned(),
        "operator_auto_approve" => text(language, "state.operator_auto_approve").to_owned(),
        "external_sandbox" => text(language, "state.external_sandbox").to_owned(),
        "unverified" => text(language, "state.unverified").to_owned(),
        _ => mode.to_owned(),
    }
}

fn message(language: &str, id: &'static str, agent: &str) -> String {
    crate::ui::messages::interpolate(text(language, id), &[("value1", agent)])
}
