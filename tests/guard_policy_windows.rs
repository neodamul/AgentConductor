#![cfg(windows)]
//! Windows spellings of workspace paths reach the same file policy as Unix paths.
use agentconductor::guard::{Disposition, Policy};
use serde_json::json;
use std::{fs, path::PathBuf};

struct Workspace(PathBuf);
impl Workspace {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ac-guard-win-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/owned.rs"), "fn main() {}\n").unwrap();
        fs::write(path.join("other.rs"), "fn other() {}\n").unwrap();
        Self(fs::canonicalize(path).unwrap())
    }
    fn policy(&self) -> Policy {
        Policy::new(&self.0, &["src".into()]).unwrap()
    }
    /// The workspace as an agent writes it: `C:\…` without the verbatim prefix.
    fn plain(&self) -> String {
        let text = self.0.to_str().unwrap();
        text.strip_prefix(r"\\?\").unwrap_or(text).to_owned()
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn drive_letter_verbatim_and_mixed_separator_paths_are_workspace_files() {
    let ws = Workspace::new("spellings");
    let policy = ws.policy();
    let plain = ws.plain();
    for path in [
        format!(r"{plain}\other.rs"),
        format!("{}/other.rs", plain.replace('\\', "/")),
        format!(r"{}\other.rs", plain.to_uppercase()),
        format!(r"\\?\{plain}\other.rs"),
    ] {
        assert_eq!(
            policy.classify("Read", &json!({ "file_path": path })),
            Ok(Disposition::Automatic),
            "{path}"
        );
    }
    assert_eq!(
        policy.classify(
            "Write",
            &json!({"file_path": format!(r"{plain}\src\new.rs"), "content": "x"})
        ),
        Ok(Disposition::Automatic)
    );
    assert_eq!(
        policy.classify(
            "Write",
            &json!({"file_path": r"src\relative.rs", "content": "x"})
        ),
        Ok(Disposition::Automatic)
    );
}

#[test]
fn windows_paths_keep_scope_traversal_and_workspace_limits() {
    let ws = Workspace::new("limits");
    let policy = ws.policy();
    let plain = ws.plain();
    assert!(
        policy
            .classify(
                "Write",
                &json!({"file_path": format!(r"{plain}\other.rs"), "content": "x"})
            )
            .unwrap_err()
            .contains("owned scopes")
    );
    assert!(
        policy
            .classify(
                "Read",
                &json!({"file_path": format!(r"{plain}\src\..\other.rs")})
            )
            .is_err()
    );
    assert!(
        policy
            .classify("Read", &json!({"file_path": r"C:\Windows\win.ini"}))
            .unwrap_err()
            .contains("outside the workspace")
    );
    assert!(
        policy
            .classify(
                "Read",
                &json!({"file_path": format!(r"{plain}-other\x.rs")})
            )
            .unwrap_err()
            .contains("outside the workspace")
    );
    assert!(
        policy
            .classify(
                "Read",
                &json!({"file_path": format!(r"{plain}\.git\config")})
            )
            .is_err()
    );
}
