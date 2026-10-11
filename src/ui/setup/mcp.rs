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
            "Operator saved MCP connection definitions with environment references",
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
                return Err("unknown MCP server".into());
            }
        } else if crate::mcp::references(&s.config.goal)?.is_empty() {
            return Err("add explicit jira:KEY-123 or confluence:PAGE_ID goal references".into());
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
        serde_json::from_slice(&data).map_err(|_| "invalid MCP result".into())
    }
    pub(super) fn apply_mcp(
        &self,
        s: &mut Saved,
        id: &str,
        selected: &[String],
    ) -> Result<serde_json::Value, String> {
        let name = s.mcp_result.as_ref().ok_or("retrieve a preview first")?;
        if !name.starts_with("preview-") {
            return Err("retrieve a source preview first".into());
        }
        let p: crate::mcp::Preview =
            serde_json::from_slice(&crate::mcp::load(&self.broker.dir.join("mcp").join(name))?)
                .map_err(|_| "invalid MCP preview")?;
        if p.id != id {
            return Err("approve the exact displayed MCP preview".into());
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
        crate::state::Store::open(self.broker.dir.join("state.sqlite"))?.record("mcp_import_approved",&serde_json::json!({"preview":id,"selected":selected}),"Operator approved selected read-only source evidence for planning; source bodies remain local")?;
        Ok(serde_json::json!({"applied":true}))
    }
    pub(super) fn source_context(&self, s: &Saved) -> Result<Option<String>, String> {
        if crate::mcp::references(&s.config.goal)?.is_empty() {
            return Ok(None);
        }
        let a = s
            .imports
            .as_ref()
            .ok_or("preview and apply referenced Jira/Confluence sources before planning")?;
        crate::mcp::context(
            &self.broker.dir,
            a,
            &s.config.mcp_settings(),
            &s.config.goal,
        )
        .map(Some)
    }
}
