use agentconductor::native_ui_probe::room::{self, State};
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
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
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
        text.contains("Tool request needs a decision")
            || labels
                .iter()
                .any(|label| label == "Tool request needs a decision"),
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
fn pending_tool_decision_uses_korean_text_and_accessible_names() {
    let id = "c".repeat(64);
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
        room::show(&mut state, Some(&snapshot), false, None, "ko", ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "ko", ui);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect::<Vec<_>>();
    assert!(text.contains("도구 요청 · 1"), "{text}");
    assert!(text.contains("도구 요청 결정을 기다립니다"), "{text}");
    assert!(
        labels
            .iter()
            .any(|label| label == &format!("한 번 허용 도구 요청 {id}")),
        "{labels:?}"
    );
    assert!(
        labels
            .iter()
            .any(|label| label == &format!("거부 도구 요청 {id}")),
        "{labels:?}"
    );
    assert!(
        labels.iter().any(|label| label == "나중에 결정"),
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
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        room::show(&mut state, Some(&snapshot), false, None, "en", ui);
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
