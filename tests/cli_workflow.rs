#[path = "support/workflow.rs"]
mod support;
use std::process::Command;

#[test]
fn binary_runs_review_approve_and_merge_with_plan_consent() {
    let (root, plan) = support::setup("shared");
    let plan_path = root.join("plan.json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let bin = env!("CARGO_BIN_EXE_agentconductor");
    let validation = Command::new(bin)
        .arg("validate")
        .arg(&plan_path)
        .output()
        .unwrap();
    assert!(validation.status.success());
    let value: serde_json::Value = serde_json::from_slice(&validation.stdout).unwrap();
    let digest = value["plan_digest"].as_str().unwrap();
    let repo = root.join("repo");
    let run = root.join("run");
    let denied = Command::new(bin)
        .arg("run")
        .arg(&plan_path)
        .arg(&repo)
        .arg(&run)
        .arg("wrong-digest")
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert!(!run.exists());
    let output = Command::new(bin)
        .arg("run")
        .arg(&plan_path)
        .arg(&repo)
        .arg(&run)
        .arg(digest)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = agentconductor::workflow::load(&run).unwrap();
    assert_eq!(report.state, "waiting_approval");
    assert!(
        Command::new(bin)
            .arg("diff")
            .arg(&run)
            .output()
            .unwrap()
            .status
            .success()
    );
    let sha = report.candidate.unwrap().commit;
    assert!(
        Command::new(bin)
            .arg("approve")
            .arg(&run)
            .arg(&sha)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert!(
        Command::new(bin)
            .arg("merge")
            .arg(&run)
            .output()
            .unwrap()
            .status
            .success()
    );
    assert_eq!(
        agentconductor::workflow::load(&run).unwrap().state,
        "completed"
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn binary_schedules_reassigns_and_selects_models_with_native_commands() {
    let (root, mut plan) = support::setup("shared");
    let mut replacement = plan.agents[0].clone();
    replacement.id = "replacement".into();
    replacement.quota_group = Some("replacement-account".into());
    replacement.args = vec!["task".into(), "{model}".into(), "{session_id}".into()];
    replacement.catalog = Some(agentconductor::plan::ModelCatalog {
        models: vec!["local/one".into(), "local/two".into()],
        source: "fixture CLI query".into(),
        queried_at: 123,
        cli_version: "mock".into(),
    });
    plan.agents[0].args = vec!["quota".into()];
    plan.agents.push(replacement);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let dir = root.join("run");
    runtime
        .block_on(agentconductor::workflow::run(
            plan,
            &root.join("repo"),
            &dir,
            10,
        ))
        .unwrap();
    let bin = env!("CARGO_BIN_EXE_agentconductor");
    let run = |args: &[&str]| {
        let owned_args: Vec<String> = args.iter().map(|s| (*s).into()).collect();
        let (send, receive) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = send.send(Command::new(bin).args(owned_args).output());
        });
        let output = match receive.recv_timeout(std::time::Duration::from_secs(120)) {
            Ok(result) => result.unwrap(),
            Err(error) => {
                let _ = agentconductor::workflow::stop(
                    std::path::Path::new(&dir),
                    "Fixture CLI output deadline",
                );
                panic!("CLI {args:?} did not close its captured output: {error}");
            }
        };
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    let dir = dir.to_str().unwrap();
    run(&["schedule", dir, "one.txt", "2099-01-01T00:00:00Z"]);
    run(&["resume-due", dir]);
    let sessions = run(&["sessions", dir]);
    assert_eq!(
        String::from_utf8(sessions.stdout).unwrap().lines().count(),
        3
    );
    for task in ["one.txt", "two.txt", "three.txt"] {
        run(&["reassign", dir, task, "replacement", "local/one"]);
    }
    run(&["session-model", dir, "one.txt", "local/two"]);
    run(&["resume-due", dir]);
    assert_eq!(
        agentconductor::workflow::load(std::path::Path::new(dir))
            .unwrap()
            .state,
        "waiting_approval"
    );
    std::fs::remove_dir_all(root).unwrap();
}
