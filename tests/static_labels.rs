//! GitHub #190: fixed texts without information are gone from the pages and the catalogs.
use serde_json::Value;

const REMOVED_KEYS: &[&str] = &[
    "startup.local_existing_cli_login",
    "startup.local_by_default",
    "startup.one_application_independent_agent_sessions_your_approval_controls",
    "startup.shared_by_default_worktrees_optional",
    "room.local_cli_authentication",
];

#[test]
fn pages_styles_and_catalogs_carry_none_of_the_removed_labels() {
    let en: Value = serde_json::from_str(include_str!("../src/ui/messages/en.json")).unwrap();
    let ko: Value = serde_json::from_str(include_str!("../src/ui/messages/ko.json")).unwrap();
    let sources = [
        include_str!("../src/ui/startup.html"),
        include_str!("../src/ui/room.html"),
        include_str!("../src/ui/startup.css"),
        include_str!("../src/ui/room.css"),
    ];
    for key in REMOVED_KEYS {
        assert!(en.get(key).is_none() && ko.get(key).is_none(), "{key}");
        assert!(sources.iter().all(|s| !s.contains(key)), "{key}");
    }
    for class in ["local-dot", "local-label", "startup-note"] {
        assert!(sources.iter().all(|s| !s.contains(class)), "{class}");
    }
}
