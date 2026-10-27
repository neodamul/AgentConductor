#![cfg(windows)]

use agentconductor::{adapters, gates::Ledger, protocol::Provider, workflow};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

#[path = "support/workflow.rs"]
mod support;

const SCRIPT: &str = r#"
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
  public static bool Click(IntPtr window, int x, int y, uint expectedPid) {
    SetForegroundWindow(window);
    if (GetForegroundWindow() != window || ProcessAt(x, y) != expectedPid || !SetCursorPos(x, y)) return false;
    mouse_event(0x0001, 0, 0, 0, UIntPtr.Zero);
    System.Threading.Thread.Sleep(100);
    if (GetForegroundWindow() != window) return false;
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

struct ChildProcess(Child);

impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

struct TestDirectory(PathBuf);

impl std::ops::Deref for TestDirectory {
    type Target = PathBuf;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
#[ignore = "Runs only against the exact packaged binary."]
fn exact_windows_package_refreshes_approves_and_rejects_tool_requests() {
    decide_with_package("refresh-approve", true);
    decide_with_package("reject", false);
}

fn decide_with_package(action: &str, allow: bool) {
    let (root, run, mut worker) = start_guarded_run();
    let initial = wait_for_request(&run, &mut worker, None);
    let home = root.join("home");
    let binary = std::env::var_os("AGENTCONDUCTOR_DESKTOP_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_agentconductor")));
    let app = ChildProcess(
        Command::new(binary)
            .args(["ui", run.to_str().unwrap()])
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );

    let request = if action == "refresh-approve" {
        drive_ui(app.0.id(), &initial.id, "refresh");
        wait_for_status(&run, &initial.id, "superseded");
        let fresh = wait_for_request(&run, &mut worker, Some(&initial.id));
        drive_ui(app.0.id(), &fresh.id, "allow");
        fresh
    } else {
        drive_ui(app.0.id(), &initial.id, "deny");
        initial
    };

    assert!(worker.0.wait().unwrap().success(), "agent run failed");
    let ledger = Ledger::open(&run.join("state.sqlite")).unwrap();
    let receipt = ledger.receipt(&request.id).unwrap().unwrap();
    let report = workflow::load(&run).unwrap();
    let repo = root.join("repo");
    assert_eq!(receipt.status, if allow { "consumed" } else { "denied" });
    assert_eq!(repo.join("approval-marker").exists(), allow);
    assert_eq!(repo.join("denied-marker").exists(), !allow);
    assert_eq!(report.tasks[0].attempt, 1);
    assert_eq!(report.tasks[0].status, "completed");
    assert!(repo.join("one.txt").exists());
    drop(app);
}

fn start_guarded_run() -> (TestDirectory, PathBuf, ChildProcess) {
    let (root, mut plan) = support::setup("shared");
    let root = TestDirectory(root);
    let executable = root.join(if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    });
    fs::copy(support::fixture(), &executable).unwrap();
    let mut profile = plan.agents[0].clone();
    profile.id = "guarded".into();
    profile.executable = executable.to_string_lossy().into_owned();
    profile.parser = Provider::Claude;
    profile.args = adapters::invocation(Provider::Claude, None, "unused")
        .unwrap()
        .args;
    profile.permission_mode = "native_callback".into();
    plan.agents.push(profile);
    plan.tasks.truncate(1);
    plan.tasks[0].agent = "guarded".into();
    plan.tasks[0]
        .paths
        .extend(["approval-marker".into(), "denied-marker".into()]);
    plan.tasks[0].test_command = vec!["cmd.exe".into(), "/C".into(), "exit".into(), "0".into()];
    let plan_path = root.join("plan.json");
    let bytes = serde_json::to_vec(&plan).unwrap();
    fs::write(&plan_path, &bytes).unwrap();
    let digest = agentconductor::workflow::digest(&bytes);
    let run = root.join("run");
    let worker = Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .env("AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS", "60")
        .args([
            "run",
            plan_path.to_str().unwrap(),
            root.join("repo").to_str().unwrap(),
            run.to_str().unwrap(),
            &digest,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    (root, run, ChildProcess(worker))
}

fn wait_for_request(
    run: &Path,
    worker: &mut ChildProcess,
    except: Option<&str>,
) -> agentconductor::gates::Receipt {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(ledger) = Ledger::open(&run.join("state.sqlite"))
            && let Some(receipt) =
                ledger.requests().unwrap().into_iter().find(|receipt| {
                    receipt.status == "pending" && Some(receipt.id.as_str()) != except
                })
        {
            return receipt;
        }
        if let Some(status) = worker.0.try_wait().unwrap() {
            panic!("worker exited before a tool request became pending: {status}");
        }
        assert!(
            Instant::now() < deadline,
            "tool request did not become pending"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn wait_for_status(run: &Path, id: &str, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if Ledger::open(&run.join("state.sqlite"))
            .and_then(|ledger| ledger.receipt(id))
            .ok()
            .flatten()
            .is_some_and(|receipt| receipt.status == expected)
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "tool receipt did not become {expected}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn drive_ui(pid: u32, id: &str, action: &str) {
    let output = Command::new("powershell.exe")
        .env("AC_DESKTOP_PID", pid.to_string())
        .env("AC_TOOL_REQUEST_ID", id)
        .env("AC_TOOL_ACTION", action)
        .args(["-NoProfile", "-NonInteractive", "-Command", SCRIPT])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success() && stdout.contains("TOOL_DECISION_SUBMITTED"),
        "UI Automation failed: stdout: {stdout}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
