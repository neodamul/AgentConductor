pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadline = (Get-Date).AddSeconds(60)
function Find-Window {
  $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*'
  } | Select-Object -First 1
}
$script:window = $null
while (-not $script:window -and (Get-Date) -lt $deadline) {
  $script:window = Find-Window
  if (-not $script:window) { Start-Sleep -Milliseconds 250 }
}
if (-not $script:window) { Write-Output 'NO_WINDOW'; exit 2 }
function Nodes {
  $script:window = Find-Window
  if (-not $script:window) { return }
  $script:window.FindAll($T::Descendants, $C)
}
function Text-Content {
  $parts = @()
  foreach ($node in (Nodes)) {
    if ($node.Current.Name) { $parts += $node.Current.Name }
    try {
      $value = $node.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
      if ($value) { $parts += $value }
    } catch {}
    try {
      $value = $node.GetCurrentPattern([System.Windows.Automation.TextPattern]::Pattern).DocumentRange.GetText(-1)
      if ($value) { $parts += $value }
    } catch {}
  }
  return $parts -join "`n"
}
function Find-Button($name) {
  Nodes | Where-Object {
    $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'
  } | Select-Object -First 1
}
function Wait-Button($name, $seconds = 25) {
  $until = (Get-Date).AddSeconds($seconds)
  while ((Get-Date) -lt $until) {
    $button = Find-Button $name
    if ($button -and $button.Current.IsEnabled -and -not $button.Current.IsOffscreen) { return $button }
    Start-Sleep -Milliseconds 200
  }
  Write-Output "BUTTON_NOT_READY: $name"
  Nodes | ForEach-Object { Write-Output "$($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name) | enabled=$($_.Current.IsEnabled) | offscreen=$($_.Current.IsOffscreen)" }
  exit 3
}
function Invoke-Button($name) {
  $button = Wait-Button $name
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Milliseconds 250
}
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class CandidateMouse {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int X, Y; public uint Data, Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct Payload { [FieldOffset(0)] public MouseInput Mouse; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public Payload Payload; }
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
  static uint ProcessId(IntPtr window) { uint pid; GetWindowThreadProcessId(window, out pid); return pid; }
  public static string Click(IntPtr window, int x, int y, uint expectedPid) {
    SetForegroundWindow(window);
    var until = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < until && ProcessId(GetForegroundWindow()) != expectedPid) System.Threading.Thread.Sleep(50);
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_NOT_OWNED";
    if (ProcessId(WindowFromPoint(new Point { X=x, Y=y })) != expectedPid) return "POINT_NOT_OWNED";
    if (!SetCursorPos(x, y)) return "CURSOR_POSITION_FAILED: " + Marshal.GetLastWin32Error();
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 2 } } };
    var up = new Input { Payload = new Payload { Mouse = new MouseInput { Flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return "SEND_INPUT_DOWN_FAILED: " + Marshal.GetLastWin32Error();
    System.Threading.Thread.Sleep(80);
    var foregroundOwned = ProcessId(GetForegroundWindow()) == expectedPid;
    if (SendInput(1, new[] { up }, size) != 1) return "SEND_INPUT_UP_FAILED: " + Marshal.GetLastWin32Error();
    if (!foregroundOwned) return "FOREGROUND_CHANGED";
    return "";
  }
}
"@
function Wait-ExpectedState($name, $expected, $seconds) {
  $until = (Get-Date).AddSeconds($seconds)
  while ((Get-Date) -lt $until) {
    if ($expected -and (Text-Content).Contains($expected)) { return 'expected' }
    if (-not (Find-Button $name)) { return 'button-closed' }
    Start-Sleep -Milliseconds 100
  }
  return 'unchanged'
}
function Activate-Button($name, $expected) {
  $button = Wait-Button $name
  try { $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch {}
  $state = Wait-ExpectedState $name $expected 2
  if ($state -eq 'expected') { Write-Output "ACTION_STATE_CONFIRMED_AFTER_UIA: $name"; return }
  if ($state -eq 'button-closed') {
    if ($expected -and (Text-Content).Contains($expected)) { Write-Output "ACTION_STATE_CONFIRMED_AFTER_UIA: $name"; return }
    if (-not $expected) { Write-Output "ACTION_DIALOG_CLOSED_AFTER_UIA: $name"; return }
    $until = (Get-Date).AddSeconds(12)
    while ((Get-Date) -lt $until) {
      if ((Text-Content).Contains($expected)) { Write-Output "ACTION_STATE_CONFIRMED_AFTER_UIA: $name"; return }
      Start-Sleep -Milliseconds 100
    }
    Write-Output "ACTION_CLOSED_WITHOUT_EXPECTED_STATE: $name"
    exit 6
  }
  $script:window = Find-Window
  $button = Wait-Button $name
  $bounds = $button.Current.BoundingRectangle
  $window = $script:window.Current.BoundingRectangle
  if ($bounds.IsEmpty -or $button.Current.IsOffscreen -or $bounds.Left -lt $window.Left -or
      $bounds.Top -lt $window.Top -or $bounds.Right -gt $window.Right -or $bounds.Bottom -gt $window.Bottom) {
    Write-Output "BUTTON_OUTSIDE_WINDOW: $name"
    exit 7
  }
  $x = [int]($bounds.Left + $bounds.Width / 2)
  $y = [int]($bounds.Top + $bounds.Height / 2)
  $failure = [CandidateMouse]::Click([IntPtr]$script:window.Current.NativeWindowHandle, $x, $y, [uint32]$env:AC_DESKTOP_PID)
  if ($failure) { Write-Output "CHECKED_POINTER_CLICK_FAILED: $failure"; exit 8 }
  $state = Wait-ExpectedState $name $expected 12
  if (-not $expected -and $state -eq 'button-closed') {
    Write-Output "ACTION_DIALOG_CLOSED_AFTER_POINTER: $name"
    return
  }
  if ($expected -and ($state -eq 'expected' -or (Text-Content).Contains($expected))) {
    Write-Output "ACTION_STATE_CONFIRMED_AFTER_POINTER: $name"
    return
  }
  Write-Output "ACTION_STATE_NOT_VISIBLE: $name expected=$expected state=$state"
  Nodes | ForEach-Object { Write-Output "$($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name) | enabled=$($_.Current.IsEnabled) | offscreen=$($_.Current.IsOffscreen)" }
  exit 9
}
function Wait-Text($value) {
  while ((Get-Date) -lt $deadline) {
    $text = Text-Content
    if ($text.Contains($value)) { return }
    Start-Sleep -Milliseconds 250
  }
  Write-Output "TEXT_NOT_FOUND: $value"
  Nodes | ForEach-Object { Write-Output "$($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name)" }
  exit 4
}
Invoke-Button 'Review candidate'
Wait-Text $env:AC_CANDIDATE_SHA
Wait-Text 'one.txt'
Wait-Text '+implemented'
Wait-Text 'exit code 0'
Wait-Text 'integration tests passed'
if ($env:AC_DECISION -eq 'reject') {
  Activate-Button 'Reject candidate' 'Reject this candidate and send feedback?'
  $reason = $null
  while (-not $reason -and (Get-Date) -lt $deadline) {
    $reason = Nodes | Where-Object {
      $_.Current.Name -eq 'Rejection reason' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit'
    } | Select-Object -First 1
    if (-not $reason) { Start-Sleep -Milliseconds 200 }
  }
  if (-not $reason) { Write-Output 'REJECTION_REASON_MISSING'; exit 5 }
  $reason.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue('Reviewed exact diff; reject fixture')
  Activate-Button 'Confirm candidate decision' $null
} else {
  Activate-Button 'Approve verified candidate' 'Confirm approval of this verified diff and tests?'
  Activate-Button 'Confirm candidate decision' $null
  Wait-Button 'Merge approved candidate'
  Activate-Button 'Merge approved candidate' 'Confirm integrating the exact approved candidate?'
  Activate-Button 'Confirm candidate decision' $null
}
Write-Output 'CANDIDATE_DECISION_VERIFIED'
"#;
