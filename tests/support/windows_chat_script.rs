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
function ChatEdit() {
  $edits = $window.FindAll($T::Descendants, $C) | Where-Object {
    $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit'
  }
  $named = $edits | Where-Object { $_.Current.Name -like 'Ask about this workspace*' } | Select-Object -First 1
  if ($named) { return $named }
  return $edits | Select-Object -Last 1
}
if ($env:AC_CHAT_MODE -in @('send', 'verify')) {
  $open = $null
  while (-not $open -and (Get-Date) -lt $deadline) {
    $open = Buttons 'Ask coordinator'
    if (-not $open) { Start-Sleep -Milliseconds 200 }
  }
  if (-not $open) { Write-Output 'NO_CHAT_LAUNCHER'; exit 3 }
  $open.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  if ($env:AC_CHAT_MODE -eq 'send') {
    $edit = $null
    while (-not $edit -and (Get-Date) -lt $deadline) {
      $edit = ChatEdit
      if (-not $edit) { Start-Sleep -Milliseconds 200 }
    }
    if (-not $edit) { Write-Output 'NO_CHAT_EDIT'; exit 4 }
    $edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue($env:AC_CHAT_QUESTION)
    $send = $null
    while (-not $send -and (Get-Date) -lt $deadline) {
      $send = Buttons 'Send'
      if (-not $send) { Start-Sleep -Milliseconds 200 }
    }
    if (-not $send) { Write-Output 'NO_CHAT_SEND'; exit 5 }
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
