use super::*;
use std::io::{BufRead, Read};
pub(super) fn text(path: &Path, limit: u64) -> Result<String, String> {
    let file = crate::ui::open_artifact(path)
        .map_err(|e| format!("Phase evidence {}: {e}", path.display()))?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(format!(
            "Phase evidence exceeds {limit} bytes: {}",
            path.display()
        ));
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
pub fn record_outcome(root: &Path, phase: &str, run: &Path) -> Result<(), String> {
    crate::phases::safe_id(phase)?;
    let run = run.canonicalize().map_err(|e| e.to_string())?;
    let report = crate::workflow::load(&run)?;
    if report.state != "completed" {
        return Err("only merged outcomes may inform the next Phase".into());
    }
    let dir = root.join("plan/outcomes");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let result_hashes = report
        .tasks
        .iter()
        .map(|task| {
            Ok((
                task.id.clone(),
                crate::workflow::file_digest(&task.artifacts.join("events.jsonl"))?,
            ))
        })
        .collect::<Result<std::collections::BTreeMap<_, _>, String>>()?;
    write(
        &dir.join(format!("{phase}.json")),
        &json!({"run":run,"candidate":report.candidate,"diff_hash":report.diff_hash,"result_hashes":result_hashes}),
    )
}
pub fn context(root: &Path, phase: &str) -> Result<Value, String> {
    let index: Value = serde_json::from_str(&text(&root.join("plan/plan.json"), 512 * 1024)?)
        .map_err(|e| e.to_string())?;
    let phases = index["phases"].as_array().ok_or("missing Phase plan")?;
    let at = phases
        .iter()
        .position(|p| p["id"] == phase)
        .ok_or("unknown Phase")?;
    for phase in phases {
        crate::phases::safe_id(phase["id"].as_str().ok_or("invalid Phase ID")?)?;
    }
    let progress = crate::slug_progress::progress(root);
    if progress["current_phase"] != phase
        || !progress["mismatches"]
            .as_array()
            .is_some_and(|a| a.is_empty())
    {
        return Err("Phase progress changed or is inconsistent".into());
    }
    let mut completed = Vec::new();
    for previous in &phases[..at] {
        let id = previous["id"].as_str().ok_or("invalid Phase ID")?;
        let receipt: Value = serde_json::from_str(&text(
            &root.join("plan/outcomes").join(format!("{id}.json")),
            64 * 1024,
        )?)
        .map_err(|e| e.to_string())?;
        let run = PathBuf::from(receipt["run"].as_str().ok_or("missing merged run")?);
        let report = crate::workflow::load(&run)?;
        if report.state != "completed"
            || json!(report.candidate) != receipt["candidate"]
            || json!(report.diff_hash) != receipt["diff_hash"]
        {
            return Err("completed Phase outcome changed".into());
        }
        let diff = log_evidence(
            &run.join("diff.patch"),
            report
                .diff_hash
                .as_deref()
                .ok_or("missing merged diff hash")?,
            32768,
        )?;
        let tests = report
            .tests
            .iter()
            .map(test_evidence)
            .collect::<Result<Vec<_>, String>>()?;
        for task in &report.tasks {
            if receipt["result_hashes"][&task.id]
                != crate::workflow::file_digest(&task.artifacts.join("events.jsonl"))?
            {
                return Err("completed Phase final report changed".into());
            }
        }
        let tasks=report.tasks.iter().map(|t|Ok(json!({"id":t.id,"status":t.status,"error":t.error,"feedback":t.feedback,"final_reports":final_reports(&t.artifacts.join("events.jsonl"))?}))).collect::<Result<Vec<_>,String>>()?;
        completed.push(json!({"phase":id,"candidate":report.candidate,"diff":diff["text"],"diff_truncated":diff["truncated"],"diff_hash":report.diff_hash,"tests":tests,"tasks":tasks,"review":report.latest_review,"execution_profiles":super::execution_fingerprint(&report.plan.agents)?}));
    }
    Ok(
        json!({"analysis":text(&root.join("plan/analysis.md"),64*1024)?,"remaining_phases":&phases[at..],"completed_phases":completed}),
    )
}

fn final_reports(path: &Path) -> Result<Vec<Value>, String> {
    let file = crate::ui::open_artifact(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 32 * 1024 * 1024 {
        return Err("Phase event evidence exceeds 32 MiB; inspect before re-planning".into());
    }
    let mut reader = std::io::BufReader::new(file);
    let mut out = Vec::new();
    loop {
        let mut line = Vec::new();
        let count = reader
            .by_ref()
            .take(1024 * 1024 + 1)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        if count > 1024 * 1024 {
            return Err("Phase result event exceeds 1 MiB".into());
        }
        let event: Value = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
        if event["kind"] == "result"
            || event
                .pointer("/payload/item/type")
                .is_some_and(|kind| kind == "agent_message")
        {
            let payload = serde_json::to_string(&event["payload"]).map_err(|e| e.to_string())?;
            let kind = if event["kind"] == "result" {
                "result"
            } else {
                "agent_message"
            };
            out.retain(|item: &Value| item["kind"] != kind);
            out.push(json!({"kind":kind,"text":payload.chars().take(2048).collect::<String>(),"truncated":payload.chars().count()>2048}));
        }
    }
    Ok(out)
}

fn test_evidence(test: &crate::workflow::TestReport) -> Result<Value, String> {
    if test.exit_code != Some(0) || test.timed_out {
        return Err("completed Phase verification is not passing".into());
    }
    Ok(
        json!({"task":test.task,"exit_code":test.exit_code,"timed_out":test.timed_out,"stdout":log_evidence(&test.stdout,&test.stdout_hash,2048)?,"stderr":log_evidence(&test.stderr,&test.stderr_hash,2048)?}),
    )
}
fn log_evidence(path: &Path, expected: &str, limit: usize) -> Result<Value, String> {
    use sha2::{Digest, Sha256};
    let mut file = crate::ui::open_artifact(path).map_err(|e| e.to_string())?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 64 * 1024 * 1024 {
        return Err("Phase evidence exceeds 64 MiB; inspect before re-planning".into());
    }
    let mut hash = Sha256::new();
    let mut sample = Vec::new();
    let mut total = 0;
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
        total += n;
        let keep = (limit - sample.len()).min(n);
        sample.extend_from_slice(&buffer[..keep]);
    }
    if format!("{:x}", hash.finalize()) != expected {
        return Err("completed Phase evidence changed".into());
    }
    Ok(
        json!({"text":String::from_utf8_lossy(&sample),"truncated":total>sample.len(),"bytes":total,"sha256":expected}),
    )
}
