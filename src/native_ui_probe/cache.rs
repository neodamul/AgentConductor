use super::{Pending, Probe};
use eframe::egui::{Align2, Button, Context, Frame, ScrollArea, Ui, Window};
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
    show_in_language(state, "en", ui, pending)
}

pub fn show_in_language(
    state: &mut State,
    language: &str,
    ui: &mut Ui,
    pending: bool,
) -> Option<Event> {
    show_content(state, language, ui, pending, true)
}

fn show_content(
    state: &mut State,
    language: &str,
    ui: &mut Ui,
    pending: bool,
    show_heading: bool,
) -> Option<Event> {
    if !state.open {
        return None;
    }
    let mut event = None;
    Frame::group(ui.style()).show(ui, |ui| {
        if show_heading {
            ui.heading(text(language, "startup.cache_cleanup"));
        }
        if let Some(result) = &state.result {
            let count = result["removed"].as_u64().unwrap_or(0).to_string();
            let size = mib(result["bytes"].as_u64().unwrap_or(0));
            ui.label(crate::ui::messages::interpolate(
                text(language, "startup.cache_removed"),
                &[("value1", &count), ("value2", &size)],
            ));
        }
        if let Some(preview) = &state.preview {
            let items = preview["items"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let count = items.len().to_string();
            let size = mib(preview["bytes"].as_u64().unwrap_or(0));
            ui.label(crate::ui::messages::interpolate(
                text(language, "startup.cache_entries"),
                &[("value1", &count), ("value2", &size)],
            ));
            let skipped = preview["skipped"].as_u64().unwrap_or(0).to_string();
            ui.label(crate::ui::messages::interpolate(
                text(language, "startup.cache_skipped"),
                &[("value1", &skipped)],
            ));
            if items.is_empty() {
                ui.label(text(language, "startup.cache_empty"));
            } else {
                ScrollArea::vertical().max_height(220.0).show_rows(
                    ui,
                    22.0,
                    items.len(),
                    |ui, rows| {
                        for index in rows {
                            ui.monospace(
                                items[index]["path"]
                                    .as_str()
                                    .unwrap_or(text(language, "startup.cache_unavailable_path")),
                            );
                        }
                    },
                );
            }
        } else if state.result.is_none() {
            ui.label(text(language, "startup.cache_previewing"));
        }
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    !pending,
                    Button::new(text(language, "startup.cache_refresh_preview")),
                )
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
                    Button::new(text(language, "startup.cache_clear_reviewed")),
                )
                .accessible_name(text(language, "startup.cache_clear_reviewed"))
                .clicked()
            {
                event = Some(Event::Clear);
            }
            if ui.button(text(language, "startup.cache_close")).clicked() {
                state.open = false;
            }
        });
        if pending {
            ui.spinner();
        }
    });
    event
}

fn show_dialog(
    state: &mut State,
    language: &str,
    context: &Context,
    pending: bool,
) -> Option<Event> {
    if !state.open {
        return None;
    }
    let mut open = state.open;
    let mut event = None;
    Window::new(text(language, "startup.cache_cleanup"))
        .open(&mut open)
        .anchor(Align2::CENTER_CENTER, eframe::egui::Vec2::ZERO)
        .default_width(480.0)
        .collapsible(false)
        .resizable(true)
        .show(context, |ui| {
            event = show_content(state, language, ui, pending, false)
        });
    state.open = open && state.open;
    event
}

fn mib(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1_048_576.0)
}

fn text(language: &str, id: &'static str) -> &'static str {
    crate::ui::messages::in_language(language, id)
}

impl Probe {
    pub(super) fn show_cache_panel(&mut self, ui: &mut Ui) -> Option<Event> {
        show_dialog(
            &mut self.cache,
            self.navigation.appearance.language(),
            ui.ctx(),
            self.pending.is_some(),
        )
    }

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
