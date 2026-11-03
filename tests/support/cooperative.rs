use crate::support;
use agentconductor::{
    cooperative::{Action, Issue},
    state::Store,
    workflow,
};

pub fn setup() -> (std::path::PathBuf, Store, workflow::Report) {
    let (root, plan) = support::setup("shared");
    let mut report: workflow::Report = serde_json::from_value(serde_json::json!({
        "schema_version":1,"state":"running","repo":root.join("repo"),"base":"base","branch":"main", "plan":plan,
        "tasks":plan.tasks.iter().map(|t|serde_json::json!({"id":t.id,"status":"running","attempt":1,"cwd":root.join("repo"),"started_ms":100,"finished_ms":null,"session_id":format!("session-{}",t.id),"error":null,"base":"base","snapshot":null,"artifacts":root.join(&t.id)})).collect::<Vec<_>>(),
        "candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")
    })).unwrap();
    report.tasks[0].session_established = true;
    let mut store = Store::open(root.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Fixture workflow",
        )
        .unwrap();
    (root, store, report)
}
pub fn issue(report: &workflow::Report) -> Issue {
    Issue {
        revision: workflow::revision(report).unwrap(),
        task: report.tasks[0].id.clone(),
        action: Action::Pause,
        instruction: "Pause at the next safe checkpoint".into(),
        issued_at_ms: 1000,
        expires_at_ms: 3000,
    }
}
