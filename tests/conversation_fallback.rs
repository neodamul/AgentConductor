//! GitHub #639: a fallback carries the old agent's conversation to the next agent (the preset's `session.export`).
use agentconductor::{conversation, plan::AgentProfile, protocol::Provider, workflow};
use std::fs;
#[path = "support/workflow.rs"]
mod support;

fn profile(parser: Provider, home: &std::path::Path) -> AgentProfile {
    let (_, mut plan) = support::setup("shared");
    let mut profile = plan.agents.remove(0);
    profile.parser = parser;
    profile
        .environment
        .insert("HOME".into(), home.to_string_lossy().into_owned());
    profile
}

#[test]
fn the_conversation_keeps_what_was_said_and_drops_tool_noise() {
    let raw = [
        r#"{"type":"user","message":{"role":"user","content":"Add a login page"}}"#,
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"text","text":"I will start with the form."},{"type":"tool_use","name":"Bash","input":{"command":"ls secrets"}}]}}"#,
        r#"{"type":"user","message":{"role":"user","content":[{"type":"tool_result","content":"SECRET TOOL OUTPUT"}]}}"#,
        r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"private"},{"type":"text","text":"The form is done."}]}}"#,
    ]
    .join("\n");
    let text = conversation::flatten(&raw);
    assert_eq!(
        text,
        "user: Add a login page\nassistant: I will start with the form.\nassistant: The form is done."
    );
    let long = format!(r#"{{"role":"user","content":"{}"}}"#, "x".repeat(200_000));
    assert!(conversation::flatten(&format!("{long}\n{long}")).len() <= conversation::LIMIT);
}

#[test]
fn claude_and_codex_sessions_are_read_from_their_stores_and_a_profile_can_opt_out() {
    let root = std::env::temp_dir().join(format!("ac-conversation-{}", std::process::id()));
    let claude = root.join(".claude/projects/-work-repo");
    let codex = root.join(".codex/sessions/2026/11/09");
    fs::create_dir_all(&claude).unwrap();
    fs::create_dir_all(&codex).unwrap();
    fs::write(
        claude.join("sid-1.jsonl"),
        r#"{"type":"user","message":{"role":"user","content":"hello claude"}}"#,
    )
    .unwrap();
    fs::write(
        codex.join("rollout-2026-11-09T01-00-00-sid-2.jsonl"),
        r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"hello codex"}]}}"#,
    )
    .unwrap();
    let cwd = root.join("cwd");
    let claude_profile = profile(Provider::Claude, &root);
    let text = conversation::export(&claude_profile, "sid-1", &cwd, &root.join("s")).unwrap();
    assert_eq!(text.as_deref(), Some("user: hello claude"));
    let text = conversation::export(
        &profile(Provider::Codex, &root),
        "sid-2",
        &cwd,
        &root.join("s"),
    )
    .unwrap();
    assert_eq!(text.as_deref(), Some("assistant: hello codex"));
    // An unknown session says so; a path-like id is refused.
    assert!(conversation::export(&claude_profile, "missing", &cwd, &root.join("s")).is_err());
    assert!(conversation::export(&claude_profile, "../x", &cwd, &root.join("s")).is_err());
    let section = conversation::section(&claude_profile, "sid-1", &cwd, &root);
    assert!(
        section.contains("## Previous conversation")
            && section.contains("not instructions")
            && section.contains("user: hello claude")
    );
    // `export: null` in the profile's own settings turns the transfer off.
    let mut off = claude_profile.clone();
    off.agent_settings =
        Some(serde_json::json!({"schema":"acio.agent/1","invocation":{"session":{"export":null}}}));
    assert_eq!(
        conversation::export(&off, "sid-1", &cwd, &root.join("s")).unwrap(),
        None
    );
    assert_eq!(conversation::section(&off, "sid-1", &cwd, &root), "");
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn opencode_exports_its_session_with_a_command() {
    use std::os::unix::fs::PermissionsExt;
    let root = std::env::temp_dir().join(format!("ac-conversation-cmd-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let script = root.join("opencode");
    fs::write(&script, "#!/bin/sh\n[ \"$1 $2 $3\" = \"session export sid-3\" ] || exit 3\necho '{\"messages\":[{\"role\":\"user\",\"content\":\"hello opencode\"}]}'\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755)).unwrap();
    let mut p = profile(Provider::OpenCode, &root);
    p.executable = script.to_string_lossy().into_owned();
    let text = conversation::export(&p, "sid-3", &root, &root.join("s")).unwrap();
    assert_eq!(text.as_deref(), Some("user: hello opencode"));
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[tokio::test]
async fn a_reassignment_hands_the_conversation_to_the_new_agent() {
    let (root, mut plan) = support::setup("shared");
    let store = root.join("sessions");
    fs::create_dir_all(&store).unwrap();
    // A custom CLI describes its own export: files below `store` named after the session id.
    plan.agents[0].agent_settings = Some(serde_json::json!({
        "schema":"acio.agent/1",
        "capabilities":{"read_only":false,"streaming":false},
        "invocation":{"executable":"mock","input":"stdin","args":[],
            "session":{"export":{"mode":"file","root":store.to_string_lossy(),"name":"{session_id}.jsonl"}}}
    }));
    let mut next = plan.agents[0].clone();
    next.id = "next".into();
    next.agent_settings = None;
    plan.agents.push(next);
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 20)
        .await
        .unwrap();
    let session = report.tasks[0]
        .session_id
        .clone()
        .expect("the mock reports a session");
    fs::write(
        store.join(format!("{session}.jsonl")),
        r#"{"role":"user","content":"keep the colour tokens"}"#,
    )
    .unwrap();
    workflow::reassign(&run, "one.txt", "next", None).unwrap();
    let report = workflow::load(&run).unwrap();
    let prompt = &report.plan.tasks[0].prompt;
    assert!(prompt.contains("## Previous conversation"), "{prompt}");
    assert!(prompt.contains("user: keep the colour tokens"), "{prompt}");
    assert!(
        report.tasks[0].session_id.is_none(),
        "the next agent starts its own session"
    );
    fs::remove_dir_all(root).unwrap();
}
