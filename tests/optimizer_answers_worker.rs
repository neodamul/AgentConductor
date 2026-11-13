use agentconductor::{mcp, native_guard};
use serde_json::{Value, json};
use std::{fs, process::Command};
#[test]
fn private_answer_input_reaches_the_real_optimizer_worker_without_applying_a_goal() {
    let root = std::env::temp_dir().join(format!(
        "ac-optimizer-worker-{}",
        native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(root.join("project")).unwrap();
    fs::create_dir(root.join("mcp")).unwrap();
    let root = root.canonicalize().unwrap();
    let exe = root.join(if cfg!(windows) {
        "optimizer.exe"
    } else {
        "optimizer"
    });
    assert!(
        Command::new("rustc")
            .args(["tests/support/optimizer_agent.rs", "-o"])
            .arg(&exe)
            .status()
            .unwrap()
            .success()
    );
    let startup = json!({"config":{"goal":"Original goal","repository":root.join("project"),"coordinator":"local","profiles":[{"id":"local","executable":exe,"args":[],"parser":"custom","permission_mode":"read_only","environment":{"AC_OPTIMIZER_SEEN":root.join("seen.txt")}}]}});
    fs::write(root.join("startup.json"), startup.to_string()).unwrap();
    for (id, goal, answers, expected) in [
        ("first", "Original goal", "", "Proposed goal"),
        (
            "second",
            "Proposed goal",
            "Question: Which environment? Answer: Windows acceptance",
            "Goal for Windows",
        ),
    ] {
        fs::write(
            root.join(format!("mcp/optimize-input-{id}.json")),
            json!({"goal":goal,"language":"English","answers":answers}).to_string(),
        )
        .unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
            .args(["optimize"])
            .arg(&root)
            .arg(id)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: Value = serde_json::from_slice(
            &mcp::load(&root.join(format!("mcp/optimize-{id}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(result["optimized"], expected);
        let seen = fs::read_to_string(root.join("seen.txt")).unwrap();
        assert!(seen.contains(goal));
        if !answers.is_empty() {
            assert!(seen.contains(answers));
            assert_eq!(result["questions"], json!([]));
        }
    }
    assert_eq!(
        fs::read_to_string(root.join("startup.json")).unwrap(),
        startup.to_string()
    );
    fs::remove_dir_all(root).unwrap();
}
