//! #943/#959: native Phase continuation, consent and evidence regressions.
#[path = "support/phase_runtime.rs"]
mod fixture;
use agentconductor::{phase_replan, workflow};
use fixture::{action, call, prepare, settled};
use serde_json::{Value, json};
use std::{
    fs,
    time::{Duration, Instant},
};
#[tokio::test]
async fn changed_phase_waits_for_review_and_original_tasks_are_preserved() {
    let (root, ui, slug, _) = prepare(true, false, false).await;
    action(&ui, "next_phase");
    let state = settled(&ui);
    assert_eq!(state["job"]["state"], "completed", "{state}");
    assert_eq!(state["proposal"]["phase"], "ready");
    assert_eq!(state["proposal"]["replan"]["unchanged"], false);
    assert!(
        state["proposal"]["replan"].get("baseline").is_none(),
        "private profile environment must not be returned to UI"
    );
    assert!(
        state["proposal"]["replan"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["kind"] == "changed")
    );
    assert!(!root.join("repo/four.txt").exists());
    let index: Value =
        serde_json::from_slice(&fs::read(slug.join("plan/plan.json")).unwrap()).unwrap();
    assert_eq!(index["phases"][1]["specs"][0]["prompt"], "four.txt");
    // A saved proposal is re-used rather than spending another generation request.
    action(&ui, "next_phase");
    let after = call(&ui, json!({"kind":"snapshot"}));
    assert_eq!(
        after["proposal"]["directory"],
        state["proposal"]["directory"]
    );
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn identical_replan_continues_and_opt_in_merge_completes_the_goal() {
    for changed in [false, true] {
        let (root, ui, slug, _) = prepare(changed, true, false).await;
        action(&ui, "next_phase");
        if changed {
            let ready = settled(&ui);
            assert_eq!(ready["proposal"]["phase"], "ready");
            call(
                &ui,
                json!({"kind":"start","revision":ready["revision"],"digest":ready["proposal"]["digest"]}),
            );
        }
        let until = Instant::now() + Duration::from_secs(25);
        loop {
            let state = call(&ui, json!({"kind":"snapshot"}));
            if state["run_state"] == "completed"
                && state["job"]["state"] != "running"
                && agentconductor::slug_progress::progress(&slug)["stage"] == "completed"
            {
                assert_eq!(state["proposal"]["automatic_continuation"], true);
                break;
            }
            assert!(Instant::now() < until, "{state}");
            std::thread::sleep(Duration::from_millis(40));
        }
        assert!(root.join("repo/four.txt").exists());
        assert!(root.join("repo/five.txt").exists());
        assert_eq!(
            agentconductor::slug_progress::progress(&slug)["stage"],
            "completed"
        );
        drop(ui);
        fs::remove_dir_all(root).unwrap();
    }
}
#[tokio::test]
async fn stop_during_replan_prevents_automatic_start() {
    let (root, ui, _, _) = prepare(false, true, true).await;
    action(&ui, "next_phase");
    call(&ui, json!({"kind":"stop_planning"}));
    let state = settled(&ui);
    assert_eq!(state["job"]["state"], "stopped", "{state}");
    assert!(!root.join("repo/four.txt").exists());
    assert!(root.join("workspace/phase-chain.stop").exists());
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn identical_tasks_still_wait_for_merge_when_auto_merge_is_off() {
    let (root, ui, slug, _) = prepare(false, false, false).await;
    let head = agentconductor::git::Repository::open(&root.join("repo"))
        .unwrap()
        .head()
        .unwrap();
    action(&ui, "next_phase");
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let state = call(&ui, json!({"kind":"snapshot"}));
        if state["run_state"] == "waiting_approval" && state["job"]["state"] != "running" {
            break;
        }
        assert!(Instant::now() < until, "{state}");
        std::thread::sleep(Duration::from_millis(30));
    }
    assert_eq!(
        agentconductor::git::Repository::open(&root.join("repo"))
            .unwrap()
            .head()
            .unwrap(),
        head
    );
    assert_eq!(
        agentconductor::slug_progress::progress(&slug)["current_phase"],
        "p2"
    );
    assert!(!root.join("repo/five.txt").exists());
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn changed_completed_evidence_is_not_used_for_replanning() {
    let (root, ui, slug, _) = prepare(true, false, false).await;
    fs::remove_file(slug.join("plan/outcomes/p1.json")).unwrap();
    phase_replan::recover_outcomes(&slug, &root.join("workspace")).unwrap();
    assert!(phase_replan::context(&slug, "p2").is_ok());
    let diff = root.join("workspace/run-first/diff.patch");
    let before = fs::read(&diff).unwrap();
    fs::write(&diff, b"changed").unwrap();
    assert!(phase_replan::context(&slug, "p2").is_err());
    fs::write(&diff, before).unwrap();
    let report = workflow::load(&root.join("workspace/run-first")).unwrap();
    let log = fs::read(&report.tests[0].stdout).unwrap();
    fs::write(&report.tests[0].stdout, b"changed").unwrap();
    assert!(phase_replan::context(&slug, "p2").is_err());
    fs::write(&report.tests[0].stdout, log).unwrap();
    fs::write(
        report.tasks[0].artifacts.join("events.jsonl"),
        b"{\"kind\":\"result\",\"payload\":\"forged\"}\n",
    )
    .unwrap();
    assert!(phase_replan::context(&slug, "p2").is_err());
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
