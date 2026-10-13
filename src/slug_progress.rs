//! DD-63: slug progress is derived from where task files are; the tree is the only truth.
use serde::Deserialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

const TASK_STAGES: [&str; 3] = ["task", "verification", "completed"];

#[derive(Deserialize)]
struct PlanFile {
    phases: Vec<Phase>,
}
#[derive(Deserialize)]
struct Phase {
    id: String,
    name: String,
    tasks: Vec<String>,
}

/// Task IDs (file stems of `*.md`) in `<root>/<stage>/<phase>/`.
fn tasks_in(root: &Path, stage: &str, phase: &str) -> BTreeSet<String> {
    fs::read_dir(root.join(stage).join(phase))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            (path.extension().is_some_and(|x| x == "md"))
                .then(|| path.file_stem()?.to_str().map(str::to_owned))
                .flatten()
        })
        .collect()
}

fn phase_dirs(root: &Path, stage: &str) -> BTreeSet<String> {
    fs::read_dir(root.join(stage))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(str::to_owned))
        .collect()
}

pub fn progress(root: &Path) -> Value {
    let plan_path = root.join("plan/plan.json");
    if !plan_path.exists() {
        let stage = if root.join("plan/analysis.md").exists() {
            "analysis"
        } else {
            "no_plan"
        };
        return json!({"stage":stage,"phases":[],"mismatches":[]});
    }
    let plan: PlanFile = match fs::read(&plan_path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
    {
        Some(plan) => plan,
        None => {
            return json!({"stage":"planned","phases":[],"mismatches":["plan/plan.json is unreadable"]});
        }
    };
    let mut mismatches = vec![];
    let declared: BTreeSet<&str> = plan.phases.iter().map(|p| p.id.as_str()).collect();
    for stage in TASK_STAGES {
        for dir in phase_dirs(root, stage) {
            if !declared.contains(dir.as_str()) {
                mismatches.push(format!("{stage}/{dir} is a phase not declared in the plan"));
            }
        }
    }
    let mut phases = vec![];
    let mut current: Option<String> = None;
    for phase in &plan.phases {
        let found: Vec<BTreeSet<String>> = TASK_STAGES
            .iter()
            .map(|stage| tasks_in(root, stage, &phase.id))
            .collect();
        for (stage, files) in TASK_STAGES.iter().zip(&found) {
            for file in files.iter().filter(|f| !phase.tasks.contains(f)) {
                mismatches.push(format!(
                    "{stage}/{}/{file}.md is not declared in the plan",
                    phase.id
                ));
            }
        }
        for task in &phase.tasks {
            match found.iter().filter(|set| set.contains(task)).count() {
                0 => mismatches.push(format!(
                    "task {task} of phase {} is missing from every stage",
                    phase.id
                )),
                1 => {}
                _ => mismatches.push(format!(
                    "task {task} of phase {} is in more than one stage",
                    phase.id
                )),
            }
        }
        let count = |i: usize| phase.tasks.iter().filter(|t| found[i].contains(*t)).count();
        let (task, verification, completed) = (count(0), count(1), count(2));
        let started = verification + completed > 0;
        let stage = if current.is_some() {
            // A later phase may not move while an earlier one is unfinished.
            if started {
                mismatches.push(format!(
                    "phase {} progressed before phase {} was completed",
                    phase.id,
                    current.as_deref().unwrap_or_default()
                ));
            }
            "pending"
        } else if completed == phase.tasks.len() {
            "completed"
        } else if task > 0 {
            "task"
        } else {
            "verification"
        };
        if stage != "completed" && current.is_none() {
            current = Some(phase.id.clone());
        }
        let ids = |i: usize| {
            phase
                .tasks
                .iter()
                .filter(|t| found[i].contains(*t))
                .cloned()
                .collect::<Vec<_>>()
        };
        phases.push(json!({"id":phase.id,"name":phase.name,"stage":stage,"task":ids(0),"verification":ids(1),"completed":ids(2)}));
    }
    let stage = if current.is_none() {
        "completed"
    } else {
        "phase"
    };
    json!({"stage":stage,"current_phase":current,"phases":phases,"mismatches":mismatches})
}

/// DD-125: the whole goal plan for review — the coordinator's analysis, and per phase its
/// briefing (the analysis item naming the phase, else its phase file) and planned tasks —
/// so every phase is reviewed up front, not only the one about to run.
pub fn briefing(root: &Path) -> Value {
    let plan: Value = fs::read(root.join("plan/plan.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or(Value::Null);
    let analysis = fs::read_to_string(root.join("plan/analysis.md")).unwrap_or_default();
    let clip = |text: &str, limit: usize| -> String {
        if text.chars().count() <= limit {
            text.to_owned()
        } else {
            text.chars().take(limit).collect::<String>() + "…"
        }
    };
    let phases: Vec<Value> = plan["phases"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|phase| {
            let id = phase["id"].as_str().unwrap_or_default();
            let brief = brief(&analysis, id).unwrap_or_else(|| {
                fs::read_to_string(root.join("phase").join(format!("{id}.md")))
                    .unwrap_or_default()
                    .lines()
                    .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            });
            let specs: Vec<Value> = phase["specs"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|spec| {
                    json!({
                        "id": spec["id"], "agent": spec["agent"], "model": spec["model"],
                        "role": spec["role"], "paths": spec["paths"],
                        "test_command": spec["test_command"], "depends_on": spec["depends_on"],
                        "prompt": clip(spec["prompt"].as_str().unwrap_or_default(), 4000),
                    })
                })
                .collect();
            json!({"id": id, "name": phase["name"], "brief": clip(&brief, 1200), "specs": specs})
        })
        .collect();
    json!({"analysis": clip(&analysis, 16 * 1024), "phases": phases})
}

/// The analysis list item that names `phase`, with its continuation lines.
fn brief(analysis: &str, phase: &str) -> Option<String> {
    let lines: Vec<&str> = analysis.lines().collect();
    let start = lines.iter().position(|l| {
        let t = l.trim_start();
        t.contains(phase) && (t.starts_with(|c: char| c.is_ascii_digit()) || t.starts_with("- "))
    })?;
    let mut out = vec![lines[start].trim()];
    for line in &lines[start + 1..] {
        let t = line.trim_start();
        let next_item = t.starts_with(|c: char| c.is_ascii_digit()) && t.contains(". ")
            || t.starts_with('#')
            || (t.starts_with("- ") && !line.starts_with(' '));
        if next_item || t.is_empty() && out.len() > 1 {
            break;
        }
        if !t.is_empty() {
            out.push(t);
        }
    }
    Some(out.join("\n").replace("**", ""))
}
