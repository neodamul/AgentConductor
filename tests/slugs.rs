#![cfg(unix)]
//! R-55 / DD-61 / QAS-46: goals become accumulating slugs with a work tree.
use agentconductor::{git::Repository, slugs, ui::setup::Startup};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

const STAGES: [&str; 5] = ["plan", "phase", "task", "verification", "completed"];

#[test]
fn slugs_are_short_readable_and_unique() {
    assert_eq!(
        slugs::suggest("Adjust the health-check interval!"),
        "adjust_the_health_check_interval"
    );
    assert!(slugs::suggest(&"word ".repeat(30)).len() <= 48);
    // Non-English goals are named semantically by the existing planner, never romanized.
    assert_eq!(
        slugs::suggest_at("사다리 게임을 만들자 그런데", 1_791_676_800),
        "goal_20261011"
    );
    assert_eq!(
        slugs::suggest_at("결제 API 개선", 1_791_676_800),
        "goal_20261011"
    );
    assert_eq!(
        slugs::suggest_at("日本語だけ", 1_791_676_800),
        "goal_20261011"
    );
    // DD-92: only date placeholders are renamed after planning.
    assert!(slugs::is_placeholder("goal_20261011") && slugs::is_placeholder("goal_20261011_2"));
    assert!(!slugs::is_placeholder("goal_setting") && !slugs::is_placeholder("goal_2026101"));
    let long = slugs::suggest(&"헬스체크 ".repeat(10));
    assert!(long.len() <= 48 && !long.ends_with('_'), "{long}");
    slugs::validate(&long).unwrap();
    assert_eq!(
        slugs::unique("goal", &["goal".into(), "goal_2".into()]),
        "goal_3"
    );
    for bad in ["", "Bad", "has-dash", "_lead", &"a".repeat(49)] {
        assert!(slugs::validate(bad).is_err(), "{bad}");
    }
    slugs::validate("healthcheck_interval_adjust").unwrap();
}

fn snapshot(s: &Startup) -> serde_json::Value {
    s.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(s: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(s)["revision"].clone();
    s.dispatch(packet.to_string().as_bytes())
}

#[test]
fn goals_accumulate_as_slugs_with_trees_outside_git_status() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo").canonicalize().unwrap();
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let save = |goal: &str| {
        mutate(&startup, serde_json::json!({"kind":"save","config":{"name":"Health","profiles":plan.agents,"repository":repo,"goal":goal,"coordinator":"mock","workspace_mode":"shared"}})).unwrap()
    };
    save("Add the health check");
    mutate(
        &startup,
        serde_json::json!({"kind":"create_slug","slug":"healthcheck_feature"}),
    )
    .unwrap();
    for stage in STAGES {
        assert!(
            repo.join("healthcheck_feature").join(stage).is_dir(),
            "{stage}"
        );
    }
    Repository::open(&repo)
        .unwrap()
        .clean()
        .expect("goal folders are excluded from Git status");
    assert!(
        mutate(
            &startup,
            serde_json::json!({"kind":"create_slug","slug":"healthcheck_feature"})
        )
        .is_err(),
        "duplicate"
    );
    assert!(
        mutate(
            &startup,
            serde_json::json!({"kind":"create_slug","slug":"Bad Slug"})
        )
        .is_err(),
        "invalid"
    );

    save("Adjust the health check interval");
    mutate(&startup, serde_json::json!({"kind":"create_slug"})).unwrap();
    let config = snapshot(&startup)["config"].clone();
    let names: Vec<_> = config["slugs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["slug"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        names,
        ["healthcheck_feature", "adjust_the_health_check_interval"],
        "goals accumulate"
    );
    assert_eq!(config["slugs"][0]["goal"], "Add the health check");
    assert_eq!(config["active_slug"], "adjust_the_health_check_interval");
    drop(startup);

    // Without a repository the tree lives in the workspace folder.
    let bare = Startup::open(
        &root.join("bare"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&bare, serde_json::json!({"kind":"save","config":{"profiles":plan.agents,"repository":"","goal":"Notes only","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    mutate(&bare, serde_json::json!({"kind":"create_slug"})).unwrap();
    assert!(root.join("bare/notes_only/task").is_dir());
    drop(bare);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn tasks_cannot_own_the_slug_work_trees() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks[0].paths = vec![".agentconductor/healthcheck_feature/plan/plan.json".into()];
    assert!(plan.validate().is_err());
    std::fs::remove_dir_all(root).unwrap();
}
