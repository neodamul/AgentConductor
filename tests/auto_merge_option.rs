//! #954: updating the merge option writes no unrelated workspace settings.
use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
#[test]
fn merge_option_is_narrow_persistent_and_revision_bound() {
    let root = std::env::temp_dir().join(format!(
        "ac-auto-merge-option-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&root, exe).unwrap();
    let before = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let request = json!({"kind":"save_auto_merge","revision":before["revision"],"enabled":true});
    startup.dispatch(request.to_string().as_bytes()).unwrap();
    assert!(startup.dispatch(request.to_string().as_bytes()).is_err());
    let after = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut expected = before["config"].clone();
    expected["auto_merge"] = json!(true);
    assert_eq!(after["config"], expected);
    let invalid = json!({"kind":"save_auto_merge","revision":after["revision"],"enabled":false,"goal":"unrelated edit"});
    assert!(startup.dispatch(invalid.to_string().as_bytes()).is_err());
    drop(startup);
    let reopened = Startup::open(&root, exe).unwrap();
    let snapshot: Value = reopened.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(snapshot["config"], expected);
    reopened
        .dispatch(
            json!({"kind":"save_auto_merge","revision":snapshot["revision"],"enabled":false})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    assert_eq!(
        reopened.dispatch(br#"{"kind":"snapshot"}"#).unwrap()["config"]["auto_merge"],
        false
    );
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}
