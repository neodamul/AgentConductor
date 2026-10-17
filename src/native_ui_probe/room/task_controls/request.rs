use super::Action;
use serde_json::Value;

pub fn request(snapshot: &Value, action: Action) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|revision| !revision.is_empty())
        .ok_or("current run revision is unavailable")?;
    let report = &snapshot["report"];
    if snapshot["job"]["state"] == "running"
        || matches!(report["state"].as_str(), Some("running" | "completed"))
        || report["tasks"]
            .as_array()
            .is_some_and(|tasks| tasks.iter().any(|task| task["status"] == "running"))
    {
        return Err("stop the active run before changing task settings".into());
    }
    let task_id = match &action {
        Action::Feedback { task, .. }
        | Action::Reassign { task, .. }
        | Action::SetModel { task, .. }
        | Action::Schedule { task, .. } => task,
    };
    let exists = report["plan"]["tasks"].as_array().is_some_and(|tasks| {
        tasks
            .iter()
            .any(|task| task["id"].as_str() == Some(task_id.as_str()))
    }) && report["tasks"].as_array().is_some_and(|tasks| {
        tasks
            .iter()
            .any(|task| task["id"].as_str() == Some(task_id.as_str()))
    });
    if !exists {
        return Err("unknown task/session ID".into());
    }
    let action = match action {
        Action::Feedback { task, feedback }
            if !feedback.trim().is_empty() && feedback.len() <= 16 * 1024 =>
        {
            serde_json::json!({"kind":"revise","task":task,"feedback":feedback})
        }
        Action::Feedback { .. } => {
            return Err("review feedback must contain 1..16384 bytes".into());
        }
        Action::Reassign { task, agent }
            if report["plan"]["agents"]
                .as_array()
                .is_some_and(|agents| agents.iter().any(|profile| profile["id"] == agent)) =>
        {
            serde_json::json!({"kind":"reassign","task":task,"agent":agent})
        }
        Action::Reassign { .. } => return Err("unknown target agent".into()),
        Action::SetModel { task, model }
            if !model.is_empty()
                && report["plan"]["tasks"]
                    .as_array()
                    .and_then(|tasks| tasks.iter().find(|task_plan| task_plan["id"] == task))
                    .and_then(|task_plan| task_plan["agent"].as_str())
                    .and_then(|agent_id| {
                        report["plan"]["agents"]
                            .as_array()?
                            .iter()
                            .find(|profile| profile["id"] == agent_id)
                    })
                    .is_some_and(|profile| {
                        !profile["catalog"]["source"]
                            .as_str()
                            .unwrap_or_default()
                            .is_empty()
                            && profile["catalog"]["models"]
                                .as_array()
                                .is_some_and(|models| {
                                    models
                                        .iter()
                                        .any(|item| item.as_str() == Some(model.as_str()))
                                })
                    }) =>
        {
            serde_json::json!({"kind":"model","task":task,"model":model})
        }
        Action::SetModel { .. } => {
            return Err("model must belong to an observed CLI catalog".into());
        }
        Action::Schedule { task, at }
            if crate::availability::parse_time(&at).is_ok()
                && report["tasks"].as_array().is_some_and(|tasks| {
                    tasks.iter().any(|task_report| {
                        task_report["id"] == task && task_report["status"] == "waiting_availability"
                    })
                }) =>
        {
            serde_json::json!({"kind":"schedule","task":task,"at":at})
        }
        Action::Schedule { .. } => {
            return Err(
                "schedule requires a waiting task and a valid timestamp with timezone".into(),
            );
        }
    };
    serde_json::to_vec(&serde_json::json!({
        "kind":"action", "revision":revision, "action":action
    }))
    .map_err(|error| error.to_string())
}
