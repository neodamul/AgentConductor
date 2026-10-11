//! R-43 phase 1 / DD-78: outbound messenger notifications (Slack Incoming Webhook or a
//! company webhook). Settings travel with the workspace; the URL is a machine-local secret.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

mod send;
pub use send::flush;

/// Notifiable run states plus `plan_ready`, in the order the dialog lists them.
pub const EVENTS: [&str; 8] = [
    "plan_ready",
    "waiting_approval",
    "waiting_tool_approval",
    "waiting_availability",
    "completed",
    "failed",
    "stopped",
    "blocked",
];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    #[default]
    Slack,
    Webhook,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub kind: Kind,
    #[serde(default = "all_events")]
    pub events: Vec<String>,
    /// Company webhook body; `None` means `{"text":"{{text}}"}`.
    #[serde(default)]
    pub template: Option<String>,
    /// The goal text is sent only when allowed (data minimization, NFR-17).
    #[serde(default)]
    pub include_goal: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: Kind::Slack,
            events: all_events(),
            template: None,
            include_goal: false,
        }
    }
}

fn all_events() -> Vec<String> {
    EVENTS.iter().map(|e| e.to_string()).collect()
}

pub struct Event {
    pub state: String,
    pub workspace: String,
    pub goal: String,
    pub completed: usize,
    pub total: usize,
}

fn label(state: &str, korean: bool) -> Option<&'static str> {
    Some(match (state, korean) {
        ("plan_ready", false) => "plan ready for review",
        ("plan_ready", true) => "계획 검토 준비 완료",
        ("waiting_approval", false) => "candidate ready for review",
        ("waiting_approval", true) => "후보 검토 대기",
        ("waiting_tool_approval", false) => "tool request needs a decision",
        ("waiting_tool_approval", true) => "도구 요청 결정 필요",
        ("waiting_availability", false) => "waiting for account availability",
        ("waiting_availability", true) => "계정 사용 한도로 대기 중",
        ("completed", false) => "run completed",
        ("completed", true) => "실행 완료",
        ("failed", false) => "run failed",
        ("failed", true) => "실행 실패",
        ("stopped", false) => "run stopped",
        ("stopped", true) => "실행 중지",
        ("blocked", false) => "run blocked",
        ("blocked", true) => "실행 차단",
        ("test", false) => "test message",
        ("test", true) => "테스트 메시지",
        _ => return None,
    })
}

/// The JSON body for one event: Slack `{"text":…}` or the company template.
pub fn payload(settings: &Settings, event: &Event) -> Result<serde_json::Value, String> {
    let korean = crate::ui::language::load() == "ko";
    let what = label(&event.state, korean).ok_or("not a messenger event")?;
    let mut text = format!("AgentConductor · {}: {what}", event.workspace);
    if event.total > 0 && event.state != "plan_ready" {
        text.push_str(&format!(" ({}/{})", event.completed, event.total));
    }
    let goal = if settings.include_goal {
        event.goal.as_str()
    } else {
        ""
    };
    if !goal.is_empty() {
        text.push_str(&format!(
            "\n{}: {goal}",
            if korean { "목표" } else { "Goal" }
        ));
    }
    if settings.kind == Kind::Slack {
        return Ok(serde_json::json!({ "text": text }));
    }
    let template = settings
        .template
        .as_deref()
        .unwrap_or(r#"{"text":"{{text}}"}"#);
    let escape = |s: &str| {
        let quoted = serde_json::to_string(s).unwrap_or_default();
        quoted[1..quoted.len() - 1].to_string()
    };
    let body = template
        .replace("{{text}}", &escape(&text))
        .replace("{{event}}", &escape(&event.state))
        .replace("{{state}}", &escape(&event.state))
        .replace("{{workspace}}", &escape(&event.workspace))
        .replace("{{goal}}", &escape(goal));
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("message body is not JSON: {e}"))?;
    if !value.is_object() {
        return Err("message body must be a JSON object".into());
    }
    Ok(value)
}

fn secrets_path() -> Result<PathBuf, String> {
    Ok(crate::desktop::workspace()?
        .parent()
        .ok_or("invalid data directory")?
        .join("messenger-secrets.json"))
}

fn key(workspace: &Path) -> String {
    let path = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    crate::workflow::digest(path.to_string_lossy().as_bytes())
}

