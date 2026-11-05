//! DD-230: the I/O around the conversation core: poll the channel, answer rejections, remember
//! the inbox. The desktop calls it from a short-lived thread (no resident listener).
use super::{
    Event, Settings,
    channel::{Fetch, Incoming},
    conversation::{self, Inbox, Verdict},
    payload, secret, send,
};
use std::path::{Path, PathBuf};

/// 30 s while a conversation is going on, 120 s when idle (DD-230).
const ACTIVE: u64 = 30;
const IDLE: u64 = 120;
const RECENT: u64 = 600;

fn file(root: &Path) -> PathBuf {
    root.join("messenger-conversation.json")
}

pub fn log(root: &Path, line: &str) {
    send::log(Some(&root.join("messenger.log")), "conversation", line);
}

pub fn load(root: &Path) -> Inbox {
    std::fs::read_to_string(file(root))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save(root: &Path, inbox: &Inbox) -> Result<(), String> {
    std::fs::write(
        file(root),
        serde_json::to_vec(inbox).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// Seconds until the next poll.
pub fn interval(inbox: &Inbox, now: u64) -> u64 {
    // Tests poll at once; production never sets this.
    if let Some(seconds) = std::env::var("AGENTCONDUCTOR_MESSENGER_POLL_SECONDS")
        .ok()
        .and_then(|v| v.parse().ok())
    {
        return seconds;
    }
    let busy = inbox.in_flight.is_some() || !inbox.queue.is_empty();
    if busy || now < inbox.last_talk + RECENT {
        ACTIVE
    } else {
        IDLE
    }
}

/// Posts one text to the saved channel as a plain notice.
pub fn say(root: &Path, settings: &Settings, name: &str, text: &str) -> Result<(), String> {
    let url = secret(root).ok_or("no webhook URL saved on this computer")?;
    let event = Event {
        state: "decision".into(),
        workspace: name.into(),
        goal: text.into(),
        completed: 0,
        total: 0,
    };
    payload(settings, &event).and_then(|body| send::deliver(&url, &body.to_string()))
}

/// Runs every fetched message through the core. The cursor moves with each message, and the first
/// poll only learns where the channel stands so its history is never answered.
pub fn apply(
    settings: &Settings,
    inbox: &mut Inbox,
    messages: Vec<Incoming>,
    now: u64,
    say: &mut dyn FnMut(String),
) {
    let (first, before) = (!inbox.started, inbox.cursor.clone());
    inbox.started = true;
    for message in messages {
        if message.ts > inbox.cursor {
            inbox.cursor = message.ts.clone();
        }
        // A channel that repeats what the cursor already passed must not be answered twice.
        if first || (!before.is_empty() && message.ts <= before) {
            continue;
        }
        match conversation::accept(settings, inbox, &message, now) {
            Verdict::Reject(id) => say(crate::ui::messages::text(id).into()),
            Verdict::Queued { place } if place > 1 => say(crate::ui::messages::render(
                "messenger.ack.queued",
                &[("value1", &place.to_string())],
            )),
            _ => {}
        }
    }
}

/// One poll: fetch, decide, answer rejections, save. Errors are returned for the log.
pub fn poll(root: &Path, settings: &Settings, name: &str, now: u64) -> Result<(), String> {
    let channel = settings.kind.channel();
    if !channel.capabilities().conversation {
        return Err("this messenger channel cannot read messages".into());
    }
    let mut inbox = load(root);
    let (messages, refused) = channel.fetch_messages(&Fetch {
        workspace: root,
        settings,
        cursor: &inbox.cursor,
    })?;
    for line in refused {
        log(root, &line);
    }
    let mut say = |text: String| {
        if let Err(e) = say(root, settings, name, &text) {
            log(root, &format!("reply: {e}"));
        }
    };
    apply(settings, &mut inbox, messages, now, &mut say);
    save(root, &inbox)
}

#[cfg(test)]
#[path = "../../tests/unit/messenger_talk.rs"]
mod tests;
