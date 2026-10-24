#![cfg_attr(not(windows), allow(dead_code))]

use agentconductor::{availability, workflow};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System.Runtime.InteropServices;
public static class AgentConductorTestInput {
  [DllImport("user32.dll")] private static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] private static extern void mouse_event(uint flags, uint x, uint y, uint data, System.UIntPtr extra);
  public static void Click(int x, int y) {
    SetCursorPos(x, y);
    mouse_event(0x0002, 0, 0, 0, System.UIntPtr.Zero);
    mouse_event(0x0004, 0, 0, 0, System.UIntPtr.Zero);
  }
}
"@
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadline = (Get-Date).AddSeconds(40)
$window = $null
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
  if (-not $window) { Start-Sleep -Milliseconds 500 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
$time = $null; $schedule = $null
while ((Get-Date) -lt $deadline -and -not ($time -and $schedule)) {
  foreach ($e in $window.FindAll($T::Descendants, $C)) {
    $name = $e.Current.Name
    if ($name -eq 'Schedule time · ISO 8601 with timezone' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { $time = $e }
    if ($name -eq 'Schedule retry' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Button') { $schedule = $e }
  }
  if (-not ($time -and $schedule)) { Start-Sleep -Milliseconds 500 }
}
if (-not $time -or -not $schedule) {
  Write-Output 'NO_SCHEDULE_CONTROLS'
  $window.FindAll($T::Descendants, $C) | Select-Object -First 50 | ForEach-Object { Write-Output $_.Current.Name }
  exit 3
}
$shell = New-Object -ComObject WScript.Shell
if (-not $shell.AppActivate([int]$env:AC_DESKTOP_PID)) {
  Write-Output 'WINDOW_NOT_ACTIVATED'
  exit 4
}
$window.SetFocus()
$bounds = $time.Current.BoundingRectangle
[AgentConductorTestInput]::Click([int](($bounds.Left + $bounds.Right) / 2), [int](($bounds.Top + $bounds.Bottom) / 2))
Start-Sleep -Milliseconds 150
$time.SetFocus()
$focusDeadline = (Get-Date).AddSeconds(5)
while (-not $time.Current.HasKeyboardFocus -and (Get-Date) -lt $focusDeadline) {
  Start-Sleep -Milliseconds 100
}
if (-not $time.Current.HasKeyboardFocus) {
  Write-Output 'SCHEDULE_INPUT_NOT_FOCUSED'
  exit 5
}
$scheduledAt = '2099-01-01T09:00:00+09:00'
[System.Windows.Forms.SendKeys]::SendWait('^a')
[System.Windows.Forms.SendKeys]::SendWait('2099-01-01T09:00:00{+}09:00')
$timePattern = $time.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
$inputDeadline = (Get-Date).AddSeconds(5)
while ($timePattern.Current.Value -ne $scheduledAt -and (Get-Date) -lt $inputDeadline) {
  Start-Sleep -Milliseconds 100
}
if ($timePattern.Current.Value -ne $scheduledAt) {
  Write-Output "SCHEDULE_VALUE_NOT_APPLIED; value=$($timePattern.Current.Value); enabled=$($time.Current.IsEnabled)"
  exit 6
}
if (-not $schedule.Current.IsEnabled) {
  Write-Output "SCHEDULE_BUTTON_DISABLED; value=$($timePattern.Current.Value)"
  exit 7
}
$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Write-Output 'SCHEDULED'
"#;

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
    if !driver.status.success() || !stdout.contains("SCHEDULED") {
        return Err(format!(
            "UI Automation failed: stdout: {stdout}; stderr: {}",
            String::from_utf8_lossy(&driver.stderr)
        ));
    }
    let expected =
        availability::parse_time("2099-01-01T09:00:00+09:00").map_err(|error| error.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        let report = workflow::load(run).map_err(|error| error.to_string())?;
        let restriction = report.tasks[0].restriction.as_ref();
        if restriction.is_some_and(|value| {
            value.source == "manual_schedule" && value.available_at == Some(expected)
        }) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err("scheduled time was not persisted by the package".into())
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
