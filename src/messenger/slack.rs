//! Slack: notices through an Incoming Webhook, decision replies read from `conversations.history`.
use super::channel::{Capabilities, Channel, Fetch, Incoming};
use super::inbound::{Reply, parse};
use super::{Event, Settings, secret_value, send};

pub struct Slack;

impl Channel for Slack {
    fn id(&self) -> &'static str {
        "slack"
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            notice: true,
            replies: true,
            conversation: true,
        }
    }
    fn notice_body(
        &self,
        _: &Settings,
        text: &str,
        _: &Event,
        _: &str,
    ) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({ "text": text }))
    }
    fn fetch_messages(&self, fetch: &Fetch<'_>) -> Result<(Vec<Incoming>, Vec<String>), String> {
        let body = self.history(fetch)?;
        Ok((slack_messages(&body)?, vec![]))
    }
    fn fetch_replies(&self, fetch: &Fetch<'_>) -> Result<(Vec<Reply>, Vec<String>), String> {
        Ok((slack_replies(&self.history(fetch)?)?, vec![]))
    }
}

impl Slack {
    /// The raw `conversations.history` body for the configured channel after the cursor.
    fn history(&self, fetch: &Fetch<'_>) -> Result<String, String> {
        let channel = fetch
            .settings
            .slack_channel
            .as_deref()
            .ok_or("Slack channel ID is not set")?;
        let token =
            secret_value(fetch.workspace, Some("token")).ok_or("Slack bot token is not saved")?;
        let oldest = if fetch.cursor.is_empty() {
            "0"
        } else {
            fetch.cursor
        };
        // Tests point this at a local HTTPS stand-in; production always uses slack.com.
        let api = std::env::var("AGENTCONDUCTOR_SLACK_API")
            .unwrap_or_else(|_| "https://slack.com/api".into());
        let url =
            format!("{api}/conversations.history?channel={channel}&oldest={oldest}&limit=100");
        send::fetch(&url, &[format!("Authorization: Bearer {token}")])
    }
}

/// Slack `conversations.history` messages → raw messages; bots and edits are ignored (own posts carry `bot_id`).
pub fn slack_messages(body: &str) -> Result<Vec<Incoming>, String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if value["ok"] != true {
        return Err(format!(
            "Slack error: {}",
            value["error"].as_str().unwrap_or("unknown")
        ));
    }
    let mut messages: Vec<Incoming> = value["messages"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["bot_id"].is_null() && m["subtype"].is_null())
        .filter_map(|m| {
            Some(Incoming {
                id: m["ts"].as_str()?.into(),
                sender: m["user"].as_str()?.into(),
                ts: m["ts"].as_str()?.into(),
                thread: m["thread_ts"].as_str().map(str::to_owned),
                text: m["text"].as_str()?.into(),
            })
        })
        .collect();
    messages.sort_by(|a, b| a.ts.cmp(&b.ts));
    Ok(messages)
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
