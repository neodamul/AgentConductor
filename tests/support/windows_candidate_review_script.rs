pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadline = (Get-Date).AddSeconds(60)
$window = $null
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*'
  } | Select-Object -First 1
  if (-not $window) { Start-Sleep -Milliseconds 250 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
function Nodes { $window.FindAll($T::Descendants, $C) }
function Text-Content {
  $parts = @()
  foreach ($node in (Nodes)) {
    if ($node.Current.Name) { $parts += $node.Current.Name }
    try {
      $value = $node.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value
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
Wait-Text 'exit code 0'
Wait-Text 'integration tests passed'
if ($env:AC_DECISION -eq 'reject') {
  Invoke-Button 'Reject candidate'
  $reason = $null
  while (-not $reason -and (Get-Date) -lt $deadline) {
    $reason = Nodes | Where-Object {
      $_.Current.Name -eq 'Rejection reason' -and $_.Current.ControlType.ProgrammaticName -eq 'ControlType.Edit'
    } | Select-Object -First 1
    if (-not $reason) { Start-Sleep -Milliseconds 200 }
  }
  if (-not $reason) { Write-Output 'REJECTION_REASON_MISSING'; exit 5 }
  $reason.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue('Reviewed exact diff; reject fixture')
  Invoke-Button 'Confirm candidate decision'
  Wait-Text 'Rejected'
} else {
  Invoke-Button 'Approve verified candidate'
  Invoke-Button 'Confirm candidate decision'
  $merge = $null
  $mergeUntil = (Get-Date).AddSeconds(20)
  while (-not $merge -and (Get-Date) -lt $mergeUntil) {
    $merge = Find-Button 'Merge approved candidate'
    if ($merge -and -not $merge.Current.IsEnabled) { $merge = $null }
    if (-not $merge) { Start-Sleep -Milliseconds 250 }
  }
  if (-not $merge) {
    Invoke-Button 'Review candidate'
    Wait-Text $env:AC_CANDIDATE_SHA
    $merge = Wait-Button 'Merge approved candidate'
  }
  $merge.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
  Invoke-Button 'Confirm candidate decision'
  Wait-Text 'Completed'
}
Write-Output 'CANDIDATE_DECISION_VERIFIED'
"#;
