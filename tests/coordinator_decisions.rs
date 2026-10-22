//! GitHub #149 / R-167 / DD-177: every coordinator judgment is listed, derived from the audit
//! records the coordinator already writes.
#[cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
#[cfg(unix)]
use agentconductor::workflow;
use agentconductor::{state::Store, workflow::decisions};
use serde_json::json;

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ac-decisions-{name}-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn each_kind_of_judgment_becomes_one_row_in_order() {
    let dir = scratch("kinds");
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    store.record("tool_orchestrator_decision", &json!({"task":"a.txt","tool":"Bash","input":"{}","allow":true,"reason":"routine build","decided_ms":5}), "tool").unwrap();
    store.record("tool_orchestrator_decision", &json!({"task":"a.txt","tool":"Bash","input":"{}","allow":false,"reason":"publishes","decided_ms":6}), "tool").unwrap();
    store.record("tool_orchestrator_decision", &json!({"task":"a.txt","tool":"Bash","input":"{}","allow":null,"error":"no answer","decided_ms":7}), "tool").unwrap();
    store.record("coordinator_review", &json!({"coordinator":"c","round":1,"proposal":{"verdict":"revise","summary":"Repair the failed task","evidence":["agent:a.txt"],"revisions":[{"task":"a.txt","feedback":"fix it"}]},"apply":true}), "review").unwrap();
    store
        .record(
            "test_command_revised",
            &json!({"task":"b.txt","from":["git","diff"],"to":["true"],"reason":"no git here"}),
            "test",
        )
        .unwrap();
    store
        .record(
            "task_timeout_resume",
            &json!({"task":"c.txt","attempt":2,"timeouts":1,"retry_at":1}),
            "timeout",
        )
        .unwrap();
    store
        .record(
            "auto_recovery",
            &json!({"status":"resume_pending","recovered":true,"attempts":1}),
            "recovery",
        )
        .unwrap();
    store
        .record(
            "handoff",
            &json!({"task":"c.txt","from":"claude","to":"codex"}),
            "handoff",
        )
        .unwrap();
    store
        .record(
            "step_lessons",
            &json!([{"task":"a.txt","cause":"exit 7","response":"fix it","at_ms":1}]),
            "lesson",
        )
        .unwrap();
    store
        .record("approval", &serde_json::Value::Null, "not a judgment")
        .unwrap();

    let rows = decisions::list(&dir).unwrap();
    let shape: Vec<(String, String, String)> = rows
        .iter()
        .map(|r| {
            (
                r["kind"].as_str().unwrap().into(),
                r["judgment"].as_str().unwrap().into(),
                r["task"].as_str().unwrap_or_default().into(),
            )
        })
        .collect();
    assert_eq!(
        shape,
        vec![
            ("tool".into(), "allow".into(), "a.txt".into()),
            ("tool".into(), "deny".into(), "a.txt".into()),
            ("tool".into(), "undecided".into(), "a.txt".into()),
            ("review".into(), "revise".into(), "a.txt".into()),
            ("test_command".into(), "replaced".into(), "b.txt".into()),
            ("timeout".into(), "resume".into(), "c.txt".into()),
            ("recovery".into(), "resume_pending".into(), "".into()),
            ("handoff".into(), "reassigned".into(), "c.txt".into()),
            ("lesson".into(), "recorded".into(), "a.txt".into()),
        ]
    );
    assert!(
        rows[0]["detail"]
            .as_str()
            .unwrap()
            .contains("routine build")
    );
    assert!(rows[4]["detail"].as_str().unwrap().contains("no git here"));
    assert!(rows[7]["detail"].as_str().unwrap().contains("codex"));
    assert_eq!(store.count_events(decisions::EVENTS).unwrap(), rows.len());
    // Windows cannot delete a folder whose database is still open.
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn a_review_that_blocks_or_accepts_is_listed_too() {
    let dir = scratch("verdicts");
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    for verdict in ["accept", "blocked"] {
        store.record("coordinator_review", &json!({"proposal":{"verdict":verdict,"summary":"s","evidence":["e"],"revisions":[]},"apply":false}), "review").unwrap();
    }
    let rows = decisions::list(&dir).unwrap();
    assert_eq!(rows[0]["judgment"], "accept");
    assert_eq!(rows[1]["judgment"], "blocked");
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn the_room_shows_every_judgment_in_the_run_and_the_task() {
    let js = include_str!("../src/ui/room.js");
    assert!(js.contains("function judgmentEntry"));
    assert!(js.contains("data.coordinator_decisions"));
    assert!(js.contains("data.decisions_all") || js.contains("data.judgments"));
    assert!(js.contains("decision_count"));
    for id in [
        "room.judgment_tool_allow",
        "room.judgment_review_revise",
        "room.judgment_test_command_replaced",
        "room.judgment_timeout_resume",
        "room.judgment_handoff_reassigned",
        "room.judgment_lesson_recorded",
        "room.decisions_count",
    ] {
        for catalog in [
            include_str!("../src/ui/messages/en.json"),
            include_str!("../src/ui/messages/ko.json"),
        ] {
            assert!(catalog.contains(&format!("\"{id}\"")), "{id}");
        }
    }
}

/// A real repair: the review, the lesson and nothing else of the run are listed, in the
/// snapshot count, the coordinator view and the task view.
#[cfg(unix)]
#[tokio::test]
async fn a_repaired_run_lists_its_judgments_everywhere() {
    let (root, mut plan) = support::setup("shared");
    // The room opens artifacts without following links, so /var -> /private/var must not appear.
    let root = root.canonicalize().unwrap();
    let dir = root.join("run");
    plan.tasks[0].prompt = "fail".into();
    let mut coordinator = plan.agents[0].clone();
    coordinator.id = "coordinator".into();
    coordinator.permission_mode = "read_only".into();
    coordinator.args = vec!["review".into()];
    std::fs::write(
        agentconductor::native_guard::orchestrator_path(&dir),
        serde_json::to_vec(&coordinator).unwrap(),
    )
    .unwrap();
    let failed = workflow::run(plan, &root.join("repo"), &dir, 10)
        .await
        .unwrap();
    workflow::repair(&dir, 10, failed).await.unwrap();

    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::open(&dir, exe).unwrap();
    let ask = |v: serde_json::Value| room.dispatch(v.to_string().as_bytes()).unwrap();
    let rows = decisions::list(&dir).unwrap();
    let kinds: Vec<&str> = rows.iter().map(|r| r["kind"].as_str().unwrap()).collect();
    assert!(
        kinds.contains(&"review") && kinds.contains(&"lesson"),
        "{kinds:?}"
    );
    assert_eq!(
        ask(json!({"kind":"snapshot"}))["decision_count"],
        rows.len()
    );
    assert_eq!(
        ask(json!({"kind":"coordinator"}))["judgments"]
            .as_array()
            .unwrap()
            .len(),
        rows.len()
    );
    let task = ask(json!({"kind":"task","index":0}));
    let own = task["coordinator_decisions"].as_array().unwrap();
    assert!(!own.is_empty() && own.iter().all(|d| d["task"] == "one.txt"));
    std::fs::remove_dir_all(root).unwrap();
}
