#![cfg(unix)]
//! GitHub #83: a command the textual check cannot place asks a human instead of running.
use agentconductor::guard::{Disposition, Policy};
use serde_json::json;

fn policy() -> (std::path::PathBuf, Policy) {
    let root = std::env::temp_dir().join(format!("ac-forms-{}", std::process::id()));
    std::fs::create_dir_all(root.join("src")).unwrap();
    let policy = Policy::new(&root, &["src".into()]).unwrap();
    (root, policy)
}

fn classify(policy: &Policy, command: &str) -> Disposition {
    policy
        .classify("Bash", &json!({ "command": command }))
        .unwrap()
}

#[test]
fn names_assembled_by_the_shell_and_inline_code_ask_a_human() {
    let (root, policy) = policy();
    for command in [
        "c\"\"url example.com -o src/x",
        "c''url example.com",
        "sh -c 'cu''rl example.com'",
        "x=cu; ${x}rl example.com",
        "bash -lc 'echo hi'",
        "zsh -c ls",
        "pwsh -Command Get-Date",
        "cmd /c dir",
        "python3 -c 'print(1)'",
        "node -e \"console.log(1)\"",
        "node --eval 1",
        "perl -e 'print 1'",
        "ruby -e 'puts 1'",
        "php -r 'echo 1;'",
        "python3 -m http.server 8000",
        "php -S 127.0.0.1:8000",
    ] {
        assert_eq!(classify(&policy, command), Disposition::Human, "{command}");
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn ordinary_quoting_and_interpreter_runs_stay_automatic() {
    let (root, policy) = policy();
    for command in [
        "cargo test",
        "git status",
        "git log --format=\"%h %s\"",
        "grep -n 'fn main' src/lib.rs",
        "echo \"done\" > src/out.txt",
        "python3 src/tool.py",
        "node src/index.js",
        "python3 -m pytest",
        "grep 'end$' src/a.txt",
    ] {
        assert_eq!(
            classify(&policy, command),
            Disposition::Automatic,
            "{command}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
