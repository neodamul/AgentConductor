//! Current-tool extraction from normalized CLI events (GitHub #100, DD-169). Provider-neutral:
//! Claude tool_use/tool_result, Codex command/MCP/file-change items, custom tool events.
use serde_json::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Start(String),
    End,
    None,
}

fn bounded(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(120)
        .collect()
}

pub fn tool_change(payload: &Value) -> Change {
    let kind = payload["type"].as_str().unwrap_or_default();
    match kind {
        "assistant" | "user" => {
            let content = payload
                .pointer("/message/content")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            if let Some(name) = content
                .iter()
                .rev()
                .find(|c| c["type"] == "tool_use")
                .and_then(|c| c["name"].as_str())
            {
                return Change::Start(bounded(name));
            }
            if content.iter().any(|c| c["type"] == "tool_result") {
                return Change::End;
            }
            Change::None
        }
        "item.started" | "item.updated" | "item.completed" => {
            let item = &payload["item"];
            let name = match item["type"].as_str() {
                Some("command_execution") => item["command"].as_str().map(bounded),
                Some("mcp_tool_call") => Some(bounded(&format!(
                    "{}.{}",
                    item["server"].as_str().unwrap_or("mcp"),
                    item["tool"].as_str().unwrap_or("tool")
                ))),
                Some("file_change") => Some("file change".to_owned()),
                _ => return Change::None,
            };
            match (kind, name) {
                ("item.completed", _) => Change::End,
                (_, Some(name)) => Change::Start(name),
                _ => Change::None,
            }
        }
        "tool" => payload["name"]
            .as_str()
            .map_or(Change::None, |name| Change::Start(bounded(name))),
        _ => Change::None,
    }
}

/// The page-facing activity of one running task: ages are facts, never a diagnosis.
pub fn summarize(doc: Option<&Value>, process: &str, now_ms: u64) -> Value {
    let age = |at: Option<u64>| at.and_then(|at| now_ms.checked_sub(at));
    let at = doc.and_then(|d| d["observed_at_ms"].as_u64());
    let tool = doc.map(|d| &d["tool"]).filter(|t| t["name"].is_string());
    serde_json::json!({
        "kind": doc.map(|d| d["kind"].clone()).unwrap_or(Value::Null),
        "observed_at_ms": at,
        "quiet_ms": age(at),
        "tool": tool.map(|t| serde_json::json!({
            "name": t["name"],
            "since_ms": t["since_ms"],
            "running_ms": age(t["since_ms"].as_u64()),
        })),
        "process": process,
    })
}
