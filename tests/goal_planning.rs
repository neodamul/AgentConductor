//! #429: inherited defaults, actual planner model argv and historical Goal isolation.
use agentconductor::{native_guard, ui::setup::Startup};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;

fn snapshot(s: &Startup) -> Value {
    s.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}
fn mutate(s: &Startup, mut v: Value) -> Result<Value, String> {
    v["revision"] = snapshot(s)["revision"].clone();
    s.dispatch(v.to_string().as_bytes())
}
fn ready(s: &Startup) -> Value {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let v = snapshot(s);
        if v["job"]["state"] != "running" {
            assert_eq!(v["job"]["state"], "completed", "{v}");
            assert_eq!(v["proposal"]["phase"], "ready", "{v}");
            return v;
        }
        assert!(Instant::now() < deadline, "{v}");
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn new_goal_uses_confirmed_profile_model_without_rewriting_previous_plan() {
    let (root, plan) = support::setup("shared");
    let capture = root.join(if cfg!(windows) {
        "capture.exe"
    } else {
        "capture"
    });
    assert!(
        Command::new("rustc")
            .args(["tests/support/planning_capture_agent.rs", "-o"])
            .arg(&capture)
            .status()
            .unwrap()
            .success()
    );
    let mut profiles = plan.agents.clone();
    profiles[0].executable = capture.to_string_lossy().into();
    profiles[0].args = vec!["plan-roles".into(), "{model}".into()];
    profiles[0].permission_mode = "read_only".into();
    profiles[0].coordinator_model = Some("local-v1".into());
    profiles[0].catalog=Some(serde_json::from_value(json!({"models":["local-v1","local-v2"],"source":"operator","queried_at":1,"cli_version":"fixture"})).unwrap());
    profiles[0].environment.insert(
        "AC_DELEGATE".into(),
        support::fixture().to_string_lossy().into(),
    );
    profiles[0].environment.insert(
        "AC_CAPTURE_ARGS".into(),
        root.join("argv").to_string_lossy().into(),
    );
    let mut alternate = profiles[0].clone();
    alternate.id = "alternate".into();
    alternate.coordinator_model = Some("local-v2".into());
    profiles.push(alternate);
    let workspace = root.join("workspace");
    let startup =
        Startup::open(&workspace, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    mutate(&startup,json!({"kind":"save","config":{"name":"Team","profiles":profiles,"coordinator":"mock","repository":root.join("repo"),"goal":"First goal","workspace_mode":"shared","roles":{"development":["mock"],"testing":["mock"],"documentation":["mock"]}}})).unwrap();
    mutate(&startup, json!({"kind":"prepare"})).unwrap();
    let first = ready(&startup);
    assert_eq!(
        fs::read_to_string(root.join("argv")).unwrap(),
        "plan-roles\nlocal-v1"
    );
    let plan_file = root.join("repo/first_goal/plan/plan.json");
    let first_plan = fs::read(&plan_file).unwrap();
    mutate(&startup, json!({"kind":"new_goal"})).unwrap();
    let new = snapshot(&startup);
    for field in ["coordinator", "profiles", "roles"] {
        assert_eq!(new["config"][field], first["config"][field], "{field}");
    }
    assert!(new["proposal"].is_null());
    assert_eq!(new["run_available"], false);
    let mut config = new["config"].clone();
    config["goal"] = json!("Second goal");
    config["coordinator"] = json!("alternate");
    mutate(&startup, json!({"kind":"save","config":config})).unwrap();
    mutate(&startup, json!({"kind":"prepare"})).unwrap();
    ready(&startup);
    assert_eq!(
        fs::read_to_string(root.join("argv")).unwrap(),
        "plan-roles\nlocal-v2"
    );
    assert_eq!(
        fs::read(&plan_file).unwrap(),
        first_plan,
        "another Goal's reviewed plan stays byte-identical"
    );
    let persisted: Value =
        serde_json::from_slice(&fs::read(workspace.join("profiles.json")).unwrap()).unwrap();
    assert_eq!(persisted[1]["coordinator_model"], "local-v2");
    mutate(&startup, json!({"kind":"new_goal"})).unwrap();
    let mut config = snapshot(&startup)["config"].clone();
    config["goal"] = json!(format!(
        "Invalid model {}",
        native_guard::session_id().unwrap()
    ));
    config["profiles"][1]["coordinator_model"] = json!("wrong-agent-model");
    mutate(&startup, json!({"kind":"save","config":config})).unwrap();
    let error = mutate(&startup, json!({"kind":"prepare"})).unwrap_err();
    assert!(error.contains("catalog"), "{error}");
    assert_eq!(
        fs::read_to_string(root.join("argv")).unwrap(),
        "plan-roles\nlocal-v2",
        "invalid selection starts no CLI"
    );
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn new_goal_keeps_a_saved_goal_that_was_never_planned() {
    // GitHub #791: R-55 says goals accumulate and a slug may exist without a plan; New Goal used to drop
    // a saved Goal that had no run yet.
    let (root, plan) = support::setup("shared");
    let s = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    mutate(&s, json!({"kind":"save","config":{"profiles":plan.agents,"repository":root.join("repo"),"goal":"Keep this goal","coordinator":"mock","workspace_mode":"shared"}})).unwrap();
    mutate(&s, json!({"kind":"new_goal"})).unwrap();
    let v = snapshot(&s);
    assert_eq!(v["config"]["goal"], "");
    let slugs = v["slugs"].as_array().unwrap();
    assert!(
        slugs.iter().any(|slug| slug["goal"] == "Keep this goal"),
        "{v}"
    );
    drop(s);
    fs::remove_dir_all(root).unwrap();
}
