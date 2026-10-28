use agentconductor::plan::{Task, TaskType};
use serde_json::json;

fn task(extra: serde_json::Value) -> Task {
    let mut value = json!({
        "id": "t1",
        "agent": "worker",
        "prompt": "implement feature",
        "paths": ["src/lib.rs"],
        "test_command": ["cargo", "test"]
    });
    value
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    serde_json::from_value(value).unwrap()
}

#[test]
fn task_type_is_optional_and_old_task_documents_round_trip() {
    let task = task(json!({}));
    assert_eq!(task.task_type, None);
    assert!(
        serde_json::to_value(task)
            .unwrap()
            .get("task_type")
            .is_none()
    );
}

#[test]
fn supported_task_types_round_trip_as_snake_case() {
    for (name, expected) in [
        ("extract", TaskType::Extract),
        ("summarize", TaskType::Summarize),
        ("analyze", TaskType::Analyze),
        ("code", TaskType::Code),
        ("agentic", TaskType::Agentic),
        ("creative", TaskType::Creative),
    ] {
        let task = task(json!({"task_type": name}));
        assert_eq!(task.task_type, Some(expected));
        assert_eq!(serde_json::to_value(task).unwrap()["task_type"], name);
    }
}

#[test]
fn unknown_task_types_are_rejected() {
    let invalid = json!({
        "id": "t1",
        "agent": "worker",
        "prompt": "implement feature",
        "paths": [],
        "test_command": [],
        "task_type": "poetry"
    });
    assert!(serde_json::from_value::<Task>(invalid).is_err());
}

#[test]
fn planning_schema_accepts_supported_task_types_without_requiring_them() {
    let schema = agentconductor::planner::schema::for_plan(&["worker".into()], &[], false);
    let task = &schema["properties"]["tasks"]["items"];
    assert_eq!(
        task["properties"]["task_type"]["enum"],
        json!([
            "extract",
            "summarize",
            "analyze",
            "code",
            "agentic",
            "creative",
            null
        ])
    );
    assert!(
        !task["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|key| key == "task_type")
    );
}
