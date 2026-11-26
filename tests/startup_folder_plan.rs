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
            assert!(project.join("ladder_game/plan/plan.json").exists());
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

#[test]
fn approving_a_folder_plan_ignores_excluded_entries_without_initializing_project_git() {
    use agentconductor::{folder, workflow};
    use std::{
        fs,
        path::Path,
        time::{Duration, Instant},
    };
    for mode in ["shared", "worktree"] {
        let (root, mut plan) = support::setup(mode);
        let project = root.join("repo");
        fs::remove_dir_all(project.join(".git")).unwrap();
        let nested = project.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("keep.txt"), b"source\r\n").unwrap();
        fs::write(nested.join(".EnV.LoCaL"), b"private fixture").unwrap();
        let old_goal = nested.join("old goal [kept]");
        fs::create_dir(&old_goal).unwrap();
        fs::write(old_goal.join(agentconductor::slugs::MARKER), b"fixture").unwrap();
        fs::write(old_goal.join("plan.md"), b"historic goal").unwrap();
        plan.tasks.truncate(1);
        plan.tasks[0].test_command = vec!["git".into(), "--version".into()];
        let starting = Startup::open(
            &root.join("workspace"),
            Path::new(env!("CARGO_BIN_EXE_agentconductor")),
        )
        .unwrap();
        let state = call(&starting, json!({"kind":"snapshot"}));
        call(
            &starting,
            json!({"kind":"save","revision":state["revision"],"config":{"name":"Folder","profiles":plan.agents,"coordinator":"mock","repository":project,"goal":plan.goal,"workspace_mode":mode,"auto_resume":false}}),
        );
        let baseline = call(&starting, json!({"kind":"baseline_status"}));
        assert_eq!(baseline["source_mode"], "folder");
        assert_eq!(baseline["needs_git"], true);
        let state = call(&starting, json!({"kind":"snapshot"}));
        call(
            &starting,
            json!({"kind":"draft","revision":state["revision"]}),
        );
        let state = call(&starting, json!({"kind":"snapshot"}));
        call(
            &starting,
            json!({"kind":"edit","revision":state["revision"],"tasks":plan.tasks,"workspace_mode":mode}),
        );
        let state = call(&starting, json!({"kind":"snapshot"}));
        let result = starting.dispatch(json!({"kind":"start","revision":state["revision"],"digest":state["proposal"]["digest"]}).to_string().as_bytes());
        let history = folder::history_directory(&project).unwrap();
        let accepted = match result {
            Ok(result) => result,
            Err(error) => {
                drop(starting);
                if history.exists() {
                    fs::remove_dir_all(history).unwrap();
                }
                fs::remove_dir_all(root).unwrap();
                panic!("Approve and run for {mode} failed: {error}");
            }
        };
        let run = Path::new(accepted["run"].as_str().unwrap());
        let deadline = Instant::now() + Duration::from_secs(24);
        loop {
            let state = call(&starting, json!({"kind":"snapshot"}));
            if state["job"]["state"] == "completed" {
                break;
            }
            assert_ne!(state["job"]["state"], "failed", "{state}");
            assert!(Instant::now() < deadline, "{state}");
            std::thread::sleep(Duration::from_millis(25));
        }
        let report = workflow::load(run).unwrap();
        assert_eq!(report.state, "waiting_approval", "{:?}", report.error);
        assert_eq!(report.repo, project.canonicalize().unwrap());
        assert_eq!(report.tests[0].exit_code, Some(0));
        assert!(workflow::merge(run).is_err());
        assert!(!project.join(".git").exists());
        assert_eq!(
            fs::read(nested.join(".EnV.LoCaL")).unwrap(),
            b"private fixture"
        );
        assert_eq!(
            fs::read(old_goal.join("plan.md")).unwrap(),
            b"historic goal"
        );
        drop(starting);
        fs::remove_dir_all(history).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
}
