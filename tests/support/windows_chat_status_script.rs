pub const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
$A = [System.Windows.Automation.AutomationElement]
$T = [System.Windows.Automation.TreeScope]
$C = [System.Windows.Automation.Condition]::TrueCondition
$deadline = (Get-Date).AddSeconds(5)
$window = $null
while (-not $window -and (Get-Date) -lt $deadline) {
  $window = $A::RootElement.FindAll($T::Children, $C) | Where-Object {
    $_.Current.ProcessId -eq [int]$env:AC_DESKTOP_PID -and $_.Current.Name -like 'AgentConductor*'
  } | Select-Object -First 1
  if (-not $window) { Start-Sleep -Milliseconds 100 }
}
if (-not $window) { Write-Output 'NO_WINDOW'; exit 2 }
$window.FindAll($T::Descendants, $C) | ForEach-Object { $_.Current.Name }
"#;
