//! R-43 phase 2 / DD-81: answer pending tool requests from the messenger. Polls only while
//! a request is pending, inside the run process; decisions go through the DD-32 path.
pub use super::slack::slack_replies;
pub use super::webhook::{hmac, relay_replies};
use super::{Settings, link_path, secret, send, sibling};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);
/// DD-229: at most this many announced requests wait for an answer; further ones queue in order.
const OUTSTANDING: usize = 20;

/// Remembered between polls: announced requests, the reply cursor and handled messages.
#[derive(Default, Serialize, Deserialize)]
struct State {
    announced: Vec<String>,
    cursor: String,
    seen: Vec<String>,
}

/// One reply from the messenger: who answered what.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub id: String,
    pub user: String,
    pub allow: bool,
    pub code: String,
    pub ts: String,
}

/// What one poll did, for tests and the run log.
#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    pub announced: usize,
    pub fetched: bool,
    pub applied: Vec<(String, bool, String)>,
    pub ignored: Vec<String>,
}

/// The 8-character code shown for a request (its SHA-256 prefix, upper case).
pub fn code(id: &str) -> String {
    id.chars().take(8).collect::<String>().to_uppercase()
}

/// `allow 3FA9C21B` / `deny 3fa9c21b`, optionally after mentions; any other leading word is not a decision (#81).
pub fn parse(text: &str) -> Option<(bool, String)> {
    let words: Vec<String> = text
        .split_whitespace()
        .skip_while(|w| w.starts_with("<@") || w.starts_with('@'))
        .take(2)
        .map(|w| {
            w.trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_lowercase()
        })
        .collect();
    words.windows(2).next().and_then(|pair| {
        let allow = match pair[0].as_str() {
            "allow" | "approve" => true,
            "deny" | "reject" => false,
            _ => return None,
        };
        let code = &pair[1];
        (code.len() == 8 && code.chars().all(|c| c.is_ascii_hexdigit()))
            .then(|| (allow, code.to_uppercase()))
    })
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn log(run: &Path, line: &str) {
    send::log(Some(&sibling(run, "messenger.log")), "decision", line);
}

/// One poll. `apply(id, allow)` performs the desktop-equivalent decision.
pub fn poll_once(
    run: &Path,
    apply: &mut dyn FnMut(&str, bool) -> Result<(), String>,
) -> Result<Outcome, String> {
    let mut outcome = Outcome::default();
    let Ok(text) = std::fs::read_to_string(link_path(run)) else {
        return Ok(outcome);
    };
    let link: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let settings: Settings =
        serde_json::from_value(link["settings"].clone()).map_err(|e| e.to_string())?;
    if !settings.enabled || !settings.decisions {
        return Ok(outcome);
    }
    let workspace = PathBuf::from(link["workspace"].as_str().unwrap_or_default());
    let name = link["name"].as_str().unwrap_or_default().to_string();
    let ledger = crate::gates::Ledger::open(&run.join("state.sqlite"))?;
    let pending: Vec<_> = ledger
        .requests()?
        .into_iter()
        .filter(|r| r.status == "pending" && r.expires_at.is_none_or(|t| t > now()))
        .collect();
    if pending.is_empty() {
        return Ok(outcome);
    }
    let state_path = sibling(run, "messenger-inbound.json");
    let mut state: State = std::fs::read_to_string(&state_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    let korean = crate::ui::language::load() == "ko";
    let post = |text: String| match secret(&workspace) {
        Some(url) => super::payload(&settings, &announcement(&name, text))
            .and_then(|body| send::deliver(&url, &body.to_string())),
        None => Err("no webhook URL saved on this computer".into()),
    };
    let mut outstanding = pending
        .iter()
        .filter(|r| state.announced.contains(&r.id))
        .count();
    for request in &pending {
        if state.announced.contains(&request.id) {
            continue;
        }
        // A request that is not announced yet keeps its place and is announced as slots free up.
        if outstanding >= OUTSTANDING {
            break;
        }
        let text = if korean {
            format!(
                "도구 요청 결정 필요 — 작업 {}, 도구 {}. `allow {code}` 또는 `deny {code}`로 답하세요.",
                request.context.task,
                request.request.tool,
                code = code(&request.id)
            )
        } else {
            format!(
                "tool request needs a decision — task {}, tool {}. Reply `allow {code}` or `deny {code}`.",
                request.context.task,
                request.request.tool,
                code = code(&request.id)
            )
        };
        match post(text) {
            Ok(()) => {
                state.announced.push(request.id.clone());
                outstanding += 1;
                outcome.announced += 1;
            }
            Err(e) => log(run, &format!("announce {}: {e}", code(&request.id))),
        }
    }
    let channel = settings.kind.channel();
    if !channel.capabilities().replies {
        return Err("this messenger channel cannot read replies".into());
    }
    let (replies, refused) = channel.fetch_replies(&super::channel::Fetch {
        workspace: &workspace,
        settings: &settings,
        cursor: &state.cursor,
    })?;
    outcome.fetched = true;
    for line in refused {
        log(run, &line);
        outcome.ignored.push(line);
    }
    for reply in replies {
        if reply.ts > state.cursor {
            state.cursor = reply.ts.clone();
        }
        if state.seen.contains(&reply.id) {
            continue;
        }
        state.seen.push(reply.id.clone());
        let verdict = if !settings.approvers.contains(&reply.user) {
            Err(format!("{} is not an approver", reply.user))
        } else {
            let mut matches = pending.iter().filter(|r| code(&r.id) == reply.code);
            match (matches.next(), matches.next()) {
                (Some(one), None) => apply(&one.id, reply.allow).map(|()| one),
                (None, _) => Err(format!("no pending request {}", reply.code)),
                _ => Err(format!("code {} is ambiguous", reply.code)),
            }
        };
        match verdict {
            Ok(request) => {
                let what = if reply.allow { "allow" } else { "deny" };
                let _ = crate::state::Store::open(run.join("state.sqlite")).and_then(|mut s| {
                    s.record(
                        &format!("messenger_decision/{}", reply.id),
                        &serde_json::json!({"request":request.id,"decision":what,"approver":reply.user,"message":reply.id}),
                        "Tool request decided from the messenger",
                    )
                });
                let text = if korean {
                    format!(
                        "{}가 {} (작업 {})",
                        reply.user,
                        if reply.allow {
                            "허용했습니다"
                        } else {
                            "거부했습니다"
                        },
                        request.context.task
                    )
                } else {
                    format!(
                        "{} by {} (task {})",
                        if reply.allow { "Allowed" } else { "Denied" },
                        reply.user,
                        request.context.task
                    )
                };
                if let Err(e) = post(text) {
                    log(run, &format!("confirm {}: {e}", reply.code));
                }
                outcome
                    .applied
                    .push((request.id.clone(), reply.allow, reply.user.clone()));
            }
            Err(reason) => {
                log(run, &format!("{}: {reason}", reply.id));
                outcome.ignored.push(reason);
            }
        }
    }
    if state.seen.len() > 500 {
        let extra = state.seen.len() - 500;
        state.seen.drain(..extra);
    }
    std::fs::write(
        &state_path,
        serde_json::to_vec(&state).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(outcome)
}

fn announcement(name: &str, text: String) -> super::Event {
    super::Event {
        state: "decision".into(),
        workspace: name.into(),
        goal: text,
        completed: 0,
        total: 0,
    }
}

/// Started by `startup-run` for a linked run; ends with the run process.
pub fn spawn(run: PathBuf) {
    let decisions = std::fs::read_to_string(link_path(&run))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .is_some_and(|link| link["settings"]["decisions"] == true);
    if !decisions {
        return;
    }
    std::thread::spawn(move || {
        let exe = std::env::current_exe().ok();
        loop {
            std::thread::sleep(INTERVAL);
            let Some(exe) = exe.as_ref() else { return };
            let mut apply = |id: &str, allow: bool| {
                let status = crate::subprocess::command(exe)
                    .args([if allow { "tool-approve" } else { "tool-reject" }])
                    .arg(&run)
                    .arg(id)
                    .stdout(std::process::Stdio::null())
                    .status()
                    .map_err(|e| e.to_string())?;
                status
                    .success()
                    .then_some(())
                    .ok_or_else(|| "decision refused".to_string())
            };
            if let Err(e) = poll_once(&run, &mut apply) {
                log(&run, &e);
            }
        }
    });
}
