#[test]
fn state_records_fit_the_requested_limit() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for file in ["src/state/records.rs", "src/state/memory.rs"] {
        let lines = std::fs::read_to_string(root.join(file))
            .unwrap()
            .lines()
            .count();
        assert!(lines <= 300, "{file} has {lines} lines; maximum is 300");
    }
}
