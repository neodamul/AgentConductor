#![cfg(unix)]
use agentconductor::{
    process::{self, ProcessSpec},
    protocol::{Decoder, EventKind, Provider},
};
use std::{path::Path, time::Duration};

async fn turn(
    root: &Path,
    name: &str,
    session: Option<&str>,
    prompt: &str,
    expected_text: &str,
) -> String {
    let mut args = vec![
        "-u".into(),
        "OPENAI_API_KEY".into(),
        "-u".into(),
        "CODEX_API_KEY".into(),
        "codex".into(),
        "exec".into(),
        "--sandbox".into(),
        "read-only".into(),
    ];
    if let Some(id) = session {
        args.extend(["resume".into(), id.into()]);
    }
    args.extend([
        "-".into(),
        "--json".into(),
        "--skip-git-repo-check".into(),
        "--ignore-user-config".into(),
        "--ignore-rules".into(),
    ]);
    let output = process::run(ProcessSpec {
        executable: "/usr/bin/env".into(),
        args,
        cwd: root.into(),
        input: prompt.as_bytes().to_vec(),
        log_dir: root.join(name),
        timeout: Duration::from_secs(45),
        signal_grace: Duration::from_secs(1),
        environment: Default::default(),
    })
    .await
    .unwrap();
    assert_eq!(
        output.exit_code,
        Some(0),
        "Native turn failed; inspect retained evidence at {}",
        root.join(name).display()
    );
    assert!(!output.timed_out);
    let mut decoder = Decoder::new(Provider::Codex, 1024 * 1024);
    let mut events = decoder
        .push(&std::fs::read(output.stdout).unwrap())
        .unwrap();
    events.extend(decoder.finish().unwrap());
    assert!(events.iter().any(|e| e.kind == EventKind::Result));
    assert!(
        events.iter().any(|e| e
            .payload
            .pointer("/item/text")
            .and_then(serde_json::Value::as_str)
            == Some(expected_text)),
        "native assistant did not produce the expected response"
    );
    assert!(
        !events
            .iter()
            .any(|e| matches!(e.kind, EventKind::Error | EventKind::Tool))
    );
    let id = events
        .iter()
        .find_map(|e| e.session_id.clone())
        .expect("native session ID");
    if let Some(expected) = session {
        assert_eq!(id, expected);
    }
    let owner: process::Ownership =
        serde_json::from_slice(&std::fs::read(root.join(name).join("owner.json")).unwrap())
            .unwrap();
    owner.ensure_quiescent().unwrap();
    id
}

#[tokio::test]
#[ignore = "Requires installed Codex CLI and existing ChatGPT login; sends two minimal generation turns"]
async fn subscribed_codex_stdin_jsonl_and_exact_session_resume_use_existing_authentication() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".agentconductor/verification")
        .join(format!("codex-native-session-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let id = turn(
        &root,
        "initial",
        None,
        "Do not use any tools. Reply exactly: AGENTCONDUCTOR_NATIVE_INITIAL",
        "AGENTCONDUCTOR_NATIVE_INITIAL",
    )
    .await;
    turn(
        &root,
        "resume",
        Some(&id),
        "Do not use any tools. What exact token were you asked to reply with in the previous user turn? Reply only that token.",
        "AGENTCONDUCTOR_NATIVE_INITIAL",
    )
    .await;
    eprintln!(
        "Native Codex subscription/session evidence retained at {}",
        root.display()
    );
}

#[tokio::test]
#[ignore = "Requires existing Codex ChatGPT login; sends four minimal turns across two concurrent sessions"]
async fn concurrent_native_sessions_preserve_independent_conversation_contexts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".agentconductor/verification")
        .join(format!("codex-native-concurrent-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let (one, two) = tokio::join!(
        turn(
            &root,
            "one-initial",
            None,
            "Do not use any tools. Reply exactly: CONDUCTOR_SESSION_ONE",
            "CONDUCTOR_SESSION_ONE"
        ),
        turn(
            &root,
            "two-initial",
            None,
            "Do not use any tools. Reply exactly: CONDUCTOR_SESSION_TWO",
            "CONDUCTOR_SESSION_TWO"
        ),
    );
    assert_ne!(one, two);
    let prompt = "Do not use any tools. What exact token were you asked to reply with in the previous user turn? Reply only that token.";
    tokio::join!(
        turn(
            &root,
            "one-resume",
            Some(&one),
            prompt,
            "CONDUCTOR_SESSION_ONE"
        ),
        turn(
            &root,
            "two-resume",
            Some(&two),
            prompt,
            "CONDUCTOR_SESSION_TWO"
        ),
    );
    eprintln!(
        "Native concurrent Codex evidence retained at {}",
        root.display()
    );
}
