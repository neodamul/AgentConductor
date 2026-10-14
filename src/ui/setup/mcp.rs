//! DD-68: explicit MCP settings and source review, using the existing async job broker.
use super::*;
impl Config {
    pub(super) fn mcp_settings(&self) -> crate::mcp::Settings {
        crate::mcp::Settings {
            mcp: self.mcp.clone(),
            goal_sources: self.goal_sources.clone(),
        }
    }
}
impl Startup {
    pub(super) fn save_mcp(&self, s: &mut Saved, text: &str) -> Result<serde_json::Value, String> {
        let settings = crate::mcp::Settings::parse(text)?;
        let mut next = s.clone();
        next.config.mcp = settings.mcp;
        next.config.goal_sources = settings.goal_sources;
        if next.config.mcp_settings().digest()? != s.config.mcp_settings().digest()? {
            next.imports = None;
            next.mcp_result = None;
            next.proposal = None;
            next.job = None;
        }
        self.store(&next)?;
        *s = next;
        crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record(
            "mcp_settings_saved",
            &serde_json::json!({"servers":s.config.mcp.keys().collect::<Vec<_>>()}),
            crate::ui::messages::text(
                "native.operator_saved_mcp_connection_definitions_with_environment_references",
            ),
        )?;
        Ok(serde_json::json!({"saved":true}))
    }
    pub(super) fn launch_mcp(
        &self,
        s: &mut Saved,
        server: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let settings = s.config.mcp_settings();
        settings.validate()?;
        if let Some(name) = server {
            if !settings.mcp.contains_key(name) {
                return Err(crate::ui::messages::text("native.unknown_mcp_server").into());
            }
        } else if crate::mcp::references(&s.config.goal)?.is_empty() {
            return Err(crate::ui::messages::text(
                "native.add_explicit_jira_key_123_or_confluence_page",
            )
            .into());
        }
        if server.is_none() {
            crate::mcp::coordinator_profile(&s.config.profiles, &s.config.coordinator)?;
        }
        let id = crate::native_guard::session_id()?;
        let name = format!(
            "{}-{id}.json",
            if server.is_some() { "tools" } else { "preview" }
        );
        s.mcp_result = Some(name.clone());
        let mut args = vec![
            if server.is_some() {
                "mcp-discover"
            } else {
                "mcp-import"
            }
            .into(),
            self.broker.dir.to_string_lossy().into(),
            name,
        ];
        if let Some(server) = server {
            args.push(server.into());
        }
        self.launch(s, "mcp", args)
    }
    pub(super) fn mcp_result(
        &self,
        s: &Saved,
        job: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        if job["kind"] != "mcp" || job["state"] != "completed" {
            return Ok(serde_json::Value::Null);
        }
        let Some(name) = &s.mcp_result else {
            return Ok(serde_json::Value::Null);
        };
        let data = crate::mcp::load(&self.broker.dir.join("mcp").join(name))?;
        serde_json::from_slice(&data)
            .map_err(|_| crate::ui::messages::text("native.invalid_mcp_result").into())
    }
    /// Title and a short summary of each applied goal source, for the goal's reference tags.
    pub(super) fn source_notes(&self, s: &Saved) -> serde_json::Value {
        let Some(applied) = &s.imports else {
            return serde_json::Value::Null;
        };
        let file = self
            .broker
            .dir
            .join("mcp")
            .join(format!("applied-{}.json", applied.id));
        let Some(preview) = crate::mcp::load(&file)
            .ok()
            .and_then(|data| serde_json::from_slice::<crate::mcp::Preview>(&data).ok())
        else {
            return serde_json::Value::Null;
        };
        let notes: serde_json::Map<String, serde_json::Value> = preview
            .records
            .iter()
            .map(|record| {
                let key = record.reference.key();
                let summary = preview
                    .summaries
                    .get(&key)
                    .map_or(record.body.as_str(), |s| s.text.as_str());
                let summary: String = summary.chars().take(600).collect();
                (
                    key,
                    serde_json::json!({"title": record.title, "summary": summary}),
                )
            })
            .collect();
        serde_json::Value::Object(notes)
    }
    pub(super) fn apply_mcp(
        &self,
        s: &mut Saved,
        id: &str,
        selected: &[String],
    ) -> Result<serde_json::Value, String> {
        let name = s
            .mcp_result
            .as_ref()
            .ok_or(crate::ui::messages::text("native.retrieve_a_preview_first"))?;
        if !name.starts_with("preview-") {
            return Err(crate::ui::messages::text("native.retrieve_a_source_preview_first").into());
        }
        let p: crate::mcp::Preview =
            serde_json::from_slice(&crate::mcp::load(&self.broker.dir.join("mcp").join(name))?)
                .map_err(|_| crate::ui::messages::text("native.invalid_mcp_preview"))?;
        if p.id != id {
            return Err(crate::ui::messages::text(
                "native.approve_the_exact_displayed_mcp_preview",
            )
            .into());
        }
        let receipt = crate::mcp::apply(
            &self.broker.dir,
            &p,
            &s.config.mcp_settings(),
            &s.config.goal,
            selected,
        )?;
        let mut next = s.clone();
        next.imports = Some(receipt);
        next.proposal = None;
        next.job = None;
        self.store(&next)?;
        *s = next;
        crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record(
            "mcp_import_approved",
            &serde_json::json!({"preview":id,"selected":selected}),
            crate::ui::messages::text(
                "native.operator_approved_selected_read_only_source_evidence_for",
            ),
        )?;
        Ok(serde_json::json!({"applied":true}))
    }
    pub(super) fn source_context(&self, s: &Saved) -> Result<Option<String>, String> {
        if crate::mcp::references(&s.config.goal)?.is_empty() {
            return Ok(None);
        }
        let a = s.imports.as_ref().ok_or(crate::ui::messages::text(
            "native.preview_and_apply_referenced_jira_confluence_sources_before",
        ))?;
        crate::mcp::context(
            &self.broker.dir,
            a,
            &s.config.mcp_settings(),
            &s.config.goal,
        )
        .map(Some)
    }
}
