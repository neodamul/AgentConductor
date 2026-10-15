use agentconductor::macos_bundle::assemble;
use std::{fs, path::PathBuf, time::SystemTime};

fn fixture() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ac-macos-bundle-{nonce}"));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn creates_finder_bundle_with_copied_executable_icon_and_escaped_version() {
    let root = fixture();
    let binary = root.join("agentconductor");
    let icon = root.join("ac.icns");
    let app = root.join("AgentConductor.app");
    fs::write(&binary, b"native executable").unwrap();
    fs::write(&icon, b"icns").unwrap();

    assemble(&binary, &icon, &app, "1<&2").unwrap();

    let contents = app.join("Contents");
    assert_eq!(
        fs::read(contents.join("MacOS/agentconductor")).unwrap(),
        b"native executable"
    );
    assert_eq!(
        fs::read(contents.join("Resources/ac.icns")).unwrap(),
        b"icns"
    );
    let plist = fs::read_to_string(contents.join("Info.plist")).unwrap();
    assert!(plist.contains("<key>CFBundleExecutable</key><string>agentconductor</string>"));
    assert!(
        plist.contains("<key>CFBundleIdentifier</key><string>dev.agentconductor.desktop</string>")
    );
    assert!(plist.contains("<key>CFBundleShortVersionString</key><string>1&lt;&amp;2</string>"));
    assert!(plist.contains("<key>NSHighResolutionCapable</key><true/>"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(contents.join("MacOS/agentconductor"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn refuses_existing_bundle_and_missing_sources_without_partial_output() {
    let root = fixture();
    let binary = root.join("agentconductor");
    let icon = root.join("missing.icns");
    let app = root.join("AgentConductor.app");
    fs::write(&binary, b"native executable").unwrap();

    assert!(assemble(&binary, &icon, &app, "1.0.0").is_err());
    assert!(!app.exists());

    fs::write(&icon, b"icns").unwrap();
    assemble(&binary, &icon, &app, "1.0.0").unwrap();
    assert!(assemble(&binary, &icon, &app, "1.0.0").is_err());

    fs::remove_dir_all(root).unwrap();
}
