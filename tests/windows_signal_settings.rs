#![cfg(windows)]

use std::{
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

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
function Find-Name($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq $name } | Select-Object -First 1
}
function Find-ActionControl($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -in @('ControlType.Button', 'ControlType.RadioButton') } | Select-Object -First 1
}
function Find-VisibleActionControl($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -in @('ControlType.Button', 'ControlType.RadioButton') -and -not $_.Current.IsOffscreen -and -not $_.Current.BoundingRectangle.IsEmpty
  } | Select-Object -First 1
}
function Find-Edit($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit' } | Select-Object -First 1
}
function Describe-ActionControls {
  $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID } | ForEach-Object {
    $root = $_
    $title = $root.Current.Name
    $root.FindAll($T::Descendants, $C) | ForEach-Object {
      $item = $_.Current
      if ($item.Name) { "window='$title' $($item.ControlType.ProgrammaticName)='$($item.Name)' offscreen=$($item.IsOffscreen)" }
    }
  } | Select-Object -First 100
}
function Wait-Name($name) {
  $found = $null
  while (-not $found -and (Get-Date) -lt $deadline) {
    $found = Find-Name $name
    if (-not $found) { Start-Sleep -Milliseconds 100 }
  }
  if (-not $found) { throw "MISSING_CONTROL: $name" }
  return $found
}
function Wait-ActionControl($name) {
  $found = $null
  while (-not $found -and (Get-Date) -lt $deadline) {
    $found = Find-ActionControl $name
    if (-not $found) { Start-Sleep -Milliseconds 100 }
  }
  if (-not $found) {
    $controls = @(Describe-ActionControls)
    throw "MISSING_ACTION_CONTROL: $name; observed=$($controls -join ' | ')"
  }
  return $found
}
function Wait-Edit($name) {
  $found = $null
  while (-not $found -and (Get-Date) -lt $deadline) {
    $found = Find-Edit $name
    if (-not $found) { Start-Sleep -Milliseconds 100 }
  }
  if (-not $found) { throw "MISSING_EDIT: $name" }
  return $found
}
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class SignalSettingsMouse {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int x, y; }
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int x, y; public uint data, flags, time; public IntPtr extraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct Payload { [FieldOffset(0)] public MouseInput mouse; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint type; public Payload payload; }
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void keybd_event(byte key, byte scan, uint flags, IntPtr extra);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll")] static extern uint SendInput(uint count, Input[] inputs, int size);
  public static void Key(byte key) {
    keybd_event(key, 0, 0, IntPtr.Zero);
    System.Threading.Thread.Sleep(50);
    keybd_event(key, 0, 2, IntPtr.Zero);
  }
  public static void ReplaceText(string value) {
    keybd_event(0x11, 0, 0, IntPtr.Zero);
    keybd_event(0x41, 0, 0, IntPtr.Zero);
    keybd_event(0x41, 0, 2, IntPtr.Zero);
    keybd_event(0x11, 0, 2, IntPtr.Zero);
    foreach (char character in value) Key((byte)character);
  }
  public static bool Focus(IntPtr window) {
    SetForegroundWindow(window);
    var deadline = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < deadline && GetForegroundWindow() != window) System.Threading.Thread.Sleep(50);
    return GetForegroundWindow() == window;
  }
  public static bool Click(IntPtr window, int x, int y, uint expectedPid) {
    if (!Focus(window)) return false;
    var target = WindowFromPoint(new Point { x = x, y = y });
    uint targetPid;
    GetWindowThreadProcessId(target, out targetPid);
    if (targetPid != expectedPid || !SetCursorPos(x, y)) return false;
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { payload = new Payload { mouse = new MouseInput { flags = 2 } } };
    var up = new Input { payload = new Payload { mouse = new MouseInput { flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return false;
    System.Threading.Thread.Sleep(80);
    return SendInput(1, new[] { up }, size) == 1;
  }
}
"@
$hwnd = [IntPtr]$window.Current.NativeWindowHandle
$expectedProcessId = [uint32]$env:AC_DESKTOP_PID
function Click-ActionControl($name) {
  $control = Wait-ActionControl $name
  $bounds = $control.Current.BoundingRectangle
  if ($bounds.IsEmpty -or $control.Current.IsOffscreen) { throw "CONTROL_OFFSCREEN: $name; bounds=$bounds" }
  if (-not [SignalSettingsMouse]::Click($hwnd, [int]($bounds.Left + $bounds.Width / 2), [int]($bounds.Top + $bounds.Height / 2), $expectedProcessId)) {
    throw "CLICK_FAILED: $name; bounds=$bounds; window=$($window.Current.BoundingRectangle)"
  }
}
function Activate-ActionControl($name, $nextName) {
  $control = Wait-ActionControl $name
  $expectedProcessId = [uint32]$env:AC_DESKTOP_PID
  try {
    if (-not [SignalSettingsMouse]::Focus($hwnd)) { throw 'PACKAGED_WINDOW_NOT_FOREGROUND' }
    $control.SetFocus()
    $focused = $A::FocusedElement
    if ($focused.Current.ProcessId -eq $expectedProcessId -and $focused.Current.Name -eq $name -and $focused.Current.ControlType.ProgrammaticName -in @('ControlType.Button', 'ControlType.RadioButton') -and $focused.Current.IsEnabled) {
      [SignalSettingsMouse]::Key(0x0D)
    }
  } catch {}
  $nextDeadline = (Get-Date).AddSeconds(3)
  while (-not (Find-ActionControl $nextName) -and -not (Find-Edit $nextName) -and (Get-Date) -lt $nextDeadline) { Start-Sleep -Milliseconds 100 }
  $activated = (Find-ActionControl $nextName) -or (Find-Edit $nextName)
  if (-not $activated) {
    Click-ActionControl $name
    $nextDeadline = (Get-Date).AddSeconds(3)
    while (-not (Find-ActionControl $nextName) -and -not (Find-Edit $nextName) -and (Get-Date) -lt $nextDeadline) { Start-Sleep -Milliseconds 100 }
    $activated = (Find-ActionControl $nextName) -or (Find-Edit $nextName)
  }
  if (-not $activated) { throw "ACTION_DID_NOT_OPEN_EXPECTED_CONTROL: $name -> $nextName" }
}
Activate-ActionControl 'Settings' 'Signal transport'
Activate-ActionControl 'Signal transport' 'Local Rust'
Activate-ActionControl 'agentmemory preferred' 'agentmemory port'
$port = Wait-Edit 'agentmemory port'
if (-not [SignalSettingsMouse]::Click($hwnd, [int]($port.Current.BoundingRectangle.Left + $port.Current.BoundingRectangle.Width / 2), [int]($port.Current.BoundingRectangle.Top + $port.Current.BoundingRectangle.Height / 2), $expectedProcessId)) {
  throw 'PORT_FIELD_CLICK_FAILED'
}
[SignalSettingsMouse]::ReplaceText('7333')
Start-Sleep -Milliseconds 250
$value = $port.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
if ($value -ne '7333') { throw "PORT_KEYBOARD_VALUE_NOT_APPLIED: $value" }
Click-ActionControl 'Local Rust'
$hideDeadline = (Get-Date).AddSeconds(5)
while ((Find-Name 'agentmemory port') -and (Get-Date) -lt $hideDeadline) { Start-Sleep -Milliseconds 100 }
if (Find-Name 'agentmemory port') { throw 'PORT_FIELD_VISIBLE_FOR_LOCAL_RUST' }
Activate-ActionControl 'agentmemory preferred' 'agentmemory port'
$port = Wait-Edit 'agentmemory port'
$value = $port.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
if ($value -ne '7333') { throw "PORT_VALUE_NOT_RETAINED: $value" }
Click-ActionControl 'Save'
$preferencePath = Join-Path $env:AC_SIGNAL_WORKSPACE 'signal-transport.json'
$persisted = $null
$persistDeadline = (Get-Date).AddSeconds(5)
while (-not $persisted -and (Get-Date) -lt $persistDeadline) {
  try {
    $candidate = Get-Content -Raw $preferencePath | ConvertFrom-Json
    if ($candidate.preference.transport -eq 'agentmemory' -and $candidate.preference.port -eq 7333) { $persisted = $candidate }
  } catch {}
  if (-not $persisted) { Start-Sleep -Milliseconds 100 }
}
if (-not $persisted) { throw "SIGNAL_PREFERENCE_NOT_PERSISTED; file=$preferencePath" }
$closeDeadline = (Get-Date).AddSeconds(10)
while ((Find-VisibleActionControl 'Local Rust') -and (Get-Date) -lt $closeDeadline) { Start-Sleep -Milliseconds 100 }
if (Find-VisibleActionControl 'Local Rust') {
  Write-Output "SIGNAL_DIALOG_STILL_VISIBLE; persisted=$($persisted | ConvertTo-Json -Compress)"
  Describe-ActionControls
  throw "SIGNAL_DIALOG_DID_NOT_CLOSE; persisted=$($persisted | ConvertTo-Json -Compress)"
}
Write-Output 'SIGNAL_SETTINGS_SAVED'
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

#[test]
#[ignore = "requires the exact packaged Windows release binary"]
fn exact_windows_package_saves_signal_transport_and_retains_port() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("ac-win-signal-{}-{nonce}", std::process::id()));
    let _cleanup = TestDirectory(root.clone());
    let home = root.join("home");
    let workspace = root.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let executable = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .expect("package job must supply the exact release binary");
    let app = RunningApp(
        Command::new(executable)
            .args(["ui-new", workspace.to_str().unwrap()])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let driver = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", app.0.id().to_string())
        .env("AC_SIGNAL_WORKSPACE", &workspace)
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .unwrap();
    let report = String::from_utf8_lossy(&driver.stdout);
    let persisted_settings = std::fs::read_to_string(workspace.join("signal-transport.json"))
        .unwrap_or_else(|error| format!("unavailable: {error}"));
    assert!(
        driver.status.success() && report.contains("SIGNAL_SETTINGS_SAVED"),
        "UI Automation exited {}: stdout: {report}; stderr: {}; persisted_settings={persisted_settings}",
        driver.status,
        String::from_utf8_lossy(&driver.stderr)
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let saved = loop {
        if let Ok(contents) = std::fs::read_to_string(workspace.join("signal-transport.json"))
            && let Ok(settings) = serde_json::from_str::<serde_json::Value>(&contents)
            && settings["preference"]["transport"] == "agentmemory"
            && settings["preference"]["port"] == 7333
        {
            break settings;
        }
        assert!(
            Instant::now() < deadline,
            "signal preference was not persisted"
        );
        std::thread::sleep(Duration::from_millis(100));
    };
    assert_eq!(saved["preference"]["transport"], "agentmemory");
    assert_eq!(saved["preference"]["port"], 7333);
}
