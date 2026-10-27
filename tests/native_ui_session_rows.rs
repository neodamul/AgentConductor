#![cfg(feature = "native-ui-probe")]
//! Agent Room session rows follow the design guide: the task name leads, a small agent icon,
//! the agent name and a compact status tag share the next line, and the tag wraps below instead
//! of leaving the list (R-128 / DD-137, GitHub #9 visual parity).

use agentconductor::native_ui_probe::{Probe, status_label};
use eframe::egui::{self, Pos2, epaint::Shape};

struct Text {
    text: String,
    x: f32,
    y: f32,
    right: f32,
    size: f32,
}

fn render(width: f32) -> Vec<Text> {
    let snapshot = serde_json::json!({
        "report": {
            "state": "waiting_approval",
            "plan": {
                "goal": "goal",
                "agents": [{"id": "claude-main", "parser": "claude"}],
                "tasks": [{"id": "one.txt", "agent": "claude-main"}]
            },
            "tasks": [{"id": "one.txt", "status": "waiting_tool_approval"}]
        }
    });
    let context = egui::Context::default();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(width, 600.0),
            )),
            ..Default::default()
        },
        |ui| Probe::render_snapshot_in_language(&snapshot, "en", ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            Shape::Text(text) => Some(Text {
                text: text.galley.job.text.clone(),
                // The first glyph, not the galley rect: wrapped layouts add leading space.
                x: text.pos.x
                    + text.galley.rows.first().map_or(0.0, |row| {
                        row.pos.x + row.glyphs.first().map_or(0.0, |glyph| glyph.pos.x)
                    }),
                y: text.pos.y,
                right: text.pos.x + text.galley.rect.max.x,
                size: text.galley.job.sections[0].format.font_id.size,
            }),
            _ => None,
        })
        .collect()
}

fn find<'a>(texts: &'a [Text], needle: &str) -> &'a Text {
    texts
        .iter()
        .find(|text| text.text.contains(needle))
        .unwrap_or_else(|| panic!("no text containing {needle:?}"))
}

#[test]
fn the_task_name_leads_and_icon_agent_and_status_follow_on_one_line() {
    let texts = render(320.0);
    let task = find(&texts, "one.txt");
    let icon = find(&texts, "CC");
    let agent = find(&texts, "claude-main");
    let status = find(&texts, &status_label("waiting_tool_approval", "en"));
    assert!(task.y < agent.y, "task name comes first");
    assert!(
        (icon.y - agent.y).abs() < 12.0,
        "icon sits on the agent line"
    );
    assert!(
        icon.x < agent.x,
        "icon precedes the agent name: icon {} agent {}",
        icon.x,
        agent.x
    );
    assert!(icon.size <= 9.5, "the icon is compact: {}", icon.size);
    assert!(
        (status.y - agent.y).abs() < 12.0,
        "status shares the agent line at 320 px"
    );
    assert!(status.x > agent.x, "status follows the agent name");
}

#[test]
fn the_status_tag_never_leaves_a_narrow_list() {
    for width in [260.0, 200.0, 160.0] {
        let texts = render(width);
        let status = find(&texts, &status_label("waiting_tool_approval", "en"));
        assert!(
            status.right <= width,
            "at {width}px the status ends at {:.0}px",
            status.right
        );
    }
}
