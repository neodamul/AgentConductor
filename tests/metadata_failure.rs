use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
fn call(s: &Startup, value: Value) -> Result<Value, String> {
    s.dispatch(value.to_string().as_bytes())
}
fn snapshot(s: &Startup) -> Value {
    call(s, json!({"kind":"snapshot"})).expect("metadata failure must not poison Snapshot")
}
#[test]
fn an_invalid_catalog_settles_as_failed_and_settings_work_after_restart() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let fixture = root.join(if cfg!(windows) {
        "metadata.exe"
    } else {
        "metadata"
    });
    assert!(
        agentconductor::subprocess::command("rustc")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/metadata_agent.rs"
            ))
            .arg("-o")
            .arg(&fixture)
            .status()
            .unwrap()
            .success()
    );
    let workspace = root.join("workspace");
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let s = Startup::open(&workspace, exe).unwrap();
    let state = snapshot(&s);
    call(&s,json!({"kind":"save","revision":state["revision"],"config":{"repository":"","goal":"","coordinator":"first","profiles":[{"id":"first","parser":"opencode","executable":fixture,"args":[],"environment":{"AC_METADATA_COUNT":"1025"}}]}})).unwrap();
    let state = snapshot(&s);
    call(
        &s,
        json!({"kind":"discover","agent":"first","revision":state["revision"]}),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let state = snapshot(&s);
        if state["metadata_job"]["state"] != "running" {
            assert_eq!(state["metadata_job"]["state"], "failed", "{state}");
            assert!(state["metadata_job"]["error"].as_str().is_some());
            assert!(state["config"]["profiles"][0]["catalog"].is_null());
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(s);
    let s = Startup::open(&workspace, exe).unwrap();
    let state = snapshot(&s);
    assert_eq!(state["metadata_job"]["state"], "failed");
    let mut config = state["config"].clone();
    config["name"] = "Saved after metadata error".into();
    call(
        &s,
        json!({"kind":"save","revision":state["revision"],"config":config}),
    )
    .unwrap();
    assert_eq!(snapshot(&s)["config"]["name"], "Saved after metadata error");
    let state = snapshot(&s);
    let mut config = state["config"].clone();
    config["profiles"][0]["environment"] = json!({"AC_METADATA_COUNT":"1"});
    call(
        &s,
        json!({"kind":"save","revision":state["revision"],"config":config}),
    )
    .unwrap();
    let state = snapshot(&s);
    call(
        &s,
        json!({"kind":"discover","agent":"first","revision":state["revision"]}),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let state = snapshot(&s);
        if state["metadata_job"]["state"] != "running" {
            assert_eq!(state["metadata_job"]["state"], "completed", "{state}");
            assert_eq!(
                state["config"]["profiles"][0]["catalog"]["models"],
                json!(["local/fixture-model"])
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(s);
    std::fs::remove_dir_all(root).unwrap();
}
