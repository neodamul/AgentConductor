//! #409: reviewed task overrides are honored by the real scheduler with deterministic CLIs.
use agentconductor::{
    plan::{Fallback, FallbackRule},
    workflow,
};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn one_role_override_does_not_reassign_other_roles_or_mutate_profile_defaults() {
    let (root, mut plan) = support::setup("shared");
    let mut inherited = plan.agents[0].clone();
    inherited.id = "inherited".into();
    inherited.quota_group = Some("default-account".into());
    let mut specific = inherited.clone();
    specific.id = "specific".into();
    specific.quota_group = Some("specific-account".into());
    plan.agents[0].args = vec!["quota-long".into()];
    plan.agents[0].quota_group = Some("paid-account".into());
    plan.agents[0].fallback = Some(Fallback {
        agent: "inherited".into(),
        model: None,
    });
    plan.agents.extend([inherited, specific]);
    plan.tasks[0].fallback = Some(FallbackRule::Agent(Fallback {
        agent: "specific".into(),
        model: None,
    }));
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert_eq!(report.plan.tasks[0].agent, "specific");
    assert!(
        report.plan.tasks[1..]
            .iter()
            .all(|t| t.agent == "inherited")
    );
    assert_eq!(
        report.plan.agents[0].fallback.as_ref().unwrap().agent,
        "inherited"
    );
    assert_eq!(report.tests.len(), 3);
    assert!(workflow::merge(&run).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn explicit_none_waits_instead_of_using_the_profile_policy() {
    let (root, mut plan) = support::setup("shared");
    let mut target = plan.agents[0].clone();
    target.id = "company".into();
    target.quota_group = Some("company".into());
    plan.agents[0].args = vec!["quota-long".into()];
    plan.agents[0].quota_group = Some("paid".into());
    plan.agents[0].fallback = Some(Fallback {
        agent: "company".into(),
        model: None,
    });
    plan.agents.push(target);
    plan.tasks.truncate(1);
    plan.tasks[0].fallback = Some(FallbackRule::Disabled(false));
    let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    assert_eq!(report.plan.tasks[0].agent, "mock");
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn an_exhausted_mutual_chain_stops_after_the_reviewed_first_hop() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-long".into()];
    plan.agents[0].quota_group = Some("paid".into());
    let mut target = plan.agents[0].clone();
    target.id = "company".into();
    target.quota_group = Some("company".into());
    target.fallback = Some(Fallback {
        agent: "mock".into(),
        model: None,
    });
    plan.agents.push(target);
    plan.tasks.truncate(1);
    plan.tasks[0].fallback = Some(FallbackRule::Agent(Fallback {
        agent: "company".into(),
        model: None,
    }));
    let report = workflow::run(plan, &root.join("repo"), &root.join("run"), 5)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    assert_eq!(report.plan.tasks[0].agent, "company");
    assert!(report.tasks[0].attempt <= 2);
    assert!(
        report.plan.tasks[0].fallback.is_none(),
        "first-hop override is consumed"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn queued_agent_change_drops_the_old_first_hop_without_editing_the_running_report() {
    let (root, mut plan) = support::setup("shared");
    let mut target = plan.agents[0].clone();
    target.id = "company".into();
    target.quota_group = Some("company".into());
    plan.agents[0].quota_group = Some("paid".into());
    plan.agents.push(target);
    plan.tasks[0].fallback = Some(FallbackRule::Agent(Fallback {
        agent: "company".into(),
        model: None,
    }));
    let run = root.join("run");
    std::fs::create_dir(&run).unwrap();
    let task = serde_json::json!({"id":plan.tasks[0].id,"status":"pending","cwd":root.join("repo"),"started_ms":null,"finished_ms":null,"session_id":null,"error":null,"base":"base","snapshot":null,"artifacts":run.join("task-0")});
    let report = serde_json::json!({"schema_version":1,"state":"running","repo":root.join("repo"),"base":"base","branch":"main","plan":plan,"tasks":[task],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")});
    let mut store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
    store.record("workflow", &report, "fixture").unwrap();
    workflow::reassign(&run, &plan.tasks[0].id, "company", None).unwrap();
    assert_eq!(
        store
            .document(&format!("pending_edit/{}", plan.tasks[0].id))
            .unwrap()["agent"],
        "company"
    );
    assert_eq!(
        store.document("workflow").unwrap()["plan"]["tasks"][0]["agent"],
        "mock"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_task_waiting_for_its_usage_limit_can_be_reassigned_during_a_run() {
    // #1054: the change used to be refused because the task was not "pending".
    let (root, mut plan) = support::setup("shared");
    let mut target = plan.agents[0].clone();
    target.id = "company".into();
    target.quota_group = Some("company".into());
    plan.agents.push(target);
    let run = root.join("run");
    std::fs::create_dir(&run).unwrap();
    let task = serde_json::json!({"id":plan.tasks[0].id,"status":"waiting_availability","cwd":root.join("repo"),"started_ms":null,"finished_ms":null,"session_id":null,"error":null,"base":"base","snapshot":null,"artifacts":run.join("task-0")});
    let report = serde_json::json!({"schema_version":1,"state":"running","repo":root.join("repo"),"base":"base","branch":"main","plan":plan,"tasks":[task],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")});
    let mut store = agentconductor::state::Store::open(run.join("state.sqlite")).unwrap();
    store.record("workflow", &report, "fixture").unwrap();
    workflow::reassign(&run, &plan.tasks[0].id, "company", None).unwrap();
    assert_eq!(
        store
            .document(&format!("pending_edit/{}", plan.tasks[0].id))
            .unwrap()["agent"],
        "company"
    );
    drop(store);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_due_fallback_moves_the_role_while_other_workers_still_run() {
    // #1059: the fallback used to wait until every other worker had finished.
    let (root, mut plan) = support::setup("shared");
    let mut slow = plan.agents[0].clone();
    slow.id = "slow".into();
    slow.args = vec!["task-slow".into()];
    slow.quota_group = Some("slow-account".into());
    let mut spare = plan.agents[0].clone();
    spare.id = "spare".into();
    spare.quota_group = Some("spare-account".into());
    plan.agents[0].args = vec!["quota-long".into()];
    plan.agents[0].quota_group = Some("paid-account".into());
    plan.agents[0].fallback = Some(Fallback {
        agent: "spare".into(),
        model: None,
    });
    plan.agents.extend([slow, spare]);
    for task in &mut plan.tasks[1..] {
        task.agent = "slow".into();
    }
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
    assert_eq!(report.plan.tasks[0].agent, "spare");
    let moved = report.tasks[0].started_ms.unwrap();
    let slow_done = report.tasks[1..]
        .iter()
        .map(|t| t.finished_ms.unwrap())
        .min()
        .unwrap();
    assert!(
        moved < slow_done,
        "the role moved at {moved}, after the other workers finished at {slow_done}"
    );
    std::fs::remove_dir_all(root).unwrap();
}
