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

#[test]
fn cff95dd_mcp_dialog_visual_evidence_does_not_claim_a_new_workflow() {
    let evidence: serde_json::Value = serde_json::from_str(include_str!(
        "../docs/evidence/issue9-cff95dd-mcp-dialog.json"
    ))
    .unwrap();
    assert_eq!(
        evidence["source_commit"],
        "cff95dd02677519ebbb7c359f5ca9e3da0c504ed"
    );
    assert_eq!(
        evidence["archive_sha256"],
        "dafc056361c6e68a1165f30db1f3bd8a0cf99d24b59876d7d6eb7d2aa1ad44d8"
    );
    assert_eq!(
        evidence["visual_inspection"]["views"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(evidence["environment"]["user_workspace_modified"], false);
    assert_eq!(evidence["acceptance"]["distinct_workflow_added"], false);
    assert_eq!(evidence["acceptance"]["after"], "11/25 (44%)");
}
