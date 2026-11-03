//! GitHub #341: a custom CLI that refuses to run tools, or prints text instead of JSON, says so instead of "invalid JSONL".
use agentconductor::{
    plan::AgentProfile,
    protocol::{Decoder, EventKind},
};
use serde_json::json;
#[cfg(unix)]
#[path = "support/workflow.rs"]
mod support;

fn profile(extra: serde_json::Value) -> AgentProfile {
    let mut base = json!({
        "id": "aipro", "executable": "aipro", "args": ["run", "{task_file}"], "parser": "custom",
        "permission_mode": "operator_auto_approve"
    });
    base.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(base).unwrap()
}

fn events(
    profile: &AgentProfile,
    output: &str,
) -> Result<Vec<(EventKind, serde_json::Value)>, String> {
    let mut decoder = Decoder::for_profile(profile, 1 << 20);
    let mut all = decoder.push(output.as_bytes())?;
    all.extend(decoder.finish()?);
    Ok(all.into_iter().map(|e| (e.kind, e.payload)).collect())
}

#[test]
fn an_invalid_line_is_quoted_in_the_failure() {
    let error = events(
        &profile(json!({})),
        "Permission denied by non-interactive policy\n",
    )
    .unwrap_err();
    assert!(
        error.contains("invalid JSONL")
            && error.contains("Permission denied by non-interactive policy"),
        "{error}"
    );
    let long = format!("{}\n", "y".repeat(5000));
    let error = events(&profile(json!({})), &long).unwrap_err();
    assert!(error.len() < 600, "the excerpt is bounded: {}", error.len());
}

#[test]
fn text_mode_records_non_json_lines_as_progress_and_still_needs_a_result() {
    let p = profile(json!({"on_invalid_json": "text"}));
    let got = events(&p, "starting up\n{\"type\":\"result\",\"ok\":true}\n").unwrap();
    assert_eq!(got[0].0, EventKind::Progress);
    assert_eq!(got[0].1["text"], "starting up");
    assert_eq!(got[1].0, EventKind::Result);
    // Without a result event nothing here is a success: the worker requires one.
    let got = events(&p, "just text\n").unwrap();
    assert!(got.iter().all(|(kind, _)| *kind != EventKind::Result));
}

#[test]
fn text_that_reads_as_a_refusal_is_a_configuration_error_not_progress() {
    let p = profile(json!({"on_invalid_json": "text"}));
    let got = events(
        &p,
        "Tool bash: Permission denied by non-interactive policy\n",
    )
    .unwrap();
    assert_eq!(got[0].0, EventKind::Error);
    let message = got[0].1["message"].as_str().unwrap();
    assert!(
        message.contains("check the CLI configuration") && message.contains("Permission denied"),
        "{message}"
    );
}

#[test]
fn a_declared_denial_on_a_tool_event_is_reported_with_tool_and_message() {
    let p = profile(json!({"denial_condition": {
        "event_type": "tool", "pointer": "/status", "equals": "error",
        "tool_pointer": "/name", "message_pointer": "/message"
    }}));
    let line = r#"{"type":"tool","name":"bash","status":"error","message":"Permission denied by non-interactive policy"}"#;
    let got = events(&p, &format!("{line}\n")).unwrap();
    assert_eq!(got[0].0, EventKind::Denied);
    let text = p.denial_condition.as_ref().unwrap().describe(&got[0].1);
    assert!(
        text.contains("permission denied")
            && text.contains("bash")
            && text.contains("non-interactive policy"),
        "{text}"
    );
    // Another event type or a successful tool is not a denial.
    let ok = r#"{"type":"tool","name":"bash","status":"ok"}"#;
    assert_eq!(
        events(&p, &format!("{ok}\n")).unwrap()[0].0,
        EventKind::Tool
    );
    let other = r#"{"type":"progress","status":"error"}"#;
    assert_ne!(
        events(&p, &format!("{other}\n")).unwrap()[0].0,
        EventKind::Denied
    );
}

#[test]
fn a_message_match_declares_a_denial_without_a_status_field() {
    let p = profile(
        json!({"denial_condition": {"pointer": "/message", "contains": "permission denied"}}),
    );
    let got = events(
        &p,
        "{\"type\":\"tool\",\"message\":\"Permission denied by policy\"}\n",
    )
    .unwrap();
    assert_eq!(got[0].0, EventKind::Denied);
}

#[test]
fn the_new_settings_are_validated_and_custom_only() {
    assert!(
        profile(json!({"denial_condition": {"pointer": "status", "equals": "error"}}))
            .validate_custom_configuration()
            .is_err()
    );
    assert!(
        profile(json!({"denial_condition": {"pointer": "/status"}}))
            .validate_custom_configuration()
            .is_err()
    );
    assert!(
        profile(
            json!({"denial_condition": {"pointer": "/status", "equals": "e", "contains": "e"}})
        )
        .validate_custom_configuration()
        .is_err()
    );
    let known: AgentProfile = serde_json::from_value(json!({"id": "c", "executable": "claude", "args": [], "parser": "claude", "on_invalid_json": "text"})).unwrap();
    assert!(known.validate_custom_configuration().is_err());
    let saved = serde_json::to_value(profile(json!({"on_invalid_json": "text"}))).unwrap();
    assert_eq!(saved["on_invalid_json"], "text");
    assert!(
        serde_json::to_value(profile(json!({})))
            .unwrap()
            .get("on_invalid_json")
            .is_none()
    );
}

