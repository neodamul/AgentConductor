#![cfg(unix)]
//! R-58 / DD-64 / QAS-49: export writes one listed archive per slug; import verifies them.
use agentconductor::{phases, slug_progress::progress, ui::setup::Startup};
use std::{path::Path, process::Command};
#[path = "support/workflow.rs"]
mod support;

fn snapshot(s: &Startup) -> serde_json::Value {
    s.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(s: &Startup, mut packet: serde_json::Value) -> Result<serde_json::Value, String> {
    packet["revision"] = snapshot(s)["revision"].clone();
    s.dispatch(packet.to_string().as_bytes())
}
fn git(dir: &Path, args: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn slugs_travel_as_verified_archives() {
    let (root, plan) = support::setup("shared");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let repo_a = root.join("repo").canonicalize().unwrap();
    let a = Startup::open(&root.join("dev-a"), exe).unwrap();
    let save = |s: &Startup, repo: &Path, goal: &str| {
        mutate(s, serde_json::json!({"kind":"save","config":{"name":"Health","profiles":plan.agents,"repository":repo,"goal":goal,"coordinator":"mock","workspace_mode":"shared"}})).unwrap()
    };
    save(&a, &repo_a, "Add the health check");
    mutate(
        &a,
        serde_json::json!({"kind":"create_slug","slug":"healthcheck_feature"}),
    )
    .unwrap();
    let tree = repo_a.join("Health/healthcheck_feature");
    let mut t = plan.tasks[0].clone();
    t.id = "a".into();
    phases::write(
        &tree,
        "# Analysis\n",
        &[phases::PhaseSpec {
            id: "01-core".into(),
            name: "core".into(),
            tasks: vec![t],
        }],
    )
    .unwrap();
    phases::advance_phase(&tree, "01-core", "verification").unwrap();
    save(&a, &repo_a, "Adjust the interval");
    mutate(
        &a,
        serde_json::json!({"kind":"create_slug","slug":"interval_adjust"}),
    )
    .unwrap();
    mutate(
        &a,
        serde_json::json!({"kind":"export_settings","folder":repo_a}),
    )
    .unwrap();
    let file: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_a.join("agentconductor.workspace.json")).unwrap(),
    )
    .unwrap();
    let slugs = file["slugs"].as_array().unwrap();
    assert_eq!(slugs.len(), 2);
    assert_eq!(slugs[0]["archive"], "healthcheck_feature.zip");
    assert_eq!(slugs[0]["progress"]["phases"][0]["stage"], "verification");
    assert!(
        repo_a.join("healthcheck_feature.zip").exists()
            && repo_a.join("interval_adjust.zip").exists()
    );

    // Developer B has another clone and receives the exported files.
    let repo_b = root.join("repo-b");
    git(
        &root,
        &[
            "clone",
            "-q",
            repo_a.to_str().unwrap(),
            repo_b.to_str().unwrap(),
        ],
    );
    for f in [
        "agentconductor.workspace.json",
        "healthcheck_feature.zip",
        "interval_adjust.zip",
    ] {
        std::fs::copy(repo_a.join(f), repo_b.join(f)).unwrap();
    }
    let repo_b = repo_b.canonicalize().unwrap();
    let b = Startup::open(&root.join("dev-b"), exe).unwrap();
    save(&b, &repo_b, "local goal");
    std::fs::rename(repo_b.join("interval_adjust.zip"), repo_b.join("moved.zip")).unwrap();
    assert!(
        mutate(
            &b,
            serde_json::json!({"kind":"import_settings","folder":repo_b,"keep_verified":false})
        )
        .is_err(),
        "missing archive"
    );
    std::fs::rename(repo_b.join("moved.zip"), repo_b.join("interval_adjust.zip")).unwrap();
    let mut tampered = file.clone();
    tampered["slugs"][0]["progress"]["phases"][0]["stage"] = "completed".into();
    std::fs::write(
        repo_b.join("agentconductor.workspace.json"),
        serde_json::to_string_pretty(&tampered).unwrap(),
    )
    .unwrap();
    assert!(
        mutate(
            &b,
            serde_json::json!({"kind":"import_settings","folder":repo_b,"keep_verified":false})
        )
        .is_err(),
        "progress mismatch"
    );
    assert!(
        !repo_b.join("Health/healthcheck_feature").exists(),
        "a refused import changes nothing"
    );
    std::fs::copy(
        repo_a.join("agentconductor.workspace.json"),
        repo_b.join("agentconductor.workspace.json"),
    )
    .unwrap();
    mutate(
        &b,
        serde_json::json!({"kind":"import_settings","folder":repo_b,"keep_verified":false}),
    )
    .unwrap();
    let restored = repo_b.join("Health/healthcheck_feature");
    assert_eq!(
        progress(&restored),
        progress(&tree),
        "same progress after import"
    );
    assert!(repo_b.join("Health/interval_adjust/task").is_dir());
    let names: Vec<_> = snapshot(&b)["config"]["slugs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["slug"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, ["healthcheck_feature", "interval_adjust"]);
    drop((a, b));
    std::fs::remove_dir_all(root).unwrap();
}
