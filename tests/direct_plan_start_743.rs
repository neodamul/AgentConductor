#[test]
fn direct_planning_and_stale_context_are_checked_in_the_browser() {
    let status = std::process::Command::new("node")
        .arg("tests/direct_plan_start_743.cjs")
        .status()
        .expect("node must be available for the UI regression test");
    assert!(status.success(), "direct plan start regression failed");
}
