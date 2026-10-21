//! Chat with the coordinator from the workspace screen (GitHub #115, DD-171). The coordinator
//! runs read-only in the project folder: it can read the project and plans, and the manual
//! below, but it cannot change files or run commands; it tells the operator what to run.
use crate::{coordinator, plan::AgentProfile};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, time::Duration};

const MANUAL: &str = include_str!("../docs/user-manual.md");
const MAX_TURN_BYTES: usize = 8 * 1024;
const MAX_TURNS: usize = 12;
const MAX_ANSWER_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub role: String,
    pub text: String,
}

/// The prompt for the operator's latest question; only the most recent turns are sent.
pub fn build_prompt(turns: &[Turn], language: &str) -> Result<String, String> {
    let last = turns.last().ok_or("the conversation is empty")?;
    if last.role != "operator" {
        return Err("the conversation must end with an operator question".into());
    }
    if turns.iter().any(|t| {
        !matches!(t.role.as_str(), "operator" | "assistant")
            || t.text.trim().is_empty()
            || t.text.len() > MAX_TURN_BYTES
    }) {
        return Err(
            "each turn needs an operator or assistant role and 1..8192 bytes of text".into(),
        );
    }
    let recent = &turns[turns.len().saturating_sub(MAX_TURNS)..];
    let conversation: String = recent
        .iter()
        .map(|t| format!("{}: {}\n\n", t.role, t.text.trim()))
        .collect();
    Ok(format!(
        "You are the coordinator agent of AgentConductor, answering the operator's questions \
from the workspace screen.\n\
Rules:\n\
- You run read-only (Read, Grep, Glob only): you cannot change files or run commands. When the \
operator wants something run, give the exact command and where to run it.\n\
- For questions about the project (what was built, how to run or test it, where results are), \
inspect the project files, plans and documents first, then answer from what you found.\n\
- For questions about using AgentConductor or the current screen, answer from the manual below.\n\
- Do not invent files or commands; say when you could not find something.\n\
- Reply in {language}. Output one JSON object only: {{\"answer\": \"<Markdown text>\"}}.\n\n\
# Manual\n\n{MANUAL}\n\n# Conversation\n\n{conversation}Answer the last operator message."
    ))
}

/// The answer text of the coordinator's JSON reply, bounded for display.
pub fn answer_from(reply: &Value) -> Result<String, String> {
    let text = reply["answer"]
        .as_str()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or("the coordinator reply has no answer")?;
    let mut end = text.len().min(MAX_ANSWER_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Ok(text[..end].to_owned())
}

/// Asks the coordinator in the project folder; `dir` must not exist yet.
pub async fn ask(
    profile: &AgentProfile,
    project: &Path,
    dir: &Path,
    turns: &[Turn],
    language: &str,
) -> Result<String, String> {
    let prompt = build_prompt(turns, language)?;
    let reply = coordinator::run_json_within(
        profile,
        project,
        dir,
        &prompt,
        "answer",
        None,
        Duration::from_secs(300),
    )
    .await?;
    answer_from(&reply)
}
