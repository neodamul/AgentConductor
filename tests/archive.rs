//! DD-64: store-only ZIP archives for slug trees, written and read without dependencies.
use agentconductor::archive;

fn tree(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("ac-archive-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("plan")).unwrap();
    std::fs::create_dir_all(root.join("task/01-core")).unwrap();
    std::fs::create_dir_all(root.join("completed")).unwrap();
    std::fs::write(root.join("plan/plan.json"), "{\"phases\":[]}").unwrap();
    std::fs::write(root.join("task/01-core/a.md"), "# a\n한국어 내용\n").unwrap();
    root
}

#[test]
fn slug_tree_round_trips_through_a_store_only_zip() {
    let source = tree("src");
    let zip = source.with_extension("zip");
    archive::pack(&source, &zip).unwrap();
    let bytes = std::fs::read(&zip).unwrap();
    assert_eq!(&bytes[..4], b"PK\x03\x04", "standard local file header");
    let target = source.with_file_name(format!(
        "{}-out",
        source.file_name().unwrap().to_str().unwrap()
    ));
    let _ = std::fs::remove_dir_all(&target);
    archive::unpack(&zip, &target).unwrap();
    assert_eq!(
        std::fs::read_to_string(target.join("task/01-core/a.md")).unwrap(),
        "# a\n한국어 내용\n"
    );
    assert!(
        target.join("completed").is_dir(),
        "empty stage folders survive"
    );
    assert_eq!(
        archive::pack_bytes(&source).unwrap(),
        archive::pack_bytes(&source).unwrap(),
        "deterministic"
    );
    for p in [source, target] {
        std::fs::remove_dir_all(p).unwrap();
    }
    std::fs::remove_file(zip).unwrap();
}

#[test]
fn unsafe_or_corrupt_archives_are_refused() {
    let source = tree("bad");
    let mut bytes = archive::pack_bytes(&source).unwrap();
    let target = source.with_file_name("ac-archive-bad-target");
    // Corrupt one content byte: the CRC check must fail and nothing is extracted.
    let pos = bytes
        .windows(4)
        .position(|w| w == "# a\n".as_bytes())
        .unwrap();
    bytes[pos] = b'X';
    let zip = source.with_extension("zip");
    std::fs::write(&zip, &bytes).unwrap();
    assert!(archive::unpack(&zip, &target).is_err());
    assert!(!target.exists());
    // Entries escaping the target are refused.
    std::fs::write(&zip, archive::single_entry("../escape.md", b"x")).unwrap();
    assert!(archive::unpack(&zip, &target).is_err());
    std::fs::write(&zip, archive::single_entry("/abs.md", b"x")).unwrap();
    assert!(archive::unpack(&zip, &target).is_err());
    std::fs::remove_dir_all(source).unwrap();
    std::fs::remove_file(zip).unwrap();
}
