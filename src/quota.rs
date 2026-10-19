pub mod custom;
mod parser;
pub use parser::{parse_claude, parse_codex};
pub(crate) use parser::{snapshot, window};
mod restriction;
pub use restriction::restriction;
mod display;
pub use display::{Badge, badge, badges, budget_rows, window_labels, window_reset_labels};
mod refresh;
pub use refresh::{due_by_profile, refresh_due};
mod budget;
mod reset_time;
pub use budget::Budget;
mod workspace;
pub use workspace::stored_for_run;
pub(crate) use workspace::workspace_cache;
mod terminal;
pub use terminal::{
    claude_blocker, claude_trust_answer, claude_trust_prompt, claude_usage_ready,
    parse_claude_usage, query_claude,
};
mod claude;
pub(crate) use claude::observe_events;
pub use claude::parse_claude_event;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Window {
    pub bucket: String,
    pub label: String,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub window_minutes: Option<u64>,
    pub resets_at: Option<i64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restriction: Option<crate::availability::Restriction>,
    #[serde(default)]
    pub budgets: Vec<Budget>,
    #[serde(default)]
    pub plan_type: Option<String>,
    pub source: String,
    pub observed_at: i64,
    pub windows: Vec<Window>,
}
#[derive(Clone, Debug, Serialize)]
pub struct View {
    pub restriction: Option<crate::availability::Restriction>,
    pub budgets: Vec<Budget>,
    pub plan_type: Option<String>,
    pub source: String,
    pub observed_at: i64,
    pub stale: bool,
    pub windows: Vec<Window>,
}
impl Snapshot {
    /// Never extrapolate capacity from old data or an expired reset window.
    pub fn view(&self, now: i64) -> View {
        let age = now.checked_sub(self.observed_at);
        let old = age.is_none_or(|age| !(0..=300).contains(&age));
        let mut stale = old;
        let windows = self
            .windows
            .iter()
            .cloned()
            .map(|mut window| {
                if old || window.resets_at.is_some_and(|reset| reset <= now) {
                    window.remaining_percent = None;
                    stale = true;
                }
                window
            })
            .collect();
        let budgets = self
            .budgets
            .iter()
            .cloned()
            .map(|mut budget| {
                if old || budget.resets_at.is_some_and(|reset| reset <= now) {
                    budget.remaining = None;
                    budget.remaining_percent = None;
                    budget.unlimited = false;
                    stale = true;
                }
                budget
            })
            .collect();
        View {
            restriction: self
                .restriction
                .clone()
                .filter(|r| !old && r.available_at.is_none_or(|at| at > now)),
            budgets,
            plan_type: self.plan_type.clone(),
            source: self.source.clone(),
            observed_at: self.observed_at,
            stale,
            windows,
        }
    }
}
pub async fn query(
    profile: &crate::plan::AgentProfile,
    dir: &std::path::Path,
) -> Result<Snapshot, String> {
    use serde_json::json;
    use std::{
        process::Stdio,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tokio::{
        io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
        process::Command,
    };
    if profile.parser != crate::protocol::Provider::Codex {
        return Err("this CLI has no verified headless quota query; quota is unavailable".into());
    }
    tokio::fs::create_dir(dir)
        .await
        .map_err(|e| e.to_string())?;
    let dir = dir.canonicalize().map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())?;
    }
    let stderr = std::fs::File::create(dir.join("stderr.log")).map_err(|e| e.to_string())?;
    let executable = crate::executable::resolve(&profile.executable)
        .unwrap_or_else(|| profile.executable.clone().into());
    let mut command = Command::new(executable);
    command
        .arg("app-server")
        .envs(&profile.environment)
        .env_remove("OPENAI_API_KEY")
        .env_remove("CODEX_API_KEY")
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(stderr)
        .kill_on_drop(true);
    crate::process::ProcessGroup::prepare(&mut command);
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let pid = child.id().ok_or("quota query PID missing")?;
    let group = crate::process::ProcessGroup::adopt(&child)?;
    let mut owner = crate::process::Ownership {
        coordinator_pid: std::process::id(),
        pid: Some(pid),
        process_group: Some(pid as i32),
        phase: "running".into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    };
    crate::process::record_owner(&dir, &owner)?;
    let mut input = child.stdin.take().ok_or("quota query stdin missing")?;
    let mut output = BufReader::new(child.stdout.take().ok_or("quota query stdout missing")?);
    let mut spool = tokio::fs::File::create(dir.join("stdout.jsonl"))
        .await
        .map_err(|e| e.to_string())?;
    async fn send(input: &mut tokio::process::ChildStdin, value: Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec(&value).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        input.write_all(&bytes).await.map_err(|e| e.to_string())
    }
    let query = async {
        send(&mut input,json!({"method":"initialize","id":1,"params":{"clientInfo":{"name":"agentconductor","title":"AgentConductor","version":env!("CARGO_PKG_VERSION")}}})).await?;
        let mut initialized = false;
        let mut total = 0usize;
        for _ in 0..128 {
            let mut line = Vec::new();
            let count = (&mut output)
                .take(1024 * 1024 + 1)
                .read_until(b'\n', &mut line)
                .await
                .map_err(|e| e.to_string())?;
            total = total.saturating_add(count);
            if count == 0 {
                return Err("CLI closed before quota metadata was returned".into());
            }
            if total > 1024 * 1024 {
                return Err("quota metadata exceeds 1 MiB".into());
            }
            spool.write_all(&line).await.map_err(|e| e.to_string())?;
            let message: Value = serde_json::from_slice(&line).map_err(|e| e.to_string())?;
            if message["id"] == 1 && !initialized {
                if message.get("error").is_some() {
                    return Err(
                        "CLI metadata initialization failed; inspect local query logs".into(),
                    );
                }
                initialized = true;
                send(&mut input, json!({"method":"initialized","params":{}})).await?;
                send(
                    &mut input,
                    json!({"method":"account/rateLimits/read","id":2}),
                )
                .await?;
            } else if message["id"] == 2 && initialized {
                if let Some(error) = message.get("error") {
                    let detail = error["message"].as_str().unwrap_or_default();
                    if detail.contains("chatgpt authentication required") {
                        return Err("Codex is signed in with an API key; usage windows are reported only for ChatGPT sign-in (run codex login and choose ChatGPT)".into());
                    }
                    return Err("account quota query failed; inspect local query logs".into());
                }
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_secs() as i64;
                return parse_codex(&message, now);
            }
        }
        Err("too many quota metadata events".into())
    };
    let result = tokio::time::timeout(Duration::from_secs(30), query).await;
    owner.timed_out = result.is_err();
    let result = result
        .map_err(|_| "quota query timed out; no generation requested".to_string())
        .and_then(|r| r);
    drop(input);
    drop(group);
    let status = tokio::time::timeout(Duration::from_secs(3), child.wait())
        .await
        .map_err(|_| "quota child reap timed out")?
        .map_err(|e| e.to_string())?;
    owner.phase = "terminal".into();
    owner.exit_code = status.code();
    owner.termination_signal = crate::process::exit_signal(&status);
    crate::process::record_owner(&dir, &owner)?;
    result
}

