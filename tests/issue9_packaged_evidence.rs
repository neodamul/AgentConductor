#[test]
fn uc79_exact_package_archive_preserves_recovery_and_project_data() {
    let evidence: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/evidence/issue9-256db1a-uc79-archive-ui.json"
    ))
    .unwrap();
    assert_eq!(evidence["application_version"], "0.1.32");
    assert_eq!(evidence["journey"]["confirmation_accepted"], true);
    assert_eq!(
        evidence["journey"]["archived_workspace_removed_from_active_registry"],
        true
    );
    assert_eq!(evidence["journey"]["project_folder_preserved"], true);
    assert!(
        evidence["journey"]["recovery_record"]
            .as_str()
            .unwrap()
            .ends_with("workspace-deletion.json")
    );
    assert_eq!(evidence["acceptance"]["adds_distinct_workflow"], false);
}

#[test]
fn mcp_registration_evidence_covers_saved_opencode_server_and_tool_discovery() {
    let evidence: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/evidence/issue9-256db1a-mcp-server-registration-ui.json"
    ))
    .unwrap();
    assert_eq!(evidence["registration"]["format"], "OpenCode JSON/JSONC");
    assert_eq!(evidence["registration"]["type"], "local");
    assert_eq!(evidence["registration"]["saved_and_reopened"], true);
    assert_eq!(evidence["discovery"]["tools"][0], "get_issue");
    assert_eq!(evidence["discovery"]["tools_call_observed"], false);
    assert_eq!(evidence["acceptance"]["after"], "8/25 (32%)");
}
