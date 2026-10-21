#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::goals;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, Rect};
use serde_json::{Value, json};

#[test]
fn opening_coordinator_analysis_does_not_push_phase_rows_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = snapshot();
    let baseline = frame(&context, &snapshot, Vec::new());
    let analysis = bounds(&baseline, "Coordinator analysis of the whole goal");
    let phase = bounds_containing(&baseline, "2. Documentation");
    clear(baseline);

    let center = Pos2::new(
        ((analysis.x0 + analysis.x1) / 2.0) as f32,
        ((analysis.y0 + analysis.y1) / 2.0) as f32,
    );
    let _ = frame(
        &context,
        &snapshot,
        vec![Event::PointerMoved(center), pointer(center, true)],
    );
    clear(frame(&context, &snapshot, vec![pointer(center, false)]));
    let opened = frame(&context, &snapshot, Vec::new());
    let after = bounds_containing(&opened, "2. Documentation");
    let accessible = opened
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.label() == Some("Coordinator analysis details"));
    let visible = opened
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    clear(opened);

    assert_eq!(after.y0, phase.y0, "analysis must overlay plan rows");
    assert!(
        visible
            .iter()
            .any(|text| text == "Coordinator analysis details"),
        "analysis popup content is missing: {visible:?}"
    );
    assert!(
        accessible,
        "analysis text must be exposed to assistive technology"
    );

    let outside = Pos2::new(850.0, 650.0);
    let _ = frame(
        &context,
        &snapshot,
        vec![Event::PointerMoved(outside), pointer(outside, true)],
    );
    clear(frame(&context, &snapshot, vec![pointer(outside, false)]));
    let closed = frame(&context, &snapshot, Vec::new());
    let remains_open = closed.shapes.iter().any(|shape| match &shape.shape {
        egui::epaint::Shape::Text(text) => text.galley.job.text == "Coordinator analysis details",
        _ => false,
    });
    clear(closed);
    assert!(!remains_open, "outside click must close the analysis popup");
}

fn snapshot() -> Value {
    json!({
        "job":{"state":"idle"}, "proposal":{"slug_phase":null},
        "slugs":[{"slug":"auth", "goal":"Refresh token rotation", "active":true,
            "progress":{"current_phase":"01-core", "mismatches":[], "phases":[
                {"id":"01-core", "name":"Core", "stage":"task", "task":[], "verification":[], "completed":[]},
                {"id":"02-docs", "name":"Documentation", "stage":"planned", "task":[], "verification":[], "completed":[]}
            ]},
            "plan":{"analysis":"Coordinator analysis details", "phases":[
                {"id":"01-core", "brief":"Implement core behavior"},
                {"id":"02-docs", "brief":"Document behavior"}
            ]}
        }]
    })
}

fn frame(context: &egui::Context, snapshot: &Value, events: Vec<Event>) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(900.0, 700.0))),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                goals::show(snapshot, "en", false, ui);
            });
        },
    )
}

fn bounds(output: &egui::FullOutput, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {label:?}"))
}

fn bounds_containing(output: &egui::FullOutput, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label().is_some_and(|text| text.contains(label)))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing control containing {label:?}"))
}

fn pointer(pos: Pos2, pressed: bool) -> Event {
    Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::default(),
    }
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
