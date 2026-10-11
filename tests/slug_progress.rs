//! R-57 / DD-63 / QAS-48: progress is derived from the slug tree; mismatches are reported.
use agentconductor::slug_progress::progress;
use std::path::Path;

fn tree(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("ac-slug-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    agentconductor::slugs::create_tree(&root).unwrap();
    root
}
fn put(root: &Path, stage: &str, phase: &str, task: &str) {
    let dir = root.join(stage).join(phase);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{task}.md")), "task").unwrap();
}
fn plan(root: &Path) {
    std::fs::write(root.join("plan/analysis.md"), "# Analysis\n").unwrap();
    std::fs::write(
        root.join("plan/plan.json"),
        r#"{"phases":[{"id":"01-core","name":"core","tasks":["a","b"]},{"id":"02-docs","name":"docs","tasks":["c"]}]}"#,
    )
    .unwrap();
}

#[test]
fn stages_follow_the_tree() {
    let root = tree("stages");
    assert_eq!(progress(&root)["stage"], "no_plan");
    std::fs::write(root.join("plan/analysis.md"), "# Analysis\n").unwrap();
    assert_eq!(progress(&root)["stage"], "analysis");
    plan(&root);
    put(&root, "task", "01-core", "a");
    put(&root, "task", "01-core", "b");
    put(&root, "task", "02-docs", "c");
    let p = progress(&root);
    assert_eq!(p["stage"], "phase");
    assert_eq!(p["current_phase"], "01-core");
    assert_eq!(p["phases"][0]["stage"], "task");
    assert_eq!(p["phases"][1]["stage"], "pending");
    assert!(p["mismatches"].as_array().unwrap().is_empty(), "{p}");

    std::fs::rename(root.join("task/01-core/a.md"), {
        std::fs::create_dir_all(root.join("verification/01-core")).unwrap();
        root.join("verification/01-core/a.md")
    })
    .unwrap();
    std::fs::rename(
        root.join("task/01-core/b.md"),
        root.join("verification/01-core/b.md"),
    )
    .unwrap();
    assert_eq!(progress(&root)["phases"][0]["stage"], "verification");
    std::fs::create_dir_all(root.join("completed/01-core")).unwrap();
    std::fs::rename(
        root.join("verification/01-core/a.md"),
        root.join("completed/01-core/a.md"),
    )
    .unwrap();
    std::fs::rename(
        root.join("verification/01-core/b.md"),
        root.join("completed/01-core/b.md"),
    )
    .unwrap();
    let p = progress(&root);
    assert_eq!(p["phases"][0]["stage"], "completed");
    assert_eq!(p["current_phase"], "02-docs");
    std::fs::create_dir_all(root.join("completed/02-docs")).unwrap();
    std::fs::rename(
        root.join("task/02-docs/c.md"),
        root.join("completed/02-docs/c.md"),
    )
    .unwrap();
    assert_eq!(progress(&root)["stage"], "completed");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn mismatches_are_reported() {
    let root = tree("mismatch");
    plan(&root);
    put(&root, "task", "01-core", "a");
    put(&root, "completed", "01-core", "a"); // in two stages
    put(&root, "task", "01-core", "stray"); // undeclared
    put(&root, "completed", "02-docs", "c"); // later phase before earlier completed
    // "b" is declared but missing everywhere
    let text = progress(&root)["mismatches"].to_string();
    for needle in ["a", "stray", "b", "02-docs"] {
        assert!(text.contains(needle), "missing {needle}: {text}");
    }
    assert!(
        text.contains("more than one stage")
            && text.contains("not declared")
            && text.contains("missing")
            && text.contains("before")
    );
    std::fs::remove_dir_all(root).unwrap();
}
