use agentconductor::native_guard::session_id;
use std::collections::HashSet;

#[test]
fn session_identifiers_remain_v4_and_unique() {
    let mut seen = HashSet::new();
    for _ in 0..128 {
        let id = session_id().unwrap();
        assert_eq!(id.len(), 36);
        assert_eq!(&id[14..15], "4");
        assert!(matches!(&id[19..20], "8" | "9" | "a" | "b"));
        assert_eq!(id.chars().filter(|c| *c == '-').count(), 4);
        assert!(id.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
        assert!(seen.insert(id));
    }
}

#[test]
fn native_guard_area_obeys_300_line_limit() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native_guard.rs");
    let lines = std::fs::read_to_string(path).unwrap().lines().count();
    assert!(
        lines <= 300,
        "native_guard.rs has {lines} lines; maximum is 300"
    );
}
