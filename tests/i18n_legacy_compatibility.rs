use std::{fs, path::Path};

#[test]
fn legacy_content_matching_helpers_are_gone() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/i18n.js")).unwrap();
    assert!(!source.contains("acSwap"));
    assert!(!source.contains("acInvert"));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/i18n_ko.js")
            .exists()
    );
}
