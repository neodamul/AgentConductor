#[test]
fn current_issue_nine_checkpoint_tracks_live_pr_release_and_utm_evidence() {
    let plan = include_str!("../docs/07-implementation-plan.md");
    let audit = include_str!("../docs/09-completion-audit.md");
    let wiki = include_str!("../knowledge/wiki/native-rust-ui-migration.md");
    let clippy = include_str!("../docs/evidence/issue9-pr-clippy-fix-20261023.json");
    assert!(clippy.contains("\"clippy_all_targets_test_fixtures_warnings_denied\": \"passed\""));
    assert!(
        clippy.contains("\"failure\": \"manual_contains in tests/native_ui_task_activity.rs\"")
    );
    assert!(plan.contains("### Fix the macOS PR lint failure — issue #9 / 2026-10-23"));
    assert!(wiki.contains("### PR #22 macOS lint correction — issue #9 / 2026-10-23"));
    assert!(audit.contains(
        "latest product/test commit on issue #9 is `72db2aa2d2f3bf9664536c1839047366c0d5decf`"
    ));
    assert!(audit.contains("15/25 (60%)"));
    assert!(audit.contains("shipped/default Rust UI replacement remains **0/25**"));
    let workflow = include_str!("../docs/evidence/issue9-workflow15-ui-isolated-20261023.json");
    assert!(workflow.contains("\"acceptance_credit\": 1"));
    let package =
        include_str!("../docs/evidence/issue9-current-source-package-manual-20261023.json");
    assert!(package.contains("\"acceptance_credit\": 0"));
    assert!(package.contains("\"manual_opened_in_exact_package\": true"));
    assert!(package.contains("\"embedded_workspace_screenshot_rendered\": true"));
    assert!(package.contains("\"embedded_agent_room_screenshot_rendered\": true"));
    assert!(audit.contains(
        "The latest product/test code checkpoint remains `72db2aa2d2f3bf9664536c1839047366c0d5decf`; current branch head is the documentation checkpoint `8b50d0bdddb04281e37b21c9177cc73f15beb1ff`."
    ));
    assert!(audit.contains(
        "`origin/main` is `602ec118`; v0.1.37 is the latest published release, while this branch is v0.1.36. Recheck and align the next stable version before packaging."
    ));
    assert!(audit.contains(
        "Current live checkpoint at branch head `8b50d0bd` reports PR #22 open/draft, `DIRTY`, and no checks."
    ));
    assert!(audit.contains("A fresh read-only `git merge-tree --write-tree --name-only HEAD origin/main` reports **39 conflict paths**"));
    let windows = include_str!("../docs/evidence/issue9-windows-utm-login-20261023.json");
    assert!(windows.contains("\"windows_build_attempted\": false"));
    assert!(windows.contains("OSStatus -2700"));
    let transfer = include_str!("../docs/evidence/issue9-utm-build-transfer-20261023.json");
    assert!(transfer.contains("\"branch_head\": \"8b50d0bdddb04281e37b21c9177cc73f15beb1ff\""));
    assert!(transfer.contains("\"guest_agent\": \"unavailable\""));
    assert!(transfer.contains("\"conflict_count\": 39"));
    assert!(transfer.contains(
        "\"sha256\": \"94a0621b4ee4d110931455763e6220e8767a7afc08d555aa0fdb316581ca7a90\""
    ));
    assert!(transfer.contains("\"windows_build_attempted\": false"));
    let share = include_str!("../docs/evidence/issue9-utm-z-share-transfer-20261023.json");
    assert!(share.contains("\"branch_head\": \"c4cbd2a6c13b5ab59f50b1efa37842a54a9c7a9c\""));
    assert!(share.contains("\"host_share_path\": \"/Users/neodamul/Downloads/UTM\""));
    assert!(share.contains("\"guest_access_verified\": false"));
    assert!(share.contains("\"restart_or_shutdown\": false"));
    assert!(audit.contains("issue9-utm-z-share-transfer-20261023.json"));

    let current_share = include_str!("../docs/evidence/issue9-utm-z-share-head-20261023.json");
    assert!(
        current_share.contains("\"branch_head\": \"d1e1b3746b3ea08cbaa09c1ac167e0c569c9810d\"")
    );
    assert!(current_share.contains(
        "\"host_copy_sha256\": \"6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd\""
    ));
    assert!(current_share.contains("\"guest_access_verified\": false"));
    assert!(current_share.contains("\"restart_or_shutdown_selected\": false"));
    assert!(audit.contains("AgentConductor-d1e1b374.tar.gz"));
    assert!(audit.contains("Source and destination SHA-256 both equal"));
    assert!(audit.contains("issue9-utm-z-share-head-20261023.json"));
    let current_utm = include_str!("../docs/evidence/issue9-utm-screen-recheck-20261023.json");
    assert!(current_utm.contains("\"vm_status\": \"started\""));
    assert!(current_utm.contains("\"restart_or_shutdown\": false"));
    assert!(current_utm.contains("\"guest_z_access_verified\": false"));
    assert!(plan.contains("Windows restart-apps confirmation is visible"));
    assert!(audit.contains("Windows restart-apps confirmation is visible"));
    assert!(wiki.contains("Windows restart-apps confirmation is visible"));
}
