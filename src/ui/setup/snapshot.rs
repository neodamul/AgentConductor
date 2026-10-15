use super::*;
impl Startup {
    pub(super) fn snapshot_state(
        &self,
        saved: &Saved,
        job: &serde_json::Value,
        metadata_job: &serde_json::Value,
        icons: bool,
        mcp_result: bool,
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
        let mut snapshot = serde_json::json!({"revision":self.revision(saved)?,"config":config,"proposal":proposal,"job":job,"quota_job":if metadata_job["kind"].as_str().is_some_and(|k|k.starts_with("quota:")){metadata_job.clone()}else{serde_json::Value::Null},"metadata_job":metadata_job,"metadata_queue":saved.metadata_queue,"root":self.broker.dir,"templates":defaults()?.profiles,"run_available":saved.proposal.as_ref().is_some_and(|p|p.phase=="starting" && p.run.join("report.json").exists()) || self.last_run(saved).is_some(),"run_state":self.last_run(saved).and_then(|run|crate::workflow::load(&run).ok()).map(|report|report.state),"quotas":quotas,"quota_badges":quota_badges,"slugs":self.slug_snapshot(saved),"plan_rules_global":crate::plan_rules::current(None),"language":language,"mcp_result":if mcp_result {self.mcp_result(saved,job)?}else{serde_json::Value::Null},"imports":saved.imports,"source_notes":self.source_notes(saved)});
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
