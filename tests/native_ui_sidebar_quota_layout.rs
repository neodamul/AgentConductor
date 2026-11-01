#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, Pos2, Shape, vec2};
use serde_json::json;

#[test]
fn quota_summary_stays_inside_a_narrow_agent_sidebar() {
    let profiles = vec![json!({
        "id":"codex","parser":"codex","executable":"codex","max_sessions":3
    })];
    let snapshot = json!({
        "config":{"profiles":profiles,"coordinator":"codex"},
        "job":{"kind":"idle","state":"completed"},
        "quotas":{"codex":{"status":"available"}},
        "quota_badges":{"codex":{"text":"73%/100%","title":"Enterprise: 73% remaining"}}
    });
    let context = egui::Context::default();
    let bounds = egui::Rect::from_min_size(Pos2::ZERO, vec2(280.0, 640.0));
    let mut input = egui::RawInput {
        screen_rect: Some(bounds),
        ..Default::default()
    };
    input
        .viewports
        .get_mut(&input.viewport_id)
        .unwrap()
        .inner_rect = Some(bounds);
    let mut output = context.run_ui(input, |ui| {
        sidebar::show(&profiles, &snapshot, "en", true, false, ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let rendered = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Text(text) => Some((
                text.galley.job.text.to_string(),
                text.pos.x + text.galley.size().x,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let quota_right = rendered
        .iter()
        .filter(|(label, _)| label.contains("73%/100%"))
        .map(|(_, right)| *right)
        .max_by(f32::total_cmp)
        .unwrap_or_else(|| panic!("the quota badge is rendered: {rendered:?}"));
    assert!(
        quota_right <= 280.0,
        "quota badge extends past the sidebar width: right={quota_right}"
    );
}
