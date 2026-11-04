//! R-211 / DD-223: native Goal actions never change selection or delete another tree.
use serde_json::{Value, json};
use std::fs;

#[path = "support/goal_actions.rs"]
mod support;
use support::Fixture;

#[test]
fn folder_and_result_are_read_only_and_unknown_goals_are_refused() {
    let f = Fixture::new();
    let before = f.snapshot();
    let folder = f
        .call(json!({"kind":"goal_folder","slug":"first_goal"}))
        .unwrap();
    assert_eq!(folder["root"], json!(f.folder()));
    assert!(
        f.call(json!({"kind":"goal_folder","slug":"../project"}))
            .is_err()
    );
    let result = f
        .call(json!({"kind":"goal_result","slug":"first_goal"}))
        .unwrap();
    assert!(result["report"].is_null());
    assert_eq!(f.snapshot()["revision"], before["revision"]);
    assert_eq!(f.snapshot()["config"], before["config"]);
}

#[test]
fn historical_result_does_not_switch_goal_and_rejects_external_run_binding() {
    let mut f = Fixture::new();
    let run = f.report("completed");
    f.edit_saved(|s| {
        s["config"]["slugs"][0]["last_run"] = json!(run);
        s["config"]["active_slug"] = Value::Null;
        s["config"]["goal"] = json!("Draft second goal");
    });
    let before = f.snapshot();
    let result = f
        .call(json!({"kind":"goal_result","slug":"first_goal"}))
        .unwrap();
    assert_eq!(result["report"]["state"], "completed");
    assert_eq!(result["evidence"]["test_output"], json!([]));
    assert_eq!(result["evidence"]["diff"], "");
    assert_eq!(result["details"], json!([]));
    assert_eq!(result["report"]["plan"]["goal"], "First goal");
    assert_eq!(f.snapshot()["config"], before["config"]);
    let outside = f.root.join("outside-run");
    fs::create_dir(&outside).unwrap();
    f.edit_saved(|s| s["config"]["slugs"][0]["last_run"] = json!(outside));
    assert!(
        f.call(json!({"kind":"goal_result","slug":"first_goal"}))
            .is_err()
    );
}

#[test]
fn deletion_requires_exact_confirmation_and_stale_revision_changes_nothing() {
    let f = Fixture::new();
    let mut req = f.delete();
    req["goal"] = json!("Wrong");
    assert!(f.mutate(req).is_err());
    let mut req = f.delete();
    req["root"] = json!(f.root.join("project"));
    assert!(f.mutate(req).is_err());
    let mut req = f.delete();
    req["revision"] = json!("stale");
    assert!(f.call(req).is_err());
    assert!(f.folder().exists());
    assert_eq!(f.snapshot()["slugs"].as_array().unwrap().len(), 1);
}

#[test]
fn last_goal_deletion_removes_only_its_folder_and_keeps_workspace_usable() {
    let f = Fixture::new();
    fs::write(f.root.join("project/source.txt"), "keep").unwrap();
    let deleted = f.mutate(f.delete()).unwrap();
    assert_eq!(deleted["deleted"], true);
    assert!(!f.folder().exists());
    assert!(f.root.join("project/source.txt").exists());
    let snapshot = f.snapshot();
    assert_eq!(snapshot["slugs"], json!([]));
    assert_eq!(snapshot["config"]["goal"], "");
    assert!(snapshot["config"]["active_slug"].is_null());
    let mut config = snapshot["config"].clone();
    config["goal"] = json!("New");
    f.mutate(json!({"kind":"save","config":config})).unwrap();
    f.mutate(json!({"kind":"create_slug","slug":"new_goal"}))
        .unwrap();
}

#[test]
fn missing_marker_and_linked_goal_root_are_refused() {
    let f = Fixture::new();
    fs::remove_file(f.folder().join(".agentconductor")).unwrap();
    assert!(f.mutate(f.delete()).is_err());
    assert!(f.folder().exists());
    #[cfg(unix)]
    {
        let actual = f.root.join("unrelated");
        fs::rename(f.folder(), &actual).unwrap();
        std::os::unix::fs::symlink(&actual, f.folder()).unwrap();
        assert!(f.mutate(f.delete()).is_err());
        assert!(actual.exists());
        assert!(
            f.call(json!({"kind":"goal_folder","slug":"first_goal"}))
                .is_err()
        );
    }
}

