use agentconductor::{state::Store, ui::Room, workflow};
use std::{
    fs,
    process::{Child, Command, Stdio},
    time::Duration,
};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn a_newer_stop_cannot_be_cleared_by_an_already_accepted_resume() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-known".into()];
    let dir = root.join("run");
    workflow::run(plan, &root.join("repo"), &dir, 5)
        .await
        .unwrap();
    workflow::stop(&dir, "First review").unwrap();
    let old = workflow::digest(&fs::read(dir.join("stop.request")).unwrap());
    workflow::stop(&dir, "Keep this new stop").unwrap();
    let resumed = workflow::resume_reviewed_stop(&dir, 5, &old).await.unwrap();
    assert_eq!(resumed.state, "stopped");
    assert_eq!(resumed.tasks[0].attempt, 1);
    let marker: serde_json::Value =
        serde_json::from_slice(&fs::read(dir.join("stop.request")).unwrap()).unwrap();
    assert_eq!(marker["reason"], "Keep this new stop");
    fs::remove_dir_all(root).unwrap();
}

struct ChildProcess(Child);

impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn agent_room_resume_action_restarts_a_stopped_task_through_its_ui_child() {
    let (root, mut plan) = support::setup("shared");
    let dir = root.join("run");
    let repo = root.join("repo");
    let started = root.join("worker.started");
    let release = root.join("worker.release");
    plan.agents[0].args = vec!["task-blocking".into()];
    plan.agents[0].environment.insert(
        "AC_TEST_BLOCKING_STARTED".into(),
        started.to_string_lossy().into_owned(),
    );
    plan.agents[0].environment.insert(
        "AC_TEST_BLOCKING_RELEASE".into(),
        release.to_string_lossy().into_owned(),
    );
    let bytes = serde_json::to_vec(&plan).unwrap();
    let plan_path = root.join("plan.json");
    fs::write(&plan_path, &bytes).unwrap();
    let mut coordinator = ChildProcess(
        Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .args(["run"])
            .arg(&plan_path)
            .arg(&repo)
            .arg(&dir)
            .arg(workflow::digest(&bytes))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    wait_for_file(&started).await;
    workflow::stop(&dir, "Review before resuming").unwrap();
    wait_for_state(&dir, "stopped").await;
    wait_for_exit(&mut coordinator.0).await;
    fs::write(&release, "continue").unwrap();

    let room = Room::open(
        &dir,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = room
        .dispatch(br#"{"kind":"snapshot","room":true}"#)
        .unwrap();
    let mut report = workflow::load(&dir).unwrap();
    report.error = Some("Coordinator finalized the stop after the UI snapshot".into());
    Store::open(dir.join("state.sqlite"))
        .unwrap()
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Coordinator finalized the stop after the UI snapshot",
        )
        .unwrap();
    let stale_decision = serde_json::json!({
        "kind":"action", "room":true, "revision":snapshot["revision"],
        "action":{"kind":"reject", "reason":"stale decision"}
    });
    let stale_error = room
        .dispatch(&serde_json::to_vec(&stale_decision).unwrap())
        .unwrap_err();
    assert!(
        stale_error.contains("실행 상태가 변경됐습니다"),
        "{stale_error}"
    );
    let action = serde_json::json!({
        "kind":"action",
        "room":true,
        "revision":snapshot["revision"],
        "action":{"kind":"resume"}
    });
    let accepted = room
        .dispatch(&serde_json::to_vec(&action).unwrap())
        .unwrap();
    assert_eq!(accepted["accepted"], true);
    let action_id = accepted["id"].as_str().unwrap();
    wait_for_state(&dir, "waiting_approval").await;
    wait_for_ui_job(&room, action_id).await;
    let report = workflow::load(&dir).unwrap();
    assert!(report.tasks[0].attempt >= 2);
    assert_eq!(report.tasks[0].status, "completed");
    drop(room);
    fs::remove_dir_all(root).unwrap();
}

async fn wait_for_file(path: &std::path::Path) {
    tokio::time::timeout(Duration::from_secs(15), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("blocking mock task did not start");
}

async fn wait_for_state(dir: &std::path::Path, state: &str) {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if workflow::load(dir).is_ok_and(|report| report.state == state) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        let observed = workflow::load(dir)
            .map(|report| {
                format!(
                    "state={}, tasks={:?}",
                    report.state,
                    report
                        .tasks
                        .iter()
                        .map(|task| format!("{}:{}:{}", task.id, task.status, task.attempt))
                        .collect::<Vec<_>>()
                )
            })
            .unwrap_or_else(|error| format!("report unavailable: {error}"));
        panic!("expected workflow state {state}; observed {observed}");
    });
}

async fn wait_for_ui_job(room: &Room, id: &str) {
    tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let snapshot = room
                .dispatch(br#"{"kind":"snapshot","room":true}"#)
                .unwrap();
            if snapshot["job"]["id"] == id && snapshot["job"]["state"] == "completed" {
                assert_eq!(snapshot["job"]["exit_code"], 0);
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("UI resume child did not complete");
}

async fn wait_for_exit(child: &mut Child) {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if child.try_wait().unwrap().is_some() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("coordinator did not exit after stop");
}
