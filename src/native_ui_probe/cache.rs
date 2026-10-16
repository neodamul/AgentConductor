use super::{Pending, Probe};
use eframe::egui::{Button, Frame, ScrollArea, Ui};
use serde_json::Value;

#[derive(Default)]
pub struct State {
    pub open: bool,
    pub preview: Option<Value>,
    pub result: Option<Value>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Event {
    Preview,
    Clear,
}

pub fn preview_request() -> &'static [u8] {
    br#"{"kind":"cache_preview"}"#
}

pub fn clear_request(preview: &Value) -> Result<Vec<u8>, String> {
    let digest = preview["digest"]
        .as_str()
        .filter(|digest| !digest.is_empty())
        .ok_or("cache preview digest is unavailable")?;
    if !preview["items"]
        .as_array()
        .is_some_and(|items| !items.is_empty())
    {
        return Err("cache preview contains no disposable entries".into());
    }
    serde_json::to_vec(&serde_json::json!({"kind":"cache_clear","digest":digest}))
        .map_err(|error| error.to_string())
}

pub fn show(state: &mut State, ui: &mut Ui, pending: bool) -> Option<Event> {
    if !state.open {
        return None;
    }
    let mut event = None;
    Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("Local cache cleanup");
        if let Some(result) = &state.result {
            ui.label(format!(
                "Removed {} disposable entries · {:.1} MiB",
                result["removed"].as_u64().unwrap_or(0),
                result["bytes"].as_u64().unwrap_or(0) as f64 / 1_048_576.0
            ));
        }
        if let Some(preview) = &state.preview {
            let items = preview["items"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            ui.label(format!(
                "{} disposable entries · {:.1} MiB",
                items.len(),
                preview["bytes"].as_u64().unwrap_or(0) as f64 / 1_048_576.0
            ));
            ui.label(format!(
                "Protected or uncertain entries retained: {}",
                preview["skipped"].as_u64().unwrap_or(0)
            ));
            if items.is_empty() {
                ui.label("No disposable cache to clear.");
            } else {
                ScrollArea::vertical().max_height(220.0).show_rows(
                    ui,
                    22.0,
                    items.len(),
                    |ui, rows| {
                        for index in rows {
                            ui.monospace(
                                items[index]["path"].as_str().unwrap_or("unavailable path"),
                            );
                        }
                    },
                );
            }
        } else if state.result.is_none() {
            ui.label("Previewing disposable cache entries…");
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!pending, Button::new("Refresh preview"))
                .clicked()
            {
                event = Some(Event::Preview);
            }
            if ui
                .add_enabled(
                    !pending
                        && state.preview.as_ref().is_some_and(|preview| {
                            preview["items"]
                                .as_array()
                                .is_some_and(|items| !items.is_empty())
                        }),
                    Button::new("Clear reviewed cache"),
                )
                .accessible_name("Clear reviewed cache")
                .clicked()
            {
                event = Some(Event::Clear);
            }
            if ui.button("Close").clicked() {
                state.open = false;
            }
        });
        if pending {
            ui.spinner();
        }
    });
    event
}

impl Probe {
    pub(super) fn open_cache(&mut self) {
        self.cache.open = true;
        self.cache.preview = None;
        self.cache.result = None;
        self.error = None;
        self.preview_cache();
    }

    pub(super) fn preview_cache(&mut self) {
        self.cache.preview = None;
        self.cache.result = None;
        self.error = None;
        match self
            .worker
            .as_ref()
            .ok_or_else(|| "Workspace controller is unavailable".to_owned())
            .and_then(|worker| worker.send(preview_request()))
        {
            Ok(()) => self.pending = Some(Pending::CachePreview),
            Err(error) => self.error = Some(error),
        }
    }

    pub(super) fn clear_cache(&mut self) {
        let request = self
            .cache
            .preview
            .as_ref()
            .ok_or_else(|| "Cache preview is unavailable".to_owned())
            .and_then(clear_request);
        match request.and_then(|request| {
            self.worker
                .as_ref()
                .ok_or_else(|| "Workspace controller is unavailable".to_owned())?
                .send(&request)
        }) {
            Ok(()) => self.pending = Some(Pending::CacheClear),
            Err(error) => self.error = Some(error),
        }
    }
}

pub(super) fn apply_preview(state: &mut State, value: Value) {
    state.preview = Some(value);
    state.result = None;
}

pub(super) fn apply_clear(state: &mut State, value: Value) {
    state.preview = None;
    state.result = Some(value);
}
