//! R-212 / DD-224: ring rows and popup rows are the same conservative cached observation.
use agentconductor::{plan::AgentProfile, quota};
use serde_json::{Value, json};
const NOW: i64 = 1_800_000_000;
fn profile(id: &str) -> AgentProfile {
    serde_json::from_value(
        json!({"id":id,"executable":"not-executed","args":[],"permission_mode":"external_sandbox"}),
    )
    .unwrap()
}
fn indicator(mut value: Value) -> Value {
    value["observed_at"] = json!(NOW);
    quota::indicators(&json!({"agent":value}), &[profile("agent")], "en", NOW)["agent"].clone()
}
#[test]
fn rings_sort_provider_windows_and_preserve_all_popup_windows() {
    let v = indicator(json!({"status":"available","windows":[
        {"bucket":"weekly","label":"seven_day","window_minutes":10080,"remaining_percent":24,"resets_at":NOW+900},
        {"bucket":"session","label":"five_hour","window_minutes":300,"used_percent":73,"remaining_percent":27},
        {"bucket":"model","label":"seven_day_opus","window_minutes":10080,"remaining_percent":61}
    ]}));
    assert_eq!(v["rings"].as_array().unwrap().len(), 2);
    assert_eq!(v["rings"][0]["label"], "5h");
    assert_eq!(v["rings"][0]["remaining_percent"].as_f64(), Some(27.0));
    assert_eq!(v["rings"][1]["remaining_percent"].as_f64(), Some(24.0));
    assert_eq!(v["windows"].as_array().unwrap().len(), 3);
    assert!(
        v["rings"][1]["reset_label"]
            .as_str()
            .unwrap()
            .starts_with("Resets ")
    );
}
#[test]
fn enterprise_allocation_is_one_truthful_ring_not_a_fake_hundred_percent_window() {
    let v = indicator(
        json!({"status":"available","plan_type":"enterprise","budgets":[
        {"scope":"credits","remaining":"50","allocated":"100"},
        {"scope":"individual","remaining":"146.73","allocated":"500.00"}
    ],"windows":[{"label":"five_hour","remaining_percent":80}]}),
    );
    assert!((v["rings"][0]["remaining_percent"].as_f64().unwrap() - 29.346).abs() < 0.001);
    assert_eq!(v["rings"][0]["scope"], "individual");
    assert_eq!(v["rings"][1]["remaining_percent"].as_f64(), Some(50.0));
    let one = indicator(
        json!({"status":"available","budgets":[{"scope":"individual","remaining_percent":64}]}),
    );
    assert_eq!(one["rings"].as_array().unwrap().len(), 1);
}
#[test]
fn stale_expired_and_invalid_values_never_fill_a_ring() {
    for value in [
        json!({"status":"stale","windows":[{"remaining_percent":73,"used_percent":27}]}),
        json!({"status":"available","windows":[{"remaining_percent":73,"used_percent":27,"resets_at":NOW}]}),
        json!({"status":"available","windows":[{"remaining_percent":101,"used_percent":-1}]}),
        json!({"status":"unavailable","error":"timeout","windows":[{"remaining_percent":73}]}),
    ] {
        let v = indicator(value);
        assert!(v["rings"][0]["remaining_percent"].is_null(), "{v}");
    }
    // GitHub #766: a value hides only after twice the refresh interval.
    let old = NOW - 1 - quota::fresh_limit(profile("agent").quota_refresh_minutes);
    let q = json!({"agent":{"status":"available","observed_at":old,"windows":[{"remaining_percent":73}]}});
    let v = quota::indicators(&q, &[profile("agent")], "en", NOW);
    assert!(v["agent"]["rings"][0]["remaining_percent"].is_null());
}
#[test]
fn used_only_capacity_is_derived_but_unlimited_is_never_a_percentage() {
    let v = indicator(json!({"status":"available","windows":[{"used_percent":70}]}));
    assert_eq!(v["rings"][0]["remaining_percent"].as_f64(), Some(30.0));
    let v = indicator(
        json!({"status":"available","budgets":[{"scope":"credits","unlimited":true,"remaining_percent":100}]}),
    );
    assert_eq!(v["rings"][0]["unlimited"], true);
    assert!(v["rings"][0]["remaining_percent"].is_null());
}
#[test]
fn errors_sources_and_unsupported_profiles_remain_inspectable_without_queries() {
    let v = indicator(
        json!({"status":"unavailable","error":"usage timed out","source":"CLI /usage","plan_type":"enterprise","windows":[]}),
    );
    assert_eq!(v["error"], "usage timed out");
    assert_eq!(v["source"], "CLI /usage");
    assert_eq!(v["rings"], json!([]));
    let v = quota::indicators(&json!({}), &[profile("custom")], "ko", NOW);
    assert_eq!(v["custom"]["status"], "unsupported");
    assert_eq!(v["custom"]["rings"], json!([]));
    let v = quota::indicators(
        &json!({"custom":{"status":"unavailable","windows":[],"error":"Account quota not queried"}}),
        &[profile("custom")],
        "en",
        NOW,
    );
    assert_eq!(v["custom"]["status"], "unsupported");
}

