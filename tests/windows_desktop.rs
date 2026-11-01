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
  const uint KEYEVENTF_KEYUP = 2;
  const uint KEYEVENTF_UNICODE = 4;
  [StructLayout(LayoutKind.Sequential)] struct Point { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int X, Y; public uint Data, Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Sequential)] struct KeyboardInput { public ushort VirtualKey, Scan; public uint Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct Payload { [FieldOffset(0)] public MouseInput Mouse; [FieldOffset(0)] public KeyboardInput Keyboard; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public Payload Payload; }
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
  static uint ProcessId(IntPtr window) { uint processId; GetWindowThreadProcessId(window, out processId); return processId; }
  static Input Key(ushort key, uint flags) { return new Input { Type = 1, Payload = new Payload { Keyboard = new KeyboardInput { VirtualKey = key, Flags = flags } } }; }
  public static bool Focus(IntPtr window, uint pid) {
    SetForegroundWindow(window);
    var until = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < until && ProcessId(GetForegroundWindow()) != pid) System.Threading.Thread.Sleep(50);
    return ProcessId(GetForegroundWindow()) == pid;
  }
  public static bool Click(IntPtr window, int x, int y, uint pid) {
    if (!Focus(window, pid) || ProcessId(WindowFromPoint(new Point { X=x, Y=y })) != pid || !SetCursorPos(x, y)) return false;
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 2 } } };
    var up = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return false;
    System.Threading.Thread.Sleep(80);
    return SendInput(1, new[] { up }, size) == 1;
  }
  public static void SelectAll() {
    var keys = new[] { Key(0x11, 0), Key(0x41, 0), Key(0x41, KEYEVENTF_KEYUP), Key(0x11, KEYEVENTF_KEYUP) };
    if (SendInput(4, keys, Marshal.SizeOf(typeof(Input))) != 4) throw new InvalidOperationException("SEND_INPUT_SELECT_ALL_FAILED");
  }
  public static void TypeText(string text) {
    foreach (char character in text) {
      var down = Key(0, KEYEVENTF_UNICODE);
      down.Payload.Keyboard.Scan = character;
      var up = Key(0, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);
      up.Payload.Keyboard.Scan = character;
      if (SendInput(2, new[] { down, up }, Marshal.SizeOf(typeof(Input))) != 2) throw new InvalidOperationException("SEND_INPUT_TEXT_FAILED");
      System.Threading.Thread.Sleep(100);
    }
  }
  public static void PressEnter() {
    var keys = new[] { Key(0x0D, 0), Key(0x0D, KEYEVENTF_KEYUP) };
    if (SendInput(2, keys, Marshal.SizeOf(typeof(Input))) != 2) throw new InvalidOperationException("SEND_INPUT_ENTER_FAILED");
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
$bounds = $goal.Current.BoundingRectangle
if ($bounds.IsEmpty) { Write-Output 'GOAL_BOUNDS_EMPTY'; exit 7 }
$centerX = [int](($bounds.Left + $bounds.Right) / 2)
$centerY = [int](($bounds.Top + $bounds.Bottom) / 2)
if (-not [DesktopInput]::Focus($hwnd, $expectedProcessId)) {
  Write-Output 'GOAL_WINDOW_NOT_FOREGROUND'
  exit 7
}
if (-not [DesktopInput]::Click($hwnd, $centerX, $centerY, $expectedProcessId)) {
  Write-Output 'GOAL_CLICK_FAILED'
  exit 7
}
$focusDeadline = (Get-Date).AddSeconds(3)
while ((Get-Date) -lt $focusDeadline -and -not $goal.Current.HasKeyboardFocus) {
  Start-Sleep -Milliseconds 50
}
if (-not $goal.Current.HasKeyboardFocus) {
  Write-Output 'GOAL_FOCUS_FAILED'
  exit 7
}
[DesktopInput]::SelectAll()
[DesktopInput]::TypeText('Windows UIA goal')
Start-Sleep -Milliseconds 750
$focused = [System.Windows.Automation.AutomationElement]::FocusedElement
$goalText = 'TEXT_PATTERN_UNAVAILABLE'
try {
  $textPattern = $goal.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern)
  $goalText = $textPattern.DocumentRange.GetText(-1) -replace '[\r\n]', ' '
} catch {}
Write-Output "GOAL_FOCUS_AFTER_INPUT:$($focused.Current.ProcessId):$($focused.Current.ControlType.ProgrammaticName):$($focused.Current.Name)"
Write-Output "GOAL_TEXT_AFTER_INPUT:$goalText"

$save.SetFocus()
$saveFocusDeadline = (Get-Date).AddSeconds(3)
while ((Get-Date) -lt $saveFocusDeadline -and -not $save.Current.HasKeyboardFocus) {
  Start-Sleep -Milliseconds 50
}
if (-not $save.Current.HasKeyboardFocus) {
  Write-Output 'SAVE_FOCUS_FAILED'
  exit 8
}
[DesktopInput]::PressEnter()
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
    assert!(
        driver.status.success(),
        "UI Automation driver exited {}: stdout: {report}; stderr: {}",
        driver.status,
        String::from_utf8_lossy(&driver.stderr)
    );
    let saved = Instant::now() + Duration::from_secs(10);
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
    assert!(report.contains("DRIVEN"), "UI Automation driver: {report}");
    assert_eq!(
        goal,
        "Windows UIA goal",
        "driver output: {report}; stderr: {}",
        String::from_utf8_lossy(&driver.stderr)
    );
    let _ = std::fs::remove_dir_all(dir);
}
