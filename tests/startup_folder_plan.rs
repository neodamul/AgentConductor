use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
#[path = "support/workflow.rs"]
mod support;
fn call(s: &Startup, value: Value) -> Value {
    s.dispatch(value.to_string().as_bytes()).unwrap()
}
#[test]
fn native_generate_plan_accepts_an_ordinary_folder_and_preserves_it() {
    let (root, mut plan) = support::setup("shared");
    let project = root.join("repo");
    std::fs::remove_dir_all(project.join(".git")).unwrap();
    plan.agents[0].args = vec!["plan".into()];
    plan.agents[0]
        .environment
        .insert("AC_PLAN_SLUG".into(), "ladder_game".into());
    plan.agents[0].permission_mode = "read_only".into();
    let startup = Startup::open(
        &root.join("workspace"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = call(&startup, json!({"kind":"snapshot"}));
    call(
        &startup,
        json!({"kind":"save","revision":state["revision"],"config":{"name":"Ladder","profiles":plan.agents,"coordinator":"mock","repository":project,"goal":"사다리 게임을 만들자 그런데","workspace_mode":"shared"}}),
    );
    let state = call(&startup, json!({"kind":"snapshot"}));
    call(
        &startup,
        json!({"kind":"prepare","revision":state["revision"]}),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let state = call(&startup, json!({"kind":"snapshot"}));
        if state["job"]["state"] != "running" {
            assert_eq!(state["job"]["state"], "completed", "{state}");
            assert_eq!(state["proposal"]["phase"], "ready");
            assert_eq!(state["proposal"]["slug"], "ladder_game");
            assert_eq!(state["config"]["active_slug"], "ladder_game");
            assert!(project.join("Ladder/ladder_game/plan/plan.json").exists());
            assert_eq!(
                state["proposal"]["plan"]["tasks"].as_array().unwrap().len(),
                3
            );
            assert_ne!(
                state["proposal"]["workspace"],
                state["config"]["repository"]
            );
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "planning did not finish"
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(!project.join(".git").exists());
    assert!(!project.join("one.txt").exists());
    assert_eq!(std::fs::read(project.join("base.txt")).unwrap(), b"base\n");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
