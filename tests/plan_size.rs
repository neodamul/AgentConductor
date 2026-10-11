#![cfg(unix)]
//! R-53/R-54, DD-60, QAS-45: 1–12 tasks and plans read from a Markdown document.
use agentconductor::{plan::Plan, ui::setup::Startup};
use std::{path::Path, process::Command};
#[path = "support/workflow.rs"]
mod support;

const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");

fn sized(plan: &Plan, n: usize) -> Plan {
    let mut plan = plan.clone();
    let template = plan.tasks[0].clone();
    plan.tasks = (0..n)
        .map(|i| {
            let mut task = template.clone();
            task.id = format!("t{i}");
            task.paths = vec![format!("f{i}.txt")];
            task
        })
        .collect();
    plan
}

#[test]
fn plans_accept_one_to_twelve_tasks() {
    let (root, plan) = support::setup("shared");
    for (n, ok) in [(0, false), (1, true), (2, true), (12, true), (13, false)] {
        assert_eq!(sized(&plan, n).validate().is_ok(), ok, "{n} tasks");
    }
    std::fs::remove_dir_all(root).unwrap();
}

fn planner_profiles(root: &Path) -> std::path::PathBuf {
    let plan = support::setup("shared").1;
    let mut planner = plan.agents[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan".into()];
    planner.permission_mode = "read_only".into();
    let path = root.join("profiles.json");
    std::fs::write(
        &path,
        serde_json::to_vec(&vec![plan.agents[0].clone(), planner]).unwrap(),
    )
    .unwrap();
    path
}

#[test]
fn plan_command_reads_a_markdown_document_as_reference() {
    let (root, _) = support::setup("shared");
    let profiles = planner_profiles(&root);
    let doc = root.join("plan.md");
    std::fs::write(
        &doc,
        "# Health plan\n\n- [x] Done item\n- [ ] Add the health module\n",
    )
    .unwrap();
    let run = |dir: &str, extra: &[&std::ffi::OsStr]| {
        Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .arg("plan")
            .arg(&profiles)
            .arg("planner")
            .arg(root.join("repo"))
            .arg(root.join(dir))
            .arg("Continue")
            .args(extra)
            .output()
            .unwrap()
    };
    let ok = run("from-md", &["--from".as_ref(), doc.as_os_str()]);
    assert!(
        ok.status.success(),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let prompt = std::fs::read_to_string(root.join("from-md/prompt.txt")).unwrap();
    assert!(
        prompt.contains("Add the health module") && prompt.contains("unfinished"),
        "{prompt}"
    );
    assert!(!prompt.contains("exactly three"), "no fixed task count");
    assert!(prompt.contains("between 1 and 12"));
    assert!(root.join("from-md/source-plan.md").exists());
    std::fs::write(&doc, "x".repeat(65 * 1024)).unwrap();
    assert!(
        !run("too-big", &["--from".as_ref(), doc.as_os_str()])
            .status
            .success()
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn read_plan_fills_an_empty_goal_from_the_document_heading() {
    let (root, plan) = support::setup("shared");
    let mut planner = plan.agents[0].clone();
    planner.id = "planner".into();
    planner.args = vec!["plan".into()];
    planner.permission_mode = "read_only".into();
    let startup = Startup::open(
        &root.join("desktop"),
        Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let snapshot = |s: &Startup| s.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let revision = snapshot(&startup)["revision"].clone();
    let config = serde_json::json!({"profiles":[plan.agents[0].clone(), planner],"repository":root.join("repo"),"goal":"","coordinator":"planner","workspace_mode":"shared"});
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":revision,"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let doc = root.join("plan.md");
    std::fs::write(&doc, "# Health plan\n\n- [ ] Add the health module\n").unwrap();
    let revision = snapshot(&startup)["revision"].clone();
    startup
        .dispatch(
            serde_json::json!({"kind":"read_plan","revision":revision,"path":doc})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let ready = loop {
        let s = snapshot(&startup);
        if s["job"]["state"] != "running" {
            break s;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    };
    assert_eq!(ready["proposal"]["phase"], "ready", "{}", ready["job"]);
    assert_eq!(ready["config"]["goal"], "Health plan");
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn review_can_add_and_remove_tasks_and_read_a_plan_file() {
    assert!(HTML.contains("id=\"start-read-plan\""));
    assert!(SCRIPT.contains("kind:'read_plan'") && SCRIPT.contains("kind:'pick_file'"));
    assert!(SCRIPT.contains("'+ Add task'") && SCRIPT.contains("'Remove task'"));
}
