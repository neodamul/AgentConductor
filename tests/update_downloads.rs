//! GitHub #80: downloaded releases are removed once an update is installed.
use agentconductor::update::clear_downloads;

#[test]
fn every_downloaded_release_folder_is_removed_but_the_folder_itself_stays() {
    let root = std::env::temp_dir().join(format!("ac-updates-{}", std::process::id()));
    for tag in ["v0.1.18", "v0.1.24"] {
        let unpacked = root.join(tag).join("unpacked/pkg");
        std::fs::create_dir_all(&unpacked).unwrap();
        std::fs::write(root.join(tag).join("agentconductor.zip"), b"zip").unwrap();
    }
    clear_downloads(&root);
    assert!(root.is_dir());
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 0);
    clear_downloads(&root.join("missing"));
    std::fs::remove_dir_all(root).unwrap();
}
