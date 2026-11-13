//! #943: comparison is complete, deterministic, and conservative about execution changes.
#[path = "support/workflow.rs"]
mod support;
use agentconductor::phase_replan;

#[test]
fn compare_marks_every_changed_added_removed_task_and_keeps_original() {
    let (root, original) = support::setup("shared");
    let mut next = original.clone();
    next.tasks[0].prompt.push_str(" changed by previous result");
    let removed = next.tasks.remove(1);
    let mut added = removed.clone();
    added.id = "new-doc".into();
    next.tasks.push(added);
    let comparison = phase_replan::compare(&original, &next).unwrap();
    assert_eq!(comparison["unchanged"], false);
    let changes = comparison["changes"].as_array().unwrap();
    assert!(changes.iter().any(|c| {
        c["kind"] == "changed"
            && c["fields"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f == "prompt")
    }));
    assert!(
        changes
            .iter()
            .any(|c| c["kind"] == "removed" && c["id"] == removed.id)
    );
    assert!(
        changes
            .iter()
            .any(|c| c["kind"] == "added" && c["id"] == "new-doc")
    );
    assert_eq!(
        phase_replan::compare(&original, &original).unwrap()["unchanged"],
        true
    );
    let mut changed_model = original.clone();
    changed_model.tasks[0].model = Some("other".into());
    assert_eq!(
        phase_replan::compare(&original, &changed_model).unwrap()["unchanged"],
        false
    );
    let mut invalid = original.clone();
    invalid.goal.push('x');
    assert!(phase_replan::compare(&original, &invalid).is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn adoption_changes_only_current_phase_and_rejects_unsafe_or_reused_ids() {
    let (root, mut base) = support::setup("shared");
    for task in &mut base.tasks {
        task.id = task.id.trim_end_matches(".txt").into();
    }
    let slug = root.join("slug");
    let mut later = base.tasks[0].clone();
    later.id = "later".into();
    agentconductor::phases::write(
        &slug,
        "analysis",
        &[
            agentconductor::phases::PhaseSpec {
                id: "first".into(),
                name: "First".into(),
                tasks: base.tasks.clone(),
            },
            agentconductor::phases::PhaseSpec {
                id: "later".into(),
                name: "Later".into(),
                tasks: vec![later],
            },
        ],
    )
    .unwrap();
    let path = slug.join("plan/plan.json");
    let before = std::fs::read(&path).unwrap();
    let mut bad = base.clone();
    bad.tasks[0].id = "../escape".into();
    assert!(phase_replan::adopt(&slug, "first", &bad).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    bad.tasks[0].id = "later".into();
    assert!(phase_replan::adopt(&slug, "first", &bad).is_err());
    let mut next = base.clone();
    next.tasks.remove(1);
    next.tasks[0].prompt.push_str(" revised");
    phase_replan::adopt(&slug, "first", &next).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&before).unwrap();
    let updated: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(updated["phases"][1], original["phases"][1]);
    assert_eq!(updated["phases"][0]["tasks"].as_array().unwrap().len(), 2);
    assert_eq!(
        std::fs::read(slug.join("plan/original.json")).unwrap(),
        before
    );
    assert!(
        agentconductor::slug_progress::progress(&slug)["mismatches"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    std::fs::remove_dir_all(root).unwrap();
}
