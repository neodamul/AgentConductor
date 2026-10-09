use agentconductor::{
    claude_control::{Controller, ToolDecision},
    process::ControlReply,
};
use serde_json::json;

fn initialized(controller: &mut Controller) -> ControlReply {
    controller.handle(json!({"type":"control_response","response":{"subtype":"success","request_id":"ac-initialize","response":{}}})).unwrap()
}

#[test]
fn prompt_waits_for_initialization_and_errors_never_release_a_user_turn() {
    let mut controller = Controller::new(
        "implement goal",
        Box::new(|_, _| Ok(ToolDecision::deny("Requires human approval"))),
    )
    .unwrap();
    let init: serde_json::Value =
        serde_json::from_slice(&controller.initial_input().unwrap()).unwrap();
    assert_eq!(
        init["request"]["hooks"]["PreToolUse"][0]["hookCallbackIds"][0],
        "ac-pre-tool"
    );
    assert!(controller.handle(json!({"type":"control_response","response":{"subtype":"error","request_id":"ac-initialize","error":"unsupported"}})).is_err());
    let reply = initialized(&mut controller);
    assert_eq!(reply.messages[0]["type"], "user");
    assert_eq!(reply.messages[0]["message"]["content"], "implement goal");
    assert!(!reply.close_input);
}

#[test]
fn pretool_denial_and_unexpected_permission_requests_do_not_allow_a_command() {
    let mut controller = Controller::new(
        "goal",
        Box::new(|_, _| Ok(ToolDecision::deny("Waiting for approval"))),
    )
    .unwrap();
    initialized(&mut controller);
    let reply=controller.handle(json!({"type":"control_request","request_id":"hook","request":{"subtype":"hook_callback","callback_id":"ac-pre-tool","tool_use_id":"tool-1","input":{"tool_name":"Bash","tool_input":{"command":"deploy production"}}}})).unwrap();
    assert_eq!(
        reply.messages[0]["response"]["response"]["hookSpecificOutput"]["permissionDecision"],
        "deny"
    );
    let reply=controller.handle(json!({"type":"control_request","request_id":"permission","request":{"subtype":"can_use_tool","tool_use_id":"unknown","tool_name":"Bash","input":{"command":"git push"}}})).unwrap();
    assert_eq!(
        reply.messages[0]["response"]["response"]["behavior"],
        "deny"
    );
    assert!(!reply.close_input);
    assert!(controller.handle(json!({"type":"control_request","request_id":"other","request":{"subtype":"hook_callback","callback_id":"invented","input":{}}})).is_err());
}

#[test]
fn permission_phase_uses_exact_pretool_decision_once_and_keeps_channel_until_result_and_idle() {
    let mut controller = Controller::new(
        "goal",
        Box::new(|tool, input| {
            assert_eq!(tool, "Bash");
            assert_eq!(input["command"], "approved command");
            Ok(ToolDecision {
                allow: true,
                reason: "Exact human approval consumed".into(),
            })
        }),
    )
    .unwrap();
    initialized(&mut controller);
    let input = json!({"command":"approved command"});
    controller.handle(json!({"type":"control_request","request_id":"pre","request":{"subtype":"hook_callback","callback_id":"ac-pre-tool","tool_use_id":"tool-1","input":{"tool_name":"Bash","tool_input":input}}})).unwrap();
    let reply=controller.handle(json!({"type":"control_request","request_id":"permit","request":{"subtype":"can_use_tool","tool_use_id":"tool-1","tool_name":"Bash","input":input}})).unwrap();
    assert_eq!(
        reply.messages[0]["response"]["response"]["behavior"],
        "allow"
    );
    assert_eq!(
        reply.messages[0]["response"]["response"]["updatedInput"],
        input
    );
    let reply=controller.handle(json!({"type":"control_request","request_id":"replay","request":{"subtype":"can_use_tool","tool_use_id":"tool-1","tool_name":"Bash","input":input}})).unwrap();
    assert_eq!(
        reply.messages[0]["response"]["response"]["behavior"],
        "deny"
    );
    controller
        .handle(json!({"type":"system","subtype":"session_state_changed","state":"running"}))
        .unwrap();
    assert!(
        !controller
            .handle(json!({"type":"result","is_error":false}))
            .unwrap()
            .close_input
    );
    assert!(
        controller
            .handle(json!({"type":"system","subtype":"session_state_changed","state":"idle"}))
            .unwrap()
            .close_input
    );
}

