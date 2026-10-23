#[test]
fn current_issue_nine_checkpoint_separates_product_head_from_last_pr_metadata() {
    let audit = include_str!("../docs/09-completion-audit.md");
    assert!(audit.contains(
        "latest product/test commit on issue #9 is `72db2aa2d2f3bf9664536c1839047366c0d5decf`"
    ));
    assert!(audit.contains("GitHub CLI query at branch head `575fef74d160d6050a878e89d3bafa7f831432f5` confirmed PR #22 is open/draft, targets `main`, and is `CONFLICTING`."));
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
        "The last product/test code checkpoint remains `72db2aa2d2f3bf9664536c1839047366c0d5decf`; documentation and verification commits follow it."
    ));
    assert!(audit.contains(
        "GitHub CLI status at branch head `02cc8eb7` reports PR #22 open/draft, targets `main`, and merge state `UNKNOWN`; no checks were reported."
    ));
    assert!(audit.contains(
        "`origin/main` is `602ec118` and v0.1.37 is published; this Rust branch remains at v0.1.36. Recheck the latest release and align the next stable version (currently v0.1.38) before packaging."
    ));
    let windows = include_str!("../docs/evidence/issue9-windows-utm-login-20261023.json");
    assert!(windows.contains("\"windows_build_attempted\": false"));
    assert!(windows.contains("OSStatus -2700"));
}
