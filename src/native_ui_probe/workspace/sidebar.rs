use super::profile_tools;
use eframe::egui::{self, Color32, CornerRadius, Frame, RichText, Stroke, vec2};
use serde_json::Value;

mod controls;
mod models;

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
    ui.horizontal(|ui| {
        ui.heading(text(language, "startup.cli_agents"))
            .accessible_name(text(language, "startup.cli_agents"));
        if ui
            .add_enabled(
                can_edit && !profile_editing && profiles.len() < 16,
                egui::Button::new(text(language, "startup.add")),
            )
            .accessible_name(text(language, "startup.register_cli_agent"))
            .clicked()
        {
            event = Some(Event::AddAgent);
        }
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
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
            let status = status(id, parser, snapshot, now);
            let dot_status = if status == "ready" {
                "completed"
            } else {
                status
            };
            let dot = super::super::status_color(dot_status, frame_time);
            if status == "running" {
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
                        avatar(ui, id, parser, language);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(id)
                                        .strong()
                                        .color(ui.visuals().widgets.noninteractive.fg_stroke.color),
                                )
                                .on_hover_text(id)
                                .accessible_name(id);
                                ui.label(RichText::new("●").size(15.0).color(dot))
                                    .accessible_name(format!(
                                        "{id}: {}",
                                        super::super::status_label(status, language)
                                    ));
                            });
                            let count = profile["max_sessions"].as_u64().unwrap_or(1).to_string();
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
                        ui.label(
                            RichText::new(model_text.clone())
                                .small()
                                .color(ui.visuals().weak_text_color()),
                        )
                        .accessible_name(model_text);
                        let saved = snapshot["config"]["profiles"]
                            .as_array()
                            .is_some_and(|items| items.iter().any(|item| item["id"] == id));
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
                        if matches!(parser, "claude" | "codex") {
                            let badge = &snapshot["quota_badges"][id];
                            let quota = badge["text"].as_str().unwrap_or("—");
                            let title = badge["title"].as_str().unwrap_or_default();
                            ui.label(
                                RichText::new(quota)
                                    .small()
                                    .strong()
                                    .monospace()
                                    .color(ui.visuals().text_color()),
                            )
                            .on_hover_text(title)
                            .accessible_name(if title.is_empty() {
                                quota.to_owned()
                            } else {
                                format!("{quota}: {title}")
                            });
                            let busy = profile_tools::metadata_active(snapshot, "quota", id);
                            let refresh = crate::ui::messages::interpolate(
                                text(language, "startup.refresh_quota"),
                                &[("value1", id)],
                            );
                            if busy {
                                ui.spinner();
                            } else if ui
                                .add_enabled(can_edit && saved, egui::Button::new("↻").small())
                                .accessible_name(refresh.clone())
                                .on_hover_text(refresh)
                                .clicked()
                            {
                                event = Some(Event::Quota(id.to_owned()));
                            }
                        }
                    });
                    let executable = profile["executable"].as_str().unwrap_or(parser);
                    let permission = profile["permission_mode"].as_str().unwrap_or("default");
                    let details = format!("{executable} · {permission}");
                    ui.label(RichText::new(details.clone()).small().monospace())
                        .accessible_name(details);
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

fn avatar(ui: &mut egui::Ui, id: &str, parser: &str, language: &str) {
    let (rect, response) = ui.allocate_exact_size(vec2(34.0, 34.0), egui::Sense::hover());
    let (fill, glyph) = match parser {
        "claude" => (Color32::from_rgb(177, 100, 76), "CC".to_owned()),
        "codex" => (Color32::from_rgb(52, 103, 91), "CX".to_owned()),
        "opencode" => (Color32::from_rgb(74, 82, 106), "OC".to_owned()),
        _ => (
            Color32::from_rgb(96, 96, 103),
            id.chars().take(2).collect::<String>().to_uppercase(),
        ),
    };
    ui.painter().circle_filled(rect.center(), 17.0, fill);
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyph,
        egui::FontId::proportional(10.0),
        Color32::WHITE,
    );
    let label = crate::ui::messages::interpolate(
        text(language, "startup.agent_icon_accessible"),
        &[("value1", id)],
    );
    response.accessible_name(label);
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

fn message(language: &str, id: &'static str, agent: &str) -> String {
    crate::ui::messages::interpolate(text(language, id), &[("value1", agent)])
}
