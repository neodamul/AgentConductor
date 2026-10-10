use crate::{
    adapters::InputMode,
    git::Repository,
    plan::{AgentProfile, Plan, Task, WorkspaceMode},
    process::{self, ProcessSpec},
};
use std::{fs, path::Path, time::Duration};

pub async fn generate(
    goal: &str,
    profiles: &[AgentProfile],
    coordinator: &str,
    repo: &Path,
    dir: &Path,
) -> Result<Plan, String> {
    if goal.trim().is_empty() {
        return Err("goal must be nonempty".into());
    }
    let profile = profiles
        .iter()
        .find(|p| p.id == coordinator)
        .ok_or("coordinator is not registered")?;
    let mut args = crate::coordinator::planning_args(profile, "{task_file}")?;
    let git = Repository::open(repo)?;
    let _repository_lease = crate::leases::WorkspaceLease::acquire(&git.root, dir)?;
    git.clean()?;
    let base = git.head()?;
    let ignored = git.command(&[
        "status",
        "--porcelain",
        "--ignored",
        "--untracked-files=all",
    ])?;
    fs::create_dir(dir).map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    if dir.starts_with(&git.root) {
        return Err("planning artifacts must be outside repository".into());
    }
    let ids: Vec<_> = profiles.iter().map(|p| &p.id).collect();
    let prompt = format!(
        "Goal: {goal}\n\nRead the codebase and propose exactly three scoped tasks. Do not edit files, execute commands, transmit data externally, or read secrets.\nRegistered agent IDs: {}\nReturn only a JSON object with a tasks array. Each task must have id, agent (registered ID), prompt (inputs, expected output and completion conditions), paths (literal disjoint repo-relative paths), depends_on (task IDs), and test_command (executable plus argv). No API keys or executable definitions.\n",
        serde_json::to_string(&ids).map_err(|e| e.to_string())?
    );
    let task_file = dir.join("prompt.txt");
    fs::write(&task_file, &prompt).map_err(|e| e.to_string())?;
    for arg in &mut args {
        *arg = arg.replace(
            "{task_file}",
            task_file.to_str().ok_or("non-UTF8 task path")?,
        );
    }
    // The coordinator never runs in the target repository; see DD-33.
    let snapshot = dir.join("snapshot");
    git.export_tree(&base, &snapshot)?;
    let process = process::run(ProcessSpec {
        executable: profile.executable.clone().into(),
        args,
        cwd: snapshot.clone(),
        input: if profile.input_mode == InputMode::Stdin {
            prompt.into_bytes()
        } else {
            vec![]
        },
        log_dir: dir.join("process"),
        timeout: Duration::from_secs(1800),
        signal_grace: Duration::from_secs(5),
        environment: profile.environment.clone(),
    })
    .await;
    let removed = fs::remove_dir_all(&snapshot);
    let process = process?;
    removed.map_err(|e| format!("planning snapshot cleanup failed: {e}"))?;
    if process.exit_code != Some(0) || process.timed_out {
        return Err("planning coordinator failed; logs retained".into());
    }
    if git.head()? != base
        || git.clean().is_err()
        || git.command(&[
            "status",
            "--porcelain",
            "--ignored",
            "--untracked-files=all",
        ])? != ignored
    {
        return Err("coordinator violated read-only repository contract".into());
    }
    let document = crate::coordinator::read_document(profile.parser, &process.stdout, "tasks")?;
    let tasks: Vec<Task> =
        serde_json::from_value(document.get("tasks").cloned().ok_or("missing tasks")?)
            .map_err(|e| e.to_string())?;
    let plan = Plan {
        goal: goal.into(),
        workspace_mode: WorkspaceMode::Shared,
        agents: profiles.to_vec(),
        tasks,
    };
    plan.validate()?;
    fs::write(
        dir.join("plan.json"),
        serde_json::to_vec_pretty(&plan).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(plan)
}
