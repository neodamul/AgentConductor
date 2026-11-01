use std::{
    env,
    process::{Command, ExitCode},
};

fn git(repo: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn check(repo: &str) -> Result<(), String> {
    if git(repo, &["rev-parse", "--is-shallow-repository"])? == "true" {
        return Err("The screenshot check needs the full Git history (fetch-depth: 0).".into());
    }
    let interface = git(
        repo,
        &["log", "-1", "--format=%H", "--", "src/native_ui_probe"],
    )?;
    if interface.is_empty() {
        return Ok(());
    }
    let captured = git(
        repo,
        &[
            "log",
            "-1",
            "--format=%H",
            "--",
            "docs/assets/ui-*-current.png",
            "docs/assets/screenshots-current.txt",
        ],
    )?;
    if captured.is_empty() {
        return Err("No captured screen exists under docs/assets/ui-*-current.png.".into());
    }
    if git(
        repo,
        &["merge-base", "--is-ancestor", &interface, &captured],
    )
    .is_ok()
    {
        return Ok(());
    }
    let changed = git(repo, &["show", "--name-only", "--format=", &interface])?
        .lines()
        .filter(|path| path.starts_with("src/native_ui_probe/") && path.ends_with(".rs"))
        .collect::<Vec<_>>()
        .join(", ");
    Err(format!(
        "The native UI changed in {} ({changed}) after screens were captured or confirmed in {}. Recapture docs/assets/ui-*-current.png, or commit docs/assets/screenshots-current.txt if the screens still match.",
        &interface[..interface.len().min(8)],
        &captured[..captured.len().min(8)]
    ))
}

fn main() -> ExitCode {
    let repo = env::args().nth(1).unwrap_or_else(|| ".".into());
    match check(&repo) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
