#![cfg(windows)]

use agentconductor::{git::Repository, workflow};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
};

#[path = "support/windows_candidate_review_script.rs"]
mod script;
#[path = "support/workflow.rs"]
mod support;

struct RunningApp(Child);

impl Drop for RunningApp {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct TestDirectory(PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
#[ignore = "Runs only against the exact packaged binary."]
async fn exact_windows_package_reviews_candidate_and_accepts_or_rejects_it() {
    review_decision("merge").await;
    review_decision("reject").await;
}

async fn review_decision(decision: &str) {
    let (root, plan) = support::setup("worktree");
    let _cleanup = TestDirectory(root.clone());
    let repo = root.join("repo");
    let run = root.join("run");
    let candidate = workflow::run(plan, &repo, &run, 10).await.unwrap();
    assert_eq!(candidate.state, "waiting_approval");
    assert!(candidate.tests.iter().all(|test| test.exit_code == Some(0)));
    let sha = candidate.candidate.as_ref().unwrap().commit.clone();
    let base = candidate.base.clone();
    let home = root.join("home");
    std::fs::create_dir_all(home.join(".agentconductor")).unwrap();
    std::fs::write(
        home.join(".agentconductor/preferences.json"),
        br#"{"language":"en"}"#,
    )
    .unwrap();
    let binary = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")));
    let app = RunningApp(
        Command::new(binary)
            .args(["ui", run.to_str().unwrap()])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_CANDIDATE_SHA", &sha)
        .env("AC_DECISION", decision)
        .args(["-NoProfile", "-NonInteractive", "-Command", script::SCRIPT])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&driver.stdout);
    assert!(
        driver.status.success() && stdout.contains("CANDIDATE_DECISION_VERIFIED"),
        "UI Automation exited {}: stdout: {stdout}; stderr: {}",
        driver.status,
        String::from_utf8_lossy(&driver.stderr)
    );
    drop(app);

    let final_report = workflow::load(&run).unwrap();
    let git = Repository::open(&repo).unwrap();
    if decision == "merge" {
        assert_eq!(final_report.state, "completed");
        assert_eq!(git.head().unwrap(), sha);
        assert!(git.clean().unwrap());
        for path in ["one.txt", "two.txt", "three.txt"] {
            assert!(repo.join(path).is_file());
        }
    } else {
        assert_eq!(final_report.state, "rejected");
        assert_eq!(git.head().unwrap(), base);
        assert!(git.clean().unwrap());
    }
}
