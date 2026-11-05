//! R-43 phase 1 / DD-79: outbound messenger notifications (Slack Incoming Webhook or a
//! company webhook). Settings travel with the workspace; the URL is a machine-local secret.
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub mod channel;
pub mod conversation;
pub mod events;
pub mod inbound;
mod secrets;
mod send;
mod slack;
mod throttle;
mod webhook;
pub use channel::{Capabilities, Channel, Incoming, Kind};
pub use secrets::{save_secret, save_secret_value, secret, secret_value, status, validate_url};
pub use send::flush;

/// Notifiable run states plus `plan_ready`, in the order the dialog lists them (DD-229).
pub const EVENTS: [&str; 8] = [
    events::CATALOG[0].id,
    events::CATALOG[1].id,
    events::CATALOG[2].id,
    events::CATALOG[3].id,
    events::CATALOG[4].id,
    events::CATALOG[5].id,
    events::CATALOG[6].id,
    events::CATALOG[7].id,
];

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
    /// DD-81: listed approvers may answer pending tool requests from the messenger.
    #[serde(default)]
    pub decisions: bool,
    /// DD-230: listed talkers may chat with the coordinator from the messenger.
    #[serde(default)]
    pub conversation: bool,
    /// Slack user IDs, or relay user names, allowed to chat (separate from `approvers`).
    #[serde(default)]
    pub talkers: Vec<String>,
    /// Slack channel ID read for replies (`C0123…`).
    #[serde(default)]
    pub slack_channel: Option<String>,
    /// Slack user IDs, or relay user names, allowed to decide.
    #[serde(default)]
    pub approvers: Vec<String>,
}

impl Settings {
    pub(crate) fn clear_shared_authority(&mut self) {
        self.enabled = false;
        self.decisions = false;
        self.approvers.clear();
        self.slack_channel = None;
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            enabled: false,
            kind: Kind::Slack,
            events: all_events(),
            template: None,
            include_goal: false,
            decisions: false,
            conversation: false,
            talkers: vec![],
            slack_channel: None,
            approvers: vec![],
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

/// The JSON body for one event: Slack `{"text":…}` or the company template.
pub fn payload(settings: &Settings, event: &Event) -> Result<serde_json::Value, String> {
    let korean = crate::ui::language::load() == "ko";
    // DD-81: decision announcements and confirmations carry their own text.
    let decision = event.state == "decision";
    let what = if decision {
        event.goal.as_str()
    } else {
        events::text(&event.state, korean).ok_or("not a messenger event")?
    };
    let mut text = format!("AgentConductor · {}: {what}", event.workspace);
    if event.total > 0 && event.state != "plan_ready" {
        text.push_str(&format!(" ({}/{})", event.completed, event.total));
    }
    let goal = if settings.include_goal && !decision {
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
    settings
        .kind
        .channel()
        .notice_body(settings, &text, event, goal)
}

pub(crate) fn sibling(run: &Path, suffix: &str) -> PathBuf {
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
    if !settings.enabled {
        return;
    }
    // Every reported state is recorded, so re-entering a state after a retry notifies again (#72).
    let _ = std::fs::write(&sent, state);
    if !settings.events.iter().any(|e| e == state) {
        return;
    }
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

/// Plan completion from the desktop (DD-79 event source 2).
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
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let actionable = events::find(&event.state).is_some_and(|d| d.actionable);
    let summary = match throttle::admit(workspace, now, actionable) {
        throttle::Admit::Fold => {
            send::log(
                log.as_deref(),
                &event.state,
                "folded: more than 10 notices a minute",
            );
            return;
        }
        throttle::Admit::SendAfterSummary(n) => Some(n),
        throttle::Admit::Send => None,
    };
    let Some(url) = secret(workspace) else {
        send::log(
            log.as_deref(),
            &event.state,
            "no webhook URL saved on this computer",
        );
        return;
    };
    if let Some(n) = summary {
        let korean = crate::ui::language::load() == "ko";
        let text = crate::ui::messages::interpolate(
            crate::ui::messages::in_language(if korean { "ko" } else { "en" }, "messenger.folded"),
            &[("n", &n.to_string())],
        );
        let notice = Event {
            state: "decision".into(),
            workspace: event.workspace.clone(),
            goal: text,
            completed: 0,
            total: 0,
        };
        match payload(settings, &notice) {
            Ok(body) => send::spawn(url.clone(), body.to_string(), "folded".into(), log.clone()),
            Err(error) => send::log(log.as_deref(), "folded", &error),
        }
    }
    match payload(settings, event) {
        Ok(body) => send::spawn(url, body.to_string(), event.state.clone(), log),
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
