use eframe::egui;
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Discover(String),
    Quota(String),
}

pub fn metadata_request(event: &Event, revision: &str) -> Vec<u8> {
    let (kind, agent) = match event {
        Event::Discover(agent) => ("discover", agent),
        Event::Quota(agent) => ("quota", agent),
    };
    serde_json::to_vec(&serde_json::json!({"kind":kind,"agent":agent,"revision":revision}))
        .expect("metadata request is serializable")
}

pub fn show(
    profiles: &[Value],
    snapshot: &Value,
    editable: bool,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    ui.separator();
    ui.label("Registered agents");
    for profile in profiles {
        let id = profile["id"].as_str().unwrap_or_default();
        let parser = profile["parser"].as_str().unwrap_or("custom");
        if id.is_empty() {
            continue;
        }
        let saved = snapshot["config"]["profiles"]
            .as_array()
            .is_some_and(|saved| saved.iter().any(|item| item["id"] == id));
        let models = profile["catalog"]["models"].as_array().map_or(0, Vec::len);
        ui.horizontal(|ui| {
            ui.strong(id);
            ui.label(format!("{models} models"));
            if let Some(badge) = snapshot["quota_badges"][id]["text"].as_str() {
                ui.label(badge).on_hover_text(
                    snapshot["quota_badges"][id]["title"]
                        .as_str()
                        .unwrap_or_default(),
                );
            }
            let catalog_busy = metadata_active(snapshot, "catalog", id);
            if parser != "custom"
                && ui
                    .add_enabled(
                        editable && saved && !catalog_busy,
                        egui::Button::new("Models ↻"),
                    )
                    .accessible_name(format!("Refresh models for {id}"))
                    .clicked()
            {
                event = Some(Event::Discover(id.into()));
            }
            if matches!(parser, "claude" | "codex") {
                let busy = metadata_active(snapshot, "quota", id);
                if ui
                    .add_enabled(editable && saved && !busy, egui::Button::new("↻"))
                    .accessible_name(format!("Refresh quota for {id}"))
                    .on_hover_text(format!("Refresh quota for {id}"))
                    .clicked()
                {
                    event = Some(Event::Quota(id.into()));
                }
                if busy {
                    ui.spinner();
                }
            }
            if catalog_busy {
                ui.spinner();
            }
        });
    }
    event
}

pub fn merge_catalogs(local: &mut [Value], saved: &[Value]) {
    for profile in local {
        let Some(observed) = saved.iter().find(|item| {
            item["id"] == profile["id"]
                && item["parser"] == profile["parser"]
                && item["executable"] == profile["executable"]
        }) else {
            continue;
        };
        if !observed["catalog"].is_null() {
            profile["catalog"] = observed["catalog"].clone();
        }
    }
}

pub fn metadata_running(snapshot: &Value) -> bool {
    snapshot["metadata_job"]["state"] == "running"
        || snapshot["metadata_queue"]
            .as_array()
            .is_some_and(|queue| !queue.is_empty())
}

fn metadata_active(snapshot: &Value, kind: &str, agent: &str) -> bool {
    let operation = format!("{kind}:{agent}");
    (snapshot["metadata_job"]["kind"] == operation
        && snapshot["metadata_job"]["state"] == "running")
        || snapshot["metadata_queue"].as_array().is_some_and(|queue| {
            queue
                .iter()
                .any(|item| item["kind"] == kind && item["agent"] == agent)
        })
}
