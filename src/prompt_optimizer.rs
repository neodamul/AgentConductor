//! Goal rewriting with the bundled `prompt-optimizer` skill (GitHub #180, DD-188). The
//! planning coordinator runs read-only in the project folder and returns a proposal; the
//! operator decides whether it replaces the goal. The skill text is vendored unmodified under
//! `third_party/prompt-optimizer` (Apache-2.0, getsentry/skills).
use crate::{coordinator, plan::AgentProfile};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::Path, time::Duration};

const SKILL: &str = include_str!("../third_party/prompt-optimizer/SKILL.md");
const CORE_PATTERNS: &str =
    include_str!("../third_party/prompt-optimizer/references/core-patterns.md");
const META_LOOP: &str =
    include_str!("../third_party/prompt-optimizer/references/meta-optimization-loop.md");
const MODEL_NOTES: &str =
    include_str!("../third_party/prompt-optimizer/references/model-family-notes.md");
const EXAMPLES: &str =
    include_str!("../third_party/prompt-optimizer/references/transformed-examples.md");
pub const MAX_GOAL_BYTES: usize = 8 * 1024;
const MAX_OPTIMIZED_BYTES: usize = 16 * 1024;
const MAX_CHANGES: usize = 12;
const MAX_QUESTIONS: usize = 8;
const MAX_NOTE_CHARS: usize = 400;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub optimized: String,
    pub changes: Vec<String>,
    pub questions: Vec<String>,
}

/// The coordinator prompt for one goal; the goal is the only operator-controlled text.
pub fn build_prompt(goal: &str, language: &str) -> Result<String, String> {
    let goal = goal.trim();
    if goal.is_empty() || goal.len() > MAX_GOAL_BYTES {
        return Err("the goal needs 1..8192 bytes of text".into());
    }
    // GitHub #211: Document (the skill, its references and the goal), Task (what to do), Format (the answer's shape).
    Ok(format!(
        "# Document\n\n## Skill: prompt-optimizer\n\n{SKILL}\n\n\
## Reference: core-patterns\n\n{CORE_PATTERNS}\n\n\
## Reference: meta-optimization-loop\n\n{META_LOOP}\n\n\
## Reference: model-family-notes\n\n{MODEL_NOTES}\n\n\
## Reference: transformed-examples\n\n{EXAMPLES}\n\n\
## Goal to rewrite\n\n{goal}\n\n\
# Task\n\nYou are the planning coordinator of AgentConductor. The operator wrote the goal above \
for a team of coding agents. Rewrite it with the prompt-optimizer skill above.\n\
Rules:\n\
- You run read-only (Read, Grep, Glob only): you cannot change files or run commands. \
Inspect the project folder only to inventory the files that govern the goal, such as \
AGENTS.md or docs, and name them by repo-relative path.\n\
- Keep the operator's intent, scope and wording of names, commands and paths. Do not invent \
requirements; when something is missing, ask in `questions` instead.\n\
- Apply the skill once. No eval cases can be run here, so do not claim measured results; \
state success criteria the operator can check.\n\n\
Rewrite this goal.\n\n\
# Format\n\n\
- The optimized goal stays in the language of the original goal. Reply in {language} for \
`changes` and `questions`.\n\
- Output one JSON object only: {{\"optimized\": \"<the rewritten goal>\", \"changes\": \
[\"<short statement of what changed and why>\"], \"questions\": [\"<what the operator should \
confirm>\"]}}.\n"
    ))
}

fn clip(text: &str, max_bytes: usize) -> &str {
    let mut end = text.len().min(max_bytes);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn notes(value: &Value, limit: usize) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .take(limit)
        .map(|t| match t.char_indices().nth(MAX_NOTE_CHARS) {
            Some((end, _)) => format!("{}…", &t[..end]),
            None => t.to_owned(),
        })
        .collect()
}

/// The coordinator's JSON reply as a bounded proposal.
pub fn proposal_from(reply: &Value) -> Result<Proposal, String> {
    let optimized = reply["optimized"]
        .as_str()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .ok_or("the coordinator reply has no optimized goal")?;
    Ok(Proposal {
        optimized: clip(optimized, MAX_OPTIMIZED_BYTES).to_owned(),
        changes: notes(&reply["changes"], MAX_CHANGES),
        questions: notes(&reply["questions"], MAX_QUESTIONS),
    })
}

/// Asks the coordinator in the project folder; `dir` must not exist yet.
pub async fn optimize(
    profile: &AgentProfile,
    project: &Path,
    dir: &Path,
    goal: &str,
    language: &str,
) -> Result<Proposal, String> {
    let prompt = build_prompt(goal, language)?;
    let reply = coordinator::run_json_within(
        profile,
        project,
        dir,
        &prompt,
        "optimized",
        None,
        Duration::from_secs(300),
    )
    .await?;
    proposal_from(&reply)
}
