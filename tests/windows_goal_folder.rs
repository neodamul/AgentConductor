#![cfg(windows)]
//! #420: exercise native Explorer opening and permanent Goal deletion on Windows.
use serde_json::json;
use std::{fs, process::Command};
#[allow(dead_code)]
#[path = "support/goal_actions.rs"]
mod support;
const OBSERVE: &str = r#"
$shell = New-Object -ComObject Shell.Application
$expected = $env:AC_GOAL_FOLDER.TrimEnd('\')
$deadline = (Get-Date).AddSeconds(30)  # GitHub #716: the first Explorer window on a fresh CI runner can take more than 12 s
$seen = $false
do {
  $owned = @()
  foreach ($window in $shell.Windows()) {
    try {
      $path = $window.Document.Folder.Self.Path.TrimEnd('\')
      if ($path -ieq $expected) { $owned += $window }
    } catch {}
  }
  if (-not $seen -and $owned.Count -gt 0) {
    $seen = $true
    foreach ($window in $owned) { $window.Quit() }
  } elseif ($seen -and $owned.Count -eq 0) {
    Write-Output 'OWNED_FOLDER_OPENED'
    exit 0
  }
  Start-Sleep -Milliseconds 100
} while ((Get-Date) -lt $deadline)
Write-Error 'The owned Goal Explorer window did not appear'
exit 1
"#;
#[test]
fn explorer_opens_only_the_registered_goal_and_deletion_keeps_project_source() {
    let f = support::Fixture::new();
    let source = f.root.join("project/source.txt");
    fs::write(&source, "keep").unwrap();
    let before = f.snapshot();
    let folder = f
        .call(json!({"kind":"goal_folder","slug":"first_goal","open":true}))
        .unwrap();
    assert_eq!(folder["opened"], true);
    assert_eq!(folder["root"], json!(f.folder()));
    let plain = f
        .folder()
        .to_string_lossy()
        .trim_start_matches(r"\\?\")
        .to_owned();
    let observed = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Sta", "-Command", OBSERVE])
        .env("AC_GOAL_FOLDER", plain)
        .output()
        .unwrap();
    assert!(
        observed.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&observed.stdout),
        String::from_utf8_lossy(&observed.stderr)
    );
    assert!(String::from_utf8_lossy(&observed.stdout).contains("OWNED_FOLDER_OPENED"));
    assert_eq!(
        f.snapshot()["revision"],
        before["revision"],
        "opening a folder is read-only"
    );
    assert_eq!(f.mutate(f.delete()).unwrap()["deleted"], true);
    assert!(!f.folder().exists());
    assert_eq!(fs::read_to_string(source).unwrap(), "keep");
    assert_eq!(f.snapshot()["slugs"], json!([]));
}
