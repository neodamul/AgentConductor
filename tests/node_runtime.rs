//! GitHub #273: every UI behavior test in `tests/*.cjs` runs in CI, not only the ones a Rust test
//! names; `start_feedback_runtime.cjs` had been failing unnoticed because nothing ran it.
#[test]
fn every_node_ui_test_passes() {
    let root = env!("CARGO_MANIFEST_DIR");
    let mut scripts: Vec<_> = std::fs::read_dir(std::path::Path::new(root).join("tests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cjs"))
        .collect();
    scripts.sort();
    assert!(scripts.len() > 20, "the UI tests are found");
    let failed: Vec<String> = scripts
        .iter()
        .filter_map(|script| {
            let result = std::process::Command::new("node")
                .current_dir(root)
                .arg(script)
                .output()
                .expect("Node runs UI behavior tests");
            (!result.status.success()).then(|| {
                format!(
                    "{}:\n{}",
                    script.display(),
                    String::from_utf8_lossy(&result.stderr)
                )
            })
        })
        .collect();
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
