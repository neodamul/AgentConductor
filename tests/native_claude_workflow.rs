#![cfg(unix)]
use agentconductor::{adapters, plan::ModelCatalog, protocol::Provider, workflow};
use std::process::{Command, Output};
#[path = "support/workflow.rs"]
mod support;

fn command(args: &[&std::ffi::OsStr]) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(args)
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("ANTHROPIC_AUTH_TOKEN")
        .env_remove("ANTHROPIC_BASE_URL")
        .env_remove("CLAUDECODE")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
#[ignore = "Requires installed Claude CLI and existing claude.ai login; three guarded haiku workers edit files"]
fn subscribed_claude_workers_run_review_approve_and_merge_in_shared_workspace() {
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
    for (task, token) in plan
        .tasks
        .iter_mut()
        .zip(["CLAUDE_ONE", "CLAUDE_TWO", "CLAUDE_THREE"])
    {
        task.agent = "claude".into();
        task.model = Some("haiku".into());
        task.prompt = format!(
            "Use only the Write tool. Create the file {} in the current directory containing exactly the single line {token}. Do not run commands or touch other files.",
            task.id
        );
        task.test_command = vec!["grep".into(), "-qx".into(), token.into(), task.id.clone()];
    }
    let plan_path = root.join("plan.json");
    std::fs::write(&plan_path, serde_json::to_vec(&plan).unwrap()).unwrap();
    let validation = command(&["validate".as_ref(), plan_path.as_os_str()]);
    let value: serde_json::Value = serde_json::from_slice(&validation.stdout).unwrap();
    let digest = value["plan_digest"].as_str().unwrap();
    let repo = root.join("repo");
    let run = root.join("run");
    command(&[
        "run".as_ref(),
        plan_path.as_os_str(),
        repo.as_os_str(),
        run.as_os_str(),
        digest.as_ref(),
    ]);
    let report = workflow::load(&run).unwrap();
    assert_eq!(
        report.state,
        "waiting_approval",
        "evidence retained at {}",
        run.display()
    );
    assert!(report.tasks.iter().all(|t| t.status == "completed"));
    assert!(report.tests.iter().all(|t| t.exit_code == Some(0)));
    let sessions: std::collections::HashSet<_> =
        report.tasks.iter().map(|t| t.session_id.clone()).collect();
    assert_eq!(sessions.len(), 3);
    command(&["diff".as_ref(), run.as_os_str()]);
    let sha = report.candidate.unwrap().commit;
    command(&["approve".as_ref(), run.as_os_str(), sha.as_ref()]);
    command(&["merge".as_ref(), run.as_os_str()]);
    assert_eq!(workflow::load(&run).unwrap().state, "completed");
    assert_eq!(
        std::fs::read_to_string(repo.join("one.txt"))
            .unwrap()
            .trim(),
        "CLAUDE_ONE"
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("two.txt"))
            .unwrap()
            .trim(),
        "CLAUDE_TWO"
    );
    eprintln!(
        "Native Claude workflow evidence retained at {}",
        root.display()
    );
}
