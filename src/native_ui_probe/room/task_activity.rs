use eframe::egui::{self, RichText};
use serde_json::Value;
#[path = "task_activity/details.rs"]
mod details;
#[path = "task_activity/render.rs"]
mod render;
mod transcript;
use self::render::{entry, localized};
use self::transcript::tool_line;
pub use self::transcript::{Entry, Tone, event_entries, transcript};
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Tab {
    #[default]
    Activity,
    Logs,
    Lessons,
}

pub fn request(index: usize) -> Vec<u8> {
    format!(r#"{{"kind":"task","index":{index}}}"#).into_bytes()
}

pub fn show(
    tab: &mut Tab,
    task_id: &str,
    provider: &str,
    plan: &Value,
    details: Option<&Value>,
    language: &str,
    ui: &mut egui::Ui,
) {
    let text = |key| crate::ui::messages::in_language(language, key);
    ui.horizontal(|ui| {
        for (choice, key) in [
            (Tab::Activity, "room.activity"),
            (Tab::Logs, "room.raw_logs"),
            (Tab::Lessons, "room.lessons"),
        ] {
            if ui
                .add(egui::Button::new(text(key)).selected(*tab == choice))
                .clicked()
            {
                *tab = choice;
            }
        }
    });
    ui.separator();
    let details = details.filter(|details| details["task"]["id"].as_str() == Some(task_id));
    if details.is_none() {
        ui.label(text("room.loading_task_details"))
            .accessible_name(text("room.loading_task_details"));
        return;
    }
    let details = details.unwrap();
    match tab {
        Tab::Activity => activity(provider, plan, details, language, ui),
        Tab::Logs => details::logs(details, language, ui),
        Tab::Lessons => details::lessons(task_id, details, language, ui),
    }
}

fn activity(provider: &str, plan: &Value, details: &Value, language: &str, ui: &mut egui::Ui) {
    let text = |key| crate::ui::messages::in_language(language, key);
    entry(
        ui,
        Entry {
            label: text("room.assigned_task").into(),
            text: plan["prompt"].as_str().unwrap_or_default().into(),
            tone: Tone::Muted,
        },
    );
    if details["task"]["feedback"].is_string() {
        entry(
            ui,
            Entry {
                label: text("room.you_latest_review").into(),
                text: details["task"]["feedback"]
                    .as_str()
                    .unwrap_or_default()
                    .into(),
                tone: Tone::Muted,
            },
        );
    }
    let events = details["events"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    for event in events {
        for mut item in event_entries(provider, event) {
            item.label = localized(&item.label, language);
            if let Some(seq) = event["seq"].as_u64() {
                item.label = format!("{} · #{seq}", item.label);
            }
            entry(ui, item);
        }
    }
    for decision in details["decisions"].as_array().into_iter().flatten() {
        let allow = decision["allow"] == true;
        let label = text(if allow {
            "room.coordinator_allowed"
        } else {
            "room.coordinator_denied"
        })
        .to_owned();
        let input = decision["input"]
            .as_str()
            .and_then(|input| serde_json::from_str::<Value>(input).ok())
            .unwrap_or(Value::Null);
        let mut summary = tool_line(
            &Value::String(decision["tool"].as_str().unwrap_or("tool").into()),
            &input,
        );
        let reason = decision["reason"]
            .as_str()
            .or_else(|| decision["error"].as_str())
            .unwrap_or_default();
        if !reason.is_empty() {
            summary.push('\n');
            summary.push_str(reason);
        }
        entry(
            ui,
            Entry {
                label,
                text: summary,
                tone: if allow { Tone::Result } else { Tone::Error },
            },
        );
    }
    if details["task"]["error"].is_string() {
        entry(
            ui,
            Entry {
                label: text("room.coordinator_report").into(),
                text: details["task"]["error"].as_str().unwrap_or_default().into(),
                tone: Tone::Error,
            },
        );
    }
    if events.is_empty() {
        ui.label(text("room.no_provider_events_have_been_reported_yet"));
    }
}

impl super::super::Probe {
    pub(in crate::native_ui_probe) fn load_task_details(&mut self, index: usize) {
        let Some(task_id) = self
            .snapshot
            .as_ref()
            .and_then(|snapshot| snapshot["report"]["tasks"][index]["id"].as_str())
            .map(str::to_owned)
        else {
            self.error = Some("Selected task is unavailable".into());
            return;
        };
        let result = self
            .worker
            .as_ref()
            .ok_or_else(|| "Agent Room controller is unavailable".to_owned())
            .and_then(|worker| worker.send(&request(index)));
        match result {
            Ok(()) => {
                if let Some(view) = self.room_view.as_mut() {
                    view.details_task = Some(task_id);
                    view.task_details = None;
                }
                self.pending = Some(super::super::Pending::TaskDetails);
                self.error = None;
            }
            Err(error) => self.error = Some(error),
        }
    }

    pub(in crate::native_ui_probe) fn refresh_selected_task_details(&mut self) {
        let Some(snapshot) = self.snapshot.as_ref() else {
            return;
        };
        let count = snapshot["report"]["tasks"].as_array().map_or(0, Vec::len);
        if count == 0 {
            return;
        }
        let index = self
            .room_view
            .as_ref()
            .and_then(|view| view.task_controls.selected_index(snapshot))
            .unwrap_or(0);
        self.load_task_details(index);
    }
}