#[test]
fn automatic_approval_without_any_way_to_see_a_denial_gets_a_hint() {
    assert!(profile(json!({})).configuration_hint().is_some());
    assert!(
        profile(json!({"denial_condition": {"pointer": "/status", "equals": "error"}}))
            .configuration_hint()
            .is_none()
    );
    assert!(
        profile(json!({"event_types": {"finished": "result"}}))
            .configuration_hint()
            .is_none()
    );
    assert!(
        profile(json!({"permission_mode": "read_only"}))
            .configuration_hint()
            .is_none()
    );
    assert!(
        profile(json!({"protocol": "acio"}))
            .configuration_hint()
            .is_none()
    );
}

/// End to end: a shell script stands in for a custom CLI that refuses tools while exiting 0.
#[cfg(unix)]
mod worker {
    use super::support;
    use agentconductor::workflow;
    use serde_json::json;

    async fn run_with(script: &str, extra: serde_json::Value) -> workflow::Report {
        let (root, _) = support::setup("shared");
        // Shells are refused as agent executables, so the stand-in CLI is a script file; it reads its input first, since a CLI that exits before the prompt is written would fail the run with a broken pipe.
        let cli = root.join("fake-cli");
        std::fs::write(&cli, format!("#!/bin/sh\ncat >/dev/null\n{script}\n")).unwrap();
        std::fs::set_permissions(&cli, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .unwrap();
        let mut agent = json!({"id":"aipro","executable":cli,"args":["run"],"parser":"custom",
            "permission_mode":"operator_auto_approve"});
        agent
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let plan = json!({"goal":"one file","workspace_mode":"shared","agents":[agent],
            "tasks":[{"id":"one.txt","agent":"aipro","prompt":"p","paths":["one.txt"],"test_command":["true"]}]});
        let report = workflow::run(
            serde_json::from_value(plan).unwrap(),
            &root.join("repo"),
            &root.join("run"),
            5,
        )
        .await
        .unwrap();
        std::fs::remove_dir_all(root).unwrap();
        report
    }

    #[tokio::test]
    async fn plain_text_output_fails_with_what_the_cli_printed() {
        let report = run_with(
            "echo 'Permission denied by non-interactive policy'; echo 'stderr note' >&2",
            json!({}),
        )
        .await;
        let error = report.tasks[0].error.clone().unwrap_or_default();
        assert_eq!(report.tasks[0].status, "failed", "{error}");
        assert!(
            error.contains("Permission denied by non-interactive policy"),
            "{error}"
        );
        assert!(
            error.contains("stderr note"),
            "stderr is shown apart: {error}"
        );
    }

    #[tokio::test]
    async fn a_declared_denial_fails_a_run_that_exited_zero_with_a_result() {
        let script = r#"echo '{"type":"tool","name":"bash","status":"error","message":"Permission denied by non-interactive policy"}'; echo '{"type":"result"}'"#;
        let denial = json!({"denial_condition":{"event_type":"tool","pointer":"/status","equals":"error","tool_pointer":"/name","message_pointer":"/message"}});
        let report = run_with(script, denial).await;
        let error = report.tasks[0].error.clone().unwrap_or_default();
        assert_eq!(report.tasks[0].status, "failed");
        assert!(error.starts_with("permission denied: tool bash"), "{error}");
        // Without the declaration the same output reads as a success, which is what the condition is for.
        let report = run_with(script, json!({})).await;
        assert_eq!(
            report.tasks[0].status, "completed",
            "{:?}",
            report.tasks[0].error
        );
    }

    #[tokio::test]
    async fn text_mode_with_a_result_event_completes_and_without_one_fails() {
        let text = json!({"on_invalid_json":"text"});
        let ok = run_with(
            "echo 'warming up'; echo '{\"type\":\"result\"}'",
            text.clone(),
        )
        .await;
        assert_eq!(ok.tasks[0].status, "completed", "{:?}", ok.tasks[0].error);
        let none = run_with("echo 'warming up'", text.clone()).await;
        assert_eq!(none.tasks[0].status, "failed");
        let refused = run_with(
            "echo 'Permission denied by policy'; echo '{\"type\":\"result\"}'",
            text,
        )
        .await;
        let error = refused.tasks[0].error.clone().unwrap_or_default();
        assert_eq!(refused.tasks[0].status, "failed");
        assert!(error.contains("check the CLI configuration"), "{error}");
    }
}
