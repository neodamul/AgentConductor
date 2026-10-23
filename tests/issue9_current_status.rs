#[test]
fn current_issue_nine_checkpoint_tracks_live_pr_release_and_utm_evidence() {
    let audit = include_str!("../docs/09-completion-audit.md");
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
}
