#[test]
fn current_issue_nine_checkpoint_records_the_open_pr_head_and_acceptance_boundary() {
    let audit = include_str!("../docs/09-completion-audit.md");
    assert!(audit.contains(
        "PR #22 was pushed through `b282fca47792890bbe85169652164f4ee56c7e42`"
    ));
    assert!(audit.contains("15/25 (60%)"));
    assert!(audit.contains("shipped/default Rust UI replacement remains **0/25**"));
    let workflow = include_str!("../docs/evidence/issue9-workflow15-ui-isolated-20261023.json");
    assert!(workflow.contains("\"acceptance_credit\": 1"));
}
