pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$targetPid = [int]$env:AC_DESKTOP_PID
$deadline = (Get-Date).AddSeconds(30)
function Find-Control($name, $type) {
  foreach ($window in $A::RootElement.FindAll($T::Children, $C)) {
    if ($window.Current.ProcessId -ne $targetPid) { continue }
    foreach ($node in $window.FindAll($T::Descendants, $C)) {
      if ($node.Current.Name -eq $name -and $node.Current.ControlType.ProgrammaticName -eq "ControlType.$type") { return $node }
    }
  }
  return $null
}
function Wait-Control($name, $type) {
  while ((Get-Date) -lt $deadline) {
    $control = Find-Control $name $type
    if ($control) { return $control }
    Start-Sleep -Milliseconds 100
  }
  throw "CONTROL_NOT_FOUND: $name ($type)"
}
function Activate($name) {
  $button = Wait-Control $name 'Button'
  if (-not $button.Current.IsEnabled) { throw "CONTROL_DISABLED: $name" }
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
$action = $env:AC_RUN_LIFECYCLE_ACTION
if ($action -eq 'stop') {
  Activate 'Stop run'
  $reason = Wait-Control 'Stop reason' 'Edit'
  $reason.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).SetValue('Issue 9 packaged lifecycle verification')
  Activate 'Confirm stop'
  Wait-Control 'Resume run' 'Button' | Out-Null
} elseif ($action -eq 'resume') {
  Activate 'Resume run'
} else {
  throw "UNKNOWN_ACTION: $action"
}
Write-Output "RUN_LIFECYCLE_ACTION_SUBMITTED:$action"
"#;
