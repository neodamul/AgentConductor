#![cfg(windows)]
//! Native Windows desktop acceptance: drive the Rust workspace
//! through Windows UI Automation — read rendered controls, set the goal and invoke Save.
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class DesktopInput {
  const uint MOUSEEVENTF_LEFTDOWN = 2, MOUSEEVENTF_LEFTUP = 4;
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int X, Y; public uint Data, Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct Payload { [FieldOffset(0)] public MouseInput Mouse; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public Payload Payload; }
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
  static uint ProcessId(IntPtr window) { uint processId; GetWindowThreadProcessId(window, out processId); return processId; }
  static Input Mouse(uint flags) { return new Input { Type = 0, Payload = new Payload { Mouse = new MouseInput { Flags = flags } } }; }
  public static bool Focus(IntPtr window, uint pid) {
    SetForegroundWindow(window);
    var until = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < until && ProcessId(GetForegroundWindow()) != pid) System.Threading.Thread.Sleep(50);
    return ProcessId(GetForegroundWindow()) == pid;
  }
  public static void Click(int x, int y) {
    if (!SetCursorPos(x, y)) throw new InvalidOperationException("SET_CURSOR_POS_FAILED");
    var clicks = new[] { Mouse(MOUSEEVENTF_LEFTDOWN), Mouse(MOUSEEVENTF_LEFTUP) };
    if (SendInput(2, clicks, Marshal.SizeOf(typeof(Input))) != 2) throw new InvalidOperationException("SEND_INPUT_MOUSE_CLICK_FAILED");
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
$readyDeadline = (Get-Date).AddSeconds(20)
while ((Get-Date) -lt $readyDeadline -and (-not $goal.Current.IsEnabled -or -not $save.Current.IsEnabled)) {
  Start-Sleep -Milliseconds 100
}
if (-not $goal.Current.IsEnabled -or -not $save.Current.IsEnabled) {
  Write-Output "CONTROLS_NOT_ENABLED:$($goal.Current.IsEnabled):$($save.Current.IsEnabled)"
  exit 6
}
$hwnd = [IntPtr]$window.Current.NativeWindowHandle
$expectedProcessId = [uint32]$env:AC_DESKTOP_PID
if (-not [DesktopInput]::Focus($hwnd, $expectedProcessId)) {
  Write-Output 'GOAL_WINDOW_NOT_FOREGROUND'
  exit 7
}
$goal.SetFocus()
$focusDeadline = (Get-Date).AddSeconds(3)
while ((Get-Date) -lt $focusDeadline -and -not $goal.Current.HasKeyboardFocus) {
  Start-Sleep -Milliseconds 50
}
if (-not $goal.Current.HasKeyboardFocus) {
  Write-Output 'GOAL_FOCUS_FAILED'
  exit 7
}
$goalValue = $goal.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
if ($goalValue.Current.IsReadOnly) { Write-Output 'GOAL_VALUE_READ_ONLY'; exit 7 }
[System.Windows.Forms.SendKeys]::SendWait('^a')
[System.Windows.Forms.SendKeys]::SendWait('Windows UIA goal')
$goalValueDeadline = (Get-Date).AddSeconds(3)
$goalText = $goalValue.Current.Value
while ($goalText -ne 'Windows UIA goal' -and (Get-Date) -lt $goalValueDeadline) {
  Start-Sleep -Milliseconds 50
  $goalText = $goalValue.Current.Value
}
$focused = [System.Windows.Automation.AutomationElement]::FocusedElement
Write-Output "GOAL_FOCUS_AFTER_INPUT:$($focused.Current.ProcessId):$($focused.Current.ControlType.ProgrammaticName):$($focused.Current.Name)"
Write-Output "GOAL_TEXT_AFTER_INPUT:$goalText"
if ($goalText -ne 'Windows UIA goal') { Write-Output "GOAL_VALUE_MISMATCH:$goalText"; exit 7 }

# Text edits may rebuild the UI Automation tree; never invoke a button captured before editing.
$save = $null
$saveDeadline = (Get-Date).AddSeconds(5)
while (-not $save -and (Get-Date) -lt $saveDeadline) {
  $all = $window.FindAll($T::Descendants, $C)
  foreach ($e in $all) {
    if ($e.Current.Name -eq 'Save workspace' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Button') {
      $save = $e
      break
    }
  }
  if (-not $save) { Start-Sleep -Milliseconds 100 }
}
if (-not $save) { Write-Output 'SAVE_NOT_FOUND_AFTER_GOAL_EDIT'; exit 8 }
$saveReadyDeadline = (Get-Date).AddSeconds(5)
while ((Get-Date) -lt $saveReadyDeadline -and -not $save.Current.IsEnabled) {
  Start-Sleep -Milliseconds 100
}
if (-not $save.Current.IsEnabled) { Write-Output 'SAVE_NOT_ENABLED_AFTER_GOAL_EDIT'; exit 8 }

$saveBounds = $save.Current.BoundingRectangle
if ($save.Current.IsOffscreen -or $saveBounds.IsEmpty) {
  Write-Output 'SAVE_NOT_VISIBLE'
  exit 8
}
if (-not [DesktopInput]::Focus($hwnd, $expectedProcessId)) {
  Write-Output 'SAVE_WINDOW_NOT_FOREGROUND'
  exit 8
}
$saveInvoke = $null
try { $saveInvoke = $save.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern) } catch {}
if ($saveInvoke) {
  $saveInvoke.Invoke()
  Write-Output 'SAVE_INVOKED'
} else {
  $saveX = [int]($saveBounds.Left + ($saveBounds.Width / 2))
  $saveY = [int]($saveBounds.Top + ($saveBounds.Height / 2))
  [DesktopInput]::Click($saveX, $saveY)
  Write-Output 'SAVE_CLICKED'
}
Write-Output 'DRIVEN'
"#;

#[test]
fn default_native_workspace_is_accessible_and_saves_through_ui_automation() {
    let dir = std::env::temp_dir().join(format!("ac-win-desktop-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    // The default launch must keep its workspace out of the operator's profile.
    let home = dir.join("home");
    let workspace = home.join(".agentconductor/desktop");
    let executable = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")));
    let mut app = Command::new(executable)
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.id().to_string())
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout).to_string();
    let driver_stderr = String::from_utf8_lossy(&driver.stderr).to_string();
    let saved = Instant::now()
        + if driver.status.success() {
            Duration::from_secs(10)
        } else {
            Duration::ZERO
        };
    let mut goal = String::new();
    while Instant::now() < saved {
        if let Ok(text) = std::fs::read_to_string(workspace.join("startup.json"))
            && let Ok(value) = serde_json::from_str::<serde_json::Value>(&text)
        {
            goal = value["config"]["goal"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            if goal == "Windows UIA goal" {
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    let _ = app.kill();
    let _ = app.wait();
    assert!(
        driver.status.success(),
        "UI Automation driver exited {}: stdout: {report}; stderr: {}",
        driver.status,
        driver_stderr
    );
    assert!(report.contains("DRIVEN"), "UI Automation driver: {report}");
    assert_eq!(
        goal, "Windows UIA goal",
        "driver output: {report}; stderr: {}",
        driver_stderr
    );
    let _ = std::fs::remove_dir_all(dir);
}
