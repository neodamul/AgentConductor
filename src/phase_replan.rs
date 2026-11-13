//! #943: outcome-based planning of one Phase, preserving the original phased plan.
use crate::{plan::Plan, roles::RoleRules};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};
mod adoption;
mod context;
pub use adoption::adopt;
pub use context::{context, record_outcome};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub root: PathBuf,
    pub phase: String,
    pub original: Plan,
    pub coordinator: String,
    pub repository: PathBuf,
    pub directory: PathBuf,
    pub roles: RoleRules,
    pub rules: String,
}

pub fn compare(original: &Plan, proposed: &Plan) -> Result<Value, String> {
    if original.goal != proposed.goal
        || original.workspace_mode != proposed.workspace_mode
        || serde_json::to_value(&original.agents).map_err(|e| e.to_string())?
            != serde_json::to_value(&proposed.agents).map_err(|e| e.to_string())?
    {
        return Err("re-planning changed the Goal or execution profiles".into());
    }
    let mut changes = Vec::new();
    for task in &proposed.tasks {
        let Some(old) = original.tasks.iter().find(|old| old.id == task.id) else {
            changes.push(json!({"id":task.id,"kind":"added","task":task,"fields":[]}));
            continue;
        };
        let a = serde_json::to_value(old).map_err(|e| e.to_string())?;
        let b = serde_json::to_value(task).map_err(|e| e.to_string())?;
        let mut fields: std::collections::BTreeSet<_> = a
            .as_object()
            .unwrap()
            .keys()
            .chain(b.as_object().unwrap().keys())
            .filter(|k| a[*k] != b[*k])
            .cloned()
            .collect();
        if original.tasks.iter().position(|t| t.id == task.id)
            != proposed.tasks.iter().position(|t| t.id == task.id)
        {
            fields.insert("order".into());
        }
        if !fields.is_empty() {
            changes.push(json!({"id":task.id,"kind":"changed","fields":fields,"before":old}));
        }
    }
    for task in &original.tasks {
        if !proposed.tasks.iter().any(|next| next.id == task.id) {
            changes.push(json!({"id":task.id,"kind":"removed","task":task,"fields":[]}));
        }
    }
    let unchanged = serde_json::to_value(&original.tasks).map_err(|e| e.to_string())?
        == serde_json::to_value(&proposed.tasks).map_err(|e| e.to_string())?;
    Ok(json!({"unchanged":unchanged,"changes":changes}))
}

pub async fn generate(request: Request) -> Result<(), String> {
    recover_outcomes(
        &request.root,
        request.directory.parent().ok_or("missing workspace")?,
    )?;
    let evidence = context(&request.root, &request.phase)?;
    let rules = format!(
        "{}\nRe-plan ONLY Phase {} from the completed outcomes below. Keep the Goal, existing task IDs when intent survives, and every later Phase unchanged. Reuse an original task byte-for-byte if its effective work is unchanged. Adjust/add/remove tasks only when justified by the merged evidence. Return the smallest nonempty valid next-Phase task list.\n",
        request.rules, request.phase
    );
    let reference=serde_json::to_string_pretty(&json!({"next_phase":request.phase,"original_tasks":request.original.tasks,"evidence":evidence})).map_err(|e|e.to_string())?;
    if reference.len() > 512 * 1024 {
        return Err("Phase evidence exceeds 512 KiB; review a smaller evidence bundle".into());
    }
    let mut plan = crate::planner::generate_from(
        &request.original.goal,
        &request.original.agents,
        &request.coordinator,
        &request.repository,
        &request.directory,
        &crate::planner::PlanInputs {
            roles: &request.roles,
            reference: Some(&reference),
            context: None,
            rules: Some(&rules),
            phases_root: None,
        },
    )
    .await?;
    plan.workspace_mode = request.original.workspace_mode;
    let mut result = compare(&request.original, &plan)?;
    let previous = evidence["completed_phases"]
        .as_array()
        .and_then(|phases| phases.last())
        .ok_or("re-planning requires a completed Phase")?;
    let profiles_changed =
        previous["execution_profiles"] != execution_fingerprint(&request.original.agents)?;
    result["execution_profiles_changed"] = profiles_changed.into();
    if profiles_changed {
        result["unchanged"] = false.into();
    }
    result["phase"] = request.phase.into();
    result["completed_phases"] = json!(evidence["completed_phases"].as_array().unwrap().iter().map(|phase|json!({"phase":phase["phase"],"candidate":phase["candidate"],"test_count":phase["tests"].as_array().map(Vec::len)})).collect::<Vec<_>>());
    write(&request.directory.join("baseline.json"), &request.original)?;
    let parent = request.directory.parent().ok_or("missing re-plan parent")?;
    let repo = crate::folder::repository(&request.repository, &parent.join("folder-history"))?;
    write(
        &request.directory.join("replan-base.json"),
        &json!({"workspace":repo.root,"repository_base":repo.head()?}),
    )?;
    write(&request.directory.join("plan.json"), &plan)?;
    write(&request.directory.join("replan.json"), &result)
}

pub(super) fn write(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let pending = path.with_extension(format!("{}.pending", crate::native_guard::session_id()?));
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    fs::write(&pending, bytes).map_err(|e| e.to_string())?;
    fs::rename(pending, path).map_err(|e| e.to_string())
}

pub fn metadata(path: &Path) -> Result<Value, String> {
    serde_json::from_str(&context::text(path, 256 * 1024)?).map_err(|e| e.to_string())
}

pub fn execution_fingerprint(profiles: &[crate::plan::AgentProfile]) -> Result<String, String> {
    let mut value = serde_json::to_value(profiles).map_err(|e| e.to_string())?;
    for profile in value.as_array_mut().ok_or("invalid profiles")? {
        profile.as_object_mut().unwrap().remove("catalog");
        profile
            .as_object_mut()
            .unwrap()
            .remove("quota_refresh_minutes");
    }
    Ok(crate::workflow::digest(
        &serde_json::to_vec(&value).map_err(|e| e.to_string())?,
    ))
}

/// Backfill retained pre-#943 runs only when their Goal-root link matches exactly.
pub fn recover_outcomes(root: &Path, workspace: &Path) -> Result<(), String> {
    let expected = root.canonicalize().map_err(|e| e.to_string())?;
    for entry in fs::read_dir(workspace).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_name().to_string_lossy().starts_with("run-")
            || !entry.file_type().map_err(|e| e.to_string())?.is_dir()
        {
            continue;
        }
        let run = entry.path();
        let Ok(bytes) = fs::read(run.join("slug-phase.json")) else {
            continue;
        };
        let Ok(link) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        let (Some(linked), Some(phase)) = (link["root"].as_str(), link["phase"].as_str()) else {
            continue;
        };
        crate::phases::safe_id(phase)?;
        if Path::new(linked).canonicalize().ok().as_ref() != Some(&expected)
            || root
                .join("plan/outcomes")
                .join(format!("{phase}.json"))
                .exists()
        {
            continue;
        }
        if crate::workflow::load(&run).is_ok_and(|report| report.state == "completed") {
            record_outcome(root, phase, &run)?;
        }
    }
    Ok(())
}
