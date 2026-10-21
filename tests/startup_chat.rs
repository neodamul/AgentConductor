use agentconductor::ui::setup::Startup;
use serde_json::json;

#[test]
fn large_chat_request_uses_the_existing_startup_job_gate() {
    let root = std::env::temp_dir().join(format!(
        "ac-chat-startup-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(&root, &std::env::current_exe().unwrap()).unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let turns = (0..9)
        .map(|index| {
            json!({"role":if index == 8 {"operator"} else {"assistant"},"text":"x".repeat(8192)})
        })
        .collect::<Vec<_>>();
    let request = serde_json::to_vec(&json!({
        "kind":"chat", "revision":snapshot["revision"], "turns":turns
    }))
    .unwrap();
    assert!(request.len() > 64 * 1024 && request.len() < 128 * 1024);
    let error = startup.dispatch(&request).unwrap_err();
    assert!(
        error.contains("코디네이터") || error.contains("coordinator"),
        "{error}"
    );
    assert!(startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["job"].is_null());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
