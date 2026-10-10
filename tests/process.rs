#![cfg(unix)]
use agentconductor::process::{ProcessSpec, run};
use std::{path::PathBuf, sync::OnceLock, time::Duration};
#[path = "support/liveness.rs"]
mod liveness;

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

fn fixture() -> PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE
        .get_or_init(|| {
            let path = std::env::temp_dir().join(format!("ac-mock-{}", std::process::id()));
            let status = std::process::Command::new("rustc")
                .args(["tests/support/mock_agent.rs", "-o"])
                .arg(&path)
                .status()
                .unwrap();
            assert!(status.success());
            path
        })
        .clone()
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

fn spec(mode: &str) -> ProcessSpec {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let cwd =
        std::env::temp_dir().join(format!("ac-run-{}-{stamp}-{sequence}", std::process::id()));
    std::fs::create_dir(&cwd).unwrap();
    ProcessSpec {
        executable: fixture(),
        args: vec![mode.into()],
        cwd: cwd.clone(),
        input: "{\"text\":\"hello 🦀\"}\n".as_bytes().to_vec(),
        log_dir: cwd.join("logs"),
        timeout: Duration::from_secs(5),
        signal_grace: Duration::from_millis(50),
        environment: Default::default(),
    }
}

#[tokio::test]
async fn closes_stdin_and_drains_large_stderr_preserving_raw_bytes() {
    let mut s = spec("echo");
    s.input = s.input.repeat(100_000);
    let cwd = s.cwd.clone();
    let input = s.input.clone();
    let result = run(s).await.unwrap();
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.timed_out);
    assert_eq!(std::fs::read(result.stdout).unwrap(), input);
    assert_eq!(
        std::fs::metadata(result.stderr).unwrap().len(),
        2 * 1024 * 1024
    );
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn records_nonzero_exit_without_retrying() {
    let s = spec("fail");
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert_eq!(result.exit_code, Some(7));
    assert!(!result.timed_out);
    let owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
    assert_eq!(owner["phase"], "terminal");
    assert_eq!(owner["exit_code"], 7);
    assert!(owner["pid"].as_u64().unwrap() > 0);
    assert_eq!(owner["pid"], owner["process_group"]);
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn timeout_terminates_descendants_and_reaps_direct_child() {
    let mut s = spec("tree");
    s.timeout = Duration::from_millis(500);
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert!(result.timed_out);
    let pid: i32 = std::fs::read_to_string(cwd.join("grandchild.pid"))
        .unwrap()
        .parse()
        .unwrap();
    liveness::assert_absent(pid).await;
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn reports_spawn_failure() {
    let mut s = spec("echo");
    let cwd = s.cwd.clone();
    s.executable = cwd.join("missing");
    assert!(run(s).await.is_err());
    let owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
    assert_eq!(owner["phase"], "not_spawned");
    assert!(owner["pid"].is_null());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn timeout_is_effective_when_agent_never_reads_stdin() {
    let mut s = spec("sleep");
    s.input = vec![b'x'; 2 * 1024 * 1024];
    s.timeout = Duration::from_millis(200);
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert!(result.timed_out);
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn escalates_to_kill_when_interrupt_and_terminate_are_ignored() {
    let mut s = spec("stubborn");
    s.timeout = Duration::from_millis(500);
    let cwd = s.cwd.clone();
    let result = run(s).await.unwrap();
    assert!(result.timed_out);
    assert_eq!(result.exit_code, None);
    assert!(cwd.join("ready").exists());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn explicit_stop_terminates_process_tree_and_is_distinct_from_timeout() {
    let s = spec("tree");
    let cwd = s.cwd.clone();
    let marker = cwd.join("stop.request");
    let request = marker.clone();
    let ready = cwd.join("grandchild.pid");
    let sender = tokio::spawn(async move {
        while !ready.exists() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        std::fs::write(request, "human stop").unwrap();
    });
    let result = agentconductor::process::run_until(s, Some(marker))
        .await
        .unwrap();
    sender.await.unwrap();
    assert!(result.stopped);
    assert!(!result.timed_out);
    let pid = std::fs::read_to_string(cwd.join("grandchild.pid")).unwrap();
    liveness::assert_absent(pid.trim().parse().unwrap()).await;
    assert!(result.stdout.exists() && result.stderr.exists());
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn stop_requested_before_launch_does_not_spawn_agent() {
    let s = spec("stubborn");
    let cwd = s.cwd.clone();
    let marker = cwd.join("stop.request");
    std::fs::write(&marker, "stop").unwrap();
    let result = agentconductor::process::run_until(s, Some(marker))
        .await
        .unwrap();
    assert!(result.stopped);
    assert!(!cwd.join("ready").exists());
    let owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
    assert_eq!(owner["phase"], "not_spawned");
    std::fs::remove_dir_all(cwd).unwrap();
}

#[tokio::test]
async fn signal_termination_is_distinct_from_a_numeric_exit_and_is_persisted() {
    for (command, exit, signal) in [
        ("kill -TERM $$", None, Some(libc::SIGTERM)),
        ("exit 143", Some(143), None),
    ] {
        let mut s = spec("unused");
        s.executable = "/bin/sh".into();
        s.args = vec!["-c".into(), command.into()];
        s.input.clear();
        let cwd = s.cwd.clone();
        let result = run(s).await.unwrap();
        assert_eq!(result.exit_code, exit);
        assert_eq!(result.termination_signal, signal);
        assert!(!result.timed_out && !result.stopped);
        let owner: agentconductor::process::Ownership =
            serde_json::from_slice(&std::fs::read(cwd.join("logs/owner.json")).unwrap()).unwrap();
        assert_eq!(owner.exit_code, exit);
        assert_eq!(owner.termination_signal, signal);
        assert_eq!(owner.phase, "terminal");
        owner.ensure_quiescent().unwrap();
        std::fs::remove_dir_all(cwd).unwrap();
    }
}
