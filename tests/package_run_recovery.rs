use agentconductor::{state::Store, workflow};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

#[path = "support/workflow.rs"]
mod support;

struct TestDirectory(PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[tokio::test]
#[ignore = "Runs only against the exact packaged binary."]
async fn exact_windows_package_stops_resumes_and_recovers_without_duplicate_work() {
    let binary = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")));
    let (root, mut plan) = support::setup("shared");
    let _cleanup = TestDirectory(root.clone());
    let run = root.join("run");
    let repo = root.join("repo");
    let started = root.join("worker.started");
    let release = root.join("worker.release");
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
    let mut coordinator = Command::new(&binary)
        .args(["run"])
        .arg(&plan_path)
        .arg(&repo)
        .arg(&run)
        .arg(workflow::digest(&bytes))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    wait_for(|| started.exists()).await;
    let stopped = Command::new(&binary)
        .arg("stop")
        .arg(&run)
        .arg("verify packaged stop and resume")
        .output()
        .unwrap();
    assert!(
        stopped.status.success(),
        "{}",
        String::from_utf8_lossy(&stopped.stderr)
    );
    wait_state(&run, "stopped").await;
    coordinator.wait().unwrap();
    std::fs::write(&release, "continue").unwrap();
    successful(&binary, ["resume", run.to_str().unwrap()]);
    wait_state(&run, "waiting_approval").await;
    let mut report = workflow::load(&run).unwrap();
    let task_id = std::fs::read_to_string(&started).unwrap();
    let task = report.tasks.iter().find(|task| task.id == task_id).unwrap();
    assert!(
        task.attempt >= 2,
        "stopped task attempt was {}",
        task.attempt
    );
    assert!(report.tasks.iter().all(|task| task.status == "completed"));

    report.state = "running".into();
    let interrupted = report
        .tasks
        .iter_mut()
        .find(|task| task.id == task_id)
        .unwrap();
    interrupted.status = "running".into();
    let owner_path = interrupted.artifacts.join("process/owner.json");
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
            "Simulated interrupted packaged run",
        )
        .unwrap();
    successful(&binary, ["recover", run.to_str().unwrap()]);
    wait_state(&run, "stopped").await;
    successful(&binary, ["resume", run.to_str().unwrap()]);
    wait_state(&run, "waiting_approval").await;
    let report = workflow::load(&run).unwrap();
    assert!(report.tasks.iter().all(|task| task.status == "completed"));
    assert!(
        report
            .tasks
            .iter()
            .find(|task| task.id == task_id)
            .unwrap()
            .attempt
            >= 3
    );
    let candidate = report.candidate.as_ref().unwrap();
    let diff = Command::new("git")
        .args(["-C", repo.to_str().unwrap(), "diff", "--name-only"])
        .arg(&report.base)
        .arg(&candidate.commit)
        .output()
        .unwrap();
    assert!(diff.status.success());
    let mut paths: Vec<_> = String::from_utf8(diff.stdout)
        .unwrap()
        .lines()
        .map(str::to_owned)
        .collect();
    paths.sort();
    assert_eq!(paths, ["one.txt", "three.txt", "two.txt"]);
}

fn successful<const N: usize>(binary: &std::path::Path, args: [&str; N]) {
    let output = Command::new(binary).args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
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
    .unwrap_or_else(|_| panic!("packaged run did not reach {expected}"));
}

async fn wait_for(mut ready: impl FnMut() -> bool) {
    tokio::time::timeout(Duration::from_secs(30), async {
        while !ready() {
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("packaged worker did not start");
}
