//! #959: the actual failed automatic worker reaches the Room snapshot as a separate error.
#[path = "support/phase_runtime.rs"]
mod fixture;
use agentconductor::{state::Store, ui::Room, workflow};
use serde_json::json;
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
#[tokio::test]
async fn blocked_auto_merge_job_is_exposed_without_replacing_human_job_state() {
    let (root, ui, _, _) = fixture::prepare(true, false, false).await;
    fixture::action(&ui, "next_phase");
    let ready = fixture::settled(&ui);
    let run = std::path::PathBuf::from(ready["proposal"]["run"].as_str().unwrap());
    fixture::call(
        &ui,
        json!({"kind":"start","revision":ready["revision"],"digest":ready["proposal"]["digest"]}),
    );
    let finished = fixture::settled(&ui);
    assert_eq!(finished["run_state"], "waiting_approval");
    let mut report = workflow::load(&run).unwrap();
    report.review_round = 1;
    workflow::set_auto_merge(&run, true, &report.plan).unwrap();
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record(
            "workflow",
            &serde_json::to_value(report).unwrap(),
            "fixture review gate",
        )
        .unwrap();
    let mut config = finished["config"].clone();
    config["auto_merge"] = json!(true);
    fixture::call(
        &ui,
        json!({"kind":"save","revision":finished["revision"],"config":config}),
    );
    let end = Instant::now() + Duration::from_secs(30);
    loop {
        let state = fixture::call(&ui, json!({"kind":"snapshot"}));
        if state["job"]["kind"] == "automatic_merge" && state["job"]["state"] == "failed" {
            break;
        }
        assert!(Instant::now() < end, "{state}");
        std::thread::sleep(Duration::from_millis(25));
    }
    drop(ui);
    let room = Room::startup(
        &root.join("workspace"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    assert_eq!(
        room.dispatch(br#"{"kind":"open_run"}"#).unwrap()["opened"],
        true
    );
    let snapshot = room
        .dispatch(br#"{"kind":"snapshot","room":true}"#)
        .unwrap();
    assert_eq!(snapshot["automatic_merge_error"]["state"], "failed");
    assert_ne!(snapshot["job"]["kind"], "automatic_merge");
    assert_eq!(snapshot["report"]["state"], "waiting_approval");
    assert!(
        snapshot["automatic_merge_error"]["error"]
            .as_str()
            .unwrap()
            .contains(agentconductor::ui::messages::text(
                "native.automatic_merge_paused_review"
            ))
    );
    drop(room);
    fs::remove_dir_all(root).unwrap();
}
