use agentconductor::{adapters, protocol::Provider, ui::setup::Startup};
use serde_json::json;
#[test]
fn explicitly_selected_callback_is_pinned_and_survives_repeated_saves() {
    let root = std::env::temp_dir().join(format!(
        "ac-callback-profile-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let mut profiles = json!([{"id":"claude","parser":"claude","executable":"claude","args":adapters::full_access_args(Provider::Claude).unwrap(),"permission_mode":"native_callback"}]);
    for _ in 0..2 {
        let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
        let result=startup.dispatch(json!({"kind":"save_profiles","revision":state["revision"],"profiles":profiles,"renames":{}}).to_string().as_bytes()).unwrap();
        profiles = result["config"]["profiles"].clone();
        assert_eq!(profiles[0]["permission_mode"], "native_callback");
        assert_eq!(
            profiles[0]["args"],
            json!(
                adapters::invocation(Provider::Claude, None, "unused")
                    .unwrap()
                    .args
            )
        );
    }
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unsupported_callback_arguments_are_rejected_without_silently_approving_tools() {
    let root = std::env::temp_dir().join(format!(
        "ac-callback-invalid-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    std::fs::create_dir(&root).unwrap();
    let startup = Startup::open(
        &root,
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let request = json!({"kind":"save_profiles","revision":state["revision"],"profiles":[{"id":"claude","parser":"claude","executable":"claude","args":["--custom"],"permission_mode":"native_callback"}],"renames":{}});
    assert!(startup.dispatch(request.to_string().as_bytes()).is_err());
    drop(startup);
    std::fs::remove_dir_all(root).unwrap();
}
