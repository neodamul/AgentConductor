#[test]
fn rust_ui_module_does_not_embed_legacy_browser_assets() {
    assert!(!std::path::Path::new("src/ui/room.js").exists());
    let ui = std::fs::read_to_string("src/ui.rs").unwrap();
    let setup = std::fs::read_to_string("src/ui/setup.rs").unwrap();
    for (source, contents) in [("src/ui.rs", ui), ("src/ui/setup.rs", setup)] {
        for asset in [
            "room.html\"",
            "room.css\"",
            "room.js\"",
            "startup.html\"",
            "startup.js\"",
        ] {
            assert!(
                !contents.contains(asset),
                "{source} still references {asset}"
            );
        }
    }
}
