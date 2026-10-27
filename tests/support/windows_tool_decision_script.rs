pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadline = (Get-Date).AddSeconds(40)
$targetPid = [int]$env:AC_DESKTOP_PID
$window = $null
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq $targetPid -and $_.Current.Name -like 'AgentConductor*'
  } | Select-Object -First 1
  if (-not $window) { Start-Sleep -Milliseconds 250 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
function Find-Button($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'
  } | Select-Object -First 1
}
function Dump-Controls {
  $window.FindAll($T::Descendants, $C) | ForEach-Object {
    Write-Output "$($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name) | enabled=$($_.Current.IsEnabled) | offscreen=$($_.Current.IsOffscreen)"
  }
}
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ToolDecisionMouse {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int x, y; }
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  public static uint ProcessAt(int x, int y) {
    uint processId;
    GetWindowThreadProcessId(WindowFromPoint(new Point { x = x, y = y }), out processId);
    return processId;
  }
  static bool ForegroundBelongsTo(uint expectedPid) {
    uint foregroundProcessId;
    GetWindowThreadProcessId(GetForegroundWindow(), out foregroundProcessId);
    return foregroundProcessId == expectedPid;
  }
  static bool Focus(IntPtr window, uint expectedPid) {
    SetForegroundWindow(window);
    var deadline = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < deadline && !ForegroundBelongsTo(expectedPid)) System.Threading.Thread.Sleep(50);
    return ForegroundBelongsTo(expectedPid);
  }
  public static bool Click(IntPtr window, int x, int y, uint expectedPid) {
    if (!Focus(window, expectedPid) || ProcessAt(x, y) != expectedPid || !SetCursorPos(x, y)) return false;
    mouse_event(0x0001, 0, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    if (!ForegroundBelongsTo(expectedPid)) return false;
    mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    return true;
  }
}
"@
function Activate($button, $expected, $visible) {
  try { $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() } catch {}
  $until = (Get-Date).AddSeconds(2)
  while ((Get-Date) -lt $until) {
    if (($null -ne (Find-Button $expected)) -eq $visible) { return }
    Start-Sleep -Milliseconds 100
  }
  $bounds = $button.Current.BoundingRectangle
  if ($button.Current.IsOffscreen -or $bounds.IsEmpty) { throw 'BUTTON_NOT_VISIBLE' }
  $x = [int]($bounds.Left + $bounds.Width / 2)
  $y = [int]($bounds.Top + $bounds.Height / 2)
  $hwnd = [IntPtr]$window.Current.NativeWindowHandle
  if (-not [ToolDecisionMouse]::Click($hwnd, $x, $y, [uint32]$targetPid)) {
    throw 'CHECKED_POINTER_CLICK_FAILED'
  }
}
$id = $env:AC_TOOL_REQUEST_ID
$action = $env:AC_TOOL_ACTION
$verb = switch ($action) {
  'allow' { 'Allow once' }
  'deny' { 'Deny' }
  'refresh' { 'Refresh request' }
  default { Write-Output "INVALID_ACTION: $action"; exit 3 }
}
$actionName = "$verb tool request $id"
$button = $null
while (-not $button -and (Get-Date) -lt $deadline) {
  $button = Find-Button $actionName
  if (-not $button) { Start-Sleep -Milliseconds 200 }
}
if (-not $button) { Write-Output "NO_ACTION_BUTTON: $actionName"; Dump-Controls; exit 4 }
try { Activate $button 'Confirm tool decision' $true } catch { Write-Output "ACTION_FAILED: $_"; Dump-Controls; exit 5 }
$confirm = $null
while (-not $confirm -and (Get-Date) -lt $deadline) {
  $confirm = Find-Button 'Confirm tool decision'
  if (-not $confirm) { Start-Sleep -Milliseconds 100 }
}
if (-not $confirm) { Write-Output 'NO_CONFIRMATION_BUTTON'; Dump-Controls; exit 6 }
try { Activate $confirm 'Confirm tool decision' $false } catch { Write-Output "CONFIRM_FAILED: $_"; Dump-Controls; exit 7 }
$settled = $false
while (-not $settled -and (Get-Date) -lt $deadline) {
  $settled = $null -eq (Find-Button 'Confirm tool decision')
  if (-not $settled) { Start-Sleep -Milliseconds 200 }
}
if (-not $settled) { Write-Output 'CONFIRMATION_DID_NOT_CLOSE'; Dump-Controls; exit 8 }
Write-Output 'TOOL_DECISION_SUBMITTED'
"#;
