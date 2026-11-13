//! #943: automatic merge uses the same exact candidate/evidence binding as human approval.
use super::*;

pub fn set_auto_merge(dir: &Path, enabled: bool, plan: &Plan) -> Result<(), String> {
    let value = serde_json::json!({"enabled":enabled,"plan_digest":digest(&serde_json::to_vec(plan).map_err(|e|e.to_string())?)});
    let path = dir.with_extension("auto-merge.json");
    let pending = path.with_extension(format!("{}.pending", crate::native_guard::session_id()?));
    fs::write(
        &pending,
        serde_json::to_vec(&value).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(pending, path).map_err(|e| e.to_string())
}

pub fn automatic_merge(dir: &Path) -> Result<bool, String> {
    let consent: serde_json::Value = match fs::read(dir.with_extension("auto-merge.json")) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string())?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e.to_string()),
    };
    if consent["enabled"] != true {
        return Ok(false);
    }
    {
        let _lock = Lock::acquire(dir)?;
        let report = load(dir)?;
        if report.state != "waiting_approval"
            || dir.join("stop.request").exists()
            || dir
                .parent()
                .is_some_and(|p| p.join("phase-chain.stop").exists())
            || report.error.is_some()
            || report.review_error.is_some()
            || report.review_round > 0
            || report.tasks.iter().any(|t| {
                t.status != "completed"
                    || t.error.is_some()
                    || t.pending_tool.is_some()
                    || t.tool_request.is_some()
                    || t.restriction.is_some()
            })
            || report
                .latest_review
                .as_ref()
                .is_some_and(|r| r.verdict != crate::coordinator::Verdict::Accept)
        {
            return Ok(false);
        }
        if consent["plan_digest"]
            != digest(&serde_json::to_vec(&report.plan).map_err(|e| e.to_string())?)
        {
            return Err("automatic merge consent no longer matches the executed plan".into());
        }
        let fingerprint = super::approval::binding(dir, &report)?;
        let candidate = report.candidate.as_ref().ok_or("no verified candidate")?;
        let mut store = Store::open(dir.join("state.sqlite"))?;
        store.record("approval",&serde_json::json!({"fingerprint":fingerprint,"candidate":candidate.commit,"automatic":true,"plan_digest":consent["plan_digest"]}),"Automatic approval of exact verified candidate under operator opt-in")?;
        let mut report = report;
        report.state = "approved".into();
        save(&mut store, dir, &report)?;
    }
    super::approval::merge(dir)?;
    Ok(true)
}
