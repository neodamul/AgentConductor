//! GitHub #514: what a running coordinator chat is doing, read from the tail of its CLI output.
//! Bounded on every side: the last 64 KiB, at most eight items, each at most 120 characters.
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const TAIL: u64 = 64 * 1024;
const ITEMS: usize = 8;
const TEXT: usize = 120;

fn clip(text: &str) -> String {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("");
    let mut out: String = line.chars().take(TEXT).collect();
    if line.chars().count() > TEXT {
        out.push('…');
    }
    out
}

/// A short hint of what a Claude tool call acts on.
fn hint(input: &Value) -> String {
    ["command", "file_path", "pattern", "path", "description"]
        .iter()
        .find_map(|key| input[*key].as_str().filter(|s| !s.trim().is_empty()))
        .map(|s| format!(" {}", clip(s)))
        .unwrap_or_default()
}

fn event_items(event: &Value) -> Vec<(&'static str, String)> {
    let mut out = vec![];
    match event["type"].as_str() {
        // Claude Code stream-json
        Some("assistant") => {
            for block in event["message"]["content"].as_array().into_iter().flatten() {
                match block["type"].as_str() {
                    Some("tool_use") => out.push((
                        "tool",
                        clip(&format!(
                            "{}{}",
                            block["name"].as_str().unwrap_or("tool"),
                            hint(&block["input"])
                        )),
                    )),
                    Some("text") => {
                        let text = clip(block["text"].as_str().unwrap_or(""));
                        if !text.is_empty() {
                            out.push(("text", text));
                        }
                    }
                    _ => {}
                }
            }
        }
        // Codex exec --json
        Some("item.started") => {
            let item = &event["item"];
            match item["type"].as_str() {
                Some("command_execution") => out.push((
                    "tool",
                    clip(&format!(
                        "command {}",
                        item["command"].as_str().unwrap_or("")
                    )),
                )),
                Some("mcp_tool_call") => out.push((
                    "tool",
                    clip(&format!(
                        "{}.{}",
                        item["server"].as_str().unwrap_or("mcp"),
                        item["tool"].as_str().unwrap_or("tool")
                    )),
                )),
                _ => {}
            }
        }
        Some("item.completed") if event["item"]["type"] == "agent_message" => {
            let text = clip(event["item"]["text"].as_str().unwrap_or(""));
            if !text.is_empty() {
                out.push(("text", text));
            }
        }
        _ => {}
    }
    out
}

/// The newest progress items of the CLI whose output is in `log_dir/stdout.jsonl`.
pub(super) fn items(log_dir: &Path) -> Vec<Value> {
    let Ok(mut file) = fs::File::open(log_dir.join("stdout.jsonl")) else {
        return vec![];
    };
    let size = file.metadata().map_or(0, |m| m.len());
    let start = size.saturating_sub(TAIL);
    let mut bytes = vec![];
    if file.seek(SeekFrom::Start(start)).is_err()
        || file.take(TAIL).read_to_end(&mut bytes).is_err()
    {
        return vec![];
    }
    let text = String::from_utf8_lossy(&bytes);
    // A tail that starts mid-file begins with a partial line.
    let mut lines = text.lines();
    if start > 0 {
        lines.next();
    }
    let mut all: Vec<(&str, String)> = lines
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .flat_map(|event| event_items(&event))
        .collect();
    let skip = all.len().saturating_sub(ITEMS);
    all.drain(..skip);
    all.into_iter()
        .map(|(kind, text)| json!({"kind": kind, "text": text}))
        .collect()
}

#[cfg(test)]
#[path = "../../../tests/unit/chat_progress.rs"]
mod tests;
