use agentconductor::plan::Plan;
use serde_json::json;

#[path = "support/workflow.rs"]
mod support;

fn with_fallback(mode: &str) -> (std::path::PathBuf, Plan) {
    let (root, plan) = support::setup(mode);
    let mut value = serde_json::to_value(plan).unwrap();
    value["agents"][0]["args"] = json!(["quota-long"]);
    value["agents"][0]["quota_group"] = json!("paid-account");
    value["agents"][0]["fallback"] = json!({"agent":"company"});
    value["agents"].as_array_mut().unwrap().push(json!({"id":"company","executable":support::fixture(),"args":["task"],"parser":"custom","permission_mode":"external_sandbox","quota_group":"company-account"}));
    (root, serde_json::from_value(value).unwrap())
}

#[tokio::test]
async fn restricted_profile_transfers_to_company_cli_and_retains_account_evidence() {
    let (root, plan) = with_fallback("shared");
    let run = root.join("run");
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(report.plan.tasks.iter().all(|task| task.agent == "company"));
    assert!(report.tasks.iter().all(|task| task.status == "completed"));
    let value = serde_json::to_value(&report).unwrap();
    assert!(value["account_restrictions"]["paid-account"].is_object());
    assert_eq!(report.tests.len(), 3);
    assert!(agentconductor::workflow::merge(&run).is_err());
    for i in 0..3 {
        let task = &report.tasks[i];
        assert!(
            run.join(format!("handoff-{i}-{}.md", task.attempt - 1))
                .is_file()
        );
        assert!(task.artifacts.join("process/stdout.jsonl").is_file());
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn fallback_rejects_unknown_models_and_same_account_substitutes() {
    let (root, plan) = with_fallback("shared");
    let value = serde_json::to_value(&plan).unwrap();
    for change in [
        json!({"agent":"missing"}),
        json!({"agent":"mock"}),
        json!({"agent":"company","model":"unobserved"}),
    ] {
        let mut changed = value.clone();
        changed["agents"][0]["fallback"] = change;
        assert!(
            serde_json::from_value::<Plan>(changed)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    // Agents may name each other as fallback.
    let mut changed = value.clone();
    changed["agents"][1]["fallback"] = json!({"agent":"mock"});
    serde_json::from_value::<Plan>(changed)
        .unwrap()
        .validate()
        .unwrap();
    let mut changed = value;
    changed["agents"][1]["quota_group"] = json!("paid-account");
    assert!(
        serde_json::from_value::<Plan>(changed)
            .unwrap()
            .validate()
            .is_err()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn no_policy_waits_and_ordinary_failure_does_not_transfer() {
    for mode in ["quota", "task"] {
        let (root, mut plan) = with_fallback("shared");
        if mode == "quota" {
            plan.agents[0].fallback = None;
        } else {
            plan.agents[0].args = vec!["task".into()];
            plan.tasks[0].prompt = "fail".into();
        }
        let report = agentconductor::workflow::run(plan, &root.join("repo"), &root.join("run"), 5)
            .await
            .unwrap();
        assert_eq!(
            report.state,
            if mode == "quota" {
                "waiting_availability"
            } else {
                "failed"
            }
        );
        assert!(report.plan.tasks.iter().all(|task| task.agent == "mock"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn reachable_fallback_permissions_are_checked_before_execution() {
    let (root, mut plan) = with_fallback("shared");
    plan.agents[1].permission_mode = "unverified".into();
    let run = root.join("run");
    assert!(
        agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
            .await
            .err()
            .unwrap()
            .contains("verified permission")
    );
    assert!(!run.exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn fallback_uses_optional_worktrees_and_retains_partial_files() {
    let (root, mut plan) = with_fallback("worktree");
    plan.agents[0].args = vec!["quota-partial-long".into()];
    plan.agents[1].args = vec!["task-preserve".into()];
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &root.join("run"), 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval");
    for task in &report.tasks {
        assert_eq!(
            std::fs::read_to_string(task.cwd.join(&task.id)).unwrap(),
            "partial\nimplemented\n"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn renaming_a_profile_does_not_clear_the_same_account_restriction() {
    let (root, mut plan) = with_fallback("shared");
    plan.agents[0].args = vec!["quota".into()];
    plan.agents[0].fallback = None;
    plan.agents[1].quota_group = Some("paid-account".into());
    let run = root.join("run");
    agentconductor::workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    for id in ["one.txt", "two.txt", "three.txt"] {
        agentconductor::workflow::reassign(&run, id, "company", None).unwrap();
    }
    let before = agentconductor::workflow::load(&run).unwrap();
    let report = agentconductor::workflow::resume_due(&run, i64::MAX, 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    for (old, current) in before.tasks.iter().zip(&report.tasks) {
        assert_eq!(old.attempt, current.attempt);
        assert_eq!(current.status, "waiting_availability");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn restricted_fallback_target_waits_without_unbounded_retries() {
    let (root, mut plan) = with_fallback("worktree");
    plan.agents[1].args = vec!["quota".into()];
    let report = agentconductor::workflow::run(plan, &root.join("repo"), &root.join("run"), 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    assert!(report.account_restrictions.contains_key("paid-account"));
    assert!(report.account_restrictions.contains_key("company-account"));
    assert!(report.tasks.iter().all(|task| task.attempt == 2));
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn human_stop_prevents_policy_transfer() {
    let (root, mut plan) = with_fallback("shared");
    plan.agents[0].args = vec!["quota-slow".into()];
    let run = root.join("run");
    let worker_run = run.clone();
    let repo = root.join("repo");
    let handle =
        tokio::spawn(
            async move { agentconductor::workflow::run(plan, &repo, &worker_run, 10).await },
        );
    // Bound preparation separately from the ten-second agent execution timeout.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if run.join("task-0/process/owner.json").is_file() {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "worker did not start"
        );
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    agentconductor::workflow::stop(&run, "Stop before quota fallback").unwrap();
    let report = handle.await.unwrap().unwrap();
    assert_eq!(report.state, "stopped");
    assert!(report.plan.tasks.iter().all(|task| task.agent == "mock"));
    assert!(!run.join("handoff-0-1.md").exists());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn mutual_fallback_with_both_accounts_restricted_waits_for_availability() {
    let (root, mut plan) = with_fallback("shared");
    let mut company = plan.agents[0].clone();
    company.id = "company".into();
    company.quota_group = Some("company-account".into());
    company.fallback = Some(serde_json::from_value(json!({"agent":"mock"})).unwrap());
    plan.agents[1] = company;
    let report = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        agentconductor::workflow::run(plan, &root.join("repo"), &root.join("run"), 5),
    )
    .await
    .expect("fallback chain must not loop")
    .unwrap();
    assert_eq!(report.state, "waiting_availability");
    let value = serde_json::to_value(&report).unwrap();
    assert!(value["account_restrictions"]["paid-account"].is_object());
    assert!(value["account_restrictions"]["company-account"].is_object());
    std::fs::remove_dir_all(root).unwrap();
}
