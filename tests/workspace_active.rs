use agentconductor::{state::Store, ui::Room};
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;
fn call(room: &Room, value: Value) -> Result<Value, String> {
    room.dispatch(value.to_string().as_bytes())
}
fn wait(mut ready: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(40);
    while !ready() {
        assert!(
            Instant::now() < deadline,
            "background operation did not progress"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
#[test]
fn active_planning_and_running_rooms_survive_creating_and_reopening_workspaces() {
    let (root, plan) = support::setup("shared");
    let home = root.join("home");
    let mut env = test_env::lock();
    env.set("HOME", &home);
    let workspace = home.join(".agentconductor/desktop");
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let fixture = root.join(if cfg!(windows) {
        "navigation-agent.exe"
    } else {
        "navigation-agent"
    });
    assert!(
        std::process::Command::new("rustc")
            .args(["tests/support/workspace_active_agent.rs", "-o"])
            .arg(&fixture)
            .status()
            .unwrap()
            .success()
    );
    let release = root.join("release");
    let room = Room::startup(&workspace, exe).unwrap();
    let state = call(&room, json!({"kind":"snapshot"})).unwrap();
    let mut config = state["config"].clone();
    config["repository"] = json!(root.join("repo"));
    config["goal"] = json!("Background plan");
    config["name"] = json!("Original");
    config["coordinator"] = json!("mock");
    config["auto_resume"] = json!(false);
    config["profiles"] = json!([{"id":"mock","parser":"custom","executable":fixture,"args":[],"permission_mode":"read_only","environment":{"AC_NAV_TEST_RELEASE":release}}]);
    call(
        &room,
        json!({"kind":"save","revision":state["revision"],"config":config}),
    )
    .unwrap();
    let state = call(&room, json!({"kind":"snapshot"})).unwrap();
    call(
        &room,
        json!({"kind":"prepare","revision":state["revision"]}),
    )
    .unwrap();
    wait(|| release.with_extension("ready").exists());
    let running = call(&room, json!({"kind":"snapshot"})).unwrap();
    assert_eq!(running["job"]["state"], "running");
    let job = running["job"]["id"].clone();
    let first = workspace.canonicalize().unwrap();
    let occupied = root.join("occupied");
    let other = Room::startup(&occupied, exe).unwrap();
    assert!(call(&room, json!({"kind":"switch_workspace","path":occupied})).is_err());
    assert_eq!(
        call(&room, json!({"kind":"snapshot"})).unwrap()["job"]["id"],
        job,
        "target lease failure preserves the active planner"
    );
    drop(other);

    let held = call(&room, json!({"kind":"create_workspace","name":"Second"}));
    if held.is_err() {
        fs::write(&release, "release").unwrap();
        wait(|| {
            call(&room, json!({"kind":"snapshot"})).is_ok_and(|s| s["job"]["state"] != "running")
        });
    }
    let second = held.expect("creating a workspace must preserve the active planner")["current"]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(
        call(&room, json!({"kind":"snapshot"})).unwrap()["config"]["goal"],
        ""
    );
    let before = fs::read_to_string(release.with_extension("progress")).unwrap();
    wait(|| fs::read_to_string(release.with_extension("progress")).is_ok_and(|p| p != before));
    assert!(
        Room::startup(&first, exe).is_err(),
        "the parked workspace keeps its ownership lease"
    );
    assert!(
        call(
            &room,
            json!({"kind":"delete_workspace","path":first,"name":"Original"})
        )
        .is_err()
    );
    call(&room, json!({"kind":"open_workspace","path":first})).unwrap();
    let restored = call(&room, json!({"kind":"snapshot"})).unwrap();
    assert_eq!(restored["job"]["id"], job);
    assert_eq!(restored["config"]["goal"], "Background plan");
    call(&room, json!({"kind":"switch_workspace","path":second})).unwrap();
    fs::write(&release, "release").unwrap();
    call(&room, json!({"kind":"open_workspace","path":first})).unwrap();
    wait(|| {
        call(&room, json!({"kind":"snapshot"})).is_ok_and(|s| s["proposal"]["phase"] == "ready")
    });
    assert_eq!(
        call(&room, json!({"kind":"snapshot"})).unwrap()["job"]["state"],
        "completed"
    );
    drop(room);
    let run = first.join("run-navigation");
    fs::create_dir(&run).unwrap();
    let mut report = json!({"schema_version":1,"state":"completed","repo":root.join("repo"),"base":"base","branch":"main","plan":plan,"tasks":[],"candidate":null,"tests":[],"diff_hash":null,"error":null,"verification_cwd":root.join("repo")});
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "fixture")
        .unwrap();
    let mut saved: Value =
        serde_json::from_slice(&fs::read(first.join("startup.json")).unwrap()).unwrap();
    saved["last_run"] = json!(run);
    saved["proposal"] = Value::Null;
    fs::write(
        first.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let room = Room::startup(&first, exe).unwrap();
    call(&room, json!({"kind":"open_run"})).unwrap();
    call(&room, json!({"kind":"home"})).unwrap();
    report["state"] = json!("running");
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "fixture")
        .unwrap();

    assert!(call(&room, json!({"kind":"switch_workspace","path":"relative"})).is_err());
    assert_eq!(
        call(&room, json!({"kind":"snapshot","room":true})).unwrap()["report"]["state"],
        "running"
    );
    call(&room, json!({"kind":"switch_workspace","path":second})).unwrap();
    assert!(!run.join("stop.request").exists());
    assert!(
        call(
            &room,
            json!({"kind":"delete_workspace","path":first,"name":"Original"})
        )
        .is_err()
    );
    report["state"] = json!("completed");
    fs::write(
        run.join("report.json"),
        serde_json::to_vec(&report).unwrap(),
    )
    .unwrap();
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record("workflow", &report, "fixture")
        .unwrap();
    call(&room, json!({"kind":"open_workspace","path":first})).unwrap();
    assert_eq!(
        call(&room, json!({"kind":"snapshot","room":true})).unwrap()["report"]["state"],
        "completed"
    );
    call(&room, json!({"kind":"switch_workspace","path":second})).unwrap();
    assert!(
        Room::startup(&first, exe).is_ok(),
        "idle workspaces release their UI lease"
    );
    drop(room);
    fs::remove_dir_all(root).unwrap();
}
