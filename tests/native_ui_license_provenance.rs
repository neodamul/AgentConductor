use sha2::{Digest, Sha256};

#[test]
fn windows_clipboard_boost_license_is_pinned_to_the_crate_source_revision() {
    let sources: serde_json::Value =
        serde_json::from_str(include_str!("../licenses/upstream/sources.json")).unwrap();
    let record = sources
        .as_array()
        .unwrap()
        .iter()
        .find(|record| record["name"] == "clipboard-win" && record["version"] == "5.4.1")
        .unwrap();
    assert_eq!(record["license"], "BSL-1.0");
    let revision = record["vcs_commit"].as_str().unwrap();
    assert_eq!(revision, "3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342");
    let relative = record["file"].as_str().unwrap();
    assert!(relative.starts_with(&format!("{revision}/")));
    let bytes = std::fs::read(format!("licenses/upstream/{relative}")).unwrap();
    assert_eq!(format!("{:x}", Sha256::digest(&bytes)), record["sha256"]);
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .starts_with("Boost Software License - Version 1.0")
    );
}
