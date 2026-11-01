#![cfg_attr(not(windows), allow(dead_code))]

use agentconductor::{availability, workflow};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[path = "support/windows_schedule.rs"]
mod schedule_support;
#[path = "support/windows_schedule_script.rs"]
mod script;
#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = script::SCRIPT;

struct RunningApp(Child);

impl Drop for RunningApp {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn package_schedules_a_waiting_agent_room_task() {
    let (root, mut plan) = support::setup("shared");
    plan.agents[0].args = vec!["quota-once".into()];
    plan.agents[0]
        .environment
        .insert("MOCK_MARKERS".into(), root.to_string_lossy().into_owned());
    let run = root.join("run");
    let report = workflow::run(plan, &root.join("repo"), &run, 10)
        .await
        .unwrap();
    assert_eq!(report.state, "waiting_availability");
    let home = root.join("home");
    let executable = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")));
    let app = RunningApp(
        Command::new(executable)
            .args(["ui", run.to_str().unwrap()])
            .env("AC_RUN_DIR", &run)
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let outcome = drive_and_verify(&app, &run);
    let _ = workflow::stop(&run, "Windows package schedule test cleanup");
    wait_for_availability_watcher_exit(&run);
    drop(app);
    let _ = std::fs::remove_dir_all(root);
    outcome.unwrap();
}

#[cfg(windows)]
#[tokio::test]
#[ignore = "Runs only against the exact packaged binary."]
async fn exact_windows_package_schedules_a_waiting_agent_room_task() {
    package_schedules_a_waiting_agent_room_task().await;
}

fn drive_and_verify(app: &RunningApp, run: &Path) -> Result<(), String> {
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .map_err(|error| error.to_string())?;
    let stdout = String::from_utf8_lossy(&driver.stdout).to_string();
    if !driver.status.success() || !stdout.contains("SCHEDULE_ACTION_SUBMITTED") {
        return Err(format!(
            "UI Automation failed: stdout: {stdout}; stderr: {}",
            String::from_utf8_lossy(&driver.stderr)
        ));
    }
    if !schedule_support::wait_for_ui_action_dispatch(run, Duration::from_secs(3))? {
        return Err(format!(
            "Schedule retry activation did not reach the UI action controller; ui={stdout}; {}",
            schedule_diagnostics(run, &stdout)
        ));
    }
    let expected =
        availability::parse_time("2099-01-01T09:00:00+09:00").map_err(|error| error.to_string())?;
    if schedule_support::wait_for_manual_schedule(run, expected, Duration::from_secs(10))? {
        return Ok(());
    }
    Err(format!(
        "scheduled time was not persisted by the package; {}",
        schedule_diagnostics(run, &stdout)
    ))
}

fn schedule_diagnostics(run: &Path, driver_output: &str) -> String {
    let report = workflow::load(run)
        .map(|report| {
            let task = report.tasks.first().map(|task| {
                format!(
                    "{}:{:?}",
                    task.status,
                    task.restriction
                        .as_ref()
                        .map(|restriction| (&restriction.source, restriction.available_at))
                )
            });
            format!("state={}, task={task:?}", report.state)
        })
        .unwrap_or_else(|error| format!("report_error={error}"));
    let actions = std::fs::read_dir(run)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("coordinator-ui-")
                .then(|| {
                    let process = entry.path().join("process");
                    format!(
                        "owner={}, stderr={}",
                        std::fs::read_to_string(process.join("owner.json")).unwrap_or_default(),
                        std::fs::read_to_string(process.join("stderr.log")).unwrap_or_default()
                    )
                })
        })
        .collect::<Vec<_>>()
        .join(" | ");
    format!("ui={driver_output}; report={report}; actions={actions}")
}

#[test]
fn schedule_failure_diagnostics_include_post_action_accessibility_output() {
    let missing_run = std::env::temp_dir().join(format!("missing-ac-run-{}", std::process::id()));
    let diagnostics = schedule_diagnostics(&missing_run, "POST_ACTION: Schedule retry");
    assert!(diagnostics.contains("POST_ACTION: Schedule retry"));
}

#[test]
fn schedule_acceptance_requires_a_controller_action_owner_record() {
    let root = std::env::temp_dir().join(format!("ac-schedule-dispatch-{}", std::process::id()));
    let process = root.join("coordinator-ui-test/process");
    std::fs::create_dir_all(&process).unwrap();
    assert!(!schedule_support::wait_for_ui_action_dispatch(&root, Duration::ZERO).unwrap());
    std::fs::write(process.join("owner.json"), "{}").unwrap();
    assert!(schedule_support::wait_for_ui_action_dispatch(&root, Duration::ZERO).unwrap());
    std::fs::remove_dir_all(root).unwrap();
}

fn wait_for_availability_watcher_exit(run: &Path) {
    let spawn = run.join("availability-spawn.lock");
    let watch = run.join("availability-watch.lock");
    let owner = run.join("availability-coordinator/process/owner.json");
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let (Ok(_spawn), Ok(_watch)) = (
            agentconductor::leases::Lease::acquire(&spawn),
            agentconductor::leases::Lease::acquire(&watch),
        ) {
            let terminal = !owner.exists()
                || serde_json::from_slice::<serde_json::Value>(&std::fs::read(&owner).unwrap())
                    .unwrap()["phase"]
                    == "terminal";
            if terminal {
                return;
            }
        }
        assert!(
            Instant::now() < deadline,
            "availability watcher did not stop"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
