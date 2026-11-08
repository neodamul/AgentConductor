//! #420: evidence references must exist even when the test lives in a nested directory.
use std::path::{Component, Path};
fn missing(root: &Path, record: &str) -> Vec<String> {
    record
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|name| [".rs", ".cjs", ".py"].iter().any(|ext| name.ends_with(ext)))
        .filter(|name| !name.chars().any(char::is_whitespace) && !name.contains(['*', '?']))
        .filter(|name| {
            !["src/", "scripts/", "examples/", "knowledge/"]
                .iter()
                .any(|prefix| name.starts_with(prefix))
        })
        .filter(|name| {
            let path = Path::new(name.strip_prefix("tests/").unwrap_or(name));
            path.components()
                .any(|part| !matches!(part, Component::Normal(_)))
                || !root.join("tests").join(path).is_file()
        })
        .map(String::from)
        .collect()
}
#[test]
fn nested_missing_and_traversing_evidence_is_rejected() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        missing(
            root,
            "`tests/choice_controls/inspector_model.cjs` `architecture_trace.rs`"
        )
        .is_empty()
    );
    assert_eq!(
        missing(root, "`tests/choice_controls/does_not_exist.cjs`"),
        ["tests/choice_controls/does_not_exist.cjs"]
    );
    assert_eq!(
        missing(root, "`tests/../src/main.rs`"),
        ["tests/../src/main.rs"]
    );
    assert!(missing(root, "`cargo test --test x.rs` `tests/*.rs` `src/main.rs`").is_empty());
}
#[test]
fn all_concrete_verification_test_references_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let record =
        std::fs::read_to_string(root.join("docs/14-architecture-verification.md")).unwrap();
    let absent = missing(root, &record);
    assert!(absent.is_empty(), "missing test evidence: {absent:?}");
}
