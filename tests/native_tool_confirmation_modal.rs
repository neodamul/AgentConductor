#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::{
    ToolAction,
    room::{self, State},
};
use eframe::egui;
use serde_json::json;

#[test]
fn explicit_deferred_and_approved_decisions_use_one_modal_without_moving_activity() {
    for (status, action) in [
        ("pending", ToolAction::Approve),
        ("pending", ToolAction::Reject),
        ("pending", ToolAction::Refresh),
        ("approved", ToolAction::Refresh),
    ] {
        let id = "a".repeat(64);
        let snapshot = json!({"revision":"rev", "report":{"state":"waiting_tool_approval","plan":{"goal":"Verify tool controls","agents":[{"id":"local"}],"tasks":[{"id":"test-suite","agent":"local"}]},"tasks":[{"id":"test-suite","status":"waiting_tool_approval"}]},
            "job":{"state":"idle"}, "tool_requests":[{"id":id,"status":status,
                "request":{"tool":"Bash","input":{"command":"cargo test"},"reason":"Run tests"},
                "context":{"task":"test-suite"}}]});
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut state = State::default();
        state.deferred_tools.insert(id.clone());
        let render = |state: &mut State| {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 892.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    room::show(state, Some(&snapshot), false, None, "en", ui);
                },
            );
            let activity = output.shapes.iter().find_map(|shape| match &shape.shape {
                egui::Shape::Text(text)
                    if text.galley.job.text.eq_ignore_ascii_case("Activity") =>
                {
                    Some(text.pos)
                }
                _ => None,
            });
            let confirmations = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter(|(_, node)| node.label() == Some("Confirm tool decision"))
                .count();
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            (activity, confirmations)
        };
        render(&mut state);
        let before = render(&mut state).0.expect("activity is visible");
        state.confirming_tool = Some((
            id.clone(),
            action,
            serde_json::to_string(&[
                &snapshot["tool_requests"][0]["request"],
                &snapshot["tool_requests"][0]["context"],
            ])
            .unwrap(),
        ));
        render(&mut state);
        let after = render(&mut state);
        assert_eq!(
            state.tool_dialog.as_deref(),
            Some(id.as_str()),
            "explicit decision must enter the existing modal"
        );
        assert_eq!(
            after.0,
            Some(before),
            "confirmation must not shift activity"
        );
        assert_eq!(after.1, 1, "render exactly one confirmation control");
    }
}
