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
fn ordinary_reads_and_owned_edits_are_allowed_but_cross_task_writes_are_refused() {
    let ws = Workspace::new();
    let policy = ws.policy();
    assert_eq!(
        policy
            .classify("Read", &json!({"file_path":ws.0.join("other.rs")}))
            .unwrap(),
        Disposition::Automatic
    );
    assert_eq!(policy.classify("Edit", &json!({"file_path":"src/owned.rs","old_string":"fn main()","new_string":"fn edited()"})).unwrap(), Disposition::Automatic);
    assert_eq!(
        policy
            .classify("Write", &json!({"file_path":"src/new.rs","content":"new"}))
            .unwrap(),
        Disposition::Automatic
    );
    assert!(
        policy
            .classify("Write", &json!({"file_path":"other.rs","content":"escape"}))
            .is_err()
    );
    assert!(
        policy
            .classify(
                "Write",
                &json!({"file_path":"src-other/new.rs","content":"escape"})
            )
            .is_err()
    );
    assert_eq!(
        fs::read_to_string(ws.0.join("other.rs")).unwrap(),
        "fn other() {}\n"
    );
    assert!(!ws.0.join("src/new.rs").exists());
}

#[test]
fn commands_searches_and_sensitive_paths_always_require_human_decisions() {
    let ws = Workspace::new();
    let policy = ws.policy();
    for command in [
        "cargo test",
        "git merge feature",
        "curl https://example.test",
        "cat .env",
        "sudo deploy",
    ] {
        assert_eq!(
            policy
                .classify("Bash", &json!({"command":command}))
                .unwrap(),
            Disposition::Human
        );
    }
    assert_eq!(
        policy
            .classify("Grep", &json!({"pattern":"TOKEN"}))
            .unwrap(),
        Disposition::Human
    );
    assert_eq!(
        policy.classify("Glob", &json!({"pattern":"**/*"})).unwrap(),
        Disposition::Human
    );
    for file in [
        "src/.env",
        "src/.env.production",
        "src/private.key",
        "src/cert.pem",
        "src/credentials.json",
        "src/secrets/config.toml",
    ] {
        if let Some(parent) = ws.0.join(file).parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(ws.0.join(file), "not a real secret").unwrap();
        assert_eq!(
            policy.classify("Read", &json!({"file_path":file})).unwrap(),
            Disposition::Human,
            "{file}"
        );
        assert_eq!(
            policy
                .classify("Write", &json!({"file_path":file,"content":"replacement"}))
                .unwrap(),
            Disposition::Human,
            "{file}"
        );
    }
}

#[test]
fn unsafe_and_unsupported_requests_do_not_become_approvable() {
    let ws = Workspace::new();
    let policy = ws.policy();
    for file in [
        "../outside",
        "src/../other.rs",
        "src\\owned.rs",
        "src//owned.rs",
        ".git/config",
        "src/.claude/settings.json",
        "src/.agents/config.json",
        "src/.agentconductor/state.sqlite",
        "src/.CoDeX/config.toml",
    ] {
        assert!(
            policy
                .classify("Write", &json!({"file_path":file,"content":"bad"}))
                .is_err(),
            "{file}"
        );
    }
    for (tool, input) in [
        ("Task", json!({"prompt":"delegate"})),
        ("WebFetch", json!({"url":"https://example.test"})),
        ("Bash", json!({"command":"deploy","run_in_background":true})),
        (
            "Read",
            json!({"file_path":"src/owned.rs","unknown_override":true}),
        ),
        ("Write", json!({"file_path":"src/new.rs"})),
        ("Bash", json!({"command":""})),
        ("Read", json!({"file_path":"src/owned.rs","offset":-1})),
    ] {
        assert!(policy.classify(tool, &input).is_err(), "{tool}: {input}");
    }
    assert!(
        policy
            .classify(
                "Write",
                &json!({"file_path":"src/new.rs","content":"x".repeat(65*1024)})
            )
            .is_err()
    );
    assert!(Policy::new(&ws.0, &["../escape".into()]).is_err());
}

#[test]
fn symlinks_hardlinks_and_special_files_are_refused_before_evaluation() {
    use std::os::unix::fs::symlink;
    let ws = Workspace::new();
    let external = Workspace::new();
    symlink(&external.0, ws.0.join("src/link")).unwrap();
    symlink(external.0.join("other.rs"), ws.0.join("src/alias.rs")).unwrap();
    fs::hard_link(external.0.join("other.rs"), ws.0.join("src/hard.rs")).unwrap();
    let fifo = std::ffi::CString::new(ws.0.join("src/fifo").to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    let policy = ws.policy();
    for path in [
        "src/link/other.rs",
        "src/link/new.rs",
        "src/alias.rs",
        "src/hard.rs",
        "src/fifo",
        "src",
    ] {
        for tool in ["Read", "Write"] {
            let input = if tool == "Read" {
                json!({"file_path":path})
            } else {
                json!({"file_path":path,"content":"bad"})
            };
            assert!(policy.classify(tool, &input).is_err(), "{tool}: {path}");
        }
    }
    assert!(Policy::new(&ws.0.join("src/link"), &["src".into()]).is_err());
    assert_eq!(
        fs::read_to_string(external.0.join("other.rs")).unwrap(),
        "fn other() {}\n"
    );
}

#[test]
fn replacing_the_workspace_at_the_same_path_invalidates_the_policy() {
    let ws = Workspace::new();
    let policy = ws.policy();
    let retained = ws.0.with_extension("retained");
    fs::rename(&ws.0, &retained).unwrap();
    fs::create_dir_all(ws.0.join("src")).unwrap();
    fs::write(ws.0.join("src/owned.rs"), "replacement").unwrap();
    let decision = policy.classify("Read", &json!({"file_path":"src/owned.rs"}));
    fs::remove_dir_all(retained).unwrap();
    assert!(
        decision.is_err(),
        "a matching path string does not prove workspace identity"
    );
}

#[test]
fn search_inputs_cannot_hide_unsupported_fields_or_escape_the_workspace() {
    let ws = Workspace::new();
    let policy = ws.policy();
    for (tool, input) in [
        ("Glob", json!({"pattern":"../../*"})),
        ("Grep", json!({"pattern":"secret","path":"../outside"})),
        (
            "Grep",
            json!({"pattern":"secret","permission_override":true}),
        ),
        ("Glob", json!({"pattern":"*","path":ws.0.parent().unwrap()})),
        ("Grep", json!({"pattern":"secret","multiline":"true"})),
    ] {
        assert!(policy.classify(tool, &input).is_err(), "{tool}: {input}");
    }
    assert_eq!(
        policy
            .classify("Grep", &json!({"pattern":"todo","path":"src","-n":true}))
            .unwrap(),
        Disposition::Human
    );
    assert_eq!(
        policy
            .classify("Glob", &json!({"pattern":"src/**/*.rs","path":"."}))
            .unwrap(),
        Disposition::Human
    );
}
