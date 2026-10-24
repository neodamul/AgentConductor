#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::task_controls::{self, Action, State};
use agentconductor::workflow;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2, accesskit::Role};
use serde_json::Value;
use std::{path::Path, time::Duration};

#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn operator_can_schedule_a_waiting_task_through_accessible_controls() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-once".into()];
    plan.agents[0]
        .environment
        .insert("MOCK_MARKERS".into(), root.to_string_lossy().into_owned());
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
    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = &snapshot;
    let mut state = State::default();

    let (_, first) = frame(&context, &mut state, snapshot, Vec::new());
    let time = bounds(
        &first,
        Role::TextInput,
        "Schedule time · ISO 8601 with timezone",
    );
    clear(first);
    let point = center(time);
    click(&context, &mut state, snapshot, point);
    let (_, typed) = frame(
        &context,
        &mut state,
        snapshot,
        vec![Event::Text("2099-01-01T09:00:00+09:00".into())],
    );
    clear(typed);

    let (_, ready) = frame(&context, &mut state, snapshot, Vec::new());
    let schedule = bounds(&ready, Role::Button, "Schedule retry");
    clear(ready);
    let action = click(&context, &mut state, snapshot, center(schedule)).unwrap();
    assert_eq!(
        action,
        Action::Schedule {
            task: "one.txt".into(),
            at: "2099-01-01T09:00:00+09:00".into()
        }
    );
    let request = task_controls::request(snapshot, action).unwrap();
    assert_eq!(room.dispatch(&request).unwrap()["accepted"], true);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let current = room
            .dispatch(br#"{"kind":"snapshot","room":true}"#)
            .unwrap();
        if current["job"]["state"] != "running" {
            assert_ne!(current["job"]["state"], "failed", "{current}");
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "schedule action timed out"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let scheduled = workflow::load(&run).unwrap();
    assert_eq!(
        scheduled.tasks[0]
            .restriction
            .as_ref()
            .and_then(|restriction| restriction.available_at),
        Some(agentconductor::availability::parse_time("2099-01-01T09:00:00+09:00").unwrap())
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

fn frame(
    context: &egui::Context,
    state: &mut State,
    snapshot: &Value,
    events: Vec<Event>,
) -> (Option<Action>, egui::FullOutput) {
    let mut action = None;
    let output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1100.0, 750.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            egui::CentralPanel::default().show(ui, |ui| {
                action = task_controls::show(state, snapshot, false, "en", ui);
            });
        },
    );
    (action, output)
}

fn bounds(output: &egui::FullOutput, role: Role, label: &str) -> egui::accesskit::Rect {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == role && node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .unwrap_or_else(|| panic!("missing {role:?} control {label:?}"))
}

fn center(rect: egui::accesskit::Rect) -> Pos2 {
    Pos2::new(
        ((rect.x0 + rect.x1) / 2.0) as f32,
        ((rect.y0 + rect.y1) / 2.0) as f32,
    )
}

fn click(
    context: &egui::Context,
    state: &mut State,
    snapshot: &Value,
    point: Pos2,
) -> Option<Action> {
    let mut action = None;
    for pressed in [true, false] {
        let (current, output) = frame(
            context,
            state,
            snapshot,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        if !pressed {
            action = current;
        }
        clear(output);
    }
    action
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
