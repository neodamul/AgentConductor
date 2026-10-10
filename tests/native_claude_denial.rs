#![cfg(unix)]
//! Live authenticated denial (DD-18/DD-19/DD-32): with real guarded Claude workers, a write
//! outside the owned paths is refused by policy and a shell command rejected by the human
//! never runs, while the third worker completes its scoped task in the same run.
use agentconductor::{adapters, gates::Ledger, plan::ModelCatalog, protocol::Provider};
use std::{
    collections::HashSet,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
#[path = "support/workflow.rs"]
mod support;

fn cli() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_agentconductor"));
    for name in [
        "ANTHROPIC_API_KEY",
        "ANTHROPIC_AUTH_TOKEN",
        "ANTHROPIC_BASE_URL",
        "CLAUDECODE",
    ] {
        command.env_remove(name);
    }
    command
}

#[test]
#[ignore = "Requires installed Claude CLI and existing claude.ai login; three guarded haiku workers"]
fn subscribed_claude_scope_escape_and_human_rejection_have_no_side_effects() {
    let (root, mut plan) = support::setup("shared");
    let mut profile = plan.agents[0].clone();
    profile.id = "claude".into();
    profile.executable = "claude".into();
    profile.parser = Provider::Claude;
    profile.args = adapters::invocation(Provider::Claude, None, "unused")
        .unwrap()
        .args;
    profile.permission_mode = "native_callback".into();
    profile.max_sessions = 3;
    profile.catalog = Some(ModelCatalog {
        models: vec!["haiku".into()],
        source: "claude --help model alias".into(),
        queried_at: 0,
        cli_version: "2.1.287".into(),
    });
    plan.agents = vec![profile];
    let prompts = [
        "Use only the Write tool to create the file outside.txt containing OUTSIDE. If it is refused, stop immediately and do nothing else.",
        "Use only the Bash tool to run exactly: touch denied-marker.txt. If it is refused, stop immediately and do nothing else.",
        "Use only the Write tool. Create the file three.txt containing exactly the single line CLAUDE_THREE. Do not touch other files.",
    ];
    let tests = [
        vec!["test", "!", "-e", "outside.txt"],
        vec!["test", "!", "-e", "denied-marker.txt"],
        vec!["grep", "-qx", "CLAUDE_THREE", "three.txt"],
    ];
    for ((task, prompt), test) in plan.tasks.iter_mut().zip(prompts).zip(tests) {
        task.agent = "claude".into();
        task.model = Some("haiku".into());
        task.prompt = prompt.into();
        task.test_command = test.into_iter().map(String::from).collect();
    }
    let plan_path = root.join("plan.json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let validation = cli().arg("validate").arg(&plan_path).output().unwrap();
    assert!(validation.status.success());
    let digest =
        serde_json::from_slice::<serde_json::Value>(&validation.stdout).unwrap()["plan_digest"]
            .as_str()
            .unwrap()
            .to_string();
    let (repo, run) = (root.join("repo"), root.join("run"));
    let mut child = cli()
        .arg("run")
        .args([&plan_path, &repo, &run])
        .arg(&digest)
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(900);
    let mut rejected = HashSet::new();
    while child.try_wait().unwrap().is_none() {
        assert!(
            Instant::now() < deadline,
            "evidence retained at {}",
            root.display()
        );
        let pending: Vec<String> = Ledger::open(&run.join("state.sqlite"))
            .and_then(|ledger| ledger.requests())
            .map(|requests| {
                requests
                    .into_iter()
                    .filter(|r| r.status == "pending" && !rejected.contains(&r.id))
                    .map(|r| r.id)
                    .collect()
            })
            .unwrap_or_default();
        for id in pending {
            let output = cli()
                .arg("tool-reject")
                .arg(&run)
                .arg(&id)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            rejected.insert(id);
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    assert!(!rejected.is_empty(), "no human decision was requested");
    assert!(!repo.join("outside.txt").exists());
    assert!(!repo.join("denied-marker.txt").exists());
    let requests = Ledger::open(&run.join("state.sqlite"))
        .unwrap()
        .requests()
        .unwrap();
    assert!(
        requests
            .iter()
            .all(|r| r.status != "consumed" || !rejected.contains(&r.id))
    );
    let status = cli().arg("status").arg(&run).output().unwrap();
    let report: serde_json::Value = serde_json::from_slice(&status.stdout).unwrap();
    let task = |id: &str| {
        report["tasks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == id)
            .unwrap()
            .clone()
    };
    // Policy escape is fail-closed: the worker is terminated before the write happens.
    assert!(
        task("one.txt")["error"]
            .as_str()
            .unwrap_or_default()
            .contains("outside the task's owned scopes")
    );
    // A human rejection denies only that call; the worker continues and finishes.
    assert_eq!(task("two.txt")["status"], "completed");
    assert_eq!(
        task("three.txt")["status"],
        "completed",
        "evidence retained at {}",
        root.display()
    );
    eprintln!(
        "state {} · rejected {} · evidence {}",
        report["state"],
        rejected.len(),
        root.display()
    );
}
