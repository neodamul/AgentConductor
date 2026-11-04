//! #420 / R-211 / DD-223: real dispatcher races and alias roots preserve ownership.
use serde_json::{Value, json};
use std::{
    fs,
    sync::{Arc, Barrier},
};
#[allow(dead_code)] // Shared fixture also exposes historical-run scenarios.
#[path = "support/goal_actions.rs"]
mod support;
use support::Fixture;
fn race(f: &Fixture, mut first: Value, mut second: Value) -> [Result<Value, String>; 2] {
    let revision = f.snapshot()["revision"].clone();
    first["revision"] = revision.clone();
    second["revision"] = revision;
    let barrier = Arc::new(Barrier::new(2));
    let service = f.startup.as_ref().unwrap();
    std::thread::scope(|scope| {
        let one = barrier.clone();
        let two = barrier;
        let first = scope.spawn(move || {
            one.wait();
            service.dispatch(first.to_string().as_bytes())
        });
        let second = scope.spawn(move || {
            two.wait();
            service.dispatch(second.to_string().as_bytes())
        });
        [first.join().unwrap(), second.join().unwrap()]
    })
}
#[test]
fn duplicate_deletion_has_one_winner_and_never_restores_the_goal() {
    let f = Fixture::new();
    let answers = race(&f, f.delete(), f.delete());
    assert_eq!(
        answers
            .iter()
            .filter(|r| r.as_ref().is_ok_and(|v| v["deleted"] == true))
            .count(),
        1,
        "{answers:?}"
    );
    assert_eq!(
        answers.iter().filter(|r| r.is_err()).count(),
        1,
        "{answers:?}"
    );
    assert!(!f.folder().exists());
    assert_eq!(f.snapshot()["slugs"], json!([]));
}
#[test]
fn a_competing_editor_or_deletion_wins_without_lost_settings() {
    let f = Fixture::new();
    let mut config = f.snapshot()["config"].clone();
    config["name"] = json!("Competing edit");
    let result = race(&f, f.delete(), json!({"kind":"save","config":config}));
    assert_eq!(result.iter().filter(|r| r.is_ok()).count(), 1, "{result:?}");
    let after = f.snapshot();
    if result[0].is_ok() {
        assert!(!f.folder().exists());
        assert_eq!(after["config"]["name"], "Project");
        assert_eq!(after["slugs"], json!([]));
    } else {
        assert!(f.folder().exists());
        assert_eq!(after["config"]["name"], "Competing edit");
        assert_eq!(after["slugs"].as_array().unwrap().len(), 1);
    }
}
#[cfg(unix)]
#[test]
fn a_goal_alias_to_the_project_root_is_refused_without_touching_source() {
    let f = Fixture::new();
    let retained = f.root.join("retained-goal");
    fs::rename(f.folder(), &retained).unwrap();
    let source = f.root.join("project/source.txt");
    fs::write(&source, "keep").unwrap();
    std::os::unix::fs::symlink(f.root.join("project"), f.folder()).unwrap();
    assert!(
        f.call(json!({"kind":"goal_folder","slug":"first_goal"}))
            .is_err()
    );
    assert!(f.mutate(f.delete()).is_err());
    assert_eq!(fs::read_to_string(source).unwrap(), "keep");
    assert!(retained.join(".agentconductor").exists());
}
#[cfg(windows)]
#[test]
fn a_reparse_goal_to_the_project_root_is_refused_without_touching_source() {
    let f = Fixture::new();
    let retained = f.root.join("retained-goal");
    fs::rename(f.folder(), &retained).unwrap();
    let source = f.root.join("project/source.txt");
    fs::write(&source, "keep").unwrap();
    let plain = |p: std::path::PathBuf| p.to_string_lossy().trim_start_matches(r"\\?\").to_owned();
    let output = std::process::Command::new("cmd.exe")
        .args([
            "/D",
            "/S",
            "/C",
            r#"mklink /J "%AC_TEST_LINK%" "%AC_TEST_TARGET%""#,
        ])
        .env("AC_TEST_LINK", plain(f.folder()))
        .env("AC_TEST_TARGET", plain(f.root.join("project")))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "junction fixture: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        f.call(json!({"kind":"goal_folder","slug":"first_goal"}))
            .is_err()
    );
    assert!(f.mutate(f.delete()).is_err());
    assert_eq!(fs::read_to_string(source).unwrap(), "keep");
    assert!(retained.join(".agentconductor").exists());
}
