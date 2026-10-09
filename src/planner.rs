use crate::{
    adapters::InputMode,
    git::Repository,
    plan::{AgentProfile, Plan, Task, WorkspaceMode},
    process::{self, ProcessSpec},
    protocol::{Decoder, EventKind, Provider},
};
use std::{fs, io::Read, path::Path, time::Duration};

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
    let mut args = profile.args.clone();
    if profile.permission_mode == "native_sandbox"
        && profile.executable.rsplit('/').next() == Some("codex")
    {
        if args != ["exec", "-", "--json", "--sandbox", "workspace-write"] {
            return Err("planner requires the verified native Codex argv template".into());
        }
        args[4] = "read-only".into();
    } else if profile.permission_mode != "read_only" {
        return Err("coordinator must have verified read-only capabilities".into());
    }
    let git = Repository::open(repo)?;
    git.clean()?;
    let base = git.head()?;
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
    let process = process::run(ProcessSpec {
        executable: profile.executable.clone().into(),
        args,
        cwd: git.root.clone(),
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
    .await?;
    if process.exit_code != Some(0) || process.timed_out {
        return Err("planning coordinator failed; logs retained".into());
    }
    if git.head()? != base || git.clean().is_err() {
        return Err("coordinator violated read-only repository contract".into());
    }
    let mut decoder = Decoder::new(profile.parser, 1024 * 1024);
    let mut bytes = [0; 8192];
    let mut file = fs::File::open(process.stdout).map_err(|e| e.to_string())?;
    let mut document = None;
    let mut opencode_text = String::new();
    let mut terminal = false;
    loop {
        let n = file.read(&mut bytes).map_err(|e| e.to_string())?;
        let batch = if n == 0 {
            decoder.finish()?
        } else {
            decoder.push(&bytes[..n])?
        };
        for event in batch {
            if event.kind == EventKind::Error {
                return Err("coordinator reported an error".into());
            }
            terminal |= event.kind == EventKind::Result;
            let value = match profile.parser {
                Provider::Claude => event
                    .payload
                    .get("structured_output")
                    .or_else(|| event.payload.get("result")),
                Provider::Codex => event.payload.pointer("/item/text"),
                Provider::OpenCode => event.payload.pointer("/part/text"),
                Provider::Custom => event.payload.get("payload"),
            };
            if let Some(value) = value {
                if matches!(profile.parser, Provider::OpenCode)
                    && let Some(text) = value.as_str()
                {
                    opencode_text.push_str(text);
                }
                if value.get("tasks").is_some() {
                    document = Some(value.clone());
                } else if let Some(text) = value.as_str()
                    && let Ok(json) = serde_json::from_str::<serde_json::Value>(text)
                    && json.get("tasks").is_some()
                {
                    document = Some(json);
                }
            }
        }
        if n == 0 {
            break;
        }
    }
    if matches!(profile.parser, Provider::OpenCode) && !opencode_text.is_empty() {
        document = serde_json::from_str(&opencode_text).ok();
        terminal = true;
    }
    if !terminal {
        return Err("coordinator did not return a successful final response".into());
    }
    let document = document.ok_or("coordinator response contains no valid JSON task plan")?;
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
