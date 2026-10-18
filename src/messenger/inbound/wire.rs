use super::Reply;
use sha2::{Digest, Sha256};

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
