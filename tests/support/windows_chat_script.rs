pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadline = (Get-Date).AddSeconds(45)
$window = $null
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*'
  } | Select-Object -First 1
  if (-not $window) { Start-Sleep -Milliseconds 250 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
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
if ($env:AC_CHAT_MODE -in @('send', 'verify')) {
  $open = $null
  while (-not $open -and (Get-Date) -lt $deadline) {
    $open = Buttons 'Ask coordinator'
    if (-not $open) { Start-Sleep -Milliseconds 200 }
  }
  if (-not $open) { Write-Output 'NO_CHAT_LAUNCHER'; exit 3 }
  $open.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Start-Sleep -Milliseconds 200
  if ($env:AC_CHAT_MODE -eq 'send') {
    $edit = $null
    while (-not $edit -and (Get-Date) -lt $deadline) {
      $edit = ChatEdit
      if (-not $edit) { Start-Sleep -Milliseconds 200 }
    }
    if (-not $edit) { ChatControlDiagnostic 'NO_CHAT_EDIT'; exit 4 }
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
  $names = ($window.FindAll($T::Descendants, $C) | ForEach-Object { $_.Current.Name }) -join "`n"
  $visible = $names.Contains($env:AC_CHAT_QUESTION) -and $names.Contains($env:AC_CHAT_ANSWER)
  if (-not $visible) { Start-Sleep -Milliseconds 250 }
}
if (-not $visible) {
  Write-Output "CHAT_TRANSCRIPT_INCOMPLETE`n$names"
  exit 6
}
Write-Output 'CHAT_VISIBLE'
Write-Output $names
"#;
