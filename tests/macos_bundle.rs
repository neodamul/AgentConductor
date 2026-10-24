use agentconductor::macos_bundle::assemble;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

fn fixture() -> PathBuf {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    loop {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let root =
            std::env::temp_dir().join(format!("ac-macos-bundle-{}-{id}", std::process::id()));
        match fs::create_dir(&root) {
            Ok(()) => return root,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create macOS bundle fixture: {error}"),
        }
    }
}

#[test]
fn parallel_fixtures_have_distinct_directories() {
    let first = fixture();
    let second = fixture();
    assert_ne!(first, second);
    fs::remove_dir_all(first).unwrap();
    fs::remove_dir_all(second).unwrap();
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
    assert!(plist.contains("<key>CFBundleDevelopmentRegion</key><string>en</string>"));
    assert!(plist.contains(
        "<key>CFBundleLocalizations</key><array><string>en</string><string>ko</string></array>"
    ));
    for language in ["en", "ko"] {
        let strings = fs::read_to_string(
            contents.join(format!("Resources/{language}.lproj/InfoPlist.strings")),
        )
        .unwrap();
        assert!(strings.contains("\"CFBundleDisplayName\" = \"AgentConductor\";"));
    }
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
