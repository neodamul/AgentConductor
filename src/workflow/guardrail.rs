//! DD-176 / DD-178: checks that decide before work starts whether a run can work. They only
//! resolve names; nothing is executed and nothing is written.
use crate::plan::Plan;
use serde_json::{Value, json};
use std::path::Path;

/// One reason a run cannot start: a code, the task it concerns (if any) and the detail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub code: &'static str,
    pub task: Option<String>,
    pub detail: String,
}

/// On PATH or the app's tool folders, an absolute path, or a path relative to the project.
pub fn resolvable(name: &str, root: &Path) -> bool {
    if name.is_empty() {
        return false;
    }
    crate::executable::resolve(name).is_some()
        || (name.contains(['/', '\\'])
            && !Path::new(name).is_absolute()
            && root.join(name).is_file())
}

/// A project-relative script a task is asked to write (inside its owned paths) does not exist yet.
fn created_by_a_task(plan: &Plan, program: &str) -> bool {
    let name = program.trim_start_matches("./");
    program.contains('/')
        && !Path::new(program).is_absolute()
        && plan.tasks.iter().flat_map(|t| &t.paths).any(|path| {
            let path = path.trim_start_matches("./").trim_end_matches('/');
            name == path || name.starts_with(&format!("{path}/"))
        })
}

/// The number of checks made and every violation found.
pub fn launch_gate(plan: &Plan, root: &Path) -> (usize, Vec<Violation>) {
    let (mut checks, mut violations) = (0, Vec::new());
    let mut agents: Vec<&str> = Vec::new();
    for task in &plan.tasks {
        if !agents.contains(&task.agent.as_str()) {
            agents.push(&task.agent);
        }
    }
    for id in agents {
        let Some(agent) = plan.agents.iter().find(|a| a.id == id) else {
            continue;
        };
        checks += 1;
        if !resolvable(&agent.executable, root) {
            violations.push(Violation {
                code: "tool_missing",
                task: None,
                detail: format!(
                    "agent {id}: executable {:?} was not found",
                    agent.executable
                ),
            });
        }
    }
    for task in &plan.tasks {
        checks += 1;
        let program = task.test_command.first().map_or("", String::as_str);
        if !resolvable(program, root) && !created_by_a_task(plan, program) {
            violations.push(Violation {
                code: "test_unrunnable",
                task: Some(task.id.clone()),
                detail: format!("test command {program:?} was not found"),
            });
        }
    }
    (checks, violations)
}

/// The message that refuses a run, with every violation.
pub fn refusal(violations: &[Violation]) -> String {
    let lines: Vec<String> = violations
        .iter()
        .map(|v| match &v.task {
            Some(task) => format!("[{}] task {task}: {}", v.code, v.detail),
            None => format!("[{}] {}", v.code, v.detail),
        })
        .collect();
    format!(
        "launch gate: the run cannot work yet; fix these and start again:\n{}",
        lines.join("\n")
    )
}

/// An amendment adds at most eight short entries of each kind.
pub fn bounded(add_paths: &[String], add_dependencies: &[String]) -> bool {
    add_paths.len() <= 8
        && add_dependencies.len() <= 8
        && add_paths
            .iter()
            .chain(add_dependencies)
            .all(|v| v.len() <= 1024 && !v.contains('\0'))
}

/// A validated amendment: the changed plan, what changed, and why it needs a person (if it does).
pub struct Amended {
    pub plan: Plan,
    pub changes: Vec<Value>,
    pub risk: Option<String>,
}

/// Names that hold secrets or keys; adding them to a task's scope always needs a person.
fn sensitive(path: &str) -> bool {
    path.to_ascii_lowercase().split('/').any(|part| {
        part.starts_with(".env")
            || matches!(
                part,
                ".ssh"
                    | ".aws"
                    | ".gnupg"
                    | ".npmrc"
                    | ".netrc"
                    | "secrets"
                    | "secret"
                    | "credentials"
                    | "id_rsa"
                    | "id_ed25519"
            )
            || [".pem", ".key", ".p12", ".pfx"]
                .iter()
                .any(|ext| part.ends_with(ext))
    })
}

/// A single-component path without an extension looks like a whole top-level directory.
fn broad(path: &str) -> bool {
    !path.contains('/') && !path.contains('.')
}

/// DD-179: validates `add_paths` and `add_dependencies` for one task. `Err` means the change
/// is unsafe, overlapping, unknown or cyclic; `risk` is set when it is valid but needs a person.
pub fn amend(
    plan: &Plan,
    task: usize,
    add_paths: &[String],
    add_dependencies: &[String],
) -> Result<Amended, String> {
    if add_paths.is_empty() && add_dependencies.is_empty() {
        return Err("an amendment adds at least one path or dependency".into());
    }
    let mut amended = plan.clone();
    let mut changes = Vec::new();
    let mut risks = Vec::new();
    for raw in add_paths {
        let path = raw
            .trim()
            .trim_start_matches("./")
            .trim_end_matches('/')
            .to_owned();
        if path.is_empty() {
            return Err("an added path must not be empty".into());
        }
        if sensitive(&path) {
            risks.push(format!("{path} is a sensitive path"));
        } else if broad(&path) {
            risks.push(format!("{path} would own a whole top-level directory"));
        }
        amended.tasks[task].paths.push(path.clone());
        changes.push(json!({"kind":"add_path","path":path}));
    }
    for id in add_dependencies {
        if !amended.tasks[task].depends_on.contains(id) {
            amended.tasks[task].depends_on.push(id.clone());
        }
        if *id == amended.tasks[task].id {
            return Err(format!("task {id} cannot depend on itself"));
        }
        changes.push(json!({"kind":"add_dependency","on":id}));
    }
    // Safe, non-overlapping scopes and known, acyclic dependencies are the plan's own rules.
    amended.validate()?;
    Ok(Amended {
        plan: amended,
        changes,
        risk: (!risks.is_empty()).then(|| risks.join("; ")),
    })
}
