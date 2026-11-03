use std::{fs, path::Path};

fn source_files(root: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    entries
        .flatten()
        .flat_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                source_files(&path)
            } else {
                vec![path]
            }
        })
        .collect()
}

#[test]
fn application_and_verification_sources_use_rust_only() {
    for directory in ["src", "tests", "scripts"] {
        for path in source_files(Path::new(directory)) {
            assert!(
                !matches!(
                    path.extension().and_then(|value| value.to_str()),
                    Some("py" | "js" | "cjs")
                ),
                "legacy source remains: {}",
                path.display()
            );
        }
    }
    let workflow = fs::read_to_string(".github/workflows/platform-validation.yml").unwrap();
    assert!(!workflow.contains("python -m unittest"));
    assert!(!workflow.contains("node "));
}
