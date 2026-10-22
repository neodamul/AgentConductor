#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::room;
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn the_room_has_an_accessible_detail_fold_control_in_both_languages() {
    for language in ["en", "ko"] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut state = room::State::default();
        state.inspector = room::inspector::Inspector::new(false);
        let snapshot = json!({"report":{"state":"running","plan":{"tasks":[]},"tasks":[]}});
        let mut output = context.run_ui(Default::default(), |ui| {
            room::show(&mut state, Some(&snapshot), false, None, language, ui);
        });
        let label = agentconductor::ui::messages::in_language(language, "room.inspector_fold");
        let visible = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == Role::Button
                    && node.label() == Some(label)
                    && node.is_expanded() == Some(true)
            });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        assert!(visible, "missing native fold control: {label}");
    }
}

#[test]
fn a_decision_opens_once_then_restores_the_operators_preference() {
    use room::inspector::Inspector;
    let mut state = Inspector::new(true);
    let mut snapshot = json!({"report":{"state":"waiting_approval","candidate":{"commit":"a"}}});
    assert!(state.observe(&snapshot) && state.open && state.preference_folded);
    state.toggle();
    assert!(state.attention());
    assert!(!state.observe(&snapshot) && !state.open);
    snapshot["report"]["candidate"]["commit"] = "b".into();
    assert!(state.observe(&snapshot) && state.open);
    snapshot["report"]["state"] = "running".into();
    assert!(!state.observe(&snapshot) && !state.open && !state.attention());
    state.toggle();
    assert!(!state.preference_folded);
    let reopened = Inspector::new(state.preference_folded);
    assert!(reopened.open);
}

#[test]
fn decision_keys_cover_gates_and_ignore_tool_request_order() {
    use room::inspector::decision_key;
    for state in ["waiting_approval", "waiting_tool_approval", "blocked"] {
        assert!(decision_key(&json!({"report":{"state":state}})).is_some());
    }
    assert!(
        decision_key(&json!({"report":{"state":"failed","error":"out-of-scope change: a"}}))
            .is_some()
    );
    assert!(decision_key(&json!({"report":{"state":"failed","error":"test failed"}})).is_none());
    let mut value = json!({"report":{"state":"running"}, "tool_requests":[
        {"id":"a","status":"pending"},{"id":"b","status":"pending"}
    ]});
    let before = decision_key(&value);
    value["tool_requests"].as_array_mut().unwrap().reverse();
    assert_eq!(before, decision_key(&value));
    value["tool_requests"][0]["status"] = "approved".into();
    assert_ne!(before, decision_key(&value));
}

#[test]
fn folded_warning_is_accessible_and_reduced_motion_is_steady() {
    let mut state = room::State::default();
    state.inspector = room::inspector::Inspector::new(true);
    let snapshot = json!({"report":{"state":"blocked","plan":{"tasks":[]},"tasks":[]}});
    state.inspector.observe(&snapshot);
    state.inspector.toggle();
    assert_eq!(state.inspector.warning_alpha(true, 0.0), 1.0);
    assert_eq!(state.inspector.warning_alpha(true, 0.7), 1.0);
    assert_ne!(
        state.inspector.warning_alpha(false, 0.0),
        state.inspector.warning_alpha(false, 0.7)
    );
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    assert!(
        nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Show task details")
                && node.is_expanded() == Some(false))
    );
    assert!(
        nodes
            .iter()
            .any(|(_, node)| node.label() == Some("A decision is waiting in the task details"))
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
