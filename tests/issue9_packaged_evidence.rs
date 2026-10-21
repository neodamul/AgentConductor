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
