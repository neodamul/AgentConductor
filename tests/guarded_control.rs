#![cfg(unix)]
use agentconductor::{
    claude_control::{Controller, ToolDecision},
    gates::{Context, Decision, Ledger, ToolRequest},
    guard::Policy,
    process::{Ownership, ProcessSpec, run_controlled},
};
use serde_json::json;
use std::{fs, time::Duration};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn policy_callback_blocks_unsupported_writes_and_requires_consumed_human_approval_for_commands()
 {
    let (root, _) = support::setup("shared");
    let cwd = root.join("repo");
    fs::create_dir(cwd.join("src")).unwrap();
    let context = Context {
        task: "owned".into(),
        session: "native-component-session".into(),
        workspace: cwd.to_str().unwrap().into(),
        plan_sha256: agentconductor::workflow::digest(b"component test plan"),
        workspace_sha256: agentconductor::workflow::digest(b"component test checkpoint"),
    };
    let db = root.join("state.sqlite");
    for (attempt, tool, input, target, success, executed) in [
        (
            0,
            "Write",
            json!({"file_path":"base.txt","content":"bad"}),
            "base.txt",
            false,
            false,
        ),
        (
            1,
            "Write",
            json!({"file_path":"src/new.rs","content":"new"}),
            "src/new.rs",
            true,
            true,
        ),
        (
            2,
            "Bash",
            json!({"command":"touch marker"}),
            "marker",
            false,
            false,
        ),
        (
            3,
            "Bash",
            json!({"command":"touch marker"}),
            "marker",
            true,
            true,
        ),
        (
            4,
            "Bash",
            json!({"command":"touch marker"}),
            "marker",
            false,
            false,
        ),
    ] {
        let mut ledger = Ledger::open(&db).unwrap();
        if attempt == 3 {
            let receipt = ledger.requests().unwrap().pop().unwrap();
            ledger
                .decide(&receipt.id, &receipt.id, true, &context, 101, 201)
                .unwrap();
        }
        if attempt == 4 {
            fs::write(cwd.join("marker"), "retained").unwrap();
        }
        let bound = context.clone();
        let evaluator = Policy::new(&cwd, &["src".into()])
            .unwrap()
            .evaluator(Box::new(move |tool, input| {
                let request = ToolRequest {
                    tool: tool.into(),
                    input: input.clone(),
                    reason: "Explicit command approval required".into(),
                };
                let id = ledger.request(&bound, &request, 100)?;
                if ledger.consume(&id, &bound, &request, 102)? != Decision::Allowed {
                    return Err(format!("Waiting for exact human approval: {id}"));
                }
                Ok(ToolDecision {
                    allow: true,
                    reason: "Exact human decision consumed".into(),
                })
            }));
        let controller = Controller::new("Implement the owned task", evaluator).unwrap();
        let log_dir = root.join(format!("attempt-{attempt}"));
        let result = run_controlled(
            ProcessSpec {
                executable: support::fixture(),
                args: vec![
                    "guard-control".into(),
                    tool.into(),
                    input.to_string(),
                    target.into(),
                ],
                cwd: cwd.clone(),
                input: controller.initial_input().unwrap(),
                log_dir: log_dir.clone(),
                timeout: Duration::from_secs(5),
                signal_grace: Duration::from_millis(50),
                environment: Default::default(),
            },
            None,
            controller.handler(),
        )
        .await;
        assert_eq!(
            result.is_ok(),
            success,
            "attempt {attempt}: {:?}",
            result.as_ref().err()
        );
        let owner: Ownership =
            serde_json::from_slice(&fs::read(log_dir.join("owner.json")).unwrap()).unwrap();
        assert_eq!(owner.phase, "terminal");
        owner.ensure_quiescent().unwrap();
        assert!(fs::metadata(log_dir.join("stdout.jsonl")).unwrap().len() > 0);
        if executed {
            assert_eq!(fs::read_to_string(cwd.join(target)).unwrap(), "executed");
        }
        if attempt == 0 {
            assert_eq!(fs::read_to_string(cwd.join("base.txt")).unwrap(), "base\n");
        }
        if attempt == 2 {
            assert!(!cwd.join("marker").exists());
        }
        if attempt == 4 {
            assert_eq!(fs::read_to_string(cwd.join("marker")).unwrap(), "retained");
        }
    }
    assert_eq!(
        Ledger::open(&db).unwrap().requests().unwrap()[0].status,
        "consumed"
    );
    fs::remove_dir_all(root).unwrap();
}
