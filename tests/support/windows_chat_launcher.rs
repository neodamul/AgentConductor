use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const APP_SCRIPT: &str = r#"
param([string]$ConfigPath)
$ErrorActionPreference = 'Stop'
$config = Get-Content -Raw $ConfigPath | ConvertFrom-Json
$env:HOME = $config.home
$env:USERPROFILE = $config.home
$arguments = @('ui-new', ('"{0}"' -f $config.workspace))
$app = Start-Process -FilePath $config.binary -ArgumentList $arguments `
    -WorkingDirectory $config.workspace -PassThru `
    -RedirectStandardOutput $config.stdout -RedirectStandardError $config.stderr
[IO.File]::WriteAllText($config.pid_file, [string]$app.Id)
Wait-Process -Id $app.Id
"#;

const REGISTER_TASK: &str = r#"
$ErrorActionPreference = 'Stop'
$taskName = $env:AC_TEST_TASK_NAME
$scriptPath = $env:AC_TEST_SCRIPT
$configPath = $env:AC_TEST_CONFIG
$powershell = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
$arguments = '-NoProfile -ExecutionPolicy Bypass -File "' + $scriptPath + '" "' + $configPath + '"'
try {
    $action = New-ScheduledTaskAction -Execute $powershell -Argument $arguments
    $trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(5)
    $principal = New-ScheduledTaskPrincipal `
        -UserId ([Security.Principal.WindowsIdentity]::GetCurrent().Name) `
        -LogonType Interactive -RunLevel Limited
    Register-ScheduledTask -TaskName $taskName -Action $action `
        -Trigger $trigger -Principal $principal -Force | Out-Null
    Start-ScheduledTask -TaskName $taskName
    $deadline = (Get-Date).AddSeconds(20)
    while (-not (Test-Path $env:AC_TEST_PID_FILE) -and (Get-Date) -lt $deadline) {
        Start-Sleep -Milliseconds 100
    }
    if (-not (Test-Path $env:AC_TEST_PID_FILE)) { throw 'Packaged app did not start.' }
    Get-Content -Raw $env:AC_TEST_PID_FILE
} catch {
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue
    throw
}
"#;

const STOP_APP: &str = r#"
$ErrorActionPreference = 'SilentlyContinue'
Stop-Process -Id ([int]$env:AC_TEST_APP_PID) -Force
$deadline = (Get-Date).AddSeconds(10)
do {
    $task = Get-ScheduledTask -TaskName $env:AC_TEST_TASK_NAME
    if (-not $task -or $task.State -eq 'Ready') { break }
    Start-Sleep -Milliseconds 100
} while ((Get-Date) -lt $deadline)
Unregister-ScheduledTask -TaskName $env:AC_TEST_TASK_NAME -Confirm:$false
"#;

pub struct RunningApp {
    pid: u32,
    task_name: String,
    root: PathBuf,
    stopped: bool,
}

impl RunningApp {
    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn stderr_report(&mut self) -> String {
        self.stop();
        fs::read_to_string(self.root.join("app.stderr.log")).unwrap_or_default()
    }

    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        let _ = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", STOP_APP])
            .env("AC_TEST_APP_PID", self.pid.to_string())
            .env("AC_TEST_TASK_NAME", &self.task_name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        self.stopped = true;
    }
}

impl Drop for RunningApp {
    fn drop(&mut self) {
        self.stop();
    }
}

pub fn launch(binary: &Path, workspace: &Path) -> RunningApp {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let task_name = format!("AgentConductorChat-{}-{stamp}", std::process::id());
    let script = workspace.join("launch-packaged-app.ps1");
    let config = workspace.join("launch-packaged-app.json");
    let pid_file = workspace.join("packaged-app.pid");
    let stdout = workspace.join("app.stdout.log");
    let stderr = workspace.join("app.stderr.log");
    let _ = fs::remove_file(&pid_file);
    fs::write(&script, APP_SCRIPT).unwrap();
    fs::write(
        &config,
        serde_json::to_vec(&json!({
            "binary": binary.to_string_lossy(),
            "workspace": workspace.to_string_lossy(),
            "home": workspace.join("home").to_string_lossy(),
            "pid_file": pid_file.to_string_lossy(),
            "stdout": stdout.to_string_lossy(),
            "stderr": stderr.to_string_lossy()
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", REGISTER_TASK])
        .env("AC_TEST_TASK_NAME", &task_name)
        .env("AC_TEST_SCRIPT", &script)
        .env("AC_TEST_CONFIG", &config)
        .env("AC_TEST_PID_FILE", &pid_file)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "interactive scheduled task failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let pid = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .expect("scheduled task must return the packaged app PID");
    RunningApp {
        pid,
        task_name,
        root: workspace.to_path_buf(),
        stopped: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{APP_SCRIPT, REGISTER_TASK};

    #[test]
    fn launches_interactively_outside_the_test_runner_job() {
        assert!(REGISTER_TASK.contains("New-ScheduledTaskPrincipal"));
        assert!(REGISTER_TASK.contains("-LogonType Interactive"));
        assert!(APP_SCRIPT.contains("Start-Process -FilePath $config.binary"));
    }
}