#[test]
fn changing_a_permission_input_cannot_reuse_an_allowed_pretool_decision() {
    let mut controller = Controller::new(
        "goal",
        Box::new(|_, _| {
            Ok(ToolDecision {
                allow: true,
                reason: "Approved exact input".into(),
            })
        }),
    )
    .unwrap();
    initialized(&mut controller);
    controller.handle(json!({"type":"control_request","request_id":"pre","request":{"subtype":"hook_callback","callback_id":"ac-pre-tool","tool_use_id":"tool-1","input":{"tool_name":"Bash","tool_input":{"command":"approved"}}}})).unwrap();
    let reply=controller.handle(json!({"type":"control_request","request_id":"permission","request":{"subtype":"can_use_tool","tool_use_id":"tool-1","tool_name":"Bash","input":{"command":"different"}}})).unwrap();
    assert_eq!(
        reply.messages[0]["response"]["response"]["behavior"],
        "deny"
    );
}

#[test]
fn a_result_does_not_discard_a_permission_callback_owed_by_active_native_work() {
    let mut controller = Controller::new(
        "goal",
        Box::new(|_, _| {
            Ok(ToolDecision {
                allow: true,
                reason: "Approved exact request".into(),
            })
        }),
    )
    .unwrap();
    initialized(&mut controller);
    controller
        .handle(json!({"type":"system","subtype":"session_state_changed","state":"running"}))
        .unwrap();
    controller.handle(json!({"type":"control_request","request_id":"pre","request":{"subtype":"hook_callback","callback_id":"ac-pre-tool","tool_use_id":"tool-1","input":{"tool_name":"Bash","tool_input":{"command":"approved"}}}})).unwrap();
    assert!(
        !controller
            .handle(json!({"type":"result","is_error":false}))
            .unwrap()
            .close_input
    );
    let reply=controller.handle(json!({"type":"control_request","request_id":"permission","request":{"subtype":"can_use_tool","tool_use_id":"tool-1","tool_name":"Bash","input":{"command":"approved"}}})).unwrap();
    assert_eq!(
        reply.messages[0]["response"]["response"]["behavior"],
        "allow"
    );
}

#[test]
fn duplicate_tool_identity_is_rejected_before_consuming_another_human_decision() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let mut controller = Controller::new(
        "goal",
        Box::new(move |_, _| {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(ToolDecision {
                allow: true,
                reason: "Exact decision".into(),
            })
        }),
    )
    .unwrap();
    initialized(&mut controller);
    for id in ["pre-1", "pre-2"] {
        let result = controller.handle(json!({"type":"control_request","request_id":id,"request":{"subtype":"hook_callback","callback_id":"ac-pre-tool","tool_use_id":"same","input":{"tool_name":"Bash","tool_input":{"command":"approved"}}}}));
        if id == "pre-1" {
            assert!(result.is_ok());
        } else {
            assert!(result.is_err());
        }
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn no_permission_can_be_released_after_the_controller_closes_input() {
    let mut controller = Controller::new(
        "goal",
        Box::new(|_, _| panic!("closed controller must not evaluate")),
    )
    .unwrap();
    initialized(&mut controller);
    assert!(
        controller
            .handle(json!({"type":"result"}))
            .unwrap()
            .close_input
    );
    assert!(controller.handle(json!({"type":"control_request","request_id":"late","request":{"subtype":"hook_callback","callback_id":"ac-pre-tool","tool_use_id":"late","input":{"tool_name":"Bash","tool_input":{"command":"late"}}}})).is_err());
}
