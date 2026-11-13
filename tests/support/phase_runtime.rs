//! #943: real native requests and subprocess planning through a merged Phase boundary.
#[path = "workflow.rs"]
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
pub fn call(ui: &Startup, packet: Value) -> Value {
    ui.dispatch(packet.to_string().as_bytes()).unwrap()
}
pub fn action(ui: &Startup, kind: &str) -> Value {
    let state = call(ui, json!({"kind":"snapshot"}));
    call(ui, json!({"kind":kind,"revision":state["revision"]}))
}
pub fn settled(ui: &Startup) -> Value {
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
pub async fn prepare(
    changed: bool,
    auto_merge: bool,
    wait: bool,
) -> (PathBuf, Startup, PathBuf, Plan) {
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
    workflow::set_auto_merge(&run, auto_merge, &first).unwrap();
    let report = workflow::run(first, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    workflow::link_phase(&run, &slug, "p1").unwrap();
    if auto_merge {
        assert!(workflow::automatic_merge(&run).unwrap());
    } else {
        workflow::approve(&run, &report.candidate.unwrap().commit).unwrap();
        workflow::merge(&run).unwrap();
    }
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
