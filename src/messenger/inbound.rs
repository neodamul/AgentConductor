//! R-43 phase 2 / DD-81: answer pending tool requests from the messenger. Polls only while
//! a request is pending, inside the run process; decisions go through the DD-32 path.
use super::{Kind, Settings, link_path, secret, secret_value, send, sibling};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

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

/// `allow 3FA9C21B` / `deny 3fa9c21b`, optionally after a mention.
pub fn parse(text: &str) -> Option<(bool, String)> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_ascii_alphanumeric())
                .to_lowercase()
        })
        .collect();
    words.windows(2).find_map(|pair| {
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

/// RFC 2104 HMAC-SHA256 with the existing `sha2` crate.
pub fn hmac(key: &[u8], message: &[u8]) -> String {
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let pad = |byte: u8| block.iter().map(|b| b ^ byte).collect::<Vec<_>>();
    let inner = Sha256::new()
        .chain_update(pad(0x36))
        .chain_update(message)
        .finalize();
    let outer = Sha256::new()
        .chain_update(pad(0x5c))
        .chain_update(inner)
        .finalize();
    format!("{outer:x}")
}

fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0, |acc, (x, y)| acc | (x ^ y))
            == 0
}

/// Slack `conversations.history` messages → replies; bots and edits are ignored.
pub fn slack_replies(body: &str) -> Result<Vec<Reply>, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if value["ok"] != true {
        return Err(format!(
            "Slack error: {}",
            value["error"].as_str().unwrap_or("unknown")
        ));
    }
    let mut replies: Vec<Reply> = value["messages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["bot_id"].is_null() && m["subtype"].is_null())
        .filter_map(|m| {
            let (allow, code) = parse(m["text"].as_str()?)?;
            Some(Reply {
                id: m["ts"].as_str()?.into(),
                user: m["user"].as_str()?.into(),
                allow,
                code,
                ts: m["ts"].as_str()?.into(),
            })
        })
        .collect();
    replies.sort_by(|a, b| a.ts.cmp(&b.ts));
    Ok(replies)
}

/// Company relay decisions; each must carry a valid HMAC over `code|decision|user|ts`.
pub fn relay_replies(body: &str, secret: &str) -> Result<(Vec<Reply>, Vec<String>), String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    let mut replies = vec![];
    let mut refused = vec![];
    for d in value["decisions"].as_array().into_iter().flatten() {
        let field = |k: &str| d[k].as_str().unwrap_or_default().to_string();
        let (code, decision, user, ts) =
            (field("code"), field("decision"), field("user"), field("ts"));
        let expected = hmac(
            secret.as_bytes(),
            format!("{code}|{decision}|{user}|{ts}").as_bytes(),
        );
        let allow = match decision.as_str() {
            "allow" => true,
            "deny" => false,
            _ => {
                refused.push(format!("{ts}: unknown decision"));
                continue;
            }
        };
        if !same(&expected, &field("signature")) {
            refused.push(format!("{ts}: invalid signature"));
            continue;
        }
        replies.push(Reply {
            id: ts.clone(),
            user,
            allow,
            code: code.to_uppercase(),
            ts,
        });
    }
    replies.sort_by(|a, b| a.ts.cmp(&b.ts));
    Ok((replies, refused))
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
    for request in &pending {
        if state.announced.contains(&request.id) {
            continue;
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
                outcome.announced += 1;
            }
            Err(e) => log(run, &format!("announce {}: {e}", code(&request.id))),
        }
    }
    let (replies, refused) = match settings.kind {
        Kind::Slack => {
            let channel = settings
                .slack_channel
                .as_deref()
                .ok_or("Slack channel ID is not set")?;
            let token =
                secret_value(&workspace, Some("token")).ok_or("Slack bot token is not saved")?;
            let oldest = if state.cursor.is_empty() {
                "0"
            } else {
                &state.cursor
            };
            // Tests point this at a local HTTPS stand-in; production always uses slack.com.
            let api = std::env::var("AGENTCONDUCTOR_SLACK_API")
                .unwrap_or_else(|_| "https://slack.com/api".into());
            let url =
                format!("{api}/conversations.history?channel={channel}&oldest={oldest}&limit=100");
            let body = send::fetch(&url, &[format!("Authorization: Bearer {token}")])?;
            (slack_replies(&body)?, vec![])
        }
        Kind::Webhook => {
            let relay = secret_value(&workspace, Some("relay")).ok_or("relay URL is not saved")?;
            let key = secret_value(&workspace, Some("relay_secret"))
                .ok_or("relay secret is not saved")?;
            let joiner = if relay.contains('?') { '&' } else { '?' };
            let body = send::fetch(&format!("{relay}{joiner}since={}", state.cursor), &[])?;
            relay_replies(&body, &key)?
        }
    };
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
                let status = std::process::Command::new(exe)
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
