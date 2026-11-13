//! #961: a read-only question uses current state despite stale display context.
#[allow(dead_code)]
#[path = "support/goal_actions.rs"]
mod support;
use serde_json::json;
#[test]
fn completed_goal_chat_survives_bookkeeping_drift_and_uses_current_context() {
    let mut f = support::Fixture::new();
    let worker = f.root.join(if cfg!(windows) {
        "chat-agent.exe"
    } else {
        "chat-agent"
    });
    assert!(
        std::process::Command::new("rustc")
            .arg(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/support/chat_state_agent.rs")
            )
            .arg("-o")
            .arg(&worker)
            .status()
            .unwrap()
            .success()
    );
    let capture = f.root.join("captured-prompt");
    let run = f.report("completed");
    f.edit_saved(|s|{s["last_run"]=json!(run);s["config"]["profiles"]=json!([{"id":"coordinator","executable":worker,"args":[],"parser":"claude","permission_mode":"read_only","environment":{"AC_CHAT_CAPTURE":capture}}]);s["config"]["coordinator"]=json!("coordinator");});
    let old = f.snapshot();
    f.edit_saved(|s| {
        s["chat_result"] = json!("chat-previous.json");
        s["config"]["goal"] = json!("Latest completed goal");
    });
    assert_ne!(f.snapshot()["chat_revision"], old["chat_revision"]);
    let accepted = f
        .call(json!({"kind":"chat","revision":old["chat_revision"],"text":"What finished?"}))
        .expect("read-only chat must not require a manual refresh");
    assert_eq!(accepted["kind"], "chat");
    assert!(
        f.call(json!({"kind":"chat","revision":old["chat_revision"],"text":"duplicate"}))
            .is_err(),
        "one chat at a time"
    );
    let end = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        let s = f.snapshot();
        if !s["chat"].is_null() {
            assert_eq!(s["chat"]["answer"], "fresh answer");
            break;
        }
        assert!(
            std::time::Instant::now() < end,
            "chat did not finish: {}",
            s["chat_job"]
        );
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    let prompt = std::fs::read_to_string(capture).unwrap();
    assert!(prompt.contains("Latest completed goal"));
    assert!(prompt.contains("What finished?"));
    assert!(
        f.call(json!({"kind":"save","revision":old["revision"],"config":old["config"]}))
            .is_err(),
        "write approvals remain revision bound"
    );
}