#[test]
fn active_run_gets_stop_request_before_any_folder_deletion() {
    let mut f = Fixture::new();
    let run = f.report("running");
    f.edit_saved(|s| {
        s["last_run"] = json!(run);
    });
    let answer = f.mutate(f.delete()).unwrap();
    assert_eq!(answer["stopping"], true);
    assert_eq!(answer["deleted"], false);
    assert!(run.join("stop.request").exists());
    assert!(f.folder().exists());
    assert_eq!(f.snapshot()["slugs"].as_array().unwrap().len(), 1);
}

#[test]
fn idle_review_can_be_deleted_but_owned_live_or_unknown_workers_are_retained() {
    let mut f = Fixture::new();
    let run = f.report("waiting_approval");
    f.edit_saved(|s| s["last_run"] = json!(run));
    let process = run.join("task-old/process");
    fs::create_dir_all(&process).unwrap();
    fs::write(process.join("owner.json"), "{}").unwrap();
    assert!(f.mutate(f.delete()).is_err());
    assert!(f.folder().exists());
    fs::remove_dir_all(run.join("task-old")).unwrap();
    let coordinator =
        agentconductor::leases::Lease::acquire(&run.join("coordinator.lock")).unwrap();
    assert_eq!(f.mutate(f.delete()).unwrap()["stopping"], true);
    assert!(f.folder().exists());
    drop(coordinator);
    assert_eq!(f.mutate(f.delete()).unwrap()["deleted"], true);
    assert!(
        agentconductor::workflow::recover(&run)
            .err()
            .expect("deleted Goal run must refuse recovery")
            .contains("Goal deletion revoked")
    );
    assert!(
        run.join("report.json").exists(),
        "central audit evidence remains"
    );
}

#[test]
fn deleting_an_inactive_goal_preserves_the_current_draft_and_unlinks_only_the_goal() {
    let f = Fixture::new();
    f.mutate(json!({"kind":"new_goal"})).unwrap();
    let mut config = f.snapshot()["config"].clone();
    config["goal"] = json!("Second");
    f.mutate(json!({"kind":"save","config":config})).unwrap();
    f.mutate(json!({"kind":"create_slug","slug":"second"}))
        .unwrap();
    #[cfg(unix)]
    {
        let unrelated = f.root.join("outside");
        fs::create_dir(&unrelated).unwrap();
        fs::write(unrelated.join("keep"), "keep").unwrap();
        std::os::unix::fs::symlink(&unrelated, f.folder().join("alias")).unwrap();
    }
    let before = f.snapshot()["config"].clone();
    assert_eq!(f.mutate(f.delete()).unwrap()["changed"], false);
    let after = f.snapshot()["config"].clone();
    assert_eq!(after["goal"], before["goal"]);
    assert_eq!(after["active_slug"], before["active_slug"]);
    assert!(f.root.join("project/second").exists());
    #[cfg(unix)]
    assert!(
        f.root.join("outside/keep").exists(),
        "link targets are never traversed"
    );
}

#[test]
fn planning_goal_deletion_requests_cancellation_and_keeps_the_folder() {
    let mut f = Fixture::new();
    let plan = f.root.join("workspace/plan-current");
    fs::create_dir(&plan).unwrap();
    let process = f.root.join("workspace/coordinator-ui-planning/process");
    fs::create_dir_all(&process).unwrap();
    let owner = serde_json::json!({"coordinator_pid":std::process::id(),"pid":std::process::id(),"process_group":std::process::id(),"phase":"running","exit_code":null,"termination_signal":null,"stopped":false,"timed_out":false});
    fs::write(
        process.join("owner.json"),
        serde_json::to_vec(&owner).unwrap(),
    )
    .unwrap();
    let run = f.root.join("workspace/run-planned");
    f.edit_saved(|s| {
        s["job"] = json!({"id":"planning","operation":"plan","started_ms":0});
        s["proposal"] = json!({"directory":plan,"repository_base":"base","run":run,"phase":"planning","slug":"first_goal"});
    });
    let answer = f.mutate(f.delete()).unwrap();
    assert_eq!(answer["stopping"], true);
    assert_eq!(answer["deleted"], false);
    assert!(plan.with_extension("stop.request").exists());
    assert!(f.folder().exists());
}
