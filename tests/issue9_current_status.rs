#[test]
fn current_issue_nine_checkpoint_separates_product_head_from_last_pr_metadata() {
    let audit = include_str!("../docs/09-completion-audit.md");
    assert!(audit.contains(
        "latest product/test commit on issue #9 is `72db2aa2d2f3bf9664536c1839047366c0d5decf`"
    ));
    assert!(audit.contains("last successful GitHub PR #22 metadata check was at `b282fca47792890bbe85169652164f4ee56c7e42`"));
    assert!(audit.contains("current PR metadata has not been re-confirmed"));
    assert!(audit.contains("15/25 (60%)"));
    assert!(audit.contains("shipped/default Rust UI replacement remains **0/25**"));
    let workflow = include_str!("../docs/evidence/issue9-workflow15-ui-isolated-20261023.json");
    assert!(workflow.contains("\"acceptance_credit\": 1"));
}
