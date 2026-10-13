#![cfg(unix)]
//! R-56 / DD-62 / QAS-47: analysis first, sequential phases, task files move with the run.
use agentconductor::{phases, slug_progress::progress};
#[path = "support/workflow.rs"]
mod support;

fn phased(plan: &agentconductor::plan::Plan) -> Vec<phases::PhaseSpec> {
    let t = |id: &str, path: &str| {
        let mut task = plan.tasks[0].clone();
        task.id = id.into();
        task.paths = vec![path.into()];
        task
    };
    vec![
        phases::PhaseSpec {
            id: "01-core".into(),
            name: "core".into(),
            tasks: vec![t("a", "a.txt"), t("b", "b.txt")],
        },
        phases::PhaseSpec {
            id: "02-docs".into(),
            name: "docs".into(),
            tasks: vec![t("c", "c.txt")],
        },
    ]
}

#[test]
fn phased_plan_is_written_and_executed_in_order() {
    let (root, plan) = support::setup("shared");
    let slug = root.join("slug");
    agentconductor::slugs::create_tree(&slug).unwrap();
    phases::write(&slug, "# Analysis\nCurrent state…\n", &phased(&plan)).unwrap();
    assert!(slug.join("plan/analysis.md").exists() && slug.join("phase/01-core.md").exists());
    assert!(slug.join("task/01-core/a.md").exists() && slug.join("task/02-docs/c.md").exists());
    assert_eq!(progress(&slug)["current_phase"], "01-core");
    assert!(progress(&slug)["mismatches"].as_array().unwrap().is_empty());

    // Phase 2 cannot start before phase 1 is completed; its run plan holds only its tasks.
    assert!(phases::run_plan(&slug, "02-docs", &plan).is_err());
    let run = phases::run_plan(&slug, "01-core", &plan).unwrap();
    assert_eq!(
        run.tasks.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
        ["a", "b"]
    );
    run.validate().unwrap();

    // Stages advance one step at a time, never backwards or skipping.
    assert!(
        phases::advance(&slug, "01-core", "a", "completed").is_err(),
        "no skipping"
    );
    phases::advance_phase(&slug, "01-core", "verification").unwrap();
    assert_eq!(progress(&slug)["phases"][0]["stage"], "verification");
    assert!(
        phases::advance(&slug, "01-core", "a", "task").is_err(),
        "no going back"
    );
    phases::advance_phase(&slug, "01-core", "completed").unwrap();
    assert_eq!(progress(&slug)["current_phase"], "02-docs");
    phases::run_plan(&slug, "02-docs", &plan).unwrap();
    assert!(progress(&slug)["mismatches"].as_array().unwrap().is_empty());
    assert!(
        phases::write(&slug, "again", &phased(&plan)).is_err(),
        "an existing plan is not overwritten"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn phases_are_bounded_and_ids_are_safe() {
    let (root, plan) = support::setup("shared");
    let slug = root.join("slug");
    agentconductor::slugs::create_tree(&slug).unwrap();
    let mut bad = phased(&plan);
    bad[0].id = "../escape".into();
    assert!(phases::write(&slug, "x", &bad).is_err());
    assert!(
        phases::write(&slug, "x", &[]).is_err(),
        "at least one phase"
    );
    let many: Vec<_> = (0..7)
        .map(|i| {
            let mut p = phased(&plan)[1].clone();
            p.id = format!("0{i}-p");
            p.tasks[0].id = format!("t{i}");
            p.tasks[0].paths = vec![format!("f{i}")];
            p
        })
        .collect();
    assert!(
        phases::write(&slug, "x", &many).is_err(),
        "at most six phases"
    );
    std::fs::remove_dir_all(root).unwrap();
}

/// A run whose process died before `link_phase` still moves its phase: the desktop
/// records the link beside the run when it launches it.
#[tokio::test]
async fn a_run_linked_only_at_launch_moves_its_phase() {
    let (root, plan) = support::setup("shared");
    let slug = root.join("slug");
    agentconductor::slugs::create_tree(&slug).unwrap();
    phases::write(&slug, "# Analysis\n", &phased(&plan)).unwrap();
    let dir = root.join("run");
    agentconductor::workflow::link_phase_at_launch(&dir, &slug, "01-core").unwrap();
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(!dir.join("slug-phase.json").exists());
    agentconductor::workflow::sync_phase(&dir).unwrap();
    assert_eq!(progress(&slug)["phases"][0]["stage"], "verification");
    std::fs::remove_dir_all(root).unwrap();
}
