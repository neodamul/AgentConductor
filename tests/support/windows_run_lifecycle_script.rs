pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$targetPid = [int]$env:AC_DESKTOP_PID
$deadline = (Get-Date).AddSeconds(30)
function Find-Control($name, $type) {
  foreach ($window in $A::RootElement.FindAll($T::Children, $C)) {
    $top = $window.Current
    if ($top.ProcessId -ne $targetPid -or $top.IsOffscreen -or $top.BoundingRectangle.IsEmpty) { continue }
    foreach ($node in $window.FindAll($T::Descendants, $C)) {
      $current = $node.Current
      if ($current.Name -eq $name -and
          $current.ControlType.ProgrammaticName -eq "ControlType.$type" -and
          -not $current.IsOffscreen -and
          -not $current.BoundingRectangle.IsEmpty) { return $node }
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
  $windows = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq $targetPid
  } | ForEach-Object {
    "name=$($_.Current.Name); offscreen=$($_.Current.IsOffscreen); bounds=$($_.Current.BoundingRectangle)"
  }
  throw "CONTROL_NOT_FOUND: $name ($type); windows=$($windows -join ' | ')"
}
function Find-EditAfterLabel($name) {
  foreach ($window in $A::RootElement.FindAll($T::Children, $C)) {
    $top = $window.Current
    if ($top.ProcessId -ne $targetPid -or $top.IsOffscreen -or $top.BoundingRectangle.IsEmpty) { continue }
    $nodes = $window.FindAll($T::Descendants, $C)
    for ($i = 0; $i -lt $nodes.Count; $i++) {
      $current = $nodes[$i].Current
      if ($current.IsOffscreen -or $current.BoundingRectangle.IsEmpty -or $current.Name -ne $name) { continue }
      if ($current.ControlType.ProgrammaticName -eq 'ControlType.Edit') { return $nodes[$i] }
      for ($j = $i + 1; $j -lt $nodes.Count; $j++) {
        $candidate = $nodes[$j].Current
        if ($candidate.ControlType.ProgrammaticName -eq 'ControlType.Edit' -and
            -not $candidate.IsOffscreen -and -not $candidate.BoundingRectangle.IsEmpty) { return $nodes[$j] }
      }
    }
  }
  return $null
}
function Wait-EditAfterLabel($name) {
  while ((Get-Date) -lt $deadline) {
    $control = Find-EditAfterLabel $name
    if ($control) { return $control }
    Start-Sleep -Milliseconds 100
  }
  throw "CONTROL_NOT_FOUND: editable input after label $name"
}
function Wait-Enabled-Control($name, $type) {
  while ((Get-Date) -lt $deadline) {
    $control = Find-Control $name $type
    if ($control -and $control.Current.IsEnabled) { return $control }
    Start-Sleep -Milliseconds 100
  }
  throw "CONTROL_NOT_ENABLED: $name ($type)"
}
function Activate($name) {
  $button = Wait-Enabled-Control $name 'Button'
  $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
}
$action = $env:AC_RUN_LIFECYCLE_ACTION
if ($action -eq 'stop') {
  Activate 'Stop run'
  $reason = Wait-EditAfterLabel 'Stop reason'
  $reason.SetFocus()
  [System.Windows.Forms.SendKeys]::SendWait('^a')
  [System.Windows.Forms.SendKeys]::SendWait('Issue 9 packaged lifecycle verification')
  Activate 'Confirm stop'
  Wait-Control 'Resume run' 'Button' | Out-Null
} elseif ($action -eq 'resume') {
  Activate 'Resume run'
} else {
  throw "UNKNOWN_ACTION: $action"
}
Write-Output "RUN_LIFECYCLE_ACTION_SUBMITTED:$action"
"#;
