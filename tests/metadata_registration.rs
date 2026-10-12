use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
fn snapshot(startup: &Startup) -> Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn call(startup: &Startup, mut packet: Value) -> Result<Value, String> {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes())
}
#[test]
fn registration_and_settings_stay_available_while_catalog_requests_are_queued() {
    let root = std::env::temp_dir().join(agentconductor::native_guard::session_id().unwrap());
    std::fs::create_dir(&root).unwrap();
    let executable = root.join(if cfg!(windows) {
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
            .arg(&executable)
            .status()
            .unwrap()
            .success()
    );
    let release = root.join("release");
    let startup = Startup::open(
        &root.join("workspace"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let profiles = json!([{"id":"first","parser":"opencode","executable":executable,"args":[],"environment":{"AC_METADATA_RELEASE":release}},{"id":"second","parser":"opencode","executable":executable,"args":[],"environment":{"AC_METADATA_RELEASE":release}}]);
    call(&startup,json!({"kind":"save","config":{"profiles":profiles,"coordinator":"first","goal":"","repository":""}})).unwrap();
    let revision = snapshot(&startup)["revision"].clone();
    call(&startup, json!({"kind":"discover","agent":"first"})).unwrap();
    assert_eq!(
        snapshot(&startup)["revision"],
        revision,
        "metadata scheduling does not invalidate editable settings"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !release.with_extension("waiting").exists() {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let paused = snapshot(&startup);
    assert!(
        paused["job"].is_null(),
        "discovery must not occupy the planning slot"
    );
    let mut changed = paused["config"]["profiles"].clone();
    changed[0]["args"] = json!(["--new-execution-setting"]);
    changed
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"company","parser":"custom","executable":"aipro","args":[]}));
    call(&startup, json!({"kind":"save_profiles","profiles":changed}))
        .expect("register another agent while querying models");
    let queued = call(&startup, json!({"kind":"discover","agent":"second"})).unwrap();
    assert_eq!(queued["queued"], true);
    let mut config = snapshot(&startup)["config"].clone();
    config["name"] = "Saved while querying".into();
    call(&startup, json!({"kind":"save","config":config})).unwrap();
    std::fs::write(&release, b"release").unwrap();
    loop {
        let state = snapshot(&startup);
        if state["metadata_job"]["state"] != "running"
            && state["metadata_queue"].as_array().unwrap().is_empty()
        {
            assert_eq!(state["config"]["profiles"].as_array().unwrap().len(), 3);
            assert!(
                state["config"]["profiles"][0]["catalog"].is_null(),
                "discard a late catalog for changed execution settings"
            );
            assert_eq!(
                state["config"]["profiles"][0]["args"],
                json!(["--new-execution-setting"])
            );
            assert_eq!(
                state["config"]["profiles"][1]["catalog"]["models"],
                json!(["local/fixture-model"])
            );
            break;
        }
        assert!(std::time::Instant::now() < deadline, "{state}");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
