#![cfg(unix)]
use agentconductor::{
    claude_control::{Controller, ToolDecision},
    process::{ControlReply, ProcessSpec, run_controlled},
};
use std::{path::PathBuf, time::Duration};

#[tokio::test]
#[ignore = "Requires installed Claude CLI; metadata only, no generation prompt is sent"]
async fn installed_claude_accepts_control_initialization_with_pretool_callback_registration() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".agentconductor/verification")
        .join(format!("claude-control-metadata-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let controller = Controller::new(
        "This prompt must not be sent",
        Box::new(|_, _| Ok(ToolDecision::deny("Metadata-only verification"))),
    )
    .unwrap();
    let result = run_controlled(
        ProcessSpec {
            executable: "claude".into(),
            args: [
                "-p",
                "--input-format",
                "stream-json",
                "--output-format",
                "stream-json",
                "--verbose",
                "--restricted",
                "--tools",
                "",
                "--strict-mcp-config",
                "--mcp-config",
                "{\"mcpServers\":{}}",
            ]
            .map(String::from)
            .to_vec(),
            cwd: root.clone(),
            input: controller.initial_input().unwrap(),
            log_dir: root.join("process"),
            timeout: Duration::from_secs(30),
            signal_grace: Duration::from_secs(2),
            environment: Default::default(),
        },
        None,
        Box::new(|message| {
            if message["type"] == "control_response" {
                assert_eq!(message["response"]["request_id"], "ac-initialize");
                assert_eq!(message["response"]["subtype"], "success");
                assert!(
                    message["response"]["response"]["models"]
                        .as_array()
                        .is_some_and(|models| !models.is_empty())
                );
                Ok(ControlReply {
                    messages: vec![],
                    close_input: true,
                })
            } else {
                Ok(ControlReply::default())
            }
        }),
    )
    .await
    .unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.timed_out);
    let raw = std::fs::read_to_string(result.stdout).unwrap();
    assert!(raw.contains("ac-initialize"));
    assert!(!raw.contains("This prompt must not be sent"));
    let owner: agentconductor::process::Ownership =
        serde_json::from_slice(&std::fs::read(root.join("process/owner.json")).unwrap()).unwrap();
    owner.ensure_quiescent().unwrap();
    eprintln!("Native metadata evidence retained at {}", root.display());
}
