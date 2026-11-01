use super::*;

#[test]
fn approve_request_uses_the_exact_candidate_and_revision() {
    let packet = candidate_decision_request(
        &candidate("waiting_approval"),
        CandidateDecision::Approve,
        "",
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-1");
    assert_eq!(
        value["action"],
        serde_json::json!({"kind":"approve","sha":"candidate-sha"})
    );
}

#[test]
fn merge_request_is_refused_until_the_candidate_is_approved() {
    assert!(
        candidate_decision_request(&candidate("waiting_approval"), CandidateDecision::Merge, "")
            .is_err()
    );
    let packet =
        candidate_decision_request(&candidate("approved"), CandidateDecision::Merge, "").unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["action"]["kind"], "merge");
    assert_eq!(value["action"]["sha"], "candidate-sha");
}

#[test]
fn reject_request_requires_a_bounded_reason_and_current_candidate() {
    assert!(
        candidate_decision_request(
            &candidate("waiting_approval"),
            CandidateDecision::Reject,
            " "
        )
        .is_err()
    );
    assert!(
        candidate_decision_request(
            &candidate("waiting_approval"),
            CandidateDecision::Reject,
            &"x".repeat(16 * 1024 + 1)
        )
        .is_err()
    );
    let packet = candidate_decision_request(
        &candidate("waiting_approval"),
        CandidateDecision::Reject,
        "missing edge case",
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(
        value["action"],
        serde_json::json!({"kind":"reject","reason":"missing edge case"})
    );
}

#[test]
fn resume_and_recover_requests_are_revision_bound_and_state_checked() {
    let packet = run_action_request(&candidate("stopped"), RunAction::Resume).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-1");
    assert_eq!(value["action"]["kind"], "resume");
    assert!(run_action_request(&candidate("waiting_approval"), RunAction::Resume).is_err());
    assert!(run_action_request(&candidate("completed"), RunAction::Resume).is_err());
    let packet = run_action_request(&candidate("running"), RunAction::Recover).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["action"]["kind"], "recover");
    assert!(run_action_request(&candidate("stopped"), RunAction::Recover).is_err());
}

#[test]
fn tool_decision_is_bound_to_a_current_pending_receipt() {
    let id = "a".repeat(64);
    let mut snapshot = candidate("running");
    snapshot["tool_requests"] = serde_json::json!([{
        "id":id,
        "status":"pending",
        "request":{"tool":"Bash","input":{"command":"make test"}},
        "context":{"task":"test-suite"}
    }]);
    let packet = tool_action_request(&snapshot, &id, ToolAction::Approve).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-1");
    assert_eq!(
        value["action"],
        serde_json::json!({"kind":"tool_approve","sha":id})
    );
    snapshot["revision"] = "run-rev-2".into();
    let packet = tool_action_request(&snapshot, &id, ToolAction::Approve).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["revision"], "run-rev-2");
    snapshot["tool_requests"][0]["request"] = serde_json::Value::Null;
    assert!(tool_action_request(&snapshot, &id, ToolAction::Approve).is_err());
    snapshot["tool_requests"][0]["request"] =
        serde_json::json!({"tool":"Bash","input":{"command":"make test"}});
    snapshot["tool_requests"][0]["context"] = serde_json::Value::Null;
    assert!(tool_action_request(&snapshot, &id, ToolAction::Approve).is_err());
    snapshot["tool_requests"][0]["context"] = serde_json::json!({"task":"test-suite"});
    snapshot["tool_requests"][0]["status"] = "approved".into();
    let packet = tool_action_request(&snapshot, &id, ToolAction::Refresh).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&packet).unwrap();
    assert_eq!(value["action"]["kind"], "tool_refresh");
    assert!(tool_action_request(&snapshot, &id, ToolAction::Approve).is_err());
    assert!(tool_action_request(&snapshot, &"b".repeat(64), ToolAction::Reject).is_err());
    snapshot["tool_requests"][0]["status"] = "expired".into();
    assert!(tool_action_request(&snapshot, &id, ToolAction::Refresh).is_err());
}

#[test]
fn native_tool_panel_shows_receipt_and_drops_confirmation_when_stale() {
    let id = "c".repeat(64);
    let mut snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{"state":"running","candidate":null},
        "tool_requests":[{
            "id":id,
            "status":"pending",
            "request":{"tool":"Bash","input":{"command":"make test"},"reason":"Run integration tests"},
            "context":{"task":"test-suite"}
        }]
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::room::State::default();
    let labels = |output: &egui::FullOutput| {
        output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .filter_map(|(_, node)| node.label().map(str::to_owned))
            .collect::<Vec<_>>()
    };
    let rendered = |output: &egui::FullOutput| {
        output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let text = rendered(&output);
    let buttons = labels(&output);
    let has_tool = text.contains("Bash · Pending");
    let has_task = text.contains("Task · test-suite");
    let no_confirmation = !buttons.contains(&"Confirm tool decision".to_owned());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(has_tool, "{text}");
    assert!(has_task, "{text}");
    assert!(no_confirmation, "{buttons:?}");

    let evidence = serde_json::to_string(&[
        &snapshot["tool_requests"][0]["request"],
        &snapshot["tool_requests"][0]["context"],
    ])
    .unwrap();
    state.confirming_tool = Some((id.clone(), ToolAction::Approve, evidence));
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let confirmation_shown = labels(&output).contains(&"Confirm tool decision".to_owned());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(confirmation_shown);

    snapshot["revision"] = "rev-2".into();
    snapshot["tool_requests"][0]["status"] = "approved".into();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    let confirmation_hidden = !labels(&output).contains(&"Confirm tool decision".to_owned());
    let confirmation_cleared = state.confirming_tool.is_none();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(confirmation_hidden);
    assert!(confirmation_cleared);

    let approved_actions = labels(&output);
    assert!(approved_actions.contains(&format!("Refresh request tool request {id}")));
    assert!(!approved_actions.contains(&format!("Allow once tool request {id}")));
}
