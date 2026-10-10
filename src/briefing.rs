//! Read-only factual team briefings; reported text is evidence, never authority.
use crate::{
    state::Store,
    workflow::{self, Report, TaskReport},
};
use serde_json::{Value, json};
use std::{io::Read, path::Path};

fn freshness(at: Option<u64>, now: u64) -> (&'static str, Option<u64>) {
    match at.and_then(|at| now.checked_sub(at)) {
        Some(age) => (if age > 300_000 { "stale" } else { "recent" }, Some(age)),
        None => (if at.is_some() { "future" } else { "unknown" }, None),
    }
}
fn process(task: &TaskReport, now: u64) -> Value {
    let path = task.artifacts.join("process/owner.json");
    let result = (|| {
        let mut bytes = Vec::new();
        std::fs::File::open(&path)
            .map_err(|e| e.to_string())?
            .take(8193)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 8192 {
            return Err("ownership evidence too large".into());
        }
        let owner: crate::process::Ownership =
            serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if owner.phase == "not_spawned" && owner.pid.is_none() && owner.process_group.is_none() {
            return Ok(json!({"state":"not_started"}));
        }
        #[cfg(unix)]
        {
            let pid = owner
                .pid
                .filter(|pid| *pid > 0 && *pid <= i32::MAX as u32)
                .ok_or("invalid owned PID")? as i32;
            if owner.process_group != Some(pid) {
                return Err("invalid owned process group".into());
            }
            let mut present = false;
            for target in [pid, -pid] {
                if unsafe { libc::kill(target, 0) } == 0 {
                    present = true;
                } else if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                    return Err("kernel presence probe inconclusive".into());
                }
            }
            Ok(
                json!({"state":if present {"present"} else {"absent"},"pid":pid,"recorded_phase":owner.phase,"exit_code":owner.exit_code,"termination_signal":owner.termination_signal,"note":"PID/group presence does not establish identity or progress"}),
            )
        }
        #[cfg(not(unix))]
        {
            let _ = owner;
            Err("native process observation unavailable on this platform".into())
        }
    })();
    let mut value=result.unwrap_or_else(|error:String|json!({"state":if task.attempt==0 {"not_started"} else {"unknown"},"error":error}));
    value["observed_at_ms"] = json!(now);
    value["source"] = json!("owned record and kernel presence probe");
    value
}
fn memory_report(store: &Store, report: &Report, index: usize, now: u64) -> Result<Value, String> {
    let task = &report.tasks[index];
    let assigned = &report.plan.tasks[index];
    let Some(mut value) =
        store.optional_document(&format!("cooperative_report/{}/{}", task.id, task.attempt))?
    else {
        return Ok(
            json!({"freshness":"unavailable","note":"No cooperative memory report recorded"}),
        );
    };
    if value["agent"] != assigned.agent
        || value["task"] != task.id
        || value["attempt"] != task.attempt
        || value["session_id"] != json!(task.session_id)
    {
        return Ok(
            json!({"freshness":"invalid","note":"Memory report does not match this assignment, attempt and session"}),
        );
    }
    for key in ["summary", "phase", "next_step", "source"] {
        let Some(text) = value[key].as_str() else {
            return Ok(json!({"freshness":"invalid","note":"Malformed cooperative report"}));
        };
        let bounded = text
            .chars()
            .take(if key == "summary" { 1600 } else { 400 })
            .collect::<String>();
        value[key] = json!(bounded);
    }
    let (fresh, age) = freshness(value["reported_at_ms"].as_u64(), now);
    // Return only named report fields; private or unexpected fields never enter the briefing.
    Ok(
        json!({"summary":value["summary"],"phase":value["phase"],"next_step":value["next_step"],"source":value["source"],"reported_at_ms":value["reported_at_ms"],"freshness":fresh,"age_ms":age,"authority":"agent-reported observation; not execution permission"}),
    )
}

