#![cfg(unix)]
use agentconductor::process::{ControlReply, Ownership, ProcessSpec, run_controlled_async};
use std::{fs, time::Duration};
#[path = "support/workflow.rs"]
mod support;

#[tokio::test]
async fn stopping_a_callback_also_terminates_its_owned_verification_process() {
    for listen_to_stop in [true, false] {
        let (root, _) = support::setup("shared");
        let cwd = root.join("repo");
        let stop = root.join("stop.request");
        let helper_log = root.join("verifier");
        let helper_ready = cwd.join("grandchild.pid");
        let helper_executable = support::fixture();
        let callback_cwd = cwd.clone();
        let callback_logs = helper_log.clone();
        let callback_stop = stop.clone();
        let parent = ProcessSpec {
            executable: support::fixture(),
            args: vec![
                "guard-control".into(),
                "Write".into(),
                "{}".into(),
                "forbidden".into(),
            ],
            cwd: cwd.clone(),
            input: b"{\"type\":\"ac-initialize\"}\n".to_vec(),
            log_dir: root.join("native"),
            timeout: Duration::from_secs(10),
            signal_grace: Duration::from_millis(50),
            environment: Default::default(),
        };
        let request = stop.clone();
        let ready = helper_ready.clone();
        let sender = tokio::spawn(async move {
            while !ready.exists() {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            fs::write(request, "human stop").unwrap();
        });
        let result = run_controlled_async(
            parent,
            Some(stop),
            Box::new(move |message| {
                let executable = helper_executable.clone();
                let cwd = callback_cwd.clone();
                let log_dir = callback_logs.clone();
                let stop = callback_stop.clone();
                Box::pin(async move {
                    if message["type"] == "control_response" {
                        return Ok(ControlReply {
                            messages: vec![serde_json::json!({"type":"user"})],
                            close_input: false,
                        });
                    }
                    if message["type"] == "system" {
                        return Ok(ControlReply::default());
                    }
                    agentconductor::process::run_until(
                        ProcessSpec {
                            executable,
                            args: vec!["tree".into()],
                            cwd,
                            input: vec![],
                            log_dir,
                            timeout: Duration::from_secs(30),
                            signal_grace: Duration::from_millis(50),
                            environment: Default::default(),
                        },
                        if listen_to_stop { Some(stop) } else { None },
                    )
                    .await?;
                    Err("stopped verifier cannot authorize a tool".into())
                })
            }),
        )
        .await
        .unwrap();
        sender.await.unwrap();
        assert!(result.stopped);
        assert!(!cwd.join("forbidden").exists());
        for logs in [root.join("native"), helper_log] {
            let owner: Ownership =
                serde_json::from_slice(&fs::read(logs.join("owner.json")).unwrap()).unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(2);
            while owner.ensure_quiescent().is_err() && std::time::Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            owner.ensure_quiescent().unwrap();
        }
        let child = fs::read_to_string(helper_ready).unwrap();
        let output = std::process::Command::new("ps")
            .args(["-o", "stat=", "-p", &child])
            .output()
            .unwrap();
        let state = String::from_utf8_lossy(&output.stdout);
        assert!(state.trim().is_empty() || state.trim().starts_with('Z'));
        fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn native_timeout_cancels_a_never_finishing_callback_without_waiting_for_its_deadline() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Dropped(Arc<AtomicBool>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    let (root, _) = support::setup("shared");
    let flag = Arc::new(AtomicBool::new(false));
    let observed = flag.clone();
    let result = run_controlled_async(
        ProcessSpec {
            executable: support::fixture(),
            args: vec![
                "guard-control".into(),
                "Write".into(),
                "{}".into(),
                "forbidden".into(),
            ],
            cwd: root.join("repo"),
            input: b"{\"type\":\"ac-initialize\"}\n".to_vec(),
            log_dir: root.join("native"),
            timeout: Duration::from_secs(1),
            signal_grace: Duration::from_millis(50),
            environment: Default::default(),
        },
        None,
        Box::new(move |message| {
            let flag = flag.clone();
            let guard = if message["type"] == "control_request" {
                Some(Dropped(flag))
            } else {
                None
            };
            Box::pin(async move {
                if message["type"] == "control_response" {
                    return Ok(ControlReply {
                        messages: vec![serde_json::json!({"type":"user"})],
                        close_input: false,
                    });
                }
                if message["type"] == "system" {
                    return Ok(ControlReply::default());
                }
                let _drop = guard;
                std::future::pending::<Result<ControlReply, String>>().await
            })
        }),
    )
    .await
    .unwrap();
    assert!(result.timed_out);
    let deadline = std::time::Instant::now() + Duration::from_secs(1);
    while !observed.load(Ordering::Acquire) && std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        observed.load(Ordering::Acquire),
        "pending callback not cancelled; evidence at {}",
        root.display()
    );
    assert!(!root.join("repo/forbidden").exists());
    fs::remove_dir_all(root).unwrap();
}

#[tokio::test]
async fn normal_native_exit_drains_a_queued_final_frame_without_cancelling_it() {
    let (root, _) = support::setup("shared");
    let result = run_controlled_async(
        ProcessSpec {
            executable: support::fixture(),
            args: vec!["control-final".into()],
            cwd: root.join("repo"),
            input: b"{\"type\":\"initialize\"}\n".to_vec(),
            log_dir: root.join("native"),
            timeout: Duration::from_secs(3),
            signal_grace: Duration::from_millis(50),
            environment: Default::default(),
        },
        None,
        Box::new(|message| {
            Box::pin(async move {
                assert_eq!(message["type"], "result");
                tokio::time::sleep(Duration::from_millis(50)).await;
                Ok(ControlReply {
                    messages: vec![],
                    close_input: true,
                })
            })
        }),
    )
    .await
    .unwrap();
    assert_eq!(result.exit_code, Some(0));
    fs::remove_dir_all(root).unwrap();
}
