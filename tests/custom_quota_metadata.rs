#![cfg(unix)]
use agentconductor::ui::setup::Startup;
use serde_json::json;
use std::{
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[test]
fn startup_metadata_job_accepts_and_persists_custom_quota_profiles() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let payload = json!({"quota":{"source":"fixture","observed_at":now,
        "windows":[{"bucket":"internal","label":"weekly","used_percent":27.0,
            "remaining_percent":73.0,"window_minutes":10080,"resets_at":now+3600}],
        "budgets":[]}})
    .to_string();
    let dir = std::env::temp_dir().join(format!(
        "ac-custom-quota-ui-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&dir).unwrap();
    let startup = Startup::open(&dir, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    startup
        .dispatch(
            json!({"kind":"save_profiles","revision":state["revision"],"profiles":[{
                "id":"internal","parser":"custom","executable":"/bin/echo","args":[],
                "quota_command":{"args":[payload],"output_pointer":"/quota"}
            }]})
            .to_string()
            .as_bytes(),
        )
        .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(state["quota_refresh_due"]["internal"], true);
    startup
        .dispatch(
            json!({"kind":"quota","agent":"internal","revision":state["revision"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let until = Instant::now() + Duration::from_secs(10);
    loop {
        let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        if state["metadata_job"]["state"] != "running" {
            assert_eq!(
                state["quotas"]["internal"]["windows"][0]["remaining_percent"],
                73.0
            );
            break;
        }
        assert!(
            Instant::now() < until,
            "custom quota metadata timed out: {state}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(startup);
    std::fs::remove_dir_all(dir).unwrap();
}
