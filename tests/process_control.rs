#![cfg(unix)]
#[path = "support/process.rs"]
mod process;
use agentconductor::process::ProcessSpec;
use process::spec;

#[tokio::test]
async fn bidirectional_permission_denial_preserves_stdin_until_idle_and_raw_large_logs() {
    let mut s = spec("control");
    s.input = b"{\"type\":\"initialize\"}\n".to_vec();
    let cwd = s.cwd.clone();
    let result = agentconductor::process::run_controlled(s, None, Box::new(|message| {
        use agentconductor::process::ControlReply;
        Ok(match message["type"].as_str() {
            Some("control_response") => ControlReply{messages:vec![serde_json::json!({"type":"user","message":"work"})],close_input:false},
            Some("control_request") => ControlReply{messages:vec![serde_json::json!({"type":"control_response","request_id":message["request_id"],"behavior":"deny"})],close_input:false},
            Some("system") => ControlReply{messages:vec![],close_input:true},
            _ => ControlReply::default(),
        })
    })).await.unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(!cwd.join("forbidden").exists());
    assert_eq!(
        std::fs::metadata(result.stderr).unwrap().len(),
        2 * 1024 * 1024
    );
    assert!(
        std::fs::read_to_string(result.stdout)
            .unwrap()
            .contains("late-permission")
    );
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn malformed_or_failed_control_terminates_before_a_tool_can_run_and_records_terminal_owner() {
    for mode in ["control-malformed", "control"] {
        let mut s = spec(mode);
        s.input = b"{\"type\":\"initialize\"}\n".to_vec();
        let cwd = s.cwd.clone();
        let error = agentconductor::process::run_controlled(
            s,
            None,
            Box::new(|message| {
                if message["type"] == "control_response" {
                    Ok(agentconductor::process::ControlReply {
                        messages: vec![serde_json::json!({"type":"user"})],
                        close_input: false,
                    })
                } else {
                    Err("permission evaluator failed".into())
                }
            }),
        )
        .await
        .err()
        .unwrap();
        assert!(error.contains("control"), "{error}");
        assert!(!cwd.join("forbidden").exists());
        let owner: agentconductor::process::Ownership =
            serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
        assert_eq!(owner.phase, "terminal");
        owner.ensure_quiescent().unwrap();
        assert!(
            std::fs::metadata(cwd.join("logs/stdout.jsonl"))
                .unwrap()
                .len()
                > 0
        );
        std::fs::remove_dir_all(cwd).unwrap();
    }
}

#[tokio::test]
async fn human_tool_approval_is_required_before_a_controlled_child_can_create_a_file() {
    use agentconductor::{
        gates::{Context, Decision, Ledger, ToolRequest},
        process::ControlReply,
    };
    let s = spec("control");
    let cwd = s.cwd.clone();
    let context = Context {
        epoch: 0,
        task: "one".into(),
        session: "native-one".into(),
        workspace: cwd.to_string_lossy().into(),
        plan_sha256: "plan".into(),
        workspace_sha256: "workspace".into(),
    };
    let request = ToolRequest {
        tool: "Bash".into(),
        input: serde_json::json!({"command":"touch forbidden"}),
        reason: "Command requires human approval".into(),
    };
    for attempt in 0..3 {
        let s = ProcessSpec {
            executable: s.executable.clone(),
            args: s.args.clone(),
            cwd: cwd.clone(),
            input: b"{\"type\":\"initialize\"}\n".to_vec(),
            log_dir: cwd.join(format!("attempt-{attempt}")),
            timeout: s.timeout,
            signal_grace: s.signal_grace,
            environment: Default::default(),
        };
        let mut ledger = Ledger::open(&cwd.join("state.sqlite")).unwrap();
        if attempt == 1 {
            let receipt = ledger.requests().unwrap().pop().unwrap();
            ledger
                .decide(&receipt.id, &receipt.id, true, &context, 101, 201)
                .unwrap();
        }
        let context = context.clone();
        let request = request.clone();
        let result=agentconductor::process::run_controlled(s,None,Box::new(move|message| {
            Ok(match message["type"].as_str() {
                Some("control_response")=>ControlReply{messages:vec![serde_json::json!({"type":"user"})],close_input:false},
                Some("control_request")=>{
                    assert_eq!(message["request"]["tool_name"],request.tool);
                    assert_eq!(message["request"]["input"],request.input);
                    let id=ledger.request(&context,&request,100)?;
                    let allowed=ledger.consume(&id,&context,&request,102)?==Decision::Allowed;
                    ControlReply{messages:vec![serde_json::json!({"type":"control_response","request_id":message["request_id"],"behavior":if allowed {"allow"}else{"deny"}})],close_input:false}
                }
                Some("system")=>ControlReply{messages:vec![],close_input:true},
                _=>ControlReply::default(),
            })
        })).await.unwrap();
        assert_eq!(result.exit_code, Some(0));
        if attempt == 0 {
            assert!(!cwd.join("forbidden").exists());
        } else if attempt == 1 {
            assert_eq!(
                std::fs::read_to_string(cwd.join("forbidden")).unwrap(),
                "command ran"
            );
            std::fs::write(cwd.join("forbidden"), "must not run again").unwrap();
        } else {
            assert_eq!(
                std::fs::read_to_string(cwd.join("forbidden")).unwrap(),
                "must not run again"
            );
        }
    }
    std::fs::remove_dir_all(cwd).unwrap();
}
