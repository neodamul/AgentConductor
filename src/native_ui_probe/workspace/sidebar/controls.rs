use super::{Event, message, profile_tools, text};
use eframe::egui::{self, RichText};
use serde_json::Value;

struct DragAgent(String);

pub(super) fn show(
    id: &str,
    snapshot: &Value,
    language: &str,
    editable: bool,
    profile_editing: bool,
    (index, count): (usize, usize),
    ui: &mut egui::Ui,
) -> Option<Event> {
    let mut event = None;
    ui.horizontal(|ui| {
        if ui
            .add_enabled(editable, egui::Button::new("✎"))
            .accessible_name(message(language, "startup.edit_agent_named", id))
            .clicked()
        {
            event = Some(Event::EditAgent(id.to_owned()));
        }
        let busy = profile_tools::metadata_active(snapshot, "quota", id)
            || profile_tools::metadata_active(snapshot, "catalog", id);
        if ui
            .add_enabled(
                editable && !profile_editing && !busy,
                egui::Button::new("🗑"),
            )
            .accessible_name(message(language, "startup.remove_agent_named", id))
            .clicked()
        {
            event = Some(Event::RemoveAgent(id.to_owned()));
        }
        let movable = editable && !profile_editing;
        for (key, glyph, target) in [
            ("startup.move_up", "↑", index.checked_sub(1)),
            (
                "startup.move_down",
                "↓",
                (index + 1 < count).then_some(index + 1),
            ),
        ] {
            let label = text(language, key);
            if ui
                .add_enabled(movable && target.is_some(), egui::Button::new(glyph))
                .accessible_name(format!("{id}: {label}"))
                .on_hover_text(label)
                .clicked()
            {
                event = target.map(|target| Event::MoveAgent(id.to_owned(), target));
            }
        }
        let drag = text(language, "startup.drag_to_reorder");
        ui.add_enabled(movable, egui::Label::new("⠿").sense(egui::Sense::drag()))
            .accessible_name(format!("{id}: {drag}"))
            .on_hover_text(drag)
            .on_hover_cursor(egui::CursorIcon::Grab)
            .dnd_set_drag_payload(DragAgent(id.to_owned()));
    });
    if snapshot["config"]["coordinator"] == id {
        ui.label(
            RichText::new(text(language, "native.coordinator"))
                .small()
                .color(ui.visuals().weak_text_color()),
        );
    }
    event
}

pub(super) fn dropped(response: &egui::Response) -> Option<String> {
    if response.dnd_hover_payload::<DragAgent>().is_some() {
        response.ctx.layer_painter(response.layer_id).rect_stroke(
            response.rect,
            10,
            egui::Stroke::new(2.0, response.ctx.global_style().visuals.hyperlink_color),
            egui::StrokeKind::Inside,
        );
    }
    response
        .dnd_release_payload::<DragAgent>()
        .map(|payload| payload.0.clone())
}
