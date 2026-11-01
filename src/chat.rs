//! Read-only coordinator guidance from the workspace screen (GitHub #115 / DD-171).
use crate::{coordinator, plan::AgentProfile};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, time::Duration};

const MANUAL: &str = include_str!("../docs/user-manual.md");
const MAX_TURN_BYTES: usize = 8 * 1024;
const MAX_TURNS: usize = 40;
const MAX_ANSWER_BYTES: usize = 32 * 1024;
const CLIPPED_TURN: &str = "\n[Earlier turn clipped]";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub role: String,
    pub text: String,
}

pub fn recent_history(turns: &[Turn]) -> Vec<Turn> {
    turns[turns.len().saturating_sub(MAX_TURNS)..].to_vec()
}

pub fn validate_question(text: &str) -> Result<(), String> {
    if text.trim().is_empty() || text.len() > MAX_TURN_BYTES {
        return Err("the latest question needs 1..8192 bytes of text".into());
    }
    Ok(())
}

pub fn build_prompt(turns: &[Turn], language: &str) -> Result<String, String> {
    build_prompt_in(turns, language, "")
}

/// Includes the bounded workspace/run evidence between the manual and conversation.
pub fn build_prompt_in(turns: &[Turn], language: &str, workspace: &str) -> Result<String, String> {
    let last = turns.last().ok_or("the conversation is empty")?;
    if last.role != "operator" {
        return Err("the conversation must end with an operator question".into());
    }
    validate_question(&last.text)?;
    for turn in turns {
        let limit = match turn.role.as_str() {
            "operator" => MAX_TURN_BYTES,
            "assistant" => MAX_ANSWER_BYTES,
            _ => return Err("each turn needs an operator or assistant role".into()),
        };
        if turn.text.trim().is_empty() || turn.text.len() > limit {
            return Err(format!(
                "each {} turn needs 1..{limit} bytes of text",
                turn.role
            ));
        }
    }
    let mut start = turns.len().saturating_sub(MAX_TURNS);
    loop {
        let conversation: String = turns[start..turns.len() - 1]
            .iter()
            .map(|turn| format!("{}: {}\n\n", turn.role, clip_earlier_turn(&turn.text)))
            .collect();
        let conversation = if conversation.is_empty() {
            "(none)\n\n".to_string()
        } else {
            conversation
        };
        let prompt = format!(
            "# Document\n\n## Manual\n\n{MANUAL}\n\n{workspace}\n\n## Earlier conversation\n\n{conversation}\\
# Task\n\nYou are the coordinator agent of AgentConductor, answering the operator's questions \
from the workspace screen.\n\
Rules:\n\
- You run read-only (Read, Grep, Glob only): you cannot change files or run commands. When the \
operator wants something run, give the exact command and where to run it.\n\
- For questions about the project (what was built, how to run or test it, where results are), \
inspect the project files, plans and documents first, then answer from what you found.\n\
- For questions about what was built, run results or how to run them, start from the Workspace \
section above, then read the files it names in the project folder. The run folder is outside your \
reach, so its evidence is only what the Workspace section quotes.\n\
- For questions about using AgentConductor or the current screen, answer from the manual in the Document.\n\
- Do not invent files or commands; say when you could not find something.\n\n\
Answer the last operator message:\n\n{}\n\n\
# Format\n\nReply in {language}. Output one JSON object only: {{\"answer\": \"<Markdown text>\"}}.\n",
            last.text.trim()
        );
        if prompt.len() <= 128 * 1024 {
            return Ok(prompt);
        }
        if start + 1 == turns.len() {
            return Err(
                "the latest question and manual exceed the coordinator prompt bound".into(),
            );
        }
        start += 1;
    }
}

fn clip_earlier_turn(text: &str) -> String {
    let max = MAX_TURN_BYTES - CLIPPED_TURN.len();
    let mut end = text.len().min(max);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    if end == text.len() {
        text.to_owned()
    } else {
        format!("{}{CLIPPED_TURN}", &text[..end])
    }
}

pub fn answer_from(reply: &Value) -> Result<String, String> {
    let text = reply["answer"]
        .as_str()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .ok_or("the coordinator reply has no answer")?;
    let mut end = text.len().min(MAX_ANSWER_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Ok(text[..end].to_owned())
}

pub async fn ask(
    profile: &AgentProfile,
    project: &Path,
    dir: &Path,
    turns: &[Turn],
    language: &str,
) -> Result<String, String> {
    ask_in(profile, project, dir, turns, language, "").await
}

pub async fn ask_in(
    profile: &AgentProfile,
    project: &Path,
    dir: &Path,
    turns: &[Turn],
    language: &str,
    workspace: &str,
) -> Result<String, String> {
    let prompt = build_prompt_in(turns, language, workspace)?;
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
