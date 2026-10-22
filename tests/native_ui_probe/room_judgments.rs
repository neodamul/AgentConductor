use crate::native_ui_probe::{Pending, Probe};
use eframe::egui;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::native_ui_probe::worker_test_support as worker_support;

fn settle(probe: &mut Probe) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while probe.pending.is_some() || probe.snapshot.is_none() {
        probe.poll();
        assert!(
            Instant::now() < deadline,
            "coordinator request did not settle"
        );
        std::thread::yield_now();
    }
    assert!(probe.error.is_none(), "{:?}", probe.error);
}

#[test]
fn coordinator_judgments_load_and_refresh_through_the_native_worker() {
    let root = std::env::temp_dir().join(format!(
        "ac-native-judgments-{}",
        crate::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = root.canonicalize().unwrap();
    let mut store = crate::state::Store::open(root.join("state.sqlite")).unwrap();
    let report = json!({"schema_version":1,"state":"stopped","repo":root,"base":"base","branch":"main",
        "plan":{"goal":"Inspect coordinator judgments","agents":[],"tasks":[],"workspace_mode":"shared"},
        "tasks":[],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root});
    store.record("workflow", &report, "fixture").unwrap();
    store
        .record(
            "coordinator_review",
            &json!({"proposal":{"verdict":"revise","summary":"Repair the contract"},"apply":true}),
            "review",
        )
        .unwrap();
    drop(store);
    let room = Arc::new(crate::ui::Room::open(&root, &std::env::current_exe().unwrap()).unwrap());
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::connect_room(room.clone(), context.clone()).unwrap();
    settle(&mut probe);
    probe.updates.background_action(Instant::now(), false);
    probe.load_coordinator_judgments();
    assert!(matches!(probe.pending, Some(Pending::Coordinator)));
    settle(&mut probe);
    assert_eq!(
        probe
            .room_view
            .as_ref()
            .unwrap()
            .coordinator_judgments
            .as_ref()
            .unwrap()["decision_count"],
        1
    );

    let mut output = context.run_ui(Default::default(), |ui| probe.update_ui(ui));
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(text.contains("Repair the contract"), "{text}");
    let mut store = crate::state::Store::open(root.join("state.sqlite")).unwrap();
    store
        .record(
            "handoff",
            &json!({"task":"one","from":"paid","to":"company"}),
            "Fallback to company agent",
        )
        .unwrap();
    drop(store);
    probe.refresh_room();
    settle(&mut probe);
    let data = probe
        .room_view
        .as_ref()
        .unwrap()
        .coordinator_judgments
        .as_ref()
        .unwrap();
    assert_eq!(data["decision_count"], 2);
    assert_eq!(data["judgments"].as_array().unwrap().len(), 2);
    drop(probe);
    worker_support::close(room);
    std::fs::remove_dir_all(root).unwrap();
}
