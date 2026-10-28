#![cfg(feature = "native-ui-probe")]

use serde_json::Value;

const KEYS: &[&str] = &[
    "startup.local_existing_cli_login",
    "startup.local_by_default",
    "startup.one_application_independent_agent_sessions_your_approval_controls",
    "startup.shared_by_default_worktrees_optional",
    "room.local_cli_authentication",
];

#[test]
fn removed_static_labels_are_absent_from_native_catalogs_and_views() {
    let catalogs = [
        include_str!("../src/ui/messages/en/startup-01.json"),
        include_str!("../src/ui/messages/en/startup-02.json"),
        include_str!("../src/ui/messages/en/room-01.json"),
        include_str!("../src/ui/messages/ko/startup-01.json"),
        include_str!("../src/ui/messages/ko/startup-02.json"),
        include_str!("../src/ui/messages/ko/room-01.json"),
    ];
    let views = concat!(
        include_str!("../src/native_ui_probe/workspace/sidebar.rs"),
        include_str!("../src/native_ui_probe/room/view.rs"),
    );
    for catalog in catalogs {
        let catalog: Value = serde_json::from_str(catalog).unwrap();
        for key in KEYS {
            assert!(catalog.get(key).is_none(), "catalog still defines {key}");
        }
    }
    for key in KEYS {
        assert!(!views.contains(key), "native view still renders {key}");
    }
}
