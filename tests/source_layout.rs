use std::path::{Path, PathBuf};

#[test]
fn source_inventory_detects_non_rust_ui_and_build_code() {
    let root = std::env::temp_dir().join(format!("ac-source-language-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for file in ["src/view.tsx", "tests/legacy.py", "scripts/build.sh"] {
        let path = root.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "fixture").unwrap();
    }
    let mut found = Vec::new();
    non_rust_source_files(&root, &mut found);
    assert_eq!(found.len(), 3);
    std::fs::remove_dir_all(root).unwrap();
}

fn source_files(root: &Path, result: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            source_files(&path, result);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some(
                "rs" | "py"
                    | "pyc"
                    | "js"
                    | "jsx"
                    | "mjs"
                    | "cjs"
                    | "ts"
                    | "tsx"
                    | "css"
                    | "html"
                    | "json"
                    | "sh"
                    | "ps1"
                    | "bat"
                    | "cmd"
            )
        ) {
            result.push(path);
        }
    }
}

fn non_rust_source_files(root: &Path, result: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            non_rust_source_files(&path, result);
        } else if matches!(
            path.extension().and_then(|extension| extension.to_str()),
            Some(
                "py" | "pyc"
                    | "js"
                    | "jsx"
                    | "mjs"
                    | "cjs"
                    | "ts"
                    | "tsx"
                    | "css"
                    | "html"
                    | "sh"
                    | "ps1"
                    | "bat"
                    | "cmd"
            )
        ) {
            result.push(path);
        }
    }
}

#[test]
fn source_and_test_files_are_bounded_and_test_bodies_are_separate() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for directory in ["src", "tests", "scripts", "examples"] {
        let directory = root.join(directory);
        if directory.is_dir() {
            source_files(&directory, &mut files);
        }
    }
    if root.join("build.rs").is_file() {
        files.push(root.join("build.rs"));
    }
    let mut non_rust = Vec::new();
    for directory in ["src", "tests", "scripts", "examples"] {
        let directory = root.join(directory);
        if directory.is_dir() {
            non_rust_source_files(&directory, &mut non_rust);
        }
    }
    assert!(
        non_rust.is_empty(),
        "project-owned UI/build source must be Rust: {}",
        non_rust
            .iter()
            .map(|path| path.strip_prefix(root).unwrap().display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    files.sort();
    let mut violations = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let relative = path.strip_prefix(root).unwrap().display();
        let lines = text.lines().count();
        if lines > 300 {
            violations.push(format!("{relative}: {lines} lines (maximum 300)"));
        }
        if path.starts_with(root.join("src"))
            && (text.contains("#[test]") || text.contains("#[tokio::test"))
        {
            violations.push(format!("{relative}: test body belongs under tests/"));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("\n"));
}
