pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition

Add-Type @"
using System;
using System.Runtime.InteropServices;
using System.Threading;
public static class CheckedGoalInput {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int X, Y; public uint Data, Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Sequential)] struct KeyboardInput { public ushort Vk, Scan; public uint Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct Payload { [FieldOffset(0)] public MouseInput Mouse; [FieldOffset(0)] public KeyboardInput Keyboard; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public Payload Payload; }
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
  static uint ProcessId(IntPtr window) { uint pid; GetWindowThreadProcessId(window, out pid); return pid; }
  static Input Key(ushort vk, ushort scan, uint flags) { return new Input { Type = 1, Payload = new Payload { Keyboard = new KeyboardInput { Vk = vk, Scan = scan, Flags = flags } } }; }
  public static string ClickAndType(IntPtr window, int x, int y, uint expectedPid, string text) {
    if (ProcessId(window) != expectedPid) return "WINDOW_PID_MISMATCH";
    SetForegroundWindow(window);
    DateTime until = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < until && ProcessId(GetForegroundWindow()) != expectedPid) Thread.Sleep(50);
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_NOT_OWNED";
    if (ProcessId(WindowFromPoint(new Point { X=x, Y=y })) != expectedPid) return "POINT_NOT_OWNED";
    if (!SetCursorPos(x, y)) return "CURSOR_POSITION_FAILED";
    int size = Marshal.SizeOf(typeof(Input));
    Input down = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 2 } } };
    Input up = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return "MOUSE_DOWN_FAILED";
    Thread.Sleep(60);
    if (SendInput(1, new[] { up }, size) != 1) return "MOUSE_UP_FAILED";
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_CHANGED_AFTER_CLICK";
    Input[] keys = new Input[4 + text.Length * 2];
    keys[0] = Key(0x11, 0, 0); keys[1] = Key(0x41, 0, 0);
    keys[2] = Key(0x41, 0, 2); keys[3] = Key(0x11, 0, 2);
    for (int i = 0; i < text.Length; i++) {
      ushort code = text[i]; keys[4 + i * 2] = Key(0, code, 4); keys[5 + i * 2] = Key(0, code, 6);
    }
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_CHANGED_BEFORE_TYPE";
    if (SendInput((uint)keys.Length, keys, size) != keys.Length) return "KEY_INPUT_INCOMPLETE";
    return "";
  }
  public static string Click(IntPtr window, int x, int y, uint expectedPid) {
    if (ProcessId(window) != expectedPid) return "WINDOW_PID_MISMATCH";
    SetForegroundWindow(window);
    DateTime until = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < until && ProcessId(GetForegroundWindow()) != expectedPid) Thread.Sleep(50);
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_NOT_OWNED";
    if (ProcessId(WindowFromPoint(new Point { X=x, Y=y })) != expectedPid) return "POINT_NOT_OWNED";
    if (!SetCursorPos(x, y)) return "CURSOR_POSITION_FAILED";
    int size = Marshal.SizeOf(typeof(Input));
    Input down = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 2 } } };
    Input up = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return "MOUSE_DOWN_FAILED";
    Thread.Sleep(60);
    if (SendInput(1, new[] { up }, size) != 1) return "MOUSE_UP_FAILED";
    return ProcessId(GetForegroundWindow()) == expectedPid ? "" : "FOREGROUND_CHANGED_AFTER_CLICK";
  }
}
"@

