use super::Event;
use eframe::egui::{self, RichText};
use serde_json::Value;

pub(super) fn show(
    agent: &str,
    snapshot: &Value,
    language: &str,
    can_refresh: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let badge = &snapshot["quota_badges"][agent];
    let quota = badge["text"].as_str().unwrap_or("—");
    let title = badge["title"].as_str().unwrap_or_default();
    let mut action = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 5.0;
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
        let busy = super::super::profile_tools::metadata_active(snapshot, "quota", agent);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let elapsed = super::super::profile_tools::quota_elapsed(snapshot, agent, now);
        if busy {
            ui.spinner();
            if let Some(elapsed) = elapsed {
                let running = crate::ui::messages::interpolate(
                    crate::ui::messages::in_language(language, "native.quota_query_running_for"),
                    &[("value1", &elapsed)],
                );
                ui.label(RichText::new(elapsed).small().monospace())
                    .accessible_name(running.clone())
                    .on_hover_text(running);
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs(1));
            }
        } else {
            let refresh = crate::ui::messages::interpolate(
                crate::ui::messages::in_language(language, "startup.refresh_quota"),
                &[("value1", agent)],
            );
            if ui
                .add_enabled(can_refresh, egui::Button::new("↻").small())
                .accessible_name(refresh.clone())
                .on_hover_text(refresh)
                .clicked()
            {
                action = Some(Event::Quota(agent.to_owned()));
            }
            let job = &snapshot["metadata_job"];
            let error = job["error"].as_str().filter(|error| {
                job["state"] == "failed"
                    && job["kind"] == format!("quota:{agent}")
                    && !error.is_empty()
            });
            if let Some(error) = error {
                ui.label(
                    RichText::new("!")
                        .strong()
                        .color(ui.visuals().error_fg_color),
                )
                .accessible_name(error.to_owned())
                .on_hover_text(error);
            }
        }
    });
    action
}
