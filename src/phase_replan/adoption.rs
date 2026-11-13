use super::*;
/// Atomically publish a revised current Phase index after staging its task documents.
pub fn adopt(root: &Path, phase: &str, plan: &Plan) -> Result<(), String> {
    plan.validate()?;
    crate::phases::safe_id(phase)?;
    for task in &plan.tasks {
        crate::phases::safe_id(&task.id)?;
    }
    let _lock = crate::leases::Lease::acquire(&root.join("phase-replan.lock"))?;
    let progress = crate::slug_progress::progress(root);
    if progress["current_phase"] != phase
        || !progress["mismatches"]
            .as_array()
            .is_some_and(|m| m.is_empty())
    {
        return Err("cannot adopt a stale Phase plan".into());
    }
    let status = progress["phases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == phase)
        .unwrap();
    if status["stage"] != "task" {
        return Err("cannot replace an executed Phase".into());
    }
    let path = root.join("plan/plan.json");
    let mut index: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let phases = index["phases"].as_array_mut().ok_or("invalid Phase plan")?;
    for p in phases.iter().filter(|p| p["id"] != phase) {
        for t in p["tasks"].as_array().ok_or("invalid Phase tasks")? {
            if plan.tasks.iter().any(|task| json!(task.id) == *t) {
                return Err("re-planned task ID belongs to another Phase".into());
            }
        }
    }
    let original = root.join("plan/original.json");
    if !original.exists() {
        write(&original, &index)?;
    }
    let phases = index["phases"].as_array_mut().unwrap();
    let target = phases
        .iter_mut()
        .find(|p| p["id"] == phase)
        .ok_or("unknown Phase")?;
    let name = target["name"].as_str().unwrap_or(phase).to_owned();
    let old = root.join("task").join(phase);
    let pending = root
        .join("task")
        .join(format!(".replan-{}", crate::native_guard::session_id()?));
    fs::create_dir(&pending).map_err(|e| e.to_string())?;
    let staged = crate::phases::PhaseSpec {
        id: phase.into(),
        name,
        tasks: plan.tasks.clone(),
    };
    for task in &plan.tasks {
        fs::write(
            pending.join(format!("{}.md", task.id)),
            crate::phases::task_markdown(&staged, task),
        )
        .map_err(|e| e.to_string())?;
    }
    let backup = old.with_extension("before-replan");
    if backup.exists() {
        return Err("unfinished Phase adoption requires recovery".into());
    }
    fs::rename(&old, &backup).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(&pending, &old) {
        let _ = fs::rename(&backup, &old);
        return Err(e.to_string());
    }
    target["tasks"] = json!(plan.tasks.iter().map(|t| &t.id).collect::<Vec<_>>());
    target["specs"] = json!(plan.tasks);
    if let Err(error) = write(&path, &index) {
        let _ = fs::remove_dir_all(&old);
        let _ = fs::rename(&backup, &old);
        return Err(error);
    }
    fs::remove_dir_all(backup).map_err(|e| e.to_string())
}
