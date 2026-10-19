#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::task_controls::{self, Action};
use agentconductor::workflow;
use eframe::egui::{self, accesskit::Role};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

#[path = "support/workflow.rs"]
mod support;

fn snapshot() -> Value {
    json!({
        "revision":"r1",
        "job":{"state":"completed"},
        "report":{
            "state":"ready",
            "plan":{
                "tasks":[{"id":"tests","agent":"codex","model":null,"prompt":"Write tests"}],
                "agents":[{"id":"codex","parser":"codex","args":[],"catalog":{"models":["codex-small","codex-large"],"source":"observed","queried_at":1,"cli_version":"test"}},{"id":"local","parser":"custom"}]
            },
            "tasks":[{"id":"tests","status":"pending","session_id":"session-tests","feedback":null}]
        }
    })
}

#[test]
fn manual_scheduling_requires_a_waiting_task_and_explicit_timezone() {
    let mut snapshot = snapshot();
    snapshot["report"]["tasks"][0]["status"] = json!("waiting_availability");
    let packet = task_controls::request(
        &snapshot,
        Action::Schedule {
            task: "tests".into(),
            at: "2099-01-01T09:00:00+09:00".into(),
        },
    )
    .unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(&packet).unwrap(),
        json!({"kind":"action","revision":"r1","action":{"kind":"schedule","task":"tests","at":"2099-01-01T09:00:00+09:00"}})
    );
    assert!(
        task_controls::request(
            &snapshot,
            Action::Schedule {
                task: "tests".into(),
                at: "2099-01-01T09:00".into(),
            }
        )
        .is_err()
    );
    snapshot["report"]["tasks"][0]["status"] = json!("pending");
    assert!(
        task_controls::request(
            &snapshot,
            Action::Schedule {
                task: "tests".into(),
                at: "2099-01-01T09:00:00+09:00".into(),
            }
        )
        .is_err()
    );
}

#[test]
fn task_controls_localize_the_task_picker_and_feedback_action() {
    for language in ["en", "ko"] {
        let mut snapshot = snapshot();
        snapshot["report"]["tasks"][0]["status"] = json!("waiting_availability");
        snapshot["report"]["tasks"][0]["restriction"] = json!({"reason":"account quota exhausted"});
        let mut state = task_controls::State::default();
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut first = context.run_ui(Default::default(), |ui| {
            assert_eq!(
                task_controls::show(&mut state, &snapshot, false, language, ui),
                None
            );
        });
        first.textures_delta.set.clear();
        first.textures_delta.free.clear();
        let mut output = context.run_ui(Default::default(), |ui| {
            assert_eq!(
                task_controls::show(&mut state, &snapshot, false, language, ui),
                None
            );
        });
        let nodes = &output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes;
        let labels = nodes
            .iter()
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        let buttons = nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Button)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for key in [
            "room.task_inspector",
            "room.send_feedback_823598",
            "startup.session_model",
            "room.schedule_timestamp_with_timezone",
        ] {
            let expected = agentconductor::ui::messages::in_language(language, key);
            assert!(labels.contains(&expected), "missing {key} for {language}");
        }
        let expected_session = format!(
            "{}session-tests",
            agentconductor::ui::messages::in_language(language, "room.session_168693")
        );
        assert!(labels.iter().any(|label| label.contains(&expected_session)));
        assert!(buttons.contains(&agentconductor::ui::messages::in_language(
            language,
            "room.send_feedback_823598"
        )));
        assert!(buttons.contains(&agentconductor::ui::messages::in_language(
            language,
            "room.schedule_retry"
        )));
    }
}

#[tokio::test]
async fn native_schedule_action_reaches_controller_and_persists_manual_time() {
    let (root, mut plan) = support::setup("worktree");
    let target_tmp = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tmp");
    assert!(root.starts_with(target_tmp));
    plan.agents[0].args = vec!["quota-once".into()];
    plan.agents[0]
        .environment
        .insert("MOCK_MARKERS".into(), root.to_str().unwrap().into());
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    let room =
        agentconductor::ui::Room::open(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor")))
            .unwrap();
    let snapshot = room
        .dispatch(br#"{"kind":"snapshot","room":true}"#)
        .unwrap();
    let timestamp = "2099-01-01T09:00:00+09:00";
    let request = task_controls::request(
        &snapshot,
        Action::Schedule {
            task: "one.txt".into(),
            at: timestamp.into(),
        },
    )
    .unwrap();
    assert_eq!(room.dispatch(&request).unwrap()["accepted"], true);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let status = room
            .dispatch(br#"{"kind":"snapshot","room":true}"#)
            .unwrap();
        if status["job"]["state"] != "running" {
            assert_ne!(status["job"]["state"], "failed", "{:?}", status["job"]);
            break;
        }
        assert!(std::time::Instant::now() < deadline, "action timed out");
        std::thread::sleep(Duration::from_millis(20));
    }
    let scheduled = workflow::load(&run).unwrap();
    assert_eq!(
        scheduled.tasks[0]
            .restriction
            .as_ref()
            .and_then(|restriction| restriction.available_at),
        Some(agentconductor::availability::parse_time(timestamp).unwrap())
    );
    assert_eq!(
        scheduled.tasks[0].restriction.as_ref().unwrap().source,
        "manual_schedule"
    );
    workflow::stop(&run, "test cleanup").unwrap();
    wait_for_availability_watcher_exit(&run);
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

fn wait_for_availability_watcher_exit(run: &Path) {
    let spawn = run.join("availability-spawn.lock");
    let watch = run.join("availability-watch.lock");
    let owner = run.join("availability-coordinator/process/owner.json");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        if let (Ok(_spawn), Ok(_watch)) = (
            agentconductor::leases::Lease::acquire(&spawn),
            agentconductor::leases::Lease::acquire(&watch),
        ) {
            let terminal = !owner.exists()
                || serde_json::from_slice::<Value>(&std::fs::read(&owner).unwrap()).unwrap()["phase"]
                    == "terminal";
            if terminal {
                return;
            }
        }
        assert!(
            std::time::Instant::now() < deadline,
            "availability watcher did not stop"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
