use agentconductor::coordinator::{ReviewProposal, Revision};
use serde_json::json;

fn proposal(add_paths: Vec<String>, add_dependencies: Vec<String>) -> ReviewProposal {
    serde_json::from_value(json!({
        "verdict":"revise",
        "summary":"The task needs a bounded contract change.",
        "evidence":["agent:auth"],
        "revisions":[{"task":"auth","feedback":"Keep the repair narrow.","add_paths":add_paths,"add_dependencies":add_dependencies}]
    })).unwrap()
}

fn validate(proposal: ReviewProposal) -> Result<(), String> {
    proposal.validate(&["auth".into()], &["agent:auth".into()], false)
}

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

#[test]
fn coordinator_amendments_are_limited_to_eight_paths_and_dependencies() {
    let paths = (0..9).map(|n| format!("src/module{n}.rs")).collect();
    assert!(validate(proposal(paths, vec![])).is_err());
    let dependencies = (0..9).map(|n| format!("task-{n}")).collect();
    assert!(validate(proposal(vec![], dependencies)).is_err());
}

#[test]
fn coordinator_amendment_entries_are_short_and_nul_free() {
    assert!(validate(proposal(vec!["x".repeat(1025)], vec![])).is_err());
    assert!(validate(proposal(vec!["src/a\0.rs".into()], vec![])).is_err());
    assert!(validate(proposal(vec![], vec!["d".repeat(1025)])).is_err());
}
