#[test]
fn current_issue_nine_checkpoint_separates_product_head_from_last_pr_metadata() {
    let audit = include_str!("../docs/09-completion-audit.md");
    assert!(audit.contains(
        "latest product/test commit on issue #9 is `72db2aa2d2f3bf9664536c1839047366c0d5decf`"
    ));
    assert!(audit.contains("GitHub CLI query after commit `bafa5335220bf1ea0330bc39662cf9d937d265cc` confirmed that PR #22 is still open and draft, targets `main`, and is `CONFLICTING`."));
    assert!(audit.contains("15/25 (60%)"));
    assert!(audit.contains("shipped/default Rust UI replacement remains **0/25**"));
    let workflow = include_str!("../docs/evidence/issue9-workflow15-ui-isolated-20261023.json");
    assert!(workflow.contains("\"acceptance_credit\": 1"));
}
