pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadlineSeconds = 45
if ($env:AC_CHAT_MODE -eq 'verify') { $deadlineSeconds = 90 }
$deadline = (Get-Date).AddSeconds($deadlineSeconds)
$window = $null
function RefreshWindow() {
  $script:window = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*'
  } | Select-Object -First 1
  return $script:window
}
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = RefreshWindow
  if (-not $window) { Start-Sleep -Milliseconds 250 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ChatMouse {
  [StructLayout(LayoutKind.Sequential)] struct Point { public int X, Y; }
  [StructLayout(LayoutKind.Sequential)] struct MouseInput { public int X, Y; public uint Data, Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Sequential)] struct KeyboardInput { public ushort VirtualKey, Scan; public uint Flags, Time; public IntPtr ExtraInfo; }
  [StructLayout(LayoutKind.Explicit)] struct Payload { [FieldOffset(0)] public MouseInput Mouse; [FieldOffset(0)] public KeyboardInput Keyboard; }
  [StructLayout(LayoutKind.Sequential)] struct Input { public uint Type; public Payload Payload; }
  [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr window);
  [DllImport("user32.dll", SetLastError=true)] static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] static extern bool GetCursorPos(out Point point);
  [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(Point point);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);
  [DllImport("user32.dll", SetLastError=true)] static extern uint SendInput(uint count, Input[] inputs, int size);
  static uint ProcessId(IntPtr window) { uint pid; GetWindowThreadProcessId(window, out pid); return pid; }
  public static bool Focus(IntPtr window, uint expectedPid) {
    SetForegroundWindow(window);
    var until = DateTime.UtcNow.AddSeconds(3);
    while (DateTime.UtcNow < until && ProcessId(GetForegroundWindow()) != expectedPid) System.Threading.Thread.Sleep(50);
    return ProcessId(GetForegroundWindow()) == expectedPid;
  }
  public static string Key(ushort key, uint expectedPid) {
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_NOT_OWNED";
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { Type = 1, Payload = new Payload { Keyboard = new KeyboardInput { VirtualKey = key } } };
    var up = new Input { Type = 1, Payload = new Payload { Keyboard = new KeyboardInput { VirtualKey = key, Flags = 2 } } };
    if (SendInput(1, new[] { down }, size) != 1) return "SEND_INPUT_KEY_DOWN_FAILED: " + Marshal.GetLastWin32Error();
    System.Threading.Thread.Sleep(50);
    if (SendInput(1, new[] { up }, size) != 1) return "SEND_INPUT_KEY_UP_FAILED: " + Marshal.GetLastWin32Error();
    return "";
  }
  public static uint ProcessAt(int x, int y) { return ProcessId(WindowFromPoint(new Point { X = x, Y = y })); }
  public static string Position(IntPtr window, int x, int y, uint expectedPid) {
    if (!Focus(window, expectedPid)) return "FOREGROUND_NOT_OWNED";
    if (ProcessAt(x, y) != expectedPid) return "POINT_NOT_OWNED";
    if (!SetCursorPos(x, y)) return "CURSOR_POSITION_FAILED: " + Marshal.GetLastWin32Error();
    if (ProcessAt(x, y) != expectedPid) return "POINT_NOT_OWNED_AFTER_CURSOR_MOVE";
    return "";
  }
  public static string Click(uint expectedPid) {
    Point point;
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_NOT_OWNED";
    if (!GetCursorPos(out point)) return "CURSOR_POSITION_READ_FAILED: " + Marshal.GetLastWin32Error();
    if (ProcessAt(point.X, point.Y) != expectedPid) return "POINT_NOT_OWNED";
    var size = Marshal.SizeOf(typeof(Input));
    var down = new Input { Type = 0, Payload = new Payload { Mouse = new MouseInput { Flags = 2 } } };
    var up = new Input { Type = 0, Payload = new Payload { Mouse = new MouseInput { Flags = 4 } } };
    if (SendInput(1, new[] { down }, size) != 1) return "SEND_INPUT_DOWN_FAILED: " + Marshal.GetLastWin32Error();
    System.Threading.Thread.Sleep(80);
    if (ProcessId(GetForegroundWindow()) != expectedPid) return "FOREGROUND_CHANGED";
    if (SendInput(1, new[] { up }, size) != 1) return "SEND_INPUT_UP_FAILED: " + Marshal.GetLastWin32Error();
    return "";
  }
}
"@
function Buttons($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.Name -eq $name -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button'
  } | Select-Object -First 1
}
function EnabledButton($name) {
  $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.Name -eq $name -and
    $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' -and
    $_.Current.IsEnabled -and
    -not $_.Current.IsOffscreen -and
    -not $_.Current.BoundingRectangle.IsEmpty
  } | Select-Object -First 1
}
function ChatEdit() {
  $edits = $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit'
  }
  return $edits | Where-Object {
    $_.Current.Name -like 'Ask about this workspace*' -and
    $_.Current.IsEnabled -and
    -not $_.Current.IsOffscreen -and
    -not $_.Current.BoundingRectangle.IsEmpty
  } | Select-Object -First 1
}
function ChatEditAny() {
  return $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit' -and
    $_.Current.Name -like 'Ask about this workspace*' -and
    -not $_.Current.IsOffscreen -and
    -not $_.Current.BoundingRectangle.IsEmpty
  } | Select-Object -First 1
}
function IsExactButtonAtPoint($button, $point) {
  $hit = $A::FromPoint($point)
  $walker = $T::ControlViewWalker
  while ($hit) {
    $current = $hit.Current
    $runtimeId = $hit.GetRuntimeId()
    if ($current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and
        $current.Name -eq 'Ask coordinator' -and
        $current.ControlType.ProgrammaticName -eq 'ControlType.Button' -and
        $current.IsEnabled -and
        (($runtimeId -join ',') -eq ($button.GetRuntimeId() -join ','))) {
      return $true
    }
    $hit = $walker.GetParent($hit)
  }
  return $false
}
function WaitChatEdit($seconds) {
  $until = (Get-Date).AddSeconds($seconds)
  while ((Get-Date) -lt $until) {
    $window = RefreshWindow
    $edit = ChatEdit
    if ($edit) { return $edit }
    Start-Sleep -Milliseconds 100
  }
  return $null
}
function WaitChatEditAny($seconds) {
  $until = (Get-Date).AddSeconds($seconds)
  while ((Get-Date) -lt $until) {
    $window = RefreshWindow
    $edit = ChatEditAny
    if ($edit) { return $edit }
    Start-Sleep -Milliseconds 100
  }
  return $null
}
function ChatControlDiagnostic($label) {
  Write-Output "CHAT_CONTROL_DIAGNOSTIC:$label"
  $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.ControlType.ProgrammaticName -in @('ControlType.Button', 'ControlType.Edit') -and
    ($_.Current.Name -eq 'Send' -or $_.Current.Name -like 'Ask about this workspace*')
  } | ForEach-Object {
    $value = ''
    try {
      $value = $_.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
    } catch { }
    Write-Output "$($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name) | enabled=$($_.Current.IsEnabled) | offscreen=$($_.Current.IsOffscreen) | value=$value"
  }
}
function ChatWindowDiagnostic($label) {
  $bounds = $window.Current.BoundingRectangle
  Write-Output "CHAT_WINDOW:$label | pid=$($window.Current.ProcessId) | hwnd=$($window.Current.NativeWindowHandle) | name=$($window.Current.Name) | enabled=$($window.Current.IsEnabled) | offscreen=$($window.Current.IsOffscreen) | bounds=$bounds"
  $A::RootElement.FindAll($T::Children, $C) | Where-Object { $_.Current.Name } | Select-Object -First 40 | ForEach-Object {
    Write-Output "CHAT_DESKTOP_WINDOW: pid=$($_.Current.ProcessId) | hwnd=$($_.Current.NativeWindowHandle) | name=$($_.Current.Name) | enabled=$($_.Current.IsEnabled) | offscreen=$($_.Current.IsOffscreen) | bounds=$($_.Current.BoundingRectangle)"
  }
}
if ($env:AC_CHAT_MODE -in @('send', 'verify')) {
  $open = $null
  while (-not $open -and (Get-Date) -lt $deadline) {
    $window = RefreshWindow
    $open = Buttons 'Ask coordinator'
    if (-not $open) { Start-Sleep -Milliseconds 200 }
  }
  if (-not $open) {
    ChatWindowDiagnostic 'NO_CHAT_LAUNCHER'
    ChatControlDiagnostic 'NO_CHAT_LAUNCHER'
    $window.FindAll($T::Descendants, $C) | Where-Object { $_.Current.Name } | Select-Object -First 40 | ForEach-Object {
      Write-Output "CHAT_CONTROL: $($_.Current.ControlType.ProgrammaticName) | $($_.Current.Name) | enabled=$($_.Current.IsEnabled)"
    }
    exit 3
  }
  $open.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  if ($env:AC_CHAT_MODE -eq 'verify') {
    $edit = WaitChatEditAny 2
    if ($edit -and -not $edit.Current.IsEnabled) {
      Write-Output 'CHAT_RECONNECTED_IN_FLIGHT'
    }
  } else {
    $edit = WaitChatEdit 0.8
  }
  if (-not $edit -and $env:AC_CHAT_MODE -ne 'verify') {
    Write-Output 'CHAT_LAUNCHER_INVOKE_NO_STATE_CHANGE'
    $window = RefreshWindow
    $open = Buttons 'Ask coordinator'
    if ($open -and $open.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $open.Current.IsEnabled) {
      $hwnd = [IntPtr]$window.Current.NativeWindowHandle
      if ([ChatMouse]::Focus($hwnd, [uint32]$env:AC_DESKTOP_PID)) {
        try { $open.SetFocus() } catch {}
        $focused = $A::FocusedElement
        if ($focused.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and
            $focused.Current.Name -eq 'Ask coordinator' -and
            $focused.Current.ControlType.ProgrammaticName -eq 'ControlType.Button' -and
            $focused.Current.IsEnabled) {
          $keyFailure = [ChatMouse]::Key(0x0D, [uint32]$env:AC_DESKTOP_PID)
          if ($keyFailure) { Write-Output "CHAT_LAUNCHER_KEY_FALLBACK_REJECTED: $keyFailure" }
        } else {
          Write-Output "CHAT_LAUNCHER_KEY_TARGET_NOT_FOCUSED: name=$($focused.Current.Name) pid=$($focused.Current.ProcessId)"
        }
      } else {
        Write-Output 'CHAT_LAUNCHER_KEY_FOREGROUND_NOT_OWNED'
      }
      $edit = WaitChatEdit 1.5
    }
  }
  if (-not $edit -and $env:AC_CHAT_MODE -ne 'verify') {
    $window = RefreshWindow
    $open = Buttons 'Ask coordinator'
    if ($open -and $open.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and
        $open.Current.IsEnabled -and -not $open.Current.IsOffscreen -and
        -not $open.Current.BoundingRectangle.IsEmpty) {
      try { $point = $open.GetClickablePoint() } catch { $point = $null }
      if ($point -and (IsExactButtonAtPoint $open $point)) {
        $x = [int]$point.X
        $y = [int]$point.Y
        $positionFailure = [ChatMouse]::Position([IntPtr]$window.Current.NativeWindowHandle, $x, $y, [uint32]$env:AC_DESKTOP_PID)
        if ($positionFailure) {
          Write-Output "CHAT_LAUNCHER_CLICK_REJECTED: $positionFailure"
        } elseif (IsExactButtonAtPoint $open $point) {
          $clickFailure = [ChatMouse]::Click([uint32]$env:AC_DESKTOP_PID)
          if ($clickFailure) { Write-Output "CHAT_LAUNCHER_CLICK_REJECTED: $clickFailure" }
          $edit = WaitChatEdit 2
        } else {
          Write-Output 'CHAT_LAUNCHER_CLICK_TARGET_CHANGED'
        }
      } else {
        Write-Output 'CHAT_LAUNCHER_CLICK_TARGET_NOT_EXACT'
      }
    }
  }
  if (-not $edit) { ChatWindowDiagnostic 'CHAT_LAUNCHER_NOT_OPEN'; ChatControlDiagnostic 'CHAT_LAUNCHER_NOT_OPEN'; exit 8 }
  if ($env:AC_CHAT_MODE -eq 'send') {
    $window = RefreshWindow
    $edit.SetFocus()
    $editPattern = $edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
    $editPattern.SetValue($env:AC_CHAT_QUESTION)
    $editDeadline = (Get-Date).AddSeconds(5)
    while ($editPattern.Current.Value -ne $env:AC_CHAT_QUESTION -and (Get-Date) -lt $editDeadline) {
      Start-Sleep -Milliseconds 100
    }
    if ($editPattern.Current.Value -ne $env:AC_CHAT_QUESTION) {
      ChatControlDiagnostic 'CHAT_EDIT_VALUE_NOT_APPLIED'
      exit 7
    }
    $send = $null
    while (-not $send -and (Get-Date) -lt $deadline) {
      $window = RefreshWindow
      $send = EnabledButton 'Send'
      if (-not $send) { Start-Sleep -Milliseconds 200 }
    }
    if (-not $send) { ChatControlDiagnostic 'NO_CHAT_SEND'; exit 5 }
    $send.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
    Write-Output 'CHAT_SENT'
    if ($env:AC_CHAT_WAIT -eq '0') { exit 0 }
  }
}
$visible = $false
while (-not $visible -and (Get-Date) -lt $deadline) {
  $window = RefreshWindow
  $names = ($window.FindAll($T::Descendants, $C) | ForEach-Object { $_.Current.Name }) -join "`n"
  $visible = $names.Contains($env:AC_CHAT_QUESTION) -and $names.Contains($env:AC_CHAT_ANSWER)
  if (-not $visible) { Start-Sleep -Milliseconds 250 }
}
if (-not $visible) {
  Write-Output "CHAT_TRANSCRIPT_INCOMPLETE`n$names"
  exit 6
}
if ($env:AC_CHAT_MODE -eq 'verify') {
  $window = RefreshWindow
  $edit = ChatEdit
  if (-not $edit) { ChatControlDiagnostic 'CHAT_EDITOR_NOT_REENABLED_AFTER_COMPLETION'; exit 9 }
  Write-Output 'CHAT_RECONNECTED_AND_REENABLED'
}
Write-Output 'CHAT_VISIBLE'
Write-Output $names
"#;
