#[test]
fn current_issue_nine_checkpoint_records_the_open_pr_head_and_acceptance_boundary() {
    let audit = include_str!("../docs/09-completion-audit.md");
    assert!(audit.contains("PR #22 head: `b6b44f7c4e59e3955ae2742d3029d3a7af791067`"));
    assert!(audit.contains("14/25 (56%)"));
    assert!(audit.contains("shipped/default Rust UI replacement remains **0/25**"));
}
