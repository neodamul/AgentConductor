use agentconductor::{state::Store, workflow};
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[path = "support/windows_run_lifecycle_script.rs"]
mod script;
#[path = "support/workflow.rs"]
mod support;

struct TestDirectory(PathBuf);
impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct ChildProcess(Child);
impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn lifecycle_driver_exposes_stop_resume_and_recovery_observations() {
    for required in [
        "AC_RUN_LIFECYCLE_ACTION",
        "Stop run",
        "Stop reason",
        "Confirm stop",
        "Resume run",
        "InvokePattern",
    ] {
        assert!(
            script::SCRIPT.contains(required),
            "missing driver contract: {required}"
        );
    }
}

#[tokio::test]
#[ignore = "Runs only against the exact packaged binary."]
async fn exact_windows_package_ui_stops_resumes_and_recovers_without_duplicate_work() {
    let (root, mut plan) = support::setup("shared");
    let _cleanup = TestDirectory(root.clone());
    let run = root.join("run");
    let repo = root.join("repo");
    let started = root.join("worker.started");
    let release = root.join("worker.release");
    plan.tasks.truncate(1);
    plan.agents[0].args = vec!["task-blocking".into()];
    plan.agents[0].environment.insert(
        "AC_TEST_BLOCKING_STARTED".into(),
        started.to_string_lossy().into_owned(),
    );
    plan.agents[0].environment.insert(
        "AC_TEST_BLOCKING_RELEASE".into(),
        release.to_string_lossy().into_owned(),
    );
    let bytes = serde_json::to_vec(&plan).unwrap();
    let plan_path = root.join("plan.json");
    std::fs::write(&plan_path, &bytes).unwrap();
    let binary = package_binary();
    let mut coordinator = ChildProcess(
        Command::new(&binary)
            .args(["run"])
            .arg(&plan_path)
            .arg(&repo)
            .arg(&run)
            .arg(workflow::digest(&bytes))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    wait_for_file(&started).await;

    let home = root.join("home");
    let app = launch(&binary, &run, &home);
    drive(&app, "stop");
    wait_state(&run, "stopped").await;
    wait_for_exit(&mut coordinator.0);
    std::fs::write(&release, "continue").unwrap();
    drive(&app, "resume");
    wait_state(&run, "waiting_approval").await;
    assert_candidate(&run, &repo, 2);
    drop(app);

    let mut report = workflow::load(&run).unwrap();
    report.state = "running".into();
    report.candidate = None;
    report.diff_hash = None;
    let task = &mut report.tasks[0];
    task.status = "running".into();
    task.finished_ms = None;
    let owner_path = task.artifacts.join("process/owner.json");
    let mut owner: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&owner_path).unwrap()).unwrap();
    owner["pid"] = (i32::MAX as u32).into();
    owner["process_group"] = i32::MAX.into();
    std::fs::write(&owner_path, serde_json::to_vec(&owner).unwrap()).unwrap();
    Store::open(run.join("state.sqlite"))
        .unwrap()
        .record(
            "workflow",
            &serde_json::to_value(&report).unwrap(),
            "Simulated interrupted packaged UI run",
        )
        .unwrap();

    let recovered_app = launch(&binary, &run, &home);
    wait_state(&run, "stopped").await;
    drive(&recovered_app, "resume");
    wait_state(&run, "waiting_approval").await;
    assert_candidate(&run, &repo, 3);
    drop(recovered_app);
}

fn package_binary() -> PathBuf {
    std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")))
}

fn launch(binary: &std::path::Path, run: &std::path::Path, home: &std::path::Path) -> ChildProcess {
    ChildProcess(
        Command::new(binary)
            .args(["ui"])
            .arg(run)
            .env("HOME", home)
            .env("USERPROFILE", home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    )
}

fn drive(app: &ChildProcess, action: &str) {
    let output = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_RUN_LIFECYCLE_ACTION", action)
        .args(["-NoProfile", "-NonInteractive", "-Command", script::SCRIPT])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("RUN_LIFECYCLE_ACTION_SUBMITTED"),
        "UI action {action} failed: stdout={stdout}; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn wait_for_exit(child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "coordinator stayed alive after UI stop"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

async fn wait_for_file(path: &std::path::Path) {
    tokio::time::timeout(Duration::from_secs(30), async {
        while !path.exists() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("blocking mock worker did not start");
}

async fn wait_state(run: &std::path::Path, expected: &str) {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if workflow::load(run).is_ok_and(|report| report.state == expected) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("packaged UI run did not reach {expected}"));
}

fn assert_candidate(run: &std::path::Path, repo: &std::path::Path, attempt: u32) {
    let report = workflow::load(run).unwrap();
    assert!(report.tasks.iter().all(|task| task.status == "completed"));
    assert!(
        report.tasks[0].attempt >= attempt,
        "task attempt was {}",
        report.tasks[0].attempt
    );
    let candidate = report
        .candidate
        .as_ref()
        .expect("completed run should retain its candidate");
    let output = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "diff", "--name-only"])
        .arg(&report.base)
        .arg(&candidate.commit)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        vec!["one.txt"]
    );
}
