#[test]
fn ci_runs_and_lints_the_workspace_menu_regressions() {
    let workflow = include_str!("../.github/workflows/platform-validation.yml");
    for test in [
        "native_ui_workspace_menu_layout",
        "native_ui_goal_menu_layout",
    ] {
        assert_eq!(
            workflow.matches(&format!("--test {test}")).count(),
            2,
            "{test} must run and lint in the native UI job"
        );
    }
}