#[test]
fn startup_and_run_snapshots_expose_the_same_cached_indicator_without_launching_cli() {
    use agentconductor::{
        native_guard,
        state::Store,
        ui::{Room, setup::Startup},
    };
    use std::{fs, path::Path};
    let root = std::env::temp_dir().join(format!(
        "ac-quota-indicators-{}",
        native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(root.join("project")).unwrap();
    let root = root.canonicalize().unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&root.join("workspace"), exe).unwrap();
    let call = |v: Value| startup.dispatch(v.to_string().as_bytes()).unwrap();
    let revision = call(json!({"kind":"snapshot"}))["revision"].clone();
    let mut p = profile("custom");
    p.quota_command = Some(serde_json::from_value(json!({"args":["usage"]})).unwrap());
    call(
        json!({"kind":"save","revision":revision,"config":{"profiles":[p],"coordinator":"custom","repository":"","goal":"quota fixture","name":"Quota","auto_resume":false}}),
    );
    let state = call(json!({"kind":"snapshot"}));
    let p: AgentProfile = serde_json::from_value(state["config"]["profiles"][0].clone()).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let observation:quota::Snapshot=serde_json::from_value(json!({"source":"cached CLI fixture","observed_at":now,"windows":[{"bucket":"primary","label":"five_hour","used_percent":27,"remaining_percent":73,"window_minutes":300,"resets_at":now+3600}]})).unwrap();
    let mut cache = Store::open(root.join("workspace/quota.sqlite")).unwrap();
    quota::persist(&mut cache, &p, Ok(observation.clone())).unwrap();
    drop(cache);
    let workspace = call(json!({"kind":"snapshot"}));
    assert_eq!(
        workspace["quota_indicators"]["custom"]["rings"][0]["remaining_percent"].as_f64(),
        Some(73.0)
    );
    assert!(workspace["metadata_job"].is_null());
    let run = root.join("run-cached");
    fs::create_dir(&run).unwrap();
    let report = json!({"schema_version":1,"state":"completed","repo":root.join("project"),"base":"base","branch":"main","plan":{"goal":"quota fixture","agents":[p],"tasks":[],"workspace_mode":"shared"},"tasks":[],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("project")});
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    let mut store = Store::open(run.join("state.sqlite")).unwrap();
    store.record("workflow", &report, "fixture").unwrap();
    quota::persist(&mut store, &p, Ok(observation)).unwrap();
    drop(store);
    let room = Room::open(&run, exe).unwrap();
    let running = room
        .dispatch(br#"{"kind":"snapshot","room":true}"#)
        .unwrap();
    assert_eq!(running["quota_indicators"], workspace["quota_indicators"]);
    drop(room);
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn a_workspace_without_a_project_folder_reports_a_code_not_a_failed_lookup() {
    // GitHub #679: the popup shows a hint for this state; any other error keeps its text and has no code.
    let text = "Choose a project folder for this workspace first; Claude reports usage only inside a trusted project folder";
    let v = indicator(json!({"status":"unavailable","error":text}));
    assert_eq!(v["error_code"], "no_project_folder");
    assert_eq!(v["error"], text);
    let other = indicator(json!({"status":"unavailable","error":"quota query timed out"}));
    assert!(other["error_code"].is_null());
}
