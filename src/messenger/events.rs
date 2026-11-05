//! DD-229: the event catalogue. One row per event the messenger can carry; the localized text
//! lives in the message catalogs (`messenger.event.<id>`), never in literals here.

/// What the notice may carry (DD-229 "Detail").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Detail {
    /// Event, workspace and counts.
    State,
    /// Adds the task id.
    Task,
    /// Adds the task id and the tool name.
    Tool,
}

/// What makes two notices "the same" for de-duplication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Run,
    Request,
    Task,
}

#[derive(Clone, Copy, Debug)]
pub struct EventDef {
    pub id: &'static str,
    /// Message id of the localized text.
    pub message: &'static str,
    /// The code path that raises the event.
    pub source: &'static str,
    /// Whether the notice carries a code a sender can answer (DD-81, DD-229).
    pub actionable: bool,
    pub key: Key,
    pub detail: Detail,
}

const fn row(
    id: &'static str,
    message: &'static str,
    source: &'static str,
    actionable: bool,
    key: Key,
    detail: Detail,
) -> EventDef {
    EventDef {
        id,
        message,
        source,
        actionable,
        key,
        detail,
    }
}

/// In the order the settings dialog lists them.
pub const CATALOG: [EventDef; 8] = [
    row(
        "plan_ready",
        "messenger.event.plan_ready",
        "notify_plan",
        false,
        Key::Run,
        Detail::State,
    ),
    row(
        "waiting_approval",
        "messenger.event.waiting_approval",
        "notify_run",
        false,
        Key::Run,
        Detail::State,
    ),
    row(
        "waiting_tool_approval",
        "messenger.event.waiting_tool_approval",
        "notify_run + tool ledger",
        true,
        Key::Request,
        Detail::Tool,
    ),
    row(
        "waiting_availability",
        "messenger.event.waiting_availability",
        "notify_run",
        false,
        Key::Run,
        Detail::State,
    ),
    row(
        "completed",
        "messenger.event.completed",
        "notify_run",
        false,
        Key::Run,
        Detail::State,
    ),
    row(
        "failed",
        "messenger.event.failed",
        "notify_run",
        false,
        Key::Run,
        Detail::State,
    ),
    row(
        "stopped",
        "messenger.event.stopped",
        "notify_run",
        false,
        Key::Run,
        Detail::State,
    ),
    row(
        "blocked",
        "messenger.event.blocked",
        "notify_run",
        false,
        Key::Run,
        Detail::State,
    ),
];

pub fn find(id: &str) -> Option<&'static EventDef> {
    CATALOG.iter().find(|def| def.id == id)
}

/// The localized text of an event; the dialog's test message has its own id.
pub fn text(state: &str, korean: bool) -> Option<&'static str> {
    let message = match find(state) {
        Some(def) => def.message,
        None if state == "test" => "messenger.event.test",
        None => return None,
    };
    Some(crate::ui::messages::in_language(
        if korean { "ko" } else { "en" },
        message,
    ))
}
