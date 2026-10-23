#[test]
fn current_issue_nine_checkpoint_records_the_open_pr_head_and_acceptance_boundary() {
    let audit = include_str!("../docs/09-completion-audit.md");
    assert!(audit.contains(
        "PR #22 head observed before this documentation synchronization: `cba882af115dacbefadb8a137802bb57f957e78a`"
    ));
    assert!(audit.contains("14/25 (56%)"));
    assert!(audit.contains("shipped/default Rust UI replacement remains **0/25**"));
}
