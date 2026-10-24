//! DD-62: analysis first, ordered phases, one run per phase, task files move with the run.
use crate::{
    plan::{Plan, Task},
    slug_progress::progress,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhaseSpec {
    pub id: String,
    pub name: String,
    pub tasks: Vec<Task>,
}

const ORDER: [&str; 3] = ["task", "verification", "completed"];

/// A phase starts only after every earlier phase is completed, so a dependency on an
/// earlier phase's task is already satisfied. Coordinators often chain the first task of a
/// phase to the last task of the previous one; drop those links before each phase is
/// validated on its own. Unknown or later-phase dependencies are kept and still rejected.
pub fn drop_satisfied_dependencies(phases: &mut [PhaseSpec]) {
    let mut earlier = HashSet::new();
    for phase in phases {
        for task in &mut phase.tasks {
            task.depends_on.retain(|id| !earlier.contains(id));
        }
        earlier.extend(phase.tasks.iter().map(|t| t.id.clone()));
    }
}

fn safe_id(id: &str) -> Result<(), String> {
    let ok = !id.is_empty()
        && id.len() <= 40
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_');
    if ok {
        Ok(())
    } else {
        Err(format!("unsafe phase or task id {id:?}"))
    }
}

fn task_markdown(phase: &PhaseSpec, task: &Task) -> String {
    format!(
        "# {}\n\nPhase: {} ({})\nRole: {}\nAgent: {}{}\nOwned paths: {}\nDepends on: {}\nTest: {}\n\n{}\n",
        task.id,
        phase.name,
        phase.id,
        task.role.as_deref().unwrap_or("—"),
        task.agent,
        task.model
            .as_ref()
            .map_or(String::new(), |m| format!(" (model {m})")),
        task.paths.join(", "),
        if task.depends_on.is_empty() {
            "—".into()
        } else {
            task.depends_on.join(", ")
        },
        task.test_command.join(" "),
        task.prompt
    )
}

/// Writes a new phased plan; an existing plan is never overwritten.
pub fn write(root: &Path, analysis: &str, phases: &[PhaseSpec]) -> Result<(), String> {
    if root.join("plan/plan.json").exists() {
        return Err("this slug already has a plan".into());
    }
    if !(1..=6).contains(&phases.len()) {
        return Err("a phased plan needs one to six phases".into());
    }
    let mut seen = HashSet::new();
    for phase in phases {
        safe_id(&phase.id)?;
        if phase.tasks.is_empty() || phase.tasks.len() > 12 {
            return Err(format!("phase {} needs one to twelve tasks", phase.id));
        }
        for task in &phase.tasks {
            safe_id(&task.id)?;
            if !seen.insert(task.id.clone()) {
                return Err(format!("task id {} is used twice", task.id));
            }
        }
    }
    crate::slugs::create_tree(root)?;
    fs::write(root.join("plan/analysis.md"), analysis).map_err(|e| e.to_string())?;
    for (index, phase) in phases.iter().enumerate() {
        let list: Vec<String> = phase.tasks.iter().map(|t| format!("- {}", t.id)).collect();
        fs::write(
            root.join("phase").join(format!("{}.md", phase.id)),
            format!(
                "# Phase {} — {}\n\nTasks:\n{}\n",
                index + 1,
                phase.name,
                list.join("\n")
            ),
        )
        .map_err(|e| e.to_string())?;
        let dir = root.join("task").join(&phase.id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        for task in &phase.tasks {
            fs::write(
                dir.join(format!("{}.md", task.id)),
                task_markdown(phase, task),
            )
            .map_err(|e| e.to_string())?;
        }
    }
    let index = serde_json::json!({"phases": phases.iter().map(|p| serde_json::json!({
        "id": p.id, "name": p.name,
        "tasks": p.tasks.iter().map(|t| &t.id).collect::<Vec<_>>(),
        "specs": p.tasks,
    })).collect::<Vec<_>>()});
    // Written last: progress treats the plan as present only once every file exists.
    let bytes = serde_json::to_vec_pretty(&index).map_err(|e| e.to_string())?;
    fs::write(root.join("plan/plan.json"), bytes).map_err(|e| e.to_string())
}

/// The run plan for `phase`, allowed only when it is the current phase.
pub fn run_plan(root: &Path, phase: &str, base: &Plan) -> Result<Plan, String> {
    let state = progress(root);
    if state["current_phase"] != phase {
        return Err(format!(
            "phase {phase} cannot start; the current phase is {}",
            state["current_phase"]
        ));
    }
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("plan/plan.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let spec = index["phases"]
        .as_array()
        .and_then(|p| p.iter().find(|p| p["id"] == phase))
        .ok_or("unknown phase")?;
    let mut tasks: Vec<Task> =
        serde_json::from_value(spec["specs"].clone()).map_err(|e| e.to_string())?;
    let ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
    // Dependencies on earlier phases are already completed.
    for task in &mut tasks {
        task.depends_on.retain(|d| ids.contains(d));
    }
    let plan_rules = match fs::read_to_string(root.join("plan/plan-rules.md")) {
        Ok(rules) => rules,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => base.plan_rules.clone(),
        Err(error) => return Err(error.to_string()),
    };
    Ok(Plan {
        goal: base.goal.clone(),
        plan_rules,
        agents: base.agents.clone(),
        tasks,
        workspace_mode: base.workspace_mode,
    })
}

/// Moves one task file a single stage forward with an atomic rename.
pub fn advance(root: &Path, phase: &str, task: &str, to: &str) -> Result<(), String> {
    let target = ORDER.iter().position(|s| *s == to).ok_or("unknown stage")?;
    if target == 0 {
        return Err("tasks never move back to task".into());
    }
    let from = ORDER[target - 1];
    let source = root.join(from).join(phase).join(format!("{task}.md"));
    if !source.exists() {
        return Err(format!("task {task} of phase {phase} is not in {from}"));
    }
    let dir = root.join(to).join(phase);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    fs::rename(&source, dir.join(format!("{task}.md"))).map_err(|e| e.to_string())
}

/// Moves every task of `phase` from the preceding stage into `to`.
pub fn advance_phase(root: &Path, phase: &str, to: &str) -> Result<(), String> {
    let target = ORDER.iter().position(|s| *s == to).ok_or("unknown stage")?;
    if target == 0 {
        return Err("tasks never move back to task".into());
    }
    let from = root.join(ORDER[target - 1]).join(phase);
    let tasks: Vec<String> = fs::read_dir(&from)
        .map_err(|e| e.to_string())?
        .flatten()
        .filter_map(|e| e.path().file_stem()?.to_str().map(str::to_owned))
        .collect();
    for task in tasks {
        advance(root, phase, &task, to)?;
    }
    Ok(())
}
