use agentconductor::native_ui_probe::room::{self, State};
use agentconductor::native_ui_probe::ToolAction;
use eframe::egui;

#[test]
fn pending_claude_tool_request_opens_an_accessible_decision_dialog() {
    let id = "a".repeat(64);
    let snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{"state":"running","candidate":null},
        "job":{"state":"running"},
        "tool_requests":[{
            "id":id,
            "status":"pending",
            "request":{"tool":"Bash","input":{"command":"cargo test"},"reason":"Run tests"},
            "context":{"task":"test-suite"}
        }]
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    let mut first = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    assert_eq!(state.tool_dialog.as_deref(), Some(id.as_str()));
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        text.contains("Tool request needs your decision")
            || labels
                .iter()
                .any(|label| label == "Tool request needs your decision"),
        "text={text}; labels={labels:?}"
    );
    assert!(text.contains("cargo test"), "{text}");
    assert!(
        labels
            .iter()
            .any(|label| label.starts_with("Allow once tool request ")),
        "{labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label.starts_with("Deny tool request ")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|label| label == "Decide later"),
        "{labels:?}"
    );
}

#[test]
fn deferred_request_stays_in_list_while_next_pending_request_opens() {
    let first_id = "a".repeat(64);
    let second_id = "b".repeat(64);
    let receipt = |id: &str, task: &str| {
        serde_json::json!({
            "id":id,
            "status":"pending",
            "request":{"tool":"Bash","input":{"command":"cargo test"},"reason":"Run tests"},
            "context":{"task":task}
        })
    };
    let snapshot = serde_json::json!({
        "revision":"rev-1",
        "report":{"state":"running","candidate":null},
        "job":{"state":"running"},
        "tool_requests":[receipt(&first_id,"first-task"),receipt(&second_id,"second-task")]
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.deferred_tools.insert(first_id.clone());
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_eq!(state.tool_dialog.as_deref(), Some(second_id.as_str()));
    assert!(
        labels
            .iter()
            .any(|label| { label == &format!("Allow once tool request {second_id}") })
    );
}

fn confirmation_view(state: &mut State, snapshot: &serde_json::Value) -> (String, Vec<String>) {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = context.run_ui(Default::default(), |ui| {
        room::show(state, Some(snapshot), false, None, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(state, Some(snapshot), false, None, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (text, labels)
}

fn confirmation_count(labels: &[String]) -> usize {
    labels
        .iter()
        .filter(|label| label.as_str() == "Confirm tool request decision")
        .count()
}

#[test]
fn deferred_confirmation_uses_the_existing_modal_instead_of_a_second_renderer() {
    let deferred = "a".repeat(64);
    let next = "b".repeat(64);
    let receipt = |id: &str, task: &str| serde_json::json!({
        "id": id, "status": "pending",
        "request": {"tool": "Bash", "input": {"command": "cargo test"}, "reason": "Run tests"},
        "context": {"task": task}
    });
    let snapshot = serde_json::json!({
        "revision": "rev-1", "report": {"state": "running"}, "job": {"state": "running"},
        "tool_requests": [receipt(&deferred, "deferred-task"), receipt(&next, "next-task")]
    });
    let mut state = State::default();
    state.deferred_tools.insert(deferred.clone());
    state.confirming_tool = Some((deferred.clone(), ToolAction::Refresh, "rev-1".into()));

    let (text, labels) = confirmation_view(&mut state, &snapshot);

    assert_eq!(state.tool_dialog.as_deref(), Some(deferred.as_str()));
    assert!(text.contains("Tool request needs your decision"), "{text}");
    assert!(text.contains("deferred-task"), "{text}");
    assert!(!text.contains("next-task"), "{text}");
    assert_eq!(confirmation_count(&labels), 1, "{labels:?}");
    assert!(state.deferred_tools.contains(&deferred));
}

#[test]
fn approved_refresh_confirmation_uses_the_existing_modal_and_current_receipt() {
    let approved = "c".repeat(64);
    let next = "d".repeat(64);
    let receipt = |id: &str, status: &str, task: &str| serde_json::json!({
        "id": id, "status": status,
        "request": {"tool": "Bash", "input": {"command": "cargo test"}, "reason": "Run tests"},
        "context": {"task": task}
    });
    let snapshot = serde_json::json!({
        "revision": "rev-1", "report": {"state": "running"}, "job": {"state": "running"},
        "tool_requests": [receipt(&approved, "approved", "approved-task"), receipt(&next, "pending", "next-task")]
    });
    let mut state = State::default();
    state.confirming_tool = Some((approved.clone(), ToolAction::Refresh, "rev-1".into()));

    let (text, labels) = confirmation_view(&mut state, &snapshot);

    assert_eq!(state.tool_dialog.as_deref(), Some(approved.as_str()));
    assert!(text.contains("Tool request needs your decision"), "{text}");
    assert!(text.contains("approved-task"), "{text}");
    assert!(!text.contains("next-task"), "{text}");
    assert_eq!(confirmation_count(&labels), 1, "{labels:?}");
}
