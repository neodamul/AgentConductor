//! Coordinator directory scopes such as `docs/` are trimmed, not rejected as unsafe.
use agentconductor::plan::{Task, trim_directory_slashes};

fn task(paths: &[&str]) -> Task {
    serde_json::from_value(serde_json::json!({
        "id": "t", "agent": "claude", "prompt": "p", "paths": paths, "test_command": ["true"],
    }))
    .unwrap()
}

#[test]
fn trailing_directory_slashes_are_trimmed() {
    let mut tasks = vec![task(&["ladder_game/docs/", "src/static//", "README.md"])];
    trim_directory_slashes(&mut tasks);
    assert_eq!(
        tasks[0].paths,
        ["ladder_game/docs", "src/static", "README.md"]
    );
}

#[test]
fn a_bare_slash_is_left_for_validation_to_reject() {
    let mut tasks = vec![task(&["/"])];
    trim_directory_slashes(&mut tasks);
    assert_eq!(tasks[0].paths, ["/"]);
}
