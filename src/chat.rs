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
const MAX_STORED_TURNS: usize = 40;
const MAX_ANSWER_BYTES: usize = 32 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    pub role: String,
    pub text: String,
}

/// The stored conversation plus the operator's new question, limited to the recent turns.
pub fn next_turns(mut history: Vec<Turn>, text: &str) -> Result<Vec<Turn>, String> {
    let text = text.trim();
    if text.is_empty() || text.len() > MAX_TURN_BYTES {
        return Err("a question needs 1..8192 bytes of text".into());
    }
    history.push(Turn {
        role: "operator".into(),
        text: text.into(),
    });
    let skip = history.len().saturating_sub(MAX_STORED_TURNS);
    Ok(history.split_off(skip))
}

/// The conversation of the latest chat (`result` is its `chat-<id>.json` artifact name):
/// the turns sent with the last question and, once it finished, the answer. Empty when
/// nothing is stored or readable.
pub fn history(root: &Path, result: &str) -> Vec<Turn> {
    let read = |name: &str| -> Option<Value> {
        let data = crate::mcp::load(&root.join("mcp").join(name)).ok()?;
        serde_json::from_slice(&data).ok()
    };
    let Some(id) = result
        .strip_prefix("chat-")
        .and_then(|r| r.strip_suffix(".json"))
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'))
    else {
        return Vec::new();
    };
    let Some(input) = read(&format!("chat-input-{id}.json")) else {
        return Vec::new();
    };
    let mut turns: Vec<Turn> = serde_json::from_value(input["turns"].clone()).unwrap_or_default();
    if let Some(answer) = read(result).and_then(|v| v["answer"].as_str().map(str::to_owned)) {
        turns.push(Turn {
            role: "assistant".into(),
            text: answer,
        });
    }
    turns
}

fn clipped(text: &str) -> &str {
    let mut end = text.len().min(MAX_TURN_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// The prompt for the operator's latest question; only the most recent turns are sent and an
/// earlier long answer is clipped rather than refused.
pub fn build_prompt(turns: &[Turn], language: &str) -> Result<String, String> {
    build_prompt_in(turns, language, "")
}

/// The same prompt with the workspace section (GitHub #243, see `chat_context`) between the manual and the conversation.
pub fn build_prompt_in(turns: &[Turn], language: &str, workspace: &str) -> Result<String, String> {
    let last = turns.last().ok_or("the conversation is empty")?;
    if last.role != "operator" {
        return Err("the conversation must end with an operator question".into());
    }
    if last.text.trim().is_empty() || last.text.len() > MAX_TURN_BYTES {
        return Err("the question needs 1..8192 bytes of text".into());
    }
    if turns
        .iter()
        .any(|t| !matches!(t.role.as_str(), "operator" | "assistant") || t.text.trim().is_empty())
    {
        return Err("each turn needs an operator or assistant role and some text".into());
    }
    let recent = &turns[turns.len().saturating_sub(MAX_TURNS)..];
    let (question, earlier) = recent.split_last().ok_or("the conversation is empty")?;
    let earlier: String = earlier
        .iter()
        .map(|t| format!("{}: {}\n\n", t.role, clipped(t.text.trim())))
        .collect();
    let earlier = if earlier.is_empty() {
        "(none)\n\n".to_string()
    } else {
        earlier
    };
    // GitHub #211: Document (manual, earlier turns), Task (rules and the current question), Format (the answer's shape).
    Ok(format!(
        "# Document\n\n## Manual\n\n{MANUAL}\n\n{workspace}\n\n## Earlier conversation\n\n{earlier}\
# Task\n\nYou are the coordinator agent of AgentConductor, answering the operator's questions \
from the workspace screen.\n\
Rules:\n\
- You run read-only (Read, Grep, Glob only): you cannot change files or run commands. When the \
operator wants something run, give the exact command and where to run it.\n\
- For questions about the project (what was built, how to run or test it, where results are), \
inspect the project files, plans and documents first, then answer from what you found.\n\
- For what was built, the run's results or how to run them, start from the Workspace section of the Document \
(goal, tasks, tests, changed files and the candidate diff), then read the files it names in the project folder. \
The run folder is outside your reach, so its evidence is only what the Workspace section quotes.\n\
- For questions about using AgentConductor or the current screen, answer from the manual in the Document.\n\
- Do not invent files or commands; say when you could not find something.\n\n\
Answer the last operator message:\n\n{}\n\n\
# Format\n\nReply in {language}. Output one JSON object only: {{\"answer\": \"<Markdown text>\"}}.\n",
        clipped(question.text.trim())
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
    ask_in(profile, project, dir, turns, language, "").await
}

/// `ask` with the workspace section of the prompt (GitHub #243).
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