pub fn collect(dir: &Path, now: u64) -> Result<Value, String> {
    if now == 0 {
        return Err("briefing needs a positive observation timestamp".into());
    }
    let store = Store::read_only_snapshot(dir.join("state.sqlite"))?;
    let report: Report =
        serde_json::from_value(store.document("workflow")?).map_err(|e| e.to_string())?;
    let mut sessions = Vec::new();
    let mut approvals = Vec::new();
    for (index, task) in report.tasks.iter().enumerate() {
        let assigned = report
            .plan
            .tasks
            .get(index)
            .filter(|assigned| assigned.id == task.id)
            .ok_or("task/report identity mismatch")?;
        let profile = report
            .plan
            .agents
            .iter()
            .find(|agent| agent.id == assigned.agent)
            .ok_or("unknown assigned agent")?;
        let events = store.task_events(&task.id, task.attempt as u64)?;
        let latest = events.last();
        let activity =
            store.optional_document(&format!("task_activity/{}/{}", task.id, task.attempt))?;
        let at = activity
            .as_ref()
            .filter(|a| latest.is_some_and(|event| event["seq"] == a["seq"]))
            .and_then(|a| a["observed_at_ms"].as_u64());
        let (fresh, age) = freshness(at, now);
        let proc = process(task, now);
        let interrupted = task.status == "running" && proc["state"] == "absent";
        let attention = if interrupted {
            "Recorded running state conflicts with an absent owned process/group"
        } else if task.status == "running" && proc["state"] == "unknown" {
            "Current process state cannot be verified"
        } else if task.status != "running" && proc["state"] == "present" {
            "PID/group is present despite idle task state; identity/reuse requires inspection"
        } else {
            ""
        };
        let next = if interrupted {
            "Inspect ownership and recover explicitly"
        } else {
            match task.status.as_str() {
                "running" => "Observe current work; silence alone does not prove failure",
                "waiting_availability" => "Review availability, schedule or reassign",
                "waiting_approval" => "Review exact tool request",
                "failed" | "blocked" => "Inspect retained evidence and revise or reassign",
                "stopped" => "Review partial work before explicit resume",
                "pending" => "Wait for dependencies and capacity",
                "completed" if report.state == "completed" => "No action required",
                "completed" => "Review integration tests and candidate",
                _ => "Inspect retained state",
            }
        };
        if let Some(id) = &task.tool_request {
            if let Some(receipt) = store.tool_receipt(id)? {
                approvals.push(json!({"kind":"tool","task":task.id,"id":receipt.id,"status":receipt.status,"tool":receipt.request.tool,"expires_at":receipt.expires_at}));
            } else {
                approvals.push(json!({"kind":"tool","task":task.id,"id":id,"status":"unknown"}));
            }
        }
        let event=latest.map(|e|json!({"seq":e["seq"],"kind":e["kind"],"raw_offset":e["raw_offset"],"text":e["payload"].to_string().chars().take(1600).collect::<String>(),"source":"normalized CLI stdout","observed_at_ms":at,"freshness":fresh,"age_ms":age}));
        sessions.push(json!({"agent":profile.id,"provider":profile.parser,"task":task.id,"work":assigned.prompt.chars().take(1600).collect::<String>(),"paths":assigned.paths,"depends_on":assigned.depends_on,"attempt":task.attempt,"session_id":task.session_id,"model":assigned.model,"state":task.status,"process":proc,"last_activity":event,"memory_report":memory_report(&store,&report,index,now)?,"restriction":task.restriction,"error":task.error,"attention":attention,"next_action":next,"artifacts":task.artifacts}));
    }
    if matches!(report.state.as_str(), "waiting_approval" | "approved") {
        approvals.push(json!({"kind":"merge","status":report.state,"candidate":report.candidate.as_ref().map(|c|&c.commit),"note":"Inspect exact diff/tests; merge remains a separate human action"}));
    }
    Ok(
        json!({"goal":report.plan.goal,"state":report.state,"revision":workflow::revision(&report)?,"observed_at_ms":now,"source":"read-only SQLite snapshot; process probes are separately observed","sessions":sessions,"approvals":approvals,"account_restrictions":report.account_restrictions,"tests":report.tests.iter().map(|t|json!({"task":t.task,"exit_code":t.exit_code,"timed_out":t.timed_out})).collect::<Vec<_>>(),"unassigned_agents":report.plan.agents.iter().filter(|a|!report.plan.tasks.iter().any(|t|t.agent==a.id)).map(|a|&a.id).collect::<Vec<_>>(),"memory_channel":"Stored cooperative reports only; this query does not contact agentmemory or an LLM"}),
    )
}
