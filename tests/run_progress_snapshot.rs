#![cfg(unix)]
//! GitHub #273 (DD-208): the run snapshot says whether the run's process is there, whether a stop
//! is in progress and which integration test runs, so the run screen never looks frozen.
use agentconductor::{state::Store, ui::Room, workflow};
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

fn snapshot(room: &Room) -> serde_json::Value {
    room.dispatch(br#"{"kind":"snapshot","room":true}"#)
        .unwrap()
}

#[tokio::test]
async fn the_test_in_progress_is_named_while_verification_runs() {
    let (root, mut plan) = support::setup("shared");
    for task in &mut plan.tasks {
        task.test_command = vec!["sh".into(), "-c".into(), "sleep 1".into()];
    }
    let run = root.join("run");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let worker = tokio::spawn({
        let (repo, run) = (root.join("repo"), run.clone());
        async move { workflow::run(plan, &repo, &run, 30).await }
    });
    let mut seen = None;
    for _ in 0..300 {
        if run.join("state.sqlite").exists()
            && let Ok(room) = Room::open(&run, exe)
        {
            let value = snapshot(&room);
            if value["verification"]["stage"] == "test" {
                seen = Some(value);
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    let value = seen.expect("the verification step is visible while its tests run");
    let step = &value["verification"];
    assert_eq!(step["total"], 3);
    assert!(step["index"].as_u64().unwrap() < 3);
    assert_eq!(
        step["task"],
        value["report"]["plan"]["tasks"][step["index"].as_u64().unwrap() as usize]["id"]
    );
    assert!(step["started_ms"].as_u64().unwrap() > 0);
    assert_eq!(
        value["runner"], "present",
        "this test process holds the run"
    );
    assert_eq!(value["stopping"], false);
    let report = worker.await.unwrap().unwrap();
    assert_eq!(report.state, "waiting_approval");
    let room = Room::open(&run, exe).unwrap();
    assert!(
        snapshot(&room)["verification"].is_null(),
        "no step once it ended"
    );
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_running_run_without_its_process_and_a_stop_in_progress_are_reported() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    let mut report = workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    // The room is open already: opening it recovers a dead run, but nothing watches it afterwards.
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::open(&run, exe).unwrap();
    report.state = "running".into();
    report.tasks[0].status = "running".into();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    store
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Fixture: the run process was killed",
        )
        .unwrap();
    drop(store);
    // The run's process record names a process that has exited.
    let mut gone = std::process::Command::new("true").spawn().unwrap();
    let pid = gone.id();
    gone.wait().unwrap();
    std::fs::write(run.join("runner.pid"), pid.to_string()).unwrap();
    let value = snapshot(&room);
    assert_eq!(value["runner"], "absent");
    assert_eq!(value["stopping"], false);
    assert!(value["verification"].is_null(), "a task is still running");
    std::fs::write(run.join("stop.request"), "operator").unwrap();
    assert_eq!(snapshot(&room)["stopping"], true);
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn a_finished_run_reports_no_runner() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    workflow::run(plan, &root.join("repo"), &run, 5)
        .await
        .unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = Room::open(&run, exe).unwrap();
    let value = snapshot(&room);
    assert!(
        value["runner"].is_null(),
        "only a running run is checked: {}",
        value["runner"]
    );
    assert!(value["job"].is_null());
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}
