#![cfg(unix)]
//! Live OpenCode contract (T-04): the product's own argv builder starts a session from a
//! task file and resumes that exact session; output normalizes to one native session ID.
use agentconductor::{
    adapters,
    plan::{AgentProfile, ModelCatalog},
    process::{self, ProcessSpec},
    protocol::{Decoder, Provider},
};
use std::{path::Path, time::Duration};
#[path = "support/workflow.rs"]
mod support;

const MODEL: &str = "opencode/nemotron-3.5-lightning-free";

fn profile() -> AgentProfile {
    let (root, plan) = support::setup("shared");
    std::fs::remove_dir_all(root).unwrap();
    let mut profile = plan.agents[0].clone();
    profile.id = "opencode".into();
    profile.executable = "opencode".into();
    profile.parser = Provider::OpenCode;
    profile.args = adapters::invocation(Provider::OpenCode, None, "{task_file}")
        .unwrap()
        .args;
    profile.catalog = Some(ModelCatalog {
        models: vec![MODEL.into()],
        source: "opencode models".into(),
        queried_at: 0,
        cli_version: "2.0.25".into(),
    });
    profile
}

async fn turn(root: &Path, name: &str, session: Option<&str>, prompt: &str) -> (String, String) {
    let dir = root.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    let task = dir.join("task.md");
    std::fs::write(&task, prompt).unwrap();
    let args =
        adapters::session_args(&profile(), Some(MODEL), session, task.to_str().unwrap()).unwrap();
    let output = process::run(ProcessSpec {
        executable: "opencode".into(),
        args,
        cwd: dir.clone(),
        input: vec![],
        log_dir: dir.join("process"),
        timeout: Duration::from_secs(180),
        signal_grace: Duration::from_secs(2),
        environment: Default::default(),
    })
    .await
    .unwrap();
    assert_eq!(output.exit_code, Some(0), "evidence at {}", dir.display());
    let mut decoder = Decoder::new(Provider::OpenCode, 1024 * 1024);
    let mut events = decoder
        .push(&std::fs::read(&output.stdout).unwrap())
        .unwrap();
    events.extend(decoder.finish().unwrap());
    let sessions: std::collections::HashSet<_> =
        events.iter().filter_map(|e| e.session_id.clone()).collect();
    assert_eq!(sessions.len(), 1, "one native session per turn");
    let text: String = events
        .iter()
        .filter_map(|e| e.payload.pointer("/part/text").and_then(|t| t.as_str()))
        .collect();
    let owner: process::Ownership =
        serde_json::from_slice(&std::fs::read(dir.join("process/owner.json")).unwrap()).unwrap();
    owner.ensure_quiescent().unwrap();
    (sessions.into_iter().next().unwrap(), text)
}

#[tokio::test]
#[ignore = "Requires installed OpenCode CLI with its free hosted models; sends two minimal turns"]
async fn opencode_task_file_turn_and_exact_session_resume() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".agentconductor/verification")
        .join(format!("opencode-native-session-{}", std::process::id()));
    let (id, text) = turn(
        &root,
        "initial",
        None,
        "Do not use any tools. Reply exactly: AGENTCONDUCTOR_OPENCODE_INITIAL",
    )
    .await;
    assert!(text.contains("AGENTCONDUCTOR_OPENCODE_INITIAL"), "{text}");
    let (resumed, text) = turn(
        &root,
        "resume",
        Some(&id),
        "Do not use any tools. What exact token were you asked to reply with in the previous user turn? Reply only that token.",
    )
    .await;
    assert_eq!(resumed, id, "resume must stay in the exact session");
    assert!(text.contains("AGENTCONDUCTOR_OPENCODE_INITIAL"), "{text}");
    eprintln!("OpenCode evidence retained at {}", root.display());
}
