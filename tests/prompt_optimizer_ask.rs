#![cfg(unix)]
//! GitHub #180: `prompt_optimizer::optimize` runs the coordinator read-only and returns its proposal.
use agentconductor::plan::AgentProfile;
use agentconductor::prompt_optimizer::optimize;
use std::os::unix::fs::PermissionsExt;

fn profile(script: &std::path::Path) -> AgentProfile {
    serde_json::from_value(serde_json::json!({
        "id":"coordinator","executable":script,"args":[],"parser":"claude",
        "permission_mode":"read_only"
    }))
    .unwrap()
}

#[tokio::test]
async fn the_coordinator_proposal_is_returned_and_the_skill_prompt_reaches_it() {
    let root = std::env::temp_dir().join(format!("ac-optimize-ask-{}", std::process::id()));
    std::fs::create_dir_all(root.join("project")).unwrap();
    let script = root.join("coordinator.sh");
    let reply = r#"{\"optimized\":\"Speed up checkout; measure with npm run bench\",\"changes\":[\"added a success check\"],\"questions\":[\"Which browser?\"]}"#;
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\ncat > \"$(dirname \"$0\")/prompt.seen\"\necho '{{\"type\":\"result\",\"is_error\":false,\"result\":\"{reply}\"}}'\n"
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let proposal = optimize(
        &profile(&script),
        &root.join("project"),
        &root.join("optimize-run"),
        "make checkout faster",
        "English",
    )
    .await
    .unwrap();
    assert_eq!(
        proposal.optimized,
        "Speed up checkout; measure with npm run bench"
    );
    assert_eq!(proposal.changes, ["added a success check"]);
    assert_eq!(proposal.questions, ["Which browser?"]);
    let seen = std::fs::read_to_string(root.join("prompt.seen")).unwrap();
    assert!(seen.contains("make checkout faster") && seen.contains("Optimize prompts with evals."));
    std::fs::remove_dir_all(root).unwrap();
}
