//! #1033: the tool check timeout is a Settings value in preferences.json, not an environment variable.
use agentconductor::{ui::Room, workflow};
use serde_json::json;
#[path = "support/workflow.rs"]
mod support;
#[path = "support/env.rs"]
mod test_env;

#[test]
fn settings_save_and_bound_the_tool_check_timeout() {
    let (root, _) = support::setup("shared");
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    env.set("USERPROFILE", root.join("home"));
    let room = Room::startup(
        &root.join("desktop"),
        std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor")),
    )
    .unwrap();
    let send = |value: serde_json::Value| room.dispatch(value.to_string().as_bytes());
    let loaded = send(json!({"kind":"update_settings"})).unwrap();
    assert_eq!(
        loaded["verifier_timeout"],
        workflow::VERIFIER_TIMEOUT_DEFAULT
    );
    assert_eq!(workflow::VERIFIER_TIMEOUT_DEFAULT, 60);

    let saved =
        send(json!({"kind":"set_update_settings","repository":"","verifier_timeout":90})).unwrap();
    assert_eq!(saved["verifier_timeout"], 90);
    assert_eq!(workflow::verifier_timeout(), 90);

    for bad in [json!(4), json!(601), json!("90")] {
        assert!(
            send(json!({"kind":"set_update_settings","repository":"","verifier_timeout":bad}))
                .is_err(),
            "{bad}"
        );
        assert_eq!(
            workflow::verifier_timeout(),
            90,
            "a rejected value saves nothing"
        );
    }
    // Saving the other settings without the field keeps the timeout.
    send(json!({"kind":"set_update_settings","repository":"","channel":"stable"})).unwrap();
    assert_eq!(workflow::verifier_timeout(), 90);
    drop(room);
    drop(env);
    std::fs::remove_dir_all(root).unwrap();
}
