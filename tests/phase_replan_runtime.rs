//! #943: real native requests and subprocess planning through a merged Phase boundary.
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{phase_replan, phases, plan::Plan, ui::setup::Startup, workflow};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
    time::{Duration, Instant},
};
fn planner() -> PathBuf {
    static CLI: OnceLock<PathBuf> = OnceLock::new();
    CLI.get_or_init(|| {
        let mut path = std::env::temp_dir().join(format!("ac-phase-replan-{}", std::process::id()));
        if cfg!(windows) {
            path.set_extension("exe");
        }
        assert!(
            Command::new("rustc")
                .args(["tests/support/phase_replan_agent.rs", "-o"])
                .arg(&path)
                .status()
                .unwrap()
                .success()
        );
        path
    })
    .clone()
}
fn call(ui: &Startup, packet: Value) -> Value {
    ui.dispatch(packet.to_string().as_bytes()).unwrap()
}
fn action(ui: &Startup, kind: &str) -> Value {
    let state = call(ui, json!({"kind":"snapshot"}));
    call(ui, json!({"kind":kind,"revision":state["revision"]}))
}
fn settled(ui: &Startup) -> Value {
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let state = call(ui, json!({"kind":"snapshot"}));
        if state["job"]["state"] != "running" {
            return state;
        }
        assert!(Instant::now() < until, "{state}");
        std::thread::sleep(Duration::from_millis(25));
    }
}
async fn prepare(changed: bool, auto_merge: bool, wait: bool) -> (PathBuf, Startup, PathBuf, Plan) {
    let (root, mut base) = support::setup("shared");
    for task in &mut base.tasks {
        task.id = task.id.trim_end_matches(".txt").into();
    }
    let mut next = base.tasks[0].clone();
    next.id = "four".into();
    next.prompt = "four.txt".into();
    next.paths = vec!["four.txt".into()];
    next.test_command = vec![
        planner().to_string_lossy().into(),
        "check".into(),
        "four.txt".into(),
    ];
    let mut coordinator = base.agents[0].clone();
    coordinator.id = "planner".into();
    coordinator.executable = planner().to_string_lossy().into();
    coordinator.args.clear();
    coordinator.environment.insert(
        "AC_REPLAN_REPLY".into(),
        root.join("reply.json").to_string_lossy().into(),
    );
    if wait {
        coordinator
            .environment
            .insert("AC_REPLAN_WAIT".into(), "1".into());
    }
    coordinator.environment.insert(
        "AC_REPLAN_REPLY_NEXT".into(),
        root.join("reply-next.json").to_string_lossy().into(),
    );
    let mut later = next.clone();
    later.id = "five".into();
    later.prompt = "five.txt".into();
    later.paths = vec!["five.txt".into()];
    later.test_command[2] = "five.txt".into();
    fs::write(
        root.join("reply-next.json"),
        json!({"type":"result","payload":{"tasks":[later]}}).to_string(),
    )
    .unwrap();
    base.agents.push(coordinator);
    let ui = Startup::open(
        &root.join("workspace"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = call(&ui, json!({"kind":"snapshot"}));
    call(
        &ui,
        json!({"kind":"save","revision":state["revision"],"config":{"profiles":base.agents,"coordinator":"planner","repository":root.join("repo"),"goal":base.goal,"auto_merge":auto_merge,"auto_resume":false}}),
    );
    let created = action(&ui, "create_slug");
    let slug = PathBuf::from(created["root"].as_str().unwrap());
    phases::write(
        &slug,
        "# Analysis\nKeep completed work",
        &[
            phases::PhaseSpec {
                id: "p1".into(),
                name: "Initial work".into(),
                tasks: base.tasks.clone(),
            },
            phases::PhaseSpec {
                id: "p2".into(),
                name: "Next work".into(),
                tasks: vec![next.clone()],
            },
            phases::PhaseSpec {
                id: "p3".into(),
                name: "Final work".into(),
                tasks: vec![later],
            },
        ],
    )
    .unwrap();
    let first = phases::run_plan(&slug, "p1", &base).unwrap();
    let run = root.join("workspace/run-first");
    let report = workflow::run(first, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    workflow::link_phase(&run, &slug, "p1").unwrap();
    workflow::approve(&run, &report.candidate.unwrap().commit).unwrap();
    workflow::merge(&run).unwrap();
    let context = phase_replan::context(&slug, "p2").unwrap();
    assert!(
        context["completed_phases"][0]["diff"]
            .as_str()
            .unwrap()
            .contains("one.txt")
    );
    assert_eq!(
        context["completed_phases"][0]["tests"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert!(context["completed_phases"][0]["tasks"][0]["final_reports"].is_array());
    let mut reply = next.clone();
    if changed {
        reply.prompt.push_str("\nPreserve the previous outcome");
    }
    fs::write(
        root.join("reply.json"),
        json!({"type":"result","payload":{"tasks":[reply]}}).to_string(),
    )
    .unwrap();
    (root, ui, slug, base)
}
#[tokio::test]
async fn changed_phase_waits_for_review_and_original_tasks_are_preserved() {
    let (root, ui, slug, _) = prepare(true, false, false).await;
    action(&ui, "next_phase");
    let state = settled(&ui);
    assert_eq!(state["job"]["state"], "completed", "{state}");
    assert_eq!(state["proposal"]["phase"], "ready");
    assert_eq!(state["proposal"]["replan"]["unchanged"], false);
    assert!(
        state["proposal"]["replan"].get("baseline").is_none(),
        "private profile environment must not be returned to UI"
    );
    assert!(
        state["proposal"]["replan"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["kind"] == "changed")
    );
    assert!(!root.join("repo/four.txt").exists());
    let index: Value =
        serde_json::from_slice(&fs::read(slug.join("plan/plan.json")).unwrap()).unwrap();
    assert_eq!(index["phases"][1]["specs"][0]["prompt"], "four.txt");
    // A saved proposal is re-used rather than spending another generation request.
    action(&ui, "next_phase");
    let after = call(&ui, json!({"kind":"snapshot"}));
    assert_eq!(
        after["proposal"]["directory"],
        state["proposal"]["directory"]
    );
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn identical_replan_continues_and_opt_in_merge_completes_the_goal() {
    let (root, ui, slug, _) = prepare(false, true, false).await;
    action(&ui, "next_phase");
    let until = Instant::now() + Duration::from_secs(25);
    loop {
        let state = call(&ui, json!({"kind":"snapshot"}));
        if state["run_state"] == "completed"
            && state["job"]["state"] != "running"
            && agentconductor::slug_progress::progress(&slug)["stage"] == "completed"
        {
            assert_eq!(state["proposal"]["automatic_continuation"], true);
            break;
        }
        assert!(Instant::now() < until, "{state}");
        std::thread::sleep(Duration::from_millis(40));
    }
    assert!(root.join("repo/four.txt").exists());
    assert!(root.join("repo/five.txt").exists());
    assert_eq!(
        agentconductor::slug_progress::progress(&slug)["stage"],
        "completed"
    );
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn stop_during_replan_prevents_automatic_start() {
    let (root, ui, _, _) = prepare(false, true, true).await;
    action(&ui, "next_phase");
    call(&ui, json!({"kind":"stop_planning"}));
    let state = settled(&ui);
    assert_eq!(state["job"]["state"], "stopped", "{state}");
    assert!(!root.join("repo/four.txt").exists());
    assert!(root.join("workspace/phase-chain.stop").exists());
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn identical_tasks_still_wait_for_merge_when_auto_merge_is_off() {
    let (root, ui, slug, _) = prepare(false, false, false).await;
    let head = agentconductor::git::Repository::open(&root.join("repo"))
        .unwrap()
        .head()
        .unwrap();
    action(&ui, "next_phase");
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let state = call(&ui, json!({"kind":"snapshot"}));
        if state["run_state"] == "waiting_approval" && state["job"]["state"] != "running" {
            break;
        }
        assert!(Instant::now() < until, "{state}");
        std::thread::sleep(Duration::from_millis(30));
    }
    assert_eq!(
        agentconductor::git::Repository::open(&root.join("repo"))
            .unwrap()
            .head()
            .unwrap(),
        head
    );
    assert_eq!(
        agentconductor::slug_progress::progress(&slug)["current_phase"],
        "p2"
    );
    assert!(!root.join("repo/five.txt").exists());
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
#[tokio::test]
async fn changed_completed_evidence_is_not_used_for_replanning() {
    let (root, ui, slug, _) = prepare(true, false, false).await;
    fs::remove_file(slug.join("plan/outcomes/p1.json")).unwrap();
    phase_replan::recover_outcomes(&slug, &root.join("workspace")).unwrap();
    assert!(phase_replan::context(&slug, "p2").is_ok());
    let diff = root.join("workspace/run-first/diff.patch");
    let before = fs::read(&diff).unwrap();
    fs::write(&diff, b"changed").unwrap();
    assert!(phase_replan::context(&slug, "p2").is_err());
    fs::write(&diff, before).unwrap();
    let report = workflow::load(&root.join("workspace/run-first")).unwrap();
    let log = fs::read(&report.tests[0].stdout).unwrap();
    fs::write(&report.tests[0].stdout, b"changed").unwrap();
    assert!(phase_replan::context(&slug, "p2").is_err());
    fs::write(&report.tests[0].stdout, log).unwrap();
    fs::write(
        report.tasks[0].artifacts.join("events.jsonl"),
        b"{\"kind\":\"result\",\"payload\":\"forged\"}\n",
    )
    .unwrap();
    assert!(phase_replan::context(&slug, "p2").is_err());
    drop(ui);
    fs::remove_dir_all(root).unwrap();
}
