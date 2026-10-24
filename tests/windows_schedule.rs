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
$inputEnabledDeadline = (Get-Date).AddSeconds(10)
while (-not $time.Current.IsEnabled -and (Get-Date) -lt $inputEnabledDeadline) {
  Start-Sleep -Milliseconds 100
}
if (-not $time.Current.IsEnabled) {
  Write-Output "SCHEDULE_INPUT_DISABLED; schedule_enabled=$($schedule.Current.IsEnabled)"
  exit 4
}
$scheduledAt = '2099-01-01T09:00:00+09:00'
$timePattern = $time.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
$timePattern.SetValue($scheduledAt)
$inputDeadline = (Get-Date).AddSeconds(5)
while ($timePattern.Current.Value -ne $scheduledAt -and (Get-Date) -lt $inputDeadline) {
  Start-Sleep -Milliseconds 100
}
if ($timePattern.Current.Value -ne $scheduledAt) {
  Write-Output "SCHEDULE_VALUE_NOT_APPLIED; value=$($timePattern.Current.Value); enabled=$($time.Current.IsEnabled)"
  exit 7
}
$enabledDeadline = (Get-Date).AddSeconds(10)
while (-not $schedule.Current.IsEnabled -and (Get-Date) -lt $enabledDeadline) {
  Start-Sleep -Milliseconds 100
}
if (-not $schedule.Current.IsEnabled) {
  Write-Output "SCHEDULE_BUTTON_DISABLED; value=$($timePattern.Current.Value)"
  exit 8
}
$schedule = $null
$refreshDeadline = (Get-Date).AddSeconds(5)
while (-not $schedule -and (Get-Date) -lt $refreshDeadline) {
  $schedule = $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Schedule retry' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1
  if (-not $schedule -or -not $schedule.Current.IsEnabled) { $schedule = $null; Start-Sleep -Milliseconds 100 }
}
if (-not $schedule) { Write-Output 'SCHEDULE_BUTTON_STALE_AFTER_VALUE_UPDATE'; exit 9 }
$hwnd = [IntPtr]$window.Current.NativeWindowHandle
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class NativeWindowInput {
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  public static bool Focus(IntPtr window) { return SetForegroundWindow(window); }
  public static bool Resize(IntPtr window, int x, int y, int width, int height) {
    return SetWindowPos(window, IntPtr.Zero, x, y, width, height, 0x0004 | 0x0010);
  }
  public static void Scroll(int x, int y) {
    SetCursorPos(x, y);
    mouse_event(0x0800, 0, 0, unchecked((uint)-120), UIntPtr.Zero);
  }
  public static void Click(int x, int y) { SetCursorPos(x, y); mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero); mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero); }
}
"@
if (-not [NativeWindowInput]::Focus($hwnd)) { Write-Output "WINDOW_FOREGROUND_FAILED; hwnd=$hwnd"; exit 12 }
$windowBounds = $window.Current.BoundingRectangle
$bounds = $schedule.Current.BoundingRectangle
if ($bounds.Bottom -gt $windowBounds.Bottom -or $schedule.Current.IsOffscreen) {
  if (-not [NativeWindowInput]::Resize($hwnd, [int]$windowBounds.Left, [int]$windowBounds.Top, [int]$windowBounds.Width, [int]($windowBounds.Height + 120))) {
    Write-Output "WINDOW_RESIZE_FAILED; error=$([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
  }
  Start-Sleep -Milliseconds 300
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
  if (-not $window) { Write-Output 'SCHEDULE_WINDOW_LOST_AFTER_RESIZE'; exit 13 }
  $windowBounds = $window.Current.BoundingRectangle
  $schedule = $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Schedule retry' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1
  if (-not $window -or -not $schedule) { Write-Output 'SCHEDULE_CONTROL_LOST_AFTER_RESIZE'; exit 13 }
  $bounds = $schedule.Current.BoundingRectangle
}
if ($bounds.Bottom -gt $windowBounds.Bottom -or $schedule.Current.IsOffscreen) {
  $x = [int]($bounds.Left + $bounds.Width / 2)
  $y = [int]($windowBounds.Bottom - 80)
  for ($attempt = 0; $attempt -lt 6; $attempt++) {
    [NativeWindowInput]::Scroll($x, $y)
    Start-Sleep -Milliseconds 150
    $schedule = $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Schedule retry' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1
    if ($schedule) { $bounds = $schedule.Current.BoundingRectangle }
    if ($schedule -and $bounds.Bottom -le $windowBounds.Bottom -and -not $schedule.Current.IsOffscreen) { break }
  }
}
$visibleDeadline = (Get-Date).AddSeconds(3)
while ((Get-Date) -lt $visibleDeadline) {
  $schedule = $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Schedule retry' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1
  if ($schedule) { $bounds = $schedule.Current.BoundingRectangle }
  if ($bounds.Width -gt 0 -and $bounds.Height -gt 0 -and -not $schedule.Current.IsOffscreen -and $bounds.Bottom -le $windowBounds.Bottom) { break }
  Start-Sleep -Milliseconds 100
}
if (-not $schedule -or $bounds.Width -le 0 -or $bounds.Height -le 0 -or $schedule.Current.IsOffscreen -or $bounds.Bottom -gt $windowBounds.Bottom) {
  Write-Output "SCHEDULE_BUTTON_NOT_VISIBLE; bounds=$bounds; window=$windowBounds"
  exit 10
}
$bounds = $schedule.Current.BoundingRectangle
[NativeWindowInput]::Click([int]($bounds.Left + $bounds.Width / 2), [int]($bounds.Top + $bounds.Height / 2))
Start-Sleep -Milliseconds 750
$window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name } | Select-Object -First 80 | ForEach-Object { Write-Output "POST_ACTION: $($_.Current.Name)" }
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