fn profile_digest(profile: &crate::plan::AgentProfile) -> Result<String, String> {
    let mut profile = profile.clone();
    profile.quota_refresh_minutes = 5;
    profile.catalog = None;
    Ok(crate::workflow::digest(
        &serde_json::to_vec(&profile).map_err(|e| e.to_string())?,
    ))
}
pub fn persist(
    store: &mut crate::state::Store,
    profile: &crate::plan::AgentProfile,
    result: Result<Snapshot, String>,
) -> Result<(), String> {
    let (snapshot, error) = match result {
        Ok(snapshot) => (Some(snapshot), None),
        Err(error) => (None, Some(error)),
    };
    store.record(&format!("quota/{}",profile.id),&serde_json::json!({"profile_digest":profile_digest(profile)?,"snapshot":snapshot,"error":error}),&format!("Account quota metadata refreshed for agent {}; no generation or execution permission granted",profile.id))
}
pub fn stored(
    store: &crate::state::Store,
    profile: &crate::plan::AgentProfile,
    now: i64,
) -> Result<Value, String> {
    let unavailable = |error: &str| serde_json::json!({"status":"unavailable","windows":[],"error":error,"refresh_minutes":profile.quota_refresh_minutes});
    let Some(value) = store.optional_document(&format!("quota/{}", profile.id))? else {
        return Ok(unavailable("Account quota not queried"));
    };
    let mut legacy = profile.clone();
    legacy.quota_refresh_minutes = 5;
    let legacy_digest =
        crate::workflow::digest(&serde_json::to_vec(&legacy).map_err(|e| e.to_string())?);
    if value["profile_digest"] != profile_digest(profile)?
        && value["profile_digest"] != legacy_digest
    {
        return Ok(unavailable("CLI profile changed; refresh quota"));
    }
    if value["snapshot"].is_null() {
        return Ok(unavailable(
            value["error"]
                .as_str()
                .unwrap_or("Quota metadata unavailable"),
        ));
    }
    let snapshot: Snapshot =
        serde_json::from_value(value["snapshot"].clone()).map_err(|e| e.to_string())?;
    let view = snapshot.view(now);
    let mut value = serde_json::to_value(&view).map_err(|e| e.to_string())?;
    value["status"] = if view.windows.iter().all(|w| w.used_percent.is_none())
        && !snapshot.budgets.iter().any(Budget::available)
    {
        "unavailable"
    } else if view.stale {
        "stale"
    } else {
        "available"
    }
    .into();
    value["refresh_minutes"] = profile.quota_refresh_minutes.into();
    value["restriction"] =
        serde_json::to_value(restriction(&value, now)).map_err(|e| e.to_string())?;
    Ok(value)
}
