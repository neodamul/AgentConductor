use std::{
    env, fs,
    path::Path,
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
    let manifest = Path::new(repo).join("docs/assets/screenshots-current.txt");
    let current = fs::read_to_string(&manifest)
        .map_err(|error| format!("Cannot read current screenshot manifest: {error}"))?
        .lines()
        .filter_map(|line| line.strip_prefix("- "))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if current.is_empty() {
        return Err("The current screenshot manifest contains no image paths.".into());
    }
    for name in &current {
        if !Path::new(repo).join("docs/assets").join(name).is_file() {
            return Err(format!("Current screenshot docs/assets/{name} is missing."));
        }
    }
    let mut capture_paths = current
        .iter()
        .map(|name| format!("docs/assets/{name}"))
        .collect::<Vec<_>>();
    capture_paths.push("docs/assets/screenshots-current.txt".into());
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(["log", "-1", "--format=%H", "--"])
        .args(&capture_paths);
    let output = command.output().map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    let captured = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if captured.is_empty() {
        return Err("No current screenshot or manifest change exists in Git history.".into());
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
        "The native UI changed in {} ({changed}) after current screens were captured or confirmed in {}. Recapture the images listed in docs/assets/screenshots-current.txt, or update that manifest if the screens still match.",
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
