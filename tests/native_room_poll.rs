#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{ToolAction, room};
use eframe::egui;
use std::time::{Duration, Instant};

#[test]
fn only_an_open_tool_dialog_suspends_background_room_refresh() {
    let mut state = room::State::default();
    let id = "a".repeat(64);
    let snapshot = serde_json::json!({"tool_requests":[{"id":id,"status":"pending"}]});
    assert!(!state.blocks_background_refresh());

    state.tool_dialog = Some(id.clone());
    assert!(state.blocks_background_refresh());

    state.tool_dialog = None;
    state.deferred_tools.insert(id);
    assert!(!state.blocks_background_refresh());

    let mut deadline = Some(Instant::now() - Duration::from_secs(1));
    assert!(room::refresh_due(
        &egui::Context::default(),
        Some(&snapshot),
        state.blocks_background_refresh(),
        &mut deadline,
    ));

    state.tool_dialog = Some("a".repeat(64));
    state.confirming_decision = Some(room::CandidateDecision::Reject);
    assert!(state.blocks_background_refresh());
}

#[test]
fn cancel_remains_available_while_an_unrelated_refresh_is_pending() {
    let id = "b".repeat(64);
    let snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{"state":"running"},
        "job":{"state":"running"},
        "tool_requests":[{
            "id":id,
            "status":"pending",
            "request":{"tool":"Bash","input":{"command":"cargo test"},"reason":"Run tests"},
            "context":{"task":"test-suite"}
        }]
    });
    let evidence = serde_json::to_string(&[
        &snapshot["tool_requests"][0]["request"],
        &snapshot["tool_requests"][0]["context"],
    ])
    .unwrap();
    let mut state = room::State::default();
    state.tool_dialog = Some(id.clone());
    state.confirming_tool = Some((id, ToolAction::Reject, evidence));
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), true, None, "en", ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), true, None, "en", ui);
    });
    let cancel_enabled = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .any(|(_, node)| node.label() == Some("Cancel tool decision") && !node.is_disabled());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(cancel_enabled, "the pending refresh trapped the modal");
}
