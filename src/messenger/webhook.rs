//! Company webhook: notices with a configurable JSON body, decision replies from a signed relay.
use super::channel::{Capabilities, Channel, Fetch};
use super::inbound::Reply;
use super::{Event, Settings, secret_value, send};
use sha2::{Digest, Sha256};

pub struct Webhook;

impl Channel for Webhook {
    fn id(&self) -> &'static str {
        "webhook"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            notice: true,
            replies: true,
            conversation: false,
        }
    }
    fn uses_template(&self) -> bool {
        true
    }
    fn notice_body(
        &self,
        settings: &Settings,
        text: &str,
        event: &Event,
        goal: &str,
    ) -> Result<serde_json::Value, String> {
        let template = settings
            .template
            .as_deref()
            .unwrap_or(r#"{"text":"{{text}}"}"#);
        let escape = |s: &str| {
            let quoted = serde_json::to_string(s).unwrap_or_default();
            quoted[1..quoted.len() - 1].to_string()
        };
        let body = template
            .replace("{{text}}", &escape(text))
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
    fn fetch_replies(&self, fetch: &Fetch<'_>) -> Result<(Vec<Reply>, Vec<String>), String> {
        let relay = secret_value(fetch.workspace, Some("relay")).ok_or("relay URL is not saved")?;
        let key = secret_value(fetch.workspace, Some("relay_secret"))
            .ok_or("relay secret is not saved")?;
        let joiner = if relay.contains('?') { '&' } else { '?' };
        let body = send::fetch(&format!("{relay}{joiner}since={}", fetch.cursor), &[])?;
        relay_replies(&body, &key)
    }
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
