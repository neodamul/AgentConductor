use super::*;
impl Startup {
    pub(super) fn snapshot_state(
        &self,
        saved: &Saved,
        job: &serde_json::Value,
        metadata_job: &serde_json::Value,
        icons: bool,
        mcp_result: bool,
        quota_refresh_last: &std::collections::BTreeMap<String, u64>,
    ) -> Result<serde_json::Value, String> {
        let mut config = serde_json::to_value(&saved.config).map_err(|e| e.to_string())?;
        for p in config["profiles"]
            .as_array_mut()
            .ok_or(crate::ui::messages::text("native.invalid_registry"))?
        {
            p.as_object_mut().unwrap().remove("environment");
        }
        let proposal = if let Some(p) = &saved.proposal {
            let path = p.directory.join("review.json");
            let mut value = serde_json::to_value(p).map_err(|e| e.to_string())?;
            if path.exists() {
                let plan: Plan = read(&path)?;
                let digest =
                    workflow::digest(&serde_json::to_vec(&plan).map_err(|e| e.to_string())?);
                let mut public = serde_json::to_value(plan).map_err(|e| e.to_string())?;
                for agent in public["agents"].as_array_mut().unwrap() {
                    agent.as_object_mut().unwrap().remove("environment");
                }
                value["plan"] = public;
                value["digest"] = digest.into();
            }
            value
        } else {
            serde_json::Value::Null
        };
        let quotas = self.quota_snapshot(saved)?;
        let language = crate::ui::language::load();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        let quota_badges = crate::quota::badges(&quotas, &language, now);
        let quota_refresh_due = crate::quota::due_by_profile(
            &saved.config.profiles,
            quota_refresh_last,
            (now as u64) * 1000,
        );
        let last_run = self.last_run(saved);
        let last_report = last_run
            .as_deref()
            .and_then(|run| crate::workflow::load(run).ok());
        let agent_activity = last_report
            .as_ref()
            .map(agent_activity)
            .unwrap_or_else(|| serde_json::json!({}));
        let run_state = last_report.as_ref().map(|report| report.state.as_str());
        let run_available = saved.proposal.as_ref().is_some_and(|proposal| {
            proposal.phase == "starting" && proposal.run.join("report.json").exists()
        }) || last_run.is_some();
        let templates = crate::custom_profile_registry::CustomProfileRegistry::append_templates(
            defaults()?.profiles,
            self.custom_profiles.clone(),
        );
        let (templates, registry_invalid) = match templates {
            Ok(templates) => (templates, self.custom_registry_invalid),
            Err(_) => (defaults()?.profiles, true),
        };
        let mut templates = serde_json::to_value(templates).map_err(|e| e.to_string())?;
        for template in templates.as_array_mut().unwrap() {
            template.as_object_mut().unwrap().remove("environment");
        }
        let mut snapshot = serde_json::json!({"revision":self.revision(saved)?,"config":config,"proposal":proposal,"job":job,"quota_job":if metadata_job["kind"].as_str().is_some_and(|k|k.starts_with("quota:")){metadata_job.clone()}else{serde_json::Value::Null},"metadata_job":metadata_job,"metadata_queue":saved.metadata_queue,"root":self.broker.dir,"templates":templates,"custom_registry_invalid":registry_invalid,"run_available":run_available,"run_state":run_state,"agent_activity":agent_activity,"quotas":quotas,"quota_badges":quota_badges,"quota_refresh_due":quota_refresh_due,"slugs":self.slug_snapshot(saved),"plan_rules_global":crate::plan_rules::current(None),"language":language,"mcp_result":if mcp_result {self.mcp_result(saved,job)?}else{serde_json::Value::Null},"imports":saved.imports,"source_notes":self.source_notes(saved)});
        if snapshot["job"]["kind"] == "plan"
            && let Some(p) = &saved.proposal
        {
            snapshot["job"]["output"] = jobs::activity(&p.directory.join("process"));
        }
        if icons {
            snapshot["icons"] = serde_json::json!(crate::icons::icon_map(
                saved.config.profiles.iter().map(|p| p.parser),
            ));
        }
        Ok(snapshot)
    }
}

fn agent_activity(report: &workflow::Report) -> serde_json::Value {
    let mut activity = serde_json::Map::new();
    for planned in &report.plan.tasks {
        let Some(task) = report.tasks.iter().find(|task| task.id == planned.id) else {
            continue;
        };
        let priority = activity_priority(&task.status);
        if priority > 0
            && activity
                .get(&planned.agent)
                .and_then(serde_json::Value::as_str)
                .is_none_or(|current| priority > activity_priority(current))
        {
            activity.insert(planned.agent.clone(), task.status.clone().into());
        }
    }
    serde_json::Value::Object(activity)
}

fn activity_priority(status: &str) -> u8 {
    match status {
        "running" => 5,
        "waiting_availability" | "waiting_approval" | "waiting_tool_approval" => 4,
        "failed" | "blocked" => 3,
        _ => 0,
    }
}