function Set-Goal($goal, $window, $expected) {
  try {
    $goalPattern = $goal.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
    $goalPattern.SetValue($expected)
    $goalDeadline = (Get-Date).AddSeconds(5)
    while ($goalPattern.Current.Value -ne $expected -and (Get-Date) -lt $goalDeadline) { Start-Sleep -Milliseconds 100 }
    if ($goalPattern.Current.Value -eq $expected) { return }
  } catch { }
  $bounds = $goal.Current.BoundingRectangle
  $windowBounds = $window.Current.BoundingRectangle
  if ($bounds.IsEmpty -or $goal.Current.IsOffscreen -or $bounds.Left -lt $windowBounds.Left -or
      $bounds.Top -lt $windowBounds.Top -or $bounds.Right -gt $windowBounds.Right -or $bounds.Bottom -gt $windowBounds.Bottom) {
    Write-Output 'GOAL_CONTROL_OUTSIDE_WINDOW'; exit 8
  }
  $x = [int]($bounds.Left + $bounds.Width / 2)
  $y = [int]($bounds.Top + $bounds.Height / 2)
  $failure = [CheckedGoalInput]::ClickAndType([IntPtr]$window.Current.NativeWindowHandle, $x, $y, [uint32]$env:AC_DESKTOP_PID, $expected)
  if ($failure) { Write-Output "CHECKED_GOAL_INPUT_FAILED: $failure"; exit 9 }
  Write-Output 'GOAL_SET_BY_GUARDED_INPUT'
}
function Save-Workspace($save, $window) {
  try {
    $save.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    return
  } catch { }
  $bounds = $save.Current.BoundingRectangle
  $windowBounds = $window.Current.BoundingRectangle
  if ($bounds.IsEmpty -or $save.Current.IsOffscreen -or -not $save.Current.IsEnabled -or
      $bounds.Left -lt $windowBounds.Left -or $bounds.Top -lt $windowBounds.Top -or
      $bounds.Right -gt $windowBounds.Right -or $bounds.Bottom -gt $windowBounds.Bottom) {
    Write-Output 'SAVE_CONTROL_OUTSIDE_WINDOW_OR_DISABLED'; exit 10
  }
  $x = [int]($bounds.Left + $bounds.Width / 2)
  $y = [int]($bounds.Top + $bounds.Height / 2)
  $failure = [CheckedGoalInput]::Click([IntPtr]$window.Current.NativeWindowHandle, $x, $y, [uint32]$env:AC_DESKTOP_PID)
  if ($failure) { Write-Output "CHECKED_SAVE_CLICK_FAILED: $failure"; exit 11 }
  Write-Output 'SAVE_CLICKED_BY_GUARDED_INPUT'
}

$deadline = (Get-Date).AddSeconds(40)
$window = $null
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*' } | Select-Object -First 1
  if (-not $window) { Start-Sleep -Milliseconds 500 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
$firstName = $null; $create = $null; $empty = $null
while ((Get-Date) -lt $deadline -and -not ($firstName -and $create -and $empty)) {
  $all = $window.FindAll($T::Descendants, $C)
  foreach ($e in $all) {
    $n = $e.Current.Name
    if ($n -eq 'Name for a new workspace' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { $firstName = $e }
    if ($n -eq 'Create workspace' -and $e.Current.ControlType.ProgrammaticName -eq 'ControlType.Button') { $create = $e }
    if ($n -eq 'No saved workspaces') { $empty = $e }
  }
  if (-not ($firstName -and $create -and $empty)) { Start-Sleep -Milliseconds 500 }
}
if (-not $firstName -or -not $create -or -not $empty) { Write-Output 'FIRST_WORKSPACE_PROMPT_MISSING'; $window.FindAll($T::Descendants, $C) | Select-Object -First 40 | ForEach-Object { Write-Output $_.Current.Name }; exit 3 }
$namePattern = $firstName.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
$namePattern.SetValue('Windows UIA workspace')
$nameDeadline = (Get-Date).AddSeconds(5)
while ($namePattern.Current.Value -ne 'Windows UIA workspace' -and (Get-Date) -lt $nameDeadline) { Start-Sleep -Milliseconds 100 }
if ($namePattern.Current.Value -ne 'Windows UIA workspace') { Write-Output "NAME_VALUE_NOT_APPLIED; value=$($namePattern.Current.Value)"; exit 6 }
$createDeadline = (Get-Date).AddSeconds(5)
while ((Get-Date) -lt $createDeadline) {
  $create = $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name -eq 'Create workspace' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' } | Select-Object -First 1
  if ($create -and $create.Current.IsEnabled) { break }
  Start-Sleep -Milliseconds 100
}
if (-not $create -or -not $create.Current.IsEnabled) { Write-Output 'CREATE_WORKSPACE_BUTTON_NOT_ENABLED'; exit 7 }
$create.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
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
Set-Goal $goal $window 'Windows UIA goal'
Save-Workspace $save $window
Write-Output 'DRIVEN'
"#;
