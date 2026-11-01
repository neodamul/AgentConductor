pub const SCRIPT: &str = r#"
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
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int X, Y; public uint Data, Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct InputPayload { [FieldOffset(0)] public MouseInput Mouse; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public InputPayload Payload; }
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr window, int command);
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
  [DllImport("user32.dll")] static extern void keybd_event(byte key, byte scan, uint flags, UIntPtr extra);
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
  public static void Key(byte key) {
    keybd_event(key, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(50);
    keybd_event(key, 0, 2, UIntPtr.Zero);
  }
  public static uint ProcessAt(int x, int y) {
    uint processId;
    GetWindowThreadProcessId(WindowFromPoint(new Point { X = x, Y = y }), out processId);
    return processId;
  }
  public static bool Click(IntPtr window, int x, int y, uint expectedPid) {
    if (!Focus(window) || ProcessAt(x, y) != expectedPid || !SetCursorPos(x - 1, y)) return false;
    mouse_event(0x0001, 1, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    if (ProcessAt(x, y) != expectedPid) return false;
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { Payload = new InputPayload { Mouse = new MouseInput { Flags = 2 } } };
    var up = new Input { Payload = new InputPayload { Mouse = new MouseInput { Flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return false;
    System.Threading.Thread.Sleep(80);
    return SendInput(1, new[] { up }, size) == 1;
  }
}
"@
function Test-ScheduleActionAccepted {
  $until = (Get-Date).AddSeconds(2)
  do {
    $currentWindow = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
    $currentButton = if ($currentWindow) { $currentWindow.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Schedule retry' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1 }
    if ($currentButton -and -not $currentButton.Current.IsEnabled) {
      Write-Output 'SCHEDULE_ACTION_BUTTON_DISABLED'
      return $true
    }
    foreach ($action in (Get-ChildItem -LiteralPath $env:AC_RUN_DIR -Directory -Filter 'coordinator-ui-*' -ErrorAction SilentlyContinue)) {
      $ownerPath = Join-Path $action.FullName 'process/owner.json'
      if (-not (Test-Path -LiteralPath $ownerPath)) { continue }
      $owner = Get-Content -LiteralPath $ownerPath -Raw | ConvertFrom-Json
      if ($owner.coordinator_pid -eq [int]$env:AC_DESKTOP_PID -and $owner.phase -in @('launching', 'running', 'terminal')) {
        Write-Output 'SCHEDULE_ACTION_CONTROLLER_DISPATCHED'
        return $true
      }
    }
    Start-Sleep -Milliseconds 100
  } while ((Get-Date) -lt $until)
  return $false
}
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
$expectedProcessId = [uint32]$env:AC_DESKTOP_PID
$targetPid = [ScheduleWindow]::ProcessAt($x, $y)
if ($targetPid -ne $expectedProcessId) {
  Write-Output "SCHEDULE_TARGET_PROCESS_MISMATCH; expected=$expectedProcessId; target=$targetPid; point=$x,$y; bounds=$bounds"
  exit 10
}
$buttonFocused = $false
try { $schedule.SetFocus(); $buttonFocused = $schedule.Current.HasKeyboardFocus }
catch { Write-Output "SCHEDULE_UIA_BUTTON_FOCUS_UNAVAILABLE: $($_.Exception.Message)" }
if ($buttonFocused) {
  try {
    $schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    if (Test-ScheduleActionAccepted) {
      Write-Output 'SCHEDULE_CLICK_SENT; method=uia'
    } else {
      Write-Output 'SCHEDULE_UIA_INVOKE_NO_STATE_CHANGE'
      $buttonFocused = $false
    }
  } catch {
    Write-Output "SCHEDULE_UIA_INVOKE_UNAVAILABLE: $($_.Exception.Message)"
    $buttonFocused = $false
  }
}
if (-not $buttonFocused) {
  $keyboardPressed = $false
  try {
    $time.SetFocus()
    $focused = $A::FocusedElement
    if ($focused.Current.ProcessId -eq $expectedProcessId -and $focused.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit' -and $focused.Current.Name -eq 'Schedule time · ISO 8601 with timezone') {
      [ScheduleWindow]::Key(0x09)
      Start-Sleep -Milliseconds 100
      $focused = $A::FocusedElement
      if ($focused.Current.ProcessId -eq $expectedProcessId -and $focused.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' -and $focused.Current.Name -eq 'Schedule retry' -and $focused.Current.IsEnabled) {
        [ScheduleWindow]::Key(0x0D)
        if (Test-ScheduleActionAccepted) {
          $keyboardPressed = $true
          Write-Output 'SCHEDULE_CLICK_SENT; method=keyboard'
        } else {
          Write-Output 'SCHEDULE_KEYBOARD_NO_STATE_CHANGE'
        }
      } else {
        Write-Output "SCHEDULE_KEYBOARD_TARGET_UNAVAILABLE: name=$($focused.Current.Name); type=$($focused.Current.ControlType.ProgrammaticName); pid=$($focused.Current.ProcessId)"
      }
    }
  } catch {
    Write-Output "SCHEDULE_KEYBOARD_FALLBACK_UNAVAILABLE: $($_.Exception.Message)"
  }
  if (-not $keyboardPressed) {
    try {
      $schedule.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
      if (Test-ScheduleActionAccepted) {
        $keyboardPressed = $true
        Write-Output 'SCHEDULE_CLICK_SENT; method=uia-invoke-fallback'
      } else {
        Write-Output 'SCHEDULE_UIA_INVOKE_NO_STATE_CHANGE'
      }
    } catch {
      Write-Output "SCHEDULE_UIA_INVOKE_UNAVAILABLE: $($_.Exception.Message)"
    }
  }
  if (-not $keyboardPressed) {
    if (-not [ScheduleWindow]::Click($hwnd, $x, $y, $expectedProcessId)) {
      Write-Output "SCHEDULE_CLICK_FAILED; point=$x,$y; target=$targetPid"
      exit 12
    }
    Write-Output 'SCHEDULE_CLICK_SENT; method=foreground-checked-pointer'
  }
}
Start-Sleep -Milliseconds 750
Write-Output "SCHEDULE_AFTER: enabled=$($schedule.Current.IsEnabled); keyboard_focus=$($schedule.Current.HasKeyboardFocus); offscreen=$($schedule.Current.IsOffscreen); window_focus=$($window.Current.HasKeyboardFocus)"
$window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name } | Select-Object -First 80 | ForEach-Object { Write-Output "POST_ACTION: $($_.Current.Name)" }
if (-not (Test-ScheduleActionAccepted)) {
  Write-Output "SCHEDULE_ACTION_NOT_ACCEPTED; enabled=$($schedule.Current.IsEnabled); point=$x,$y; bounds=$bounds"
  exit 13
}
Write-Output 'SCHEDULE_ACTION_SUBMITTED'
"#;
