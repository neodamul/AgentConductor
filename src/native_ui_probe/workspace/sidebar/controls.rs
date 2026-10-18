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
        if icon_button(ui, editable, false)
            .accessible_name(message(language, "startup.edit_agent_named", id))
            .clicked()
        {
            event = Some(Event::EditAgent(id.to_owned()));
        }
        let busy = profile_tools::metadata_active(snapshot, "quota", id)
            || profile_tools::metadata_active(snapshot, "catalog", id);
        if icon_button(ui, editable && !profile_editing && !busy, true)
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
        ui.add_enabled_ui(movable, |ui| {
            ui.dnd_drag_source(
                egui::Id::unique(("native-agent-order", id)),
                DragAgent(id.to_owned()),
                |ui| {
                    ui.add(egui::Label::new("⠿").selectable(false));
                },
            )
        })
        .inner
        .response
        .accessible_name(format!("{id}: {drag}"))
        .on_hover_text(drag)
        .on_hover_cursor(egui::CursorIcon::Grab);
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

fn icon_button(ui: &mut egui::Ui, enabled: bool, trash: bool) -> egui::Response {
    let response = ui.add_enabled(
        enabled,
        egui::Button::new("").min_size(egui::vec2(22.0, 20.0)),
    );
    let center = response.rect.center();
    let stroke = ui.style().interact(&response).fg_stroke;
    let line = |a, b| ui.painter().line_segment([center + a, center + b], stroke);
    if trash {
        line(egui::vec2(-5.0, -4.0), egui::vec2(5.0, -4.0));
        line(egui::vec2(-2.0, -6.0), egui::vec2(2.0, -6.0));
        for x in [-4.0, 4.0] {
            line(egui::vec2(x, -2.0), egui::vec2(x, 5.0));
        }
        line(egui::vec2(-4.0, 5.0), egui::vec2(4.0, 5.0));
    } else {
        line(egui::vec2(-5.0, 5.0), egui::vec2(4.0, -4.0));
        line(egui::vec2(-2.0, 5.0), egui::vec2(5.0, -2.0));
        line(egui::vec2(-5.0, 5.0), egui::vec2(-2.0, 5.0));
        line(egui::vec2(2.0, -5.0), egui::vec2(5.0, -2.0));
    }
    response
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
