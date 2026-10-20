//! GitHub #100 (DD-169): the task-activity record carries the current tool.
use agentconductor::protocol::{Event, EventKind};
use agentconductor::state::Store;
use serde_json::json;

fn event(seq: u64, payload: serde_json::Value) -> Event {
    Event {
        seq,
        raw_offset: 0,
        session_id: None,
        kind: EventKind::Progress,
        payload,
    }
}

#[test]
fn a_tool_start_is_recorded_and_its_result_clears_it() {
    let dir = std::env::temp_dir().join(format!("ac-activity-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut store = Store::open(dir.join("state.sqlite")).unwrap();
    let start = event(
        1,
        json!({"type":"assistant","message":{"content":[{"type":"tool_use","id":"a","name":"Bash"}]}}),
    );
    store.append_task_events("t", 1, &[start]).unwrap();
    let doc = store
        .optional_document("task_activity/t/1")
        .unwrap()
        .unwrap();
    assert_eq!(doc["tool"]["name"], "Bash");
    assert!(doc["tool"]["since_ms"].as_u64().unwrap() > 0);
    assert_eq!(doc["seq"], 1);

    let text = event(
        2,
        json!({"type":"assistant","message":{"content":[{"type":"text","text":"still"}]}}),
    );
    store.append_task_events("t", 1, &[text]).unwrap();
    let doc = store
        .optional_document("task_activity/t/1")
        .unwrap()
        .unwrap();
    assert_eq!(
        doc["tool"]["name"], "Bash",
        "other output keeps the running tool"
    );

    let end = event(
        3,
        json!({"type":"user","message":{"content":[{"type":"tool_result","tool_use_id":"a"}]}}),
    );
    store.append_task_events("t", 1, &[end]).unwrap();
    let doc = store
        .optional_document("task_activity/t/1")
        .unwrap()
        .unwrap();
    assert!(doc["tool"].is_null());
    assert_eq!(doc["seq"], 3);
    drop(store);
    std::fs::remove_dir_all(dir).unwrap();
}

