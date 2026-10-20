//! GitHub #79: macOS and Windows file systems ignore case, so scopes must too.
use agentconductor::plan::overlaps;

#[test]
fn scopes_that_differ_only_by_case_overlap() {
    assert!(overlaps("README.md", "readme.md"));
    assert!(overlaps("Docs/guide.md", "docs"));
    assert!(overlaps("src", "SRC/lib.rs"));
}

#[test]
fn unrelated_scopes_do_not_overlap() {
    assert!(!overlaps("src", "srcs"));
    assert!(!overlaps("docs/a.md", "docs/b.md"));
    assert!(!overlaps("README.md", "README.mdx"));
}

#[test]
fn a_write_is_owned_when_its_path_differs_from_the_scope_only_by_case() {
    use agentconductor::guard::Policy;
    let root = std::env::temp_dir().join(format!("ac-scope-case-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    let policy = Policy::new(&root, &["src".into()]).unwrap();
    let write = |path: &str| {
        policy.classify(
            "Write",
            &serde_json::json!({"file_path": path, "content": "x"}),
        )
    };
    assert!(write("SRC/new.rs").is_ok());
    assert!(write("srcs/new.rs").is_err());
    std::fs::remove_dir_all(root).unwrap();
}
