//! DD-180 / DD-182: checks that decide before work starts whether a run can work. They only
//! resolve names; nothing is executed and nothing is written.
use crate::plan::Plan;
use serde_json::{Value, json};
use std::path::Path;

mod charter;
pub use charter::{Charter, charter_drift, record_charter_drift};

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

/// A validated amendment and the risk that requires human review, if any.
pub struct Amended {
    pub plan: Plan,
    pub changes: Vec<Value>,
    pub risk: Option<String>,
}

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

/// Validates coordinator-proposed paths and dependencies through the existing plan rules.
pub fn amend(
    plan: &Plan,
    task: usize,
    paths: &[String],
    dependencies: &[String],
) -> Result<Amended, String> {
    amend_contract(plan, Path::new("."), task, paths, dependencies, None)
}

/// Validates scoped additions and an optional replacement test command.
pub fn amend_contract(
    plan: &Plan,
    root: &Path,
    task: usize,
    paths: &[String],
    dependencies: &[String],
    test_command: Option<&[String]>,
) -> Result<Amended, String> {
    if task >= plan.tasks.len() {
        return Err("unknown task for amendment".into());
    }
    if paths.is_empty() && dependencies.is_empty() && test_command.is_none() {
        return Err("an amendment must change a path, dependency, or test command".into());
    }
    let mut amended = plan.clone();
    let mut changes = Vec::new();
    let mut risks = Vec::new();
    for raw in paths {
        let raw = raw.trim().trim_start_matches("./");
        let directory_scope = raw.ends_with('/');
        let path = raw
            .trim_start_matches("./")
            .trim_end_matches('/')
            .to_owned();
        if path.is_empty() {
            return Err("an added path must not be empty".into());
        }
        if sensitive(&path) {
            risks.push(format!("{path} is a sensitive path"));
        } else if !path.contains('/') && (directory_scope || !path.contains('.')) {
            risks.push(format!("{path} would own a whole top-level directory"));
        }
        amended.tasks[task].paths.push(path.clone());
        changes.push(json!({"kind":"add_path","path":path}));
    }
    for id in dependencies {
        if id == &amended.tasks[task].id {
            return Err(format!("task {id} cannot depend on itself"));
        }
        if amended.tasks[task].depends_on.contains(id) {
            return Err(format!(
                "task {} already depends on {id}",
                amended.tasks[task].id
            ));
        }
        amended.tasks[task].depends_on.push(id.clone());
        changes.push(json!({"kind":"add_dependency","on":id}));
    }
    if let Some(command) = test_command {
        if command.is_empty()
            || command.len() > 32
            || command
                .iter()
                .any(|part| part.is_empty() || part.len() > 4096 || part.contains('\0'))
        {
            return Err("replacement test command must contain one to 32 bounded arguments".into());
        }
        if !resolvable(&command[0], root) && !created_by_a_task(&amended, &command[0]) {
            return Err(format!(
                "replacement test command {:?} was not found",
                command[0]
            ));
        }
        if command == amended.tasks[task].test_command {
            return Err("replacement test command must differ from the current command".into());
        }
        amended.tasks[task].test_command = command.to_vec();
        changes.push(json!({"kind":"replace_test_command","test_command":command}));
    }
    amended.validate()?;
    Ok(Amended {
        plan: amended,
        changes,
        risk: (!risks.is_empty()).then(|| risks.join("; ")),
    })
}
