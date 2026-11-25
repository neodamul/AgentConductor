use agentconductor::{plan::Plan, workflow};
use std::{fs, path::Path, process::Command};
#[path = "support/workflow.rs"]
mod support;

fn setup(mode: &str, review: &str) -> (std::path::PathBuf, Plan) {
    let (root, mut plan) = support::setup(mode);
    let fixture = root.join(if cfg!(windows) { "red.exe" } else { "red" });
    assert!(
        Command::new("rustc")
            .arg("tests/support/red_wrapper.rs")
            .arg("-o")
            .arg(&fixture)
            .status()
            .unwrap()
            .success()
    );
    plan.tasks[0].test_expect_failure = true;
    plan.tasks[0].test_command = vec![fixture.to_string_lossy().into_owned(), "test".into()];
    let mut reviewer = plan.agents[0].clone();
    reviewer.id = "coordinator".into();
    reviewer.executable = fixture.to_string_lossy().into_owned();
    reviewer.permission_mode = "read_only".into();
    reviewer.args = vec![review.into()];
    plan.agents.push(reviewer);
    (root, plan)
}

async fn run(root: &Path, plan: Plan) -> workflow::Report {
    workflow::run(plan, &root.join("repo"), &root.join("run"), 20)
        .await
        .unwrap()
}

#[tokio::test]
async fn coordinator_can_accept_zero_exit_red_after_other_tests_pass() {
    let (root, plan) = setup("worktree", "accept");
    let dir = root.join("run");
    fs::write(
        agentconductor::native_guard::orchestrator_path(&dir),
        serde_json::to_vec(plan.agents.last().unwrap()).unwrap(),
    )
    .unwrap();
    workflow::set_auto_merge(&dir, true, &plan).unwrap();
    let report = run(&root, plan).await;
    assert_eq!(report.state, "failed");
    assert!(!report.test_passed(&report.tests[0]));
    let accepted = workflow::repair(&dir, 20, report).await.unwrap();
    assert_eq!(
        accepted.state, "waiting_approval",
        "{:?}",
        accepted.review_error
    );
    assert_eq!(accepted.tests.len(), 3);
    assert_eq!(accepted.tests[0].exit_code, Some(0));
    assert!(accepted.tests.iter().all(|t| accepted.test_passed(t)));
    {
        let room =
            agentconductor::ui::Room::open(&dir, Path::new(env!("CARGO_BIN_EXE_agentconductor")))
                .unwrap();
        let snapshot = room.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        assert_eq!(snapshot["report"]["tests"][0]["passed"], true);
        assert_eq!(snapshot["report"]["tests"][0]["exit_code"], 0);
    }
    assert!(!workflow::automatic_merge(&dir).unwrap());
    assert!(workflow::merge(&dir).is_err());
    let sha = &accepted.candidate.as_ref().unwrap().commit;
    let log = &accepted.tests[0].stdout;
    let captured = fs::read(log).unwrap();
    fs::write(log, "changed after review").unwrap();
    assert!(workflow::approve(&dir, sha).is_err());
    fs::write(log, captured).unwrap();
    workflow::approve(&dir, sha).unwrap();
    workflow::merge(&dir).unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn review_cannot_waive_ordinary_failures_or_omit_red_test_evidence() {
    for (review, ordinary_failure) in [("accept", true), ("omit", false), ("block", false)] {
        let (root, mut plan) = setup("shared", review);
        if ordinary_failure {
            plan.tasks[1].test_command[1] = "fail".into();
        }
        let dir = root.join("run");
        let report = run(&root, plan).await;
        let result = workflow::review(&dir, "coordinator", true).await;
        if review == "block" {
            assert_eq!(
                result.unwrap().verdict,
                agentconductor::coordinator::Verdict::Blocked
            );
        } else {
            assert!(result.is_err());
        }
        assert!(workflow::approve(&dir, &report.candidate.unwrap().commit).is_err());
        assert!(!workflow::load(&dir).unwrap().test_passed(&report.tests[0]));
        fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn saved_red_acceptance_is_bound_to_unchanged_test_evidence() {
    let (root, plan) = setup("shared", "accept");
    let dir = root.join("run");
    let report = run(&root, plan).await;
    let proposal = workflow::review(&dir, "coordinator", false).await.unwrap();
    let hash = workflow::digest(&serde_json::to_vec(&proposal).unwrap());
    assert_eq!(workflow::load(&dir).unwrap().state, "failed");
    let original = fs::read(&report.tests[0].stdout).unwrap();
    fs::write(&report.tests[0].stdout, "red ok but changed").unwrap();
    assert!(workflow::apply_review(&dir, &hash).is_err());
    assert_eq!(workflow::load(&dir).unwrap().state, "failed");
    fs::write(&report.tests[0].stdout, original).unwrap();
    workflow::apply_review(&dir, &hash).unwrap();
    let accepted = workflow::load(&dir).unwrap();
    assert!(accepted.test_passed(&accepted.tests[0]));
    assert_eq!(accepted.state, "waiting_approval");
    assert!(workflow::apply_review(&dir, &hash).is_err());
    fs::remove_dir_all(root).unwrap();
}
