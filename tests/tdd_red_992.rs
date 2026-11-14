//! #992: expected Red is verified evidence, not a failed Goal or a successful worker crash.
use agentconductor::{phases, plan::Task, slug_progress::progress, workflow};
use std::fs;
#[path = "support/workflow.rs"]
mod support;

fn red(task: Task) -> Task {
    let mut value = serde_json::to_value(task).unwrap();
    value["test_expect_failure"] = true.into();
    serde_json::from_value(value).expect("a task supports an explicit Red expectation")
}

#[tokio::test]
async fn red_phase_can_be_approved_and_merged_before_the_same_check_turns_green() {
    for mode in ["shared", "worktree"] {
        let (root, mut base) = support::setup(mode);
        let root = root.canonicalize().unwrap();
        for task in &mut base.tasks {
            task.id = task.id.replace(".txt", "");
        }
        let slug = root.join("slug");
        let specs = [
            phases::PhaseSpec {
                id: "red".into(),
                name: "Failing check".into(),
                tasks: vec![red(base.tasks[0].clone())],
            },
            phases::PhaseSpec {
                id: "green".into(),
                name: "Implement missing files".into(),
                tasks: base.tasks[1..].to_vec(),
            },
        ];
        phases::write(&slug, "# Analysis\nRed then Green", &specs).unwrap();
        let dir = root.join("red-run");
        workflow::link_phase_at_launch(&dir, &slug, "red").unwrap();
        let report = workflow::run(
            phases::run_plan(&slug, "red", &base).unwrap(),
            &root.join("repo"),
            &dir,
            20,
        )
        .await
        .unwrap();
        assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
        assert!(report.tests[0].exit_code.is_some_and(|c| c != 0));
        workflow::sync_phase(&dir).unwrap();
        assert_eq!(progress(&slug)["phases"][0]["stage"], "verification");
        {
            let room = agentconductor::ui::Room::open(
                &dir,
                std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
            )
            .unwrap();
            assert_eq!(
                room.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["report"]["tests"][0]["passed"],
                true
            );
            assert_eq!(
                room.dispatch(br#"{"kind":"evidence"}"#).unwrap()["test_output"][0]["passed"],
                true
            );
        }
        workflow::approve(&dir, &report.candidate.unwrap().commit).unwrap();
        workflow::merge(&dir).unwrap();
        assert_eq!(progress(&slug)["current_phase"], "green");
        let context = agentconductor::phase_replan::context(&slug, "green").unwrap();
        assert_eq!(context["completed_phases"][0]["tests"][0]["passed"], true);
        let green = phases::run_plan(&slug, "green", &base).unwrap();
        assert_eq!(green.tasks[0].test_command, specs[0].tasks[0].test_command);
        let dir = root.join("green-run");
        workflow::link_phase_at_launch(&dir, &slug, "green").unwrap();
        let report = workflow::run(green, &root.join("repo"), &dir, 20)
            .await
            .unwrap();
        assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
        assert!(report.tests.iter().all(|t| t.exit_code == Some(0)));
        workflow::approve(&dir, &report.candidate.unwrap().commit).unwrap();
        workflow::merge(&dir).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn unexpected_red_pass_blocks_approval_and_worker_failure_is_not_red() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks[0] = red(plan.tasks[0].clone());
    let dir = root.join("unexpected-pass");
    let report = workflow::run(plan.clone(), &root.join("repo"), &dir, 20)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    assert!(report.error.unwrap().contains("expected failure"));
    assert!(workflow::approve(&dir, &report.candidate.unwrap().commit).is_err());
    fs::remove_dir_all(root).unwrap();

    let (root, mut plan) = support::setup("shared");
    plan.tasks = vec![red(plan.tasks[0].clone())];
    plan.agents[0].args = vec!["fail".into()];
    let report = workflow::run(plan, &root.join("repo"), &root.join("worker-failure"), 20)
        .await
        .unwrap();
    assert_eq!(report.state, "failed");
    assert_eq!(report.tasks[0].status, "failed");
    assert!(report.tests.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn expectation_is_strict_and_old_task_serialization_is_unchanged() {
    let value = serde_json::json!({"id":"t","agent":"mock","prompt":"p","paths":["one.txt"],"test_command":["node","--test"],"depends_on":[],"model":null});
    let mut task: Task = serde_json::from_value(value.clone()).unwrap();
    assert!(!task.test_expect_failure);
    assert_eq!(serde_json::to_value(&task).unwrap(), value);
    for expected in [false, true] {
        task.test_expect_failure = expected;
        for (exit, timed_out, signal, passed) in [
            (Some(0), false, None, !expected),
            (Some(1), false, None, expected),
            (Some(7), false, None, expected),
            (None, false, None, false),
            (Some(-1), false, None, false),
            (Some(1), true, None, false),
            (Some(0), true, None, false),
            (Some(1), false, Some(15), false),
        ] {
            assert_eq!(task.test_passed(exit, timed_out, signal), passed);
        }
    }
    for phased in [false, true] {
        let schema = agentconductor::planner::schema::for_plan(&["mock".into()], &[], phased);
        let tasks = if phased {
            &schema["properties"]["phases"]["items"]["properties"]["tasks"]
        } else {
            &schema["properties"]["tasks"]
        };
        assert_eq!(
            tasks["items"]["properties"]["test_expect_failure"]["type"],
            "boolean"
        );
        assert!(
            tasks["items"]["required"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f == "test_expect_failure")
        );
    }
}

#[tokio::test]
async fn a_missing_verification_executable_is_not_an_expected_test_failure() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks = vec![red(plan.tasks[0].clone())];
    plan.tasks[0].test_command = vec![root.join("absent-runner").to_string_lossy().into_owned()];
    let error = workflow::run(plan, &root.join("repo"), &root.join("missing-runner"), 20)
        .await
        .err()
        .expect("missing runners must be rejected, not accepted as Red");
    assert!(error.contains("test_unrunnable"));
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn coordinator_accepts_verified_red_without_approving_merge() {
    let (root, mut plan) = support::setup("shared");
    plan.tasks[0] = red(plan.tasks[0].clone());
    plan.tasks[0].test_command = vec![
        support::fixture().to_string_lossy().into_owned(),
        "fail".into(),
    ];
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec!["review-accept".into()];
    plan.agents.push(reviewer);
    let dir = root.join("review-red");
    let report = workflow::run(plan, &root.join("repo"), &dir, 20)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    let proposal = workflow::review(&dir, "coordinator", true).await.unwrap();
    assert_eq!(
        proposal.verdict,
        agentconductor::coordinator::Verdict::Accept
    );
    assert_eq!(workflow::load(&dir).unwrap().state, "waiting_approval");
    assert!(
        workflow::merge(&dir).is_err(),
        "a coordinator never grants human merge approval"
    );
    fs::remove_dir_all(root).unwrap();
}
