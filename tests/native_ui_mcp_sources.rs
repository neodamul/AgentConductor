#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::mcp::{
    SourceImport, append_reference, source_apply_request, source_preview_request,
};
use eframe::egui;
use serde_json::{Value, json};

#[test]
fn native_source_preview_requires_explicit_goal_references_and_saved_revision() {
    let snapshot = json!({
        "revision": "rev-8",
        "config": {"goal": "Implement jira:EMM-52599 and confluence:53553565"}
    });

    let request: Value =
        serde_json::from_slice(&source_preview_request(&snapshot).unwrap()).unwrap();

    assert_eq!(request["kind"], "mcp_preview");
    assert_eq!(request["revision"], "rev-8");
    assert!(
        source_preview_request(&json!({
            "revision": "rev-8", "config": {"goal": "Add generic source imports"}
        }))
        .is_err()
    );
    assert!(
        source_preview_request(&json!({
            "revision": "", "config": {"goal": "jira:EMM-52599"}
        }))
        .is_err()
    );
}

#[test]
fn native_source_apply_uses_only_displayed_records_and_keeps_jira_required() {
    let snapshot = json!({"revision": "rev-8"});
    let preview = json!({
        "id": "preview-1",
        "records": [
            {"reference": {"kind": "jira", "id": "EMM-52599"}},
            {"reference": {"kind": "confluence", "id": "53553565"}}
        ]
    });

    let request: Value = serde_json::from_slice(
        &source_apply_request(&snapshot, &preview, &["jira:EMM-52599".into()]).unwrap(),
    )
    .unwrap();

    assert_eq!(request["kind"], "mcp_apply");
    assert_eq!(request["revision"], "rev-8");
    assert_eq!(request["preview_id"], "preview-1");
    assert_eq!(request["selected"], json!(["jira:EMM-52599"]));
    assert!(source_apply_request(&snapshot, &preview, &["confluence:999".into()]).is_err());
    assert!(source_apply_request(&snapshot, &preview, &[]).is_err());
    assert!(source_apply_request(&snapshot, &preview, &["confluence:53553565".into()]).is_err());
    assert!(source_apply_request(&json!({}), &preview, &["jira:EMM-52599".into()]).is_err());
    let mut failed = preview;
    failed["failures"] = json!([{"reference": {"kind": "jira", "id": "EMM-52599"}}]);
    assert!(source_apply_request(&snapshot, &failed, &["jira:EMM-52599".into()]).is_err());
}

#[test]
fn native_source_reference_insertion_keeps_valid_explicit_keys_without_duplicates() {
    let mut goal = "Implement authentication".to_owned();

    append_reference(&mut goal, "jira", "EMM-52599").unwrap();
    append_reference(&mut goal, "confluence", "53553565").unwrap();
    append_reference(&mut goal, "jira", "EMM-52599").unwrap();

    assert_eq!(
        goal,
        "Implement authentication jira:EMM-52599 confluence:53553565"
    );
    assert!(append_reference(&mut goal, "jira", "unsafe key").is_err());
    assert!(append_reference(&mut goal, "confluence", "not-a-page").is_err());
}

#[test]
fn native_workspace_exposes_accessible_reference_and_preview_controls() {
    let snapshot = json!({
        "config": {
            "goal": "Implement jira:EMM-52599",
            "coordinator": "claude",
            "profiles": [{"id": "claude"}],
            "mcp": {"company": {"type": "local", "command": ["company-mcp"]}}
        }
    });
    let mut goal = "Implement jira:EMM-52599".to_owned();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = SourceImport::default();
    let mut first = context.run_ui(Default::default(), |ui| {
        state.show(&mut goal, Some(&snapshot), "en", true, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        state.show(&mut goal, Some(&snapshot), "en", true, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    assert!(labels.contains(&"+ Jira story"), "{labels:?}");
    assert!(labels.contains(&"+ Confluence page"), "{labels:?}");
    assert!(labels.contains(&"Preview imports"), "{labels:?}");
}

#[test]
fn native_workspace_source_controls_are_accessible_in_korean() {
    let snapshot = json!({
        "config": {
            "goal": "Implement jira:EMM-52599",
            "coordinator": "claude",
            "profiles": [{"id": "claude"}],
            "mcp": {"company": {"type": "local", "command": ["company-mcp"]}}
        }
    });
    let mut goal = "Implement jira:EMM-52599".to_owned();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = SourceImport::default();
    let mut first = context.run_ui(Default::default(), |ui| {
        state.show(&mut goal, Some(&snapshot), "ko", true, ui);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        state.show(&mut goal, Some(&snapshot), "ko", true, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    assert!(labels.contains(&"+ Jira 스토리"), "{labels:?}");
    assert!(labels.contains(&"+ Confluence 문서"), "{labels:?}");
    assert!(labels.contains(&"문서 미리보기"), "{labels:?}");
}
