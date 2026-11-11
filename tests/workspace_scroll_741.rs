#[test]
fn long_goal_tools_remain_visible_above_the_fixed_footer() {
    let status = std::process::Command::new("node")
        .arg("tests/workspace_scroll_741.cjs")
        .status()
        .expect("node must be available for the workspace scroll test");
    assert!(status.success(), "workspace scrolling regression failed");
}
