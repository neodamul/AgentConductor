#![cfg_attr(not(windows), allow(dead_code))]

use agentconductor::{availability, workflow};
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[path = "support/windows_schedule.rs"]
mod schedule_support;
#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
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
$hwnd = [IntPtr]$window.Current.NativeWindowHandle
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ScheduleWindow {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int X, Y; }
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr window, int command);
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  public static void Maximize(IntPtr window) { ShowWindow(window, 3); }
  public static bool Focus(IntPtr window) {
    SetForegroundWindow(window);
    var deadline = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < deadline && GetForegroundWindow() != window) System.Threading.Thread.Sleep(50);
    return GetForegroundWindow() == window;
  }
  public static void Scroll(int x, int y) {
    SetCursorPos(x, y);
    mouse_event(0x0800, 0, 0, unchecked((uint)-360), UIntPtr.Zero);
  }
  public static uint ProcessAt(int x, int y) {
    uint processId;
    GetWindowThreadProcessId(WindowFromPoint(new Point { X = x, Y = y }), out processId);
    return processId;
  }
}
"@
[ScheduleWindow]::Maximize($hwnd)
if (-not [ScheduleWindow]::Focus($hwnd)) { Write-Output 'SCHEDULE_FOREGROUND_FAILED'; exit 9 }
Start-Sleep -Milliseconds 300
$window = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
if (-not $window) { Write-Output 'NO_MAXIMIZED_WINDOW'; exit 3 }
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
$window.SetFocus()
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
$windowBounds = $window.Current.BoundingRectangle
$scrollX = [int]($windowBounds.Right - 40); $scrollY = [int]($windowBounds.Bottom - 64)
for ($attempt = 0; $attempt -lt 10; $attempt++) {
  $bounds = $schedule.Current.BoundingRectangle
  $windowBounds = $window.Current.BoundingRectangle
  if ($bounds.Left -ge $windowBounds.Left -and $bounds.Top -ge $windowBounds.Top -and $bounds.Right -le $windowBounds.Right -and $bounds.Bottom -le $windowBounds.Bottom) { break }
  [ScheduleWindow]::Scroll($scrollX, $scrollY)
  Start-Sleep -Milliseconds 200
}
$bounds = $schedule.Current.BoundingRectangle
$windowBounds = $window.Current.BoundingRectangle
if ($bounds.Left -lt $windowBounds.Left -or $bounds.Top -lt $windowBounds.Top -or $bounds.Right -gt $windowBounds.Right -or $bounds.Bottom -gt $windowBounds.Bottom -or $schedule.Current.IsOffscreen) {
  Write-Output "SCHEDULE_BUTTON_OUTSIDE_WINDOW; button=$bounds; window=$windowBounds; offscreen=$($schedule.Current.IsOffscreen)"
  exit 11
}
if (-not [ScheduleWindow]::Focus($hwnd)) {
  Write-Output "SCHEDULE_FOREGROUND_FAILED; expected=$env:AC_DESKTOP_PID"
  exit 9
}
$window = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
try { $window.SetFocus() }
catch { Write-Output "SCHEDULE_UIA_FOCUS_UNAVAILABLE: $($_.Exception.Message)" }
$postFocusDeadline = (Get-Date).AddSeconds(5)
do {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
  $schedule = $null
  if ($window) { $schedule = $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Schedule retry' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1 }
  if ($schedule -and $schedule.Current.IsEnabled) { break }
  Start-Sleep -Milliseconds 100
} while ((Get-Date) -lt $postFocusDeadline)
if (-not $window) { Write-Output 'SCHEDULE_WINDOW_NOT_AVAILABLE_AFTER_FOCUS'; exit 12 }
if (-not $schedule) { Write-Output 'SCHEDULE_CONTROL_NOT_AVAILABLE_AFTER_FOCUS'; exit 12 }
if (-not $schedule.Current.IsEnabled) { Write-Output "SCHEDULE_CONTROL_DISABLED_AFTER_FOCUS; value=$($timePattern.Current.Value)"; exit 12 }
$bounds = $schedule.Current.BoundingRectangle
$x = [int]($bounds.Left + $bounds.Width / 2); $y = [int]($bounds.Top + $bounds.Height / 2)
$targetPid = [ScheduleWindow]::ProcessAt($x, $y)
if ($targetPid -ne [uint32]$env:AC_DESKTOP_PID) {
  Write-Output "SCHEDULE_TARGET_PROCESS_MISMATCH; expected=$env:AC_DESKTOP_PID; target=$targetPid; point=$x,$y; bounds=$bounds"
  exit 10
}
$schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Start-Sleep -Milliseconds 750
Write-Output "SCHEDULE_AFTER: enabled=$($schedule.Current.IsEnabled); keyboard_focus=$($schedule.Current.HasKeyboardFocus); offscreen=$($schedule.Current.IsOffscreen); window_focus=$($window.Current.HasKeyboardFocus)"
$window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name } | Select-Object -First 80 | ForEach-Object { Write-Output "POST_ACTION: $($_.Current.Name)" }
Write-Output 'SCHEDULE_ACTION_SUBMITTED'
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
