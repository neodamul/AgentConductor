#![cfg(windows)]
//! Native Windows desktop acceptance: drive the Rust workspace
//! through Windows UI Automation — read rendered controls, set the goal and invoke Save.
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};

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
$heading = $null; $goal = $null; $save = $null
while ((Get-Date) -lt $deadline -and -not ($heading -and $goal -and $save)) {
  $all = $window.FindAll($T::Descendants, $C)
  foreach ($e in $all) {
    $n = $e.Current.Name
    if ($n -eq 'What should the team work on?') { $heading = $e }
    if ($n -eq 'Goal' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { $goal = $e }
    if ($n -eq 'Save workspace' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Button') { $save = $e }
  }
  if (-not ($heading -and $goal -and $save)) { Start-Sleep -Milliseconds 500 }
}
if (-not $heading) { Write-Output 'NO_HEADING'; $window.FindAll($T::Descendants, $C) | Select-Object -First 30 | ForEach-Object { Write-Output $_.Current.Name }; exit 3 }
if (-not $goal) { Write-Output 'NO_GOAL'; exit 4 }
if (-not $save) { Write-Output 'NO_SAVE'; exit 5 }
$goal.SetFocus()
[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal 윈도우')
Start-Sleep -Milliseconds 500
$save.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
Write-Output 'DRIVEN'
"#;

#[test]
fn native_workspace_is_accessible_and_saves_through_ui_automation() {
    let dir = std::env::temp_dir().join(format!("ac-win-desktop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // An isolated HOME keeps this fixture out of the operator's workspace list.
    let mut app = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .args(["ui-new", dir.to_str().unwrap()])
        .env("HOME", dir.join("home"))
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.id().to_string())
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout).to_string();
    assert!(
        driver.status.success(),
        "UI Automation driver failed: {}",
        String::from_utf8_lossy(&driver.stderr)
    );
    let saved = Instant::now() + Duration::from_secs(10);
    let mut goal = String::new();
    while Instant::now() < saved {
        if let Ok(text) = std::fs::read_to_string(dir.join("startup.json"))
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
        {
            goal = value["config"]["goal"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            if goal == "Windows UIA goal 윈도우" {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = app.kill();
    let _ = app.wait();
    assert!(report.contains("DRIVEN"), "UI Automation driver: {report}");
    assert_eq!(goal, "Windows UIA goal 윈도우");
    let _ = std::fs::remove_dir_all(dir);
}
