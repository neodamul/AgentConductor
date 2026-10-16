#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::room::task_activity::{self, Tab};
use agentconductor::native_ui_probe::room::{self, Event, State};
use agentconductor::ui::Room;
use eframe::egui::{self, accesskit::Role};
use serde_json::{Value, json};
use std::path::Path;

#[path = "support/workflow.rs"]
mod support;

#[test]
fn provider_events_become_cli_like_entries_without_raw_json() {
    let codex = task_activity::transcript(
        "codex",
        &json!({"type":"item.completed","item":{"type":"agent_message","text":"Added refresh guard"}}),
    );
    assert_eq!(codex[0].text, "Added refresh guard");
    assert!(!codex[0].text.contains("item.completed"));

    let claude = task_activity::transcript(
        "claude",
        &json!({"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"cargo test"}}]}}),
    );
    assert!(claude[0].text.contains("Bash(cargo test)"));

    let opencode = task_activity::transcript(
        "opencode",
        &json!({"type":"text","part":{"text":"Review complete"}}),
    );
    assert_eq!(opencode[0].text, "Review complete");

    let unknown = task_activity::transcript(
        "custom",
        &json!({"type":"vendor.notice","message":"queued"}),
    );
    assert_eq!(unknown[0].label, "vendor.notice");
    assert_eq!(unknown[0].text, "queued");

    let warning = task_activity::event_entries(
        "codex",
        &json!({"kind":"warning","payload":{"type":"error","message":"Reconnecting"}}),
    );
    assert_eq!(warning[0].label, "room.warning");
    assert_eq!(warning[0].tone, task_activity::Tone::Warning);
}

#[test]
fn task_details_use_the_existing_indexed_room_request() {
    assert_eq!(
        serde_json::from_slice::<Value>(&task_activity::request(2)).unwrap(),
        json!({"kind":"task","index":2})
    );
}

#[tokio::test]
async fn selected_task_details_load_through_the_existing_room_controller() {
    let (root, plan) = support::setup("shared");
    let run = root.join("run");
    agentconductor::workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    let room = Room::open(&run, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let details = room.dispatch(&task_activity::request(0)).unwrap();
    assert_eq!(details["task"]["id"], "one.txt");
    assert!(details["events"].is_array());
    assert!(details["stdout"].is_string());
    assert!(details["stderr"].is_string());
    assert!(details["knowledge"].is_array());
    drop(room);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn selecting_a_task_requests_its_details_from_the_room() {
    let snapshot = json!({
        "revision":"r1",
        "report":{
            "state":"waiting_availability",
            "plan":{"tasks":[{"id":"one","agent":"local","prompt":"Do the work","paths":[]}],"agents":[{"id":"local","parser":"custom","catalog":{"source":"test","models":[]}}]},
            "tasks":[{"id":"one","status":"waiting_availability","restriction":{"reason":"quota"}}],
            "candidate":null
        },
        "quotas":{},"tool_requests":[],"job":null
    });
    let mut state = State::default();
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            assert_eq!(
                room::show(&mut state, Some(&snapshot), false, None, "en", ui),
                Some(Event::TaskDetails(0))
            );
        });
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn activity_logs_and_reviewed_lessons_are_visible_in_both_languages() {
    let plan = json!({"id":"one","prompt":"Implement the refresh guard"});
    let details = json!({
        "task":{"id":"one","feedback":"Add the boundary test","started_ms":1,"finished_ms":2},
        "events":[{"seq":4,"kind":"progress","payload":{"type":"item.completed","item":{"type":"agent_message","text":"Guard implemented"}}},{"seq":5,"kind":"warning","payload":{"type":"error","message":"Retry scheduled"}}],
        "decisions":[{"allow":false,"tool":"Bash","input":"{\"command\":\"rm -rf .\"}","reason":"Publishing requires review"}],
        "stdout":"{\"type\":\"text\"}\n",
        "stderr":"compiler warning\n",
        "knowledge":[{"task":"one","status":"fresh","lesson":{"context":"Keep one timer","confidence":0.9,"response":"Use the existing timer","sources":[{"path":"src/timer.rs","sha256":"abc"}],"revalidate":"Check the timer owner"}}]
    });
    for language in ["en", "ko"] {
        for tab in [Tab::Activity, Tab::Logs, Tab::Lessons] {
            let context = egui::Context::default();
            context.enable_accesskit();
            let mut output = context.run_ui(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let mut selected = tab;
                    task_activity::show(
                        &mut selected,
                        "one",
                        "codex",
                        &plan,
                        Some(&details),
                        language,
                        ui,
                    );
                });
            });
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            let labels = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter_map(|(_, node)| node.label())
                .collect::<Vec<_>>();
            for key in ["room.activity", "room.raw_logs", "room.lessons"] {
                assert!(labels.contains(&agentconductor::ui::messages::in_language(language, key)));
            }
            let expected = match tab {
                Tab::Activity => "Guard implemented",
                Tab::Logs => "compiler warning",
                Tab::Lessons => "Keep one timer",
            };
            assert!(
                labels.iter().any(|label| label.contains(expected)),
                "{language}: {labels:?}"
            );
            if tab == Tab::Activity {
                assert!(!labels.iter().any(|label| label.contains("\"type\"")));
                let warning = agentconductor::ui::messages::in_language(language, "room.warning");
                assert!(labels.iter().any(|label| label.starts_with(warning)));
                assert!(labels.iter().any(|label| label.contains("Bash(rm -rf .)")));
                assert!(
                    labels
                        .iter()
                        .any(|label| label.contains("Publishing requires review"))
                );
            }
            assert!(
                output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .any(|(_, node)| node.role() == Role::Button)
            );
        }
    }
}
