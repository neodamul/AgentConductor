#![cfg(unix)]

use agentconductor::guard::{Disposition, Policy};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Workspace(PathBuf);

impl Workspace {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ac-guard-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/owned.rs"), "fn main() {}\n").unwrap();
        fs::write(path.join("other.rs"), "fn other() {}\n").unwrap();
        Self(path)
    }

    fn policy(&self) -> Policy {
        Policy::new(&self.0, &["src".into()]).unwrap()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn reading_a_missing_workspace_file_is_denied_without_aborting_the_worker() {
    let ws = Workspace::new();
    let policy = ws.policy();
    for path in ["src/missing.rs", "src/absent/nested.rs"] {
        assert!(matches!(
            policy
                .classify("Read", &json!({"file_path":ws.0.join(path)}))
                .unwrap(),
            Disposition::Deny(_)
        ));
    }
    assert!(matches!(
        policy
            .classify(
                "Edit",
                &json!({"file_path":"src/missing.rs","old_string":"a","new_string":"b"})
            )
            .unwrap(),
        Disposition::Deny(_)
    ));
    assert!(
        policy
            .classify("Read", &json!({"file_path":"/etc/hosts"}))
            .is_err()
    );
}
