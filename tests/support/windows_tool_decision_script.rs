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
  $script:window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'
  } | Select-Object -First 1
}
function Find-ModalButton($name) {
  foreach ($modal in $script:window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Window'
  }) {
    $buttons = @($modal.FindAll($T::Descendants, $C) | Where-Object {
      $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'
    })
    if ($buttons | Where-Object { $_.Current.Name -eq 'Decide later' }) {
      $target = $buttons | Where-Object { $_.Current.Name -eq $name } | Select-Object -First 1
      if ($target) { return $target }
    }
  }
  return $null
}
function Ensure-ButtonInsideWindow($name) {
  $hwnd = [IntPtr]$script:window.Current.NativeWindowHandle
  [ToolDecisionMouse]::Maximize($hwnd)
  if (-not [ToolDecisionMouse]::Focus($hwnd, [uint32]$targetPid)) { throw 'FOREGROUND_NOT_OWNED_BEFORE_SCROLL' }
  Start-Sleep -Milliseconds 300
  for ($attempt = 0; $attempt -lt 10; $attempt++) {
    $script:window = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
      $_.Current.ProcessId -eq $targetPid -and $_.Current.Name -like 'AgentConductor*'
    } | Select-Object -First 1
    $button = Find-Button $name
    if (-not $button) { throw "ACTION_BUTTON_DISAPPEARED: $name" }
    $bounds = $button.Current.BoundingRectangle
    $windowBounds = $script:window.Current.BoundingRectangle
    if (-not $bounds.IsEmpty -and -not $button.Current.IsOffscreen -and
        $bounds.Left -ge $windowBounds.Left -and $bounds.Top -ge $windowBounds.Top -and
        $bounds.Right -le $windowBounds.Right -and $bounds.Bottom -le $windowBounds.Bottom) {
      return $button
    }
    $x = [int][Math]::Max($windowBounds.Left + 20, [Math]::Min($bounds.Left + $bounds.Width / 2, $windowBounds.Right - 20))
    $y = [int]($windowBounds.Bottom - 48)
    $scrollFailure = [ToolDecisionMouse]::Scroll($x, $y, [uint32]$targetPid)
    if ($scrollFailure) { throw "SCROLL_TO_ACTION_FAILED: $scrollFailure; button=$bounds; window=$windowBounds" }
    Start-Sleep -Milliseconds 200
  }
  throw "BUTTON_OUTSIDE_WINDOW: $name; button=$bounds; window=$windowBounds; offscreen=$($button.Current.IsOffscreen)"
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
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr window, int command);
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  public static uint ProcessAt(int x, int y) {
    uint processId;
    GetWindowThreadProcessId(WindowFromPoint(new Point { x = x, y = y }), out processId);
    return processId;
  }
  public static void Maximize(IntPtr window) { ShowWindow(window, 3); }
  public static string Scroll(int x, int y, uint expectedPid) {
    if (!ForegroundBelongsTo(expectedPid)) return "FOREGROUND_NOT_OWNED";
    var pointProcessId = ProcessAt(x, y);
    if (pointProcessId != expectedPid) return "POINT_NOT_OWNED: pid=" + pointProcessId + " expected=" + expectedPid + " x=" + x + " y=" + y;
    if (!SetCursorPos(x, y)) return "CURSOR_POSITION_FAILED: error=" + Marshal.GetLastWin32Error();
    mouse_event(0x0800, 0, 0, unchecked((uint)-360), UIntPtr.Zero);
    return "";
  }
  static bool ForegroundBelongsTo(uint expectedPid) {
    uint foregroundProcessId;
    GetWindowThreadProcessId(GetForegroundWindow(), out foregroundProcessId);
    return foregroundProcessId == expectedPid;
  }
  static uint ForegroundProcessId() {
    uint processId;
    GetWindowThreadProcessId(GetForegroundWindow(), out processId);
    return processId;
  }
  public static bool Focus(IntPtr window, uint expectedPid) {
    SetForegroundWindow(window);
    var deadline = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < deadline && !ForegroundBelongsTo(expectedPid)) System.Threading.Thread.Sleep(50);
    return ForegroundBelongsTo(expectedPid);
  }
  public static string Click(IntPtr window, int x, int y, uint expectedPid) {
    if (!Focus(window, expectedPid)) return "FOREGROUND_NOT_OWNED: pid=" + ForegroundProcessId() + " hwnd=" + window;
    var pointProcessId = ProcessAt(x, y);
    if (pointProcessId != expectedPid) return "POINT_NOT_OWNED: pid=" + pointProcessId + " expected=" + expectedPid + " x=" + x + " y=" + y;
    if (!SetCursorPos(x, y)) return "CURSOR_POSITION_FAILED: error=" + Marshal.GetLastWin32Error() + " x=" + x + " y=" + y;
    mouse_event(0x0001, 0, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    if (!ForegroundBelongsTo(expectedPid)) return "FOREGROUND_CHANGED: pid=" + ForegroundProcessId() + " expected=" + expectedPid;
    mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);
    return "";
  }
}
"@
function Activate($button, $expected, $visible) {
  $invokeError = $null
  try { $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke() }
  catch { $invokeError = $_.Exception.Message }
  $until = (Get-Date).AddSeconds(2)
  while ((Get-Date) -lt $until) {
    if (($null -ne (Find-Button $expected)) -eq $visible) {
      Write-Output "ACTION_STATE_CONFIRMED_AFTER_UIA: expected=$expected visible=$visible"
      return
    }
    Start-Sleep -Milliseconds 100
  }
  Write-Output "ACTION_INVOKE_NO_STATE_CHANGE: name=$($button.Current.Name) expected=$expected visible=$visible invokeError=$invokeError"
  $button = Ensure-ButtonInsideWindow $button.Current.Name
  $bounds = $button.Current.BoundingRectangle
  $x = [int]($bounds.Left + $bounds.Width / 2)
  $y = [int]($bounds.Top + $bounds.Height / 2)
  $hwnd = [IntPtr]$script:window.Current.NativeWindowHandle
  $clickFailure = [ToolDecisionMouse]::Click($hwnd, $x, $y, [uint32]$targetPid)
  if ($clickFailure) { throw "CHECKED_POINTER_CLICK_FAILED: $clickFailure" }
  Write-Output "ACTION_POINTER_CLICKED: name=$($button.Current.Name) x=$x y=$y"
  $until = (Get-Date).AddSeconds(2)
  while ((Get-Date) -lt $until) {
    if (($null -ne (Find-Button $expected)) -eq $visible) {
      Write-Output "ACTION_STATE_CONFIRMED_AFTER_POINTER: expected=$expected visible=$visible"
      return
    }
    Start-Sleep -Milliseconds 100
  }
  throw "ACTION_STATE_NOT_VISIBLE_AFTER_POINTER: expected=$expected visible=$visible"
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
if ($action -eq 'refresh') {
  $later = Find-ModalButton 'Decide later'
  if ($later) { Activate $later 'Decide later' $false }
  $button = Find-Button $actionName
} else {
  $button = Find-ModalButton $actionName
}
while (-not $button -and (Get-Date) -lt $deadline) {
  if ($action -eq 'refresh') {
    $button = Find-Button $actionName
  } else {
    $button = Find-ModalButton $actionName
  }
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
