pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName WindowsBase
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
  foreach ($window in $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq $targetPid
  }) {
    $button = $window.FindAll($T::Descendants, $C) | Where-Object {
      $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'
    } | Select-Object -First 1
    if ($button) { $script:window = $window; return $button }
  }
  return $null
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
function Find-CurrentButton($name) {
  $modalButton = Find-ModalButton $name
  if ($modalButton) { return $modalButton }
  return Find-Button $name
}
function Wait-EnabledButton($name) {
  $until = (Get-Date).AddSeconds(20)
  while ((Get-Date) -lt $until) {
    $button = Find-CurrentButton $name
    if ($button -and $button.Current.IsEnabled) { return $button }
    Start-Sleep -Milliseconds 100
  }
  $button = Find-CurrentButton $name
  $enabled = if ($button) { $button.Current.IsEnabled } else { 'missing' }
  throw "ACTION_BUTTON_NOT_ENABLED: name=$name enabled=$enabled"
}
$actionStateTimeoutMs = 4000
function Wait-ActionState($expected, $visible) {
  $until = (Get-Date).AddMilliseconds($actionStateTimeoutMs)
  while ((Get-Date) -lt $until) {
    if (($null -ne (Find-Button $expected)) -eq $visible) { return $true }
    Start-Sleep -Milliseconds 50
  }
  return (($null -ne (Find-Button $expected)) -eq $visible)
}
function PointHitsButton($x, $y, $name) {
  $point = [System.Windows.Point]::new([double]$x, [double]$y)
  $element = $A::FromPoint($point)
  $walker = [System.Windows.Automation.TreeWalker]::ControlViewWalker
  for ($depth = 0; $element -and $depth -lt 8; $depth++) {
    if ($element.Current.ProcessId -ne $targetPid) { return $false }
    if ($element.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' -and $element.Current.Name -eq $name) { return $true }
    $element = $walker.GetParent($element)
  }
  return $false
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
    $button = Find-CurrentButton $name
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
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int x, y; public uint data, flags, time; public IntPtr extraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct InputPayload { [FieldOffset(0)] public MouseInput mouse; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint type; public InputPayload payload; }
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr window, int command);
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
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
    if (!SetCursorPos(x - 1, y)) return "CURSOR_POSITION_FAILED: error=" + Marshal.GetLastWin32Error() + " x=" + x + " y=" + y;
    mouse_event(0x0001, 1, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    pointProcessId = ProcessAt(x, y);
    if (pointProcessId != expectedPid) return "POINT_NOT_OWNED_AFTER_CURSOR_MOVE: pid=" + pointProcessId + " expected=" + expectedPid + " x=" + x + " y=" + y;
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { payload = new InputPayload { mouse = new MouseInput { flags = 2 } } };
    var up = new Input { payload = new InputPayload { mouse = new MouseInput { flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return "SEND_INPUT_DOWN_FAILED: error=" + Marshal.GetLastWin32Error();
    System.Threading.Thread.Sleep(80);
    if (!ForegroundBelongsTo(expectedPid)) return "FOREGROUND_CHANGED: pid=" + ForegroundProcessId() + " expected=" + expectedPid;
    if (SendInput(1, new[] { up }, size) != 1) return "SEND_INPUT_UP_FAILED: error=" + Marshal.GetLastWin32Error();
    return "";
  }
}
"@
function Activate($button, $expected, $visible) {
  $name = $button.Current.Name
  $button = Ensure-ButtonInsideWindow $name
  $button = Wait-EnabledButton $name
  if (($null -ne (Find-Button $expected)) -eq $visible) { return }
  $point = $button.GetClickablePoint()
  $x = [int][Math]::Round($point.X)
  $y = [int][Math]::Round($point.Y)
  if (-not (PointHitsButton $x $y $name)) {
    throw "POINTER_NOT_ON_TARGET_BUTTON: name=$name x=$x y=$y"
  }
  $hwnd = [IntPtr]$script:window.Current.NativeWindowHandle
  $clickFailure = [ToolDecisionMouse]::Click($hwnd, $x, $y, [uint32]$targetPid)
  if ($clickFailure) { throw "CHECKED_POINTER_CLICK_FAILED: $clickFailure" }
  Write-Output "ACTION_POINTER_CLICKED: name=$name x=$x y=$y"
  if (Wait-ActionState $expected $visible) {
    Write-Output "ACTION_STATE_CONFIRMED_AFTER_POINTER: expected=$expected visible=$visible"
    return
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
