use agentconductor::coordinator::Revision;

#[test]
fn coordinator_revision_can_request_typed_path_and_dependency_amendments() {
    let revision: Result<Revision, _> = serde_json::from_value(serde_json::json!({
        "task": "auth",
        "feedback": "Add the rotation table before implementing refresh.",
        "add_paths": ["src/auth/rotation.rs"],
        "add_dependencies": ["schema"]
    }));

    let revision = revision.expect("typed amendments are part of the review contract");
    assert_eq!(revision.add_paths, ["src/auth/rotation.rs"]);
    assert_eq!(revision.add_dependencies, ["schema"]);
}

#[test]
fn old_review_records_without_amendment_fields_remain_readable() {
    let revision: Revision = serde_json::from_value(serde_json::json!({
        "task": "auth",
        "feedback": "Keep the change within the original scope."
    }))
    .unwrap();

    assert!(revision.add_paths.is_empty());
    assert!(revision.add_dependencies.is_empty());
}