fn secrets() -> serde_json::Map<String, serde_json::Value> {
    secrets_path()
        .ok()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

/// The workspace's webhook URL, if one is saved on this computer.
pub fn secret(workspace: &Path) -> Option<String> {
    secrets()
        .get(&key(workspace))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

pub fn validate_url(url: &str) -> Result<(), String> {
    if !url.starts_with("https://") || url.len() > 2048 || url.chars().any(char::is_control) {
        return Err("webhook URL must be an https:// address".into());
    }
    Ok(())
}

/// Saves (`Some`) or removes (`None`) the URL; the file is readable by the owner only.
pub fn save_secret(workspace: &Path, url: Option<&str>) -> Result<(), String> {
    let mut all = secrets();
    match url {
        Some(url) => {
            validate_url(url)?;
            all.insert(key(workspace), url.into());
        }
        None => {
            all.remove(&key(workspace));
        }
    }
    let path = secrets_path()?;
    std::fs::create_dir_all(path.parent().ok_or("invalid secrets path")?)
        .map_err(|e| e.to_string())?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(
        &temporary,
        serde_json::to_vec_pretty(&all).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))
            .map_err(|e| e.to_string())?;
    }
    std::fs::rename(&temporary, &path).map_err(|e| e.to_string())
}

fn sibling(run: &Path, suffix: &str) -> PathBuf {
    let name = run.file_name().map(|n| n.to_string_lossy().to_string());
    run.with_file_name(format!("{}.{suffix}", name.unwrap_or_default()))
}

pub fn link_path(run: &Path) -> PathBuf {
    sibling(run, "messenger.json")
}

/// Written by the desktop before a run starts: which workspace and settings it reports to.
pub fn link_run(
    run: &Path,
    workspace: &Path,
    name: &str,
    goal: &str,
    settings: &Settings,
) -> Result<(), String> {
    let path = link_path(run);
    std::fs::create_dir_all(path.parent().ok_or("invalid run path")?).map_err(|e| e.to_string())?;
    let link =
        serde_json::json!({"workspace":workspace,"name":name,"goal":goal,"settings":settings});
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&link).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// Called on every report save; sends at most one message per new notifiable state.
/// Never fails the workflow: problems go to `<run>.messenger.log`.
pub fn notify_run(run: &Path, state: &str, completed: usize, total: usize) {
    let Ok(text) = std::fs::read_to_string(link_path(run)) else {
        return;
    };
    let Ok(link) = serde_json::from_str::<serde_json::Value>(&text) else {
        return;
    };
    let Ok(settings) = serde_json::from_value::<Settings>(link["settings"].clone()) else {
        return;
    };
    let sent = sibling(run, "messenger-sent");
    if std::fs::read_to_string(&sent).is_ok_and(|last| last == state) {
        return;
    }
    if !settings.enabled || !settings.events.iter().any(|e| e == state) {
        return;
    }
    let _ = std::fs::write(&sent, state);
    let workspace = PathBuf::from(link["workspace"].as_str().unwrap_or_default());
    let event = Event {
        state: state.into(),
        workspace: link["name"].as_str().unwrap_or_default().into(),
        goal: link["goal"].as_str().unwrap_or_default().into(),
        completed,
        total,
    };
    deliver_later(
        &workspace,
        &settings,
        &event,
        Some(sibling(run, "messenger.log")),
    );
}

/// Plan completion from the desktop (DD-78 event source 2).
pub fn notify_plan(workspace: &Path, settings: &Settings, name: &str, goal: &str, tasks: usize) {
    if settings.enabled && settings.events.iter().any(|e| e == "plan_ready") {
        let event = Event {
            state: "plan_ready".into(),
            workspace: name.into(),
            goal: goal.into(),
            completed: 0,
            total: tasks,
        };
        deliver_later(
            workspace,
            settings,
            &event,
            Some(workspace.join("messenger.log")),
        );
    }
}

fn deliver_later(workspace: &Path, settings: &Settings, event: &Event, log: Option<PathBuf>) {
    let result = secret(workspace)
        .ok_or_else(|| "no webhook URL saved on this computer".to_string())
        .and_then(|url| Ok((url, payload(settings, event)?.to_string())));
    match result {
        Ok((url, body)) => send::spawn(url, body, event.state.clone(), log),
        Err(error) => send::log(log.as_deref(), &event.state, &error),
    }
}

/// A representative event for validating a template before it is saved.
pub fn sample() -> Event {
    Event {
        state: "completed".into(),
        workspace: "Workspace".into(),
        goal: "Goal \"quoted\"\nsecond line".into(),
        completed: 1,
        total: 1,
    }
}

/// The dialog's **Send test message**: waits for the result.
pub fn test(
    workspace: &Path,
    settings: &Settings,
    url: Option<&str>,
    name: &str,
) -> Result<(), String> {
    let url = match url {
        Some(url) => {
            validate_url(url)?;
            url.to_string()
        }
        None => secret(workspace).ok_or("enter the webhook URL first")?,
    };
    let event = Event {
        state: "test".into(),
        workspace: name.into(),
        goal: String::new(),
        completed: 0,
        total: 0,
    };
    send::deliver(&url, &payload(settings, &event)?.to_string())
}
