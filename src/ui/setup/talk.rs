//! DD-230: the desktop side of the messenger conversation. Each request tick does at most one
//! step: finish an answer, hand the next queued message to the coordinator chat, or start a
//! poll. Network work runs on one short-lived thread; nothing polls while the desktop is closed.
use super::*;
use crate::messenger::{conversation, talk};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

#[derive(Default)]
pub(super) struct Talk {
    busy: Arc<AtomicBool>,
    next_poll: AtomicU64,
}

fn seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl Startup {
    pub(super) fn talk_tick(&self, saved: &mut Saved) {
        let settings = saved.config.messenger.clone();
        if !(settings.enabled && settings.conversation)
            || self.talk.busy.swap(true, Ordering::SeqCst)
        {
            return;
        }
        let (root, name, now) = (
            self.broker.dir.clone(),
            saved.config.name.clone(),
            seconds(),
        );
        let busy = self.talk.busy.clone();
        let mut inbox = talk::load(&root);
        let running = self
            .chat_job(saved)
            .is_ok_and(|job| job["state"] == "running");
        // The thread posts, then lets the next tick go on.
        let post = |texts: Vec<String>| {
            let (root, settings, name, busy) =
                (root.clone(), settings.clone(), name.clone(), busy.clone());
            std::thread::spawn(move || {
                for text in texts {
                    if let Err(e) = talk::say(&root, &settings, &name, &text) {
                        talk::log(&root, &format!("answer: {e}"));
                        break;
                    }
                }
                busy.store(false, Ordering::SeqCst);
            });
        };
        if inbox.in_flight.is_some() {
            if running && saved.chat_result == inbox.chat {
                return busy.store(false, Ordering::SeqCst);
            }
            let answer = inbox
                .chat
                .take()
                .and_then(|file| crate::mcp::load(&root.join("mcp").join(file)).ok())
                .and_then(|data| serde_json::from_slice::<serde_json::Value>(&data).ok())
                .and_then(|v| v["answer"].as_str().map(str::to_owned));
            inbox.in_flight = None;
            let _ = talk::save(&root, &inbox);
            let text = answer.unwrap_or_else(|| messages::text("messenger.reply.failed").into());
            return post(conversation::split_answer(&text));
        }
        if !running && let Some(item) = conversation::next(&mut inbox) {
            let started = self.chat_revision(saved).and_then(|revision| {
                self.launch_chat(saved, conversation::operator_turn(&item), &revision)
            });
            let failed = started.is_err();
            if let Err(e) = started {
                talk::log(&root, &format!("hand-off {}: {e}", item.id));
                inbox.in_flight = None;
            } else {
                inbox.chat = saved.chat_result.clone();
            }
            let _ = talk::save(&root, &inbox);
            return if failed {
                post(vec![messages::text("messenger.reply.failed").into()])
            } else {
                busy.store(false, Ordering::SeqCst)
            };
        }
        if now < self.talk.next_poll.load(Ordering::SeqCst) {
            return busy.store(false, Ordering::SeqCst);
        }
        self.talk
            .next_poll
            .store(now + talk::interval(&inbox, now), Ordering::SeqCst);
        std::thread::spawn(move || {
            if let Err(e) = talk::poll(&root, &settings, &name, now) {
                talk::log(&root, &format!("poll: {e}"));
            }
            busy.store(false, Ordering::SeqCst);
        });
    }
}
