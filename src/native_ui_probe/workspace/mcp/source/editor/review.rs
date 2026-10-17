use super::Event;
use eframe::egui::{self, RichText};
use serde_json::Value;
use std::collections::BTreeSet;

pub(super) fn show(
    preview: &Value,
    selected: &mut BTreeSet<String>,
    applying: bool,
    previewing: bool,
    can_apply: bool,
    language: &str,
    ui: &mut egui::Ui,
) -> Option<Event> {
    let text = |id| crate::ui::messages::in_language(language, id);
    let records = preview["records"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    let failures = preview["failures"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default();
    egui::ScrollArea::vertical()
        .max_height(340.0)
        .show(ui, |ui| {
            for record in records {
                show_record(record, preview, language, selected, ui);
            }
            for failure in failures {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    format!(
                        "{}: {}",
                        reference_key(&failure["reference"]),
                        failure["error"]
                    ),
                );
            }
        });
    let required_jira_failed = failures
        .iter()
        .any(|failure| failure["reference"]["kind"] == "jira");
    let selected = selected.iter().cloned().collect::<Vec<_>>();
    if ui
        .add_enabled(
            !records.is_empty()
                && !selected.is_empty()
                && !required_jira_failed
                && !applying
                && !previewing
                && can_apply,
            egui::Button::new(text("mcp.apply_selected_sources")),
        )
        .accessible_name(text("mcp.apply_selected_sources"))
        .clicked()
    {
        return Some(Event::Apply(selected));
    }
    None
}

fn show_record(
    record: &Value,
    preview: &Value,
    language: &str,
    selected: &mut BTreeSet<String>,
    ui: &mut egui::Ui,
) {
    let key = reference_key(&record["reference"]);
    let kind = record["reference"]["kind"].as_str().unwrap_or_default();
    ui.group(|ui| {
        let mut checked = selected.contains(&key);
        ui.add_enabled_ui(kind != "jira", |ui| {
            if ui.checkbox(&mut checked, &key).changed() {
                if checked {
                    selected.insert(key.clone());
                } else {
                    selected.remove(&key);
                }
            }
        });
        let unavailable = crate::ui::messages::in_language(language, "mcp.not_supplied");
        let title = record["title"].as_str().unwrap_or_else(|| {
            crate::ui::messages::in_language(language, "mcp.title_not_supplied")
        });
        let url = record["url"]
            .as_str()
            .unwrap_or_else(|| crate::ui::messages::in_language(language, "mcp.url_not_supplied"));
        let fetched = preview["fetched_at"]
            .as_u64()
            .unwrap_or_default()
            .to_string();
        let metadata = crate::ui::messages::render(
            "mcp.version_updated_retrieved_sha_256",
            &[
                ("value1", record["server"].as_str().unwrap_or_default()),
                ("value2", record["tool"].as_str().unwrap_or_default()),
                ("value3", title),
                ("value4", url),
                ("value5", record["version"].as_str().unwrap_or(unavailable)),
                (
                    "value6",
                    record["updated_at"].as_str().unwrap_or(unavailable),
                ),
                ("value7", &fetched),
                ("value8", record["sha256"].as_str().unwrap_or_default()),
            ],
        );
        ui.label(RichText::new(metadata).small());
        if let Some(summary) = preview["summaries"][&key]["text"].as_str() {
            ui.label(summary);
        }
        ui.collapsing(
            crate::ui::messages::in_language(language, "mcp.full_source"),
            |ui| {
                egui::ScrollArea::vertical()
                    .max_height(240.0)
                    .show(ui, |ui| {
                        ui.label(record["body"].as_str().unwrap_or_default());
                    });
            },
        );
    });
}

pub(super) fn reference_key(reference: &Value) -> String {
    format!(
        "{}:{}",
        reference["kind"].as_str().unwrap_or_default(),
        reference["id"].as_str().unwrap_or_default()
    )
}
