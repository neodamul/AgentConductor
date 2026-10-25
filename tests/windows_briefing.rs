#![cfg(windows)]

use agentconductor::workflow;
use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
};

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
function Find-Button($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1
}
$load = $null
while (-not $load -and (Get-Date) -lt $deadline) {
  $load = Find-Button 'Load read-only team briefing'
  if (-not $load) { Start-Sleep -Milliseconds 250 }
}
if (-not $load) { Write-Output 'NO_BRIEFING_ACTION'; exit 3 }
$hwnd = [IntPtr]$window.Current.NativeWindowHandle
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class BriefingMouse {
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  public static bool Click(IntPtr window, int x, int y) {
    SetForegroundWindow(window);
    var deadline = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < deadline && GetForegroundWindow() != window) System.Threading.Thread.Sleep(50);
    if (GetForegroundWindow() != window) return false;
    SetCursorPos(x, y);
    mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(80);
    mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    return true;
  }
}
"@
$bounds = $load.Current.BoundingRectangle
if (-not [BriefingMouse]::Click($hwnd, [int]($bounds.Left + $bounds.Width / 2), [int]($bounds.Top + $bounds.Height / 2))) { Write-Output 'WINDOW_FOREGROUND_FAILED_BEFORE_BRIEFING_CLICK'; exit 6 }
$menu = $null
while (-not $menu -and (Get-Date) -lt $deadline) {
  $menu = Find-Button 'Team briefing · read-only'
  if (-not $menu) { Start-Sleep -Milliseconds 250 }
}
if (-not $menu) {
  Write-Output "NO_BRIEFING_RESULT; window=$($window.Current.BoundingRectangle)"
  $window.FindAll($T::Descendants, $C) | ForEach-Object {
    Write-Output "$($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name) | enabled=$($_.Current.IsEnabled) | offscreen=$($_.Current.IsOffscreen)"
  }
  exit 4
}
$menu.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
$required = @('mock', 'one.txt', 'two.txt', 'three.txt', 'Review requests: merge')
$names = ''
$visible = $false
while (-not $visible -and (Get-Date) -lt $deadline) {
  $names = ($window.FindAll($T::Descendants, $C) | ForEach-Object { $_.Current.Name }) -join "`n"
  $visible = $true
  foreach ($value in $required) { if (-not $names.Contains($value)) { $visible = $false } }
  if (-not $visible) { Start-Sleep -Milliseconds 250 }
}
if (-not $visible) { Write-Output "BRIEFING_INCOMPLETE`n$names"; exit 5 }
Write-Output 'BRIEFING_VISIBLE'
Write-Output $names
"#;

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
async fn exact_windows_package_reads_the_team_briefing_without_changing_the_run() {
    let (root, plan) = support::setup("shared");
    let _cleanup = TestDirectory(root.clone());
    let run = root.join("run");
    let before = workflow::run(plan, &root.join("repo"), &run, 3)
        .await
        .unwrap();
    assert_eq!(before.state, "waiting_approval");
    assert!(before.tasks.iter().all(|task| task.status == "completed"));
    let before = serde_json::to_value(before).unwrap();
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
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout);
    assert!(
        driver.status.success() && report.contains("BRIEFING_VISIBLE"),
        "UI Automation exited {}: stdout: {report}; stderr: {}",
        driver.status,
        String::from_utf8_lossy(&driver.stderr)
    );
    drop(app);
    assert_eq!(
        serde_json::to_value(workflow::load(&run).unwrap()).unwrap(),
        before
    );
}
