use serde_json::Value;

#[derive(Debug, Eq, PartialEq)]
pub struct Entry {
    pub label: String,
    pub text: String,
    pub tone: Tone,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tone {
    Normal,
    Muted,
    Tool,
    Result,
    Warning,
    Error,
}

pub fn event_entries(provider: &str, event: &Value) -> Vec<Entry> {
    let mut entries = transcript(provider, &event["payload"]);
    if event["kind"] == "warning" {
        for entry in &mut entries {
            entry.label = "room.warning".into();
            entry.tone = Tone::Warning;
        }
    }
    entries
}
pub fn transcript(provider: &str, raw: &Value) -> Vec<Entry> {
    let mut entries = Vec::new();
    let (kind, item) = (raw["type"].as_str().unwrap_or_default(), &raw["item"]);
    match provider {
        "claude" => match kind {
            "system" => add(&mut entries, "room.session", "Session started", Tone::Muted),
            "assistant" => {
                for content in raw
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    match content["type"].as_str().unwrap_or_default() {
                        "text" => {
                            add_value(&mut entries, "room.agent", &content["text"], Tone::Normal)
                        }
                        "tool_use" => add(
                            &mut entries,
                            "room.tool",
                            &tool_line(&content["name"], &content["input"]),
                            Tone::Tool,
                        ),
                        "thinking" => add(
                            &mut entries,
                            "room.thinking",
                            &clip_lines(&content["thinking"], 4),
                            Tone::Muted,
                        ),
                        _ => {}
                    }
                }
            }
            "user" => {
                for content in raw
                    .pointer("/message/content")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                {
                    if content["type"] == "tool_result" {
                        let error = content["is_error"] == true;
                        add(
                            &mut entries,
                            if error {
                                "room.tool_error"
                            } else {
                                "room.result"
                            },
                            &clip_lines(&content["content"], 8),
                            if error { Tone::Error } else { Tone::Result },
                        );
                    }
                }
            }
            "result" => add_value(
                &mut entries,
                if raw["is_error"] == true {
                    "room.failed_031a8f"
                } else {
                    "room.finished"
                },
                &raw["result"],
                if raw["is_error"] == true {
                    Tone::Error
                } else {
                    Tone::Result
                },
            ),
            "error" => add_value(&mut entries, "room.error", &raw["message"], Tone::Error),
            _ => unknown(&mut entries, kind, raw),
        },
        "codex" => match kind {
            "thread.started" => add(&mut entries, "room.session", "Session started", Tone::Muted),
            "turn.completed" => {
                let usage = &raw["usage"];
                if usage.is_object() {
                    add(
                        &mut entries,
                        "room.usage",
                        &format!(
                            "{} in / {} out",
                            usage["input_tokens"].as_u64().unwrap_or(0),
                            usage["output_tokens"].as_u64().unwrap_or(0)
                        ),
                        Tone::Muted,
                    );
                }
            }
            "turn.failed" | "error" => add_value(
                &mut entries,
                "room.error",
                raw.pointer("/error/message").unwrap_or(&raw["message"]),
                Tone::Error,
            ),
            "item.started" | "item.updated" | "item.completed" => {
                match item["type"].as_str().unwrap_or_default() {
                    "agent_message" if kind == "item.completed" => {
                        add_value(&mut entries, "room.agent", &item["text"], Tone::Normal)
                    }
                    "reasoning" if kind == "item.completed" => add(
                        &mut entries,
                        "room.thinking",
                        &clip_lines(&item["text"], 4),
                        Tone::Muted,
                    ),
                    "command_execution" if kind == "item.started" => add(
                        &mut entries,
                        "room.tool",
                        &format!("● $ {}", item["command"].as_str().unwrap_or("")),
                        Tone::Tool,
                    ),
                    "command_execution" if kind == "item.completed" => {
                        let error = item["exit_code"].as_i64().unwrap_or(1) != 0;
                        add(
                            &mut entries,
                            if error {
                                "room.tool_error"
                            } else {
                                "room.result"
                            },
                            &format!(
                                "exit {}{}",
                                item["exit_code"]
                                    .as_i64()
                                    .map_or("?".into(), |code| code.to_string()),
                                suffix(&item["aggregated_output"])
                            ),
                            if error { Tone::Error } else { Tone::Result },
                        );
                    }
                    "file_change" if kind == "item.completed" => add(
                        &mut entries,
                        "room.tool",
                        &format!(
                            "{} files changed",
                            item["changes"].as_array().map_or(0, Vec::len)
                        ),
                        Tone::Tool,
                    ),
                    "mcp_tool_call" if kind == "item.completed" => add(
                        &mut entries,
                        "room.tool",
                        &format!(
                            "● {}.{}",
                            item["server"].as_str().unwrap_or("mcp"),
                            item["tool"].as_str().unwrap_or("")
                        ),
                        Tone::Tool,
                    ),
                    "web_search" if kind == "item.completed" => {
                        add_value(&mut entries, "room.tool", &item["query"], Tone::Tool)
                    }
                    "todo_list" => add(&mut entries, "room.plan", "Task list updated", Tone::Muted),
                    "error" => add_value(&mut entries, "room.error", &item["message"], Tone::Error),
                    _ => {}
                }
            }
            _ => unknown(&mut entries, kind, raw),
        },
        "opencode" => match kind {
            "text" => add_value(
                &mut entries,
                "room.agent",
                raw.pointer("/part/text").unwrap_or(&raw["text"]),
                Tone::Normal,
            ),
            "tool_use" => {
                let state = &raw["part"]["state"];
                add(
                    &mut entries,
                    "room.tool",
                    &tool_line(&raw["part"]["tool"], &state["input"]),
                    Tone::Tool,
                );
                if state["output"].is_string() {
                    add(
                        &mut entries,
                        "room.result",
                        &clip_lines(&state["output"], 8),
                        Tone::Result,
                    );
                }
            }
            "error" => add_value(
                &mut entries,
                "room.error",
                raw.pointer("/error/message").unwrap_or(&raw["message"]),
                Tone::Error,
            ),
            "step_start" | "step_finish" => {}
            _ => unknown(&mut entries, kind, raw),
        },
        _ => unknown(&mut entries, kind, raw),
    }
    entries
}

fn unknown(entries: &mut Vec<Entry>, kind: &str, raw: &Value) {
    let message = raw
        .pointer("/payload/summary")
        .or_else(|| raw.get("summary"))
        .or_else(|| raw.get("text"))
        .or_else(|| raw.get("message"));
    if kind.is_empty() && message.is_none() {
        return;
    }
    let text = message
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    add(
        entries,
        if kind.is_empty() { "room.status" } else { kind },
        &text,
        Tone::Muted,
    );
}

pub(super) fn tool_line(name: &Value, input: &Value) -> String {
    let argument = [
        "command",
        "file_path",
        "filePath",
        "path",
        "pattern",
        "url",
        "query",
    ]
    .iter()
    .find_map(|key| input[*key].as_str())
    .unwrap_or_default();
    format!(
        "● {}({})",
        name.as_str().unwrap_or("tool"),
        argument
            .lines()
            .next()
            .unwrap_or_default()
            .chars()
            .take(200)
            .collect::<String>()
    )
}

fn suffix(value: &Value) -> String {
    value
        .as_str()
        .filter(|value| !value.is_empty())
        .map(|value| format!("\n{}", clip(value, 4000)))
        .unwrap_or_default()
}

fn result_text(value: &Value) -> String {
    if let Some(value) = value.as_str() {
        return value.into();
    }
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|item| item["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn clip_lines(value: &Value, maximum: usize) -> String {
    let text = result_text(value);
    let lines = text.lines().collect::<Vec<_>>();
    let mut output = lines
        .iter()
        .take(maximum)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    if lines.len() > maximum {
        output.push_str(&format!("\n… {} more lines", lines.len() - maximum));
    }
    clip(&output, 4000)
}

fn clip(value: &str, maximum: usize) -> String {
    let mut chars = value.chars();
    let output = chars.by_ref().take(maximum).collect::<String>();
    if chars.next().is_some() {
        format!("{output}…")
    } else {
        output
    }
}

fn add(entries: &mut Vec<Entry>, label: &str, text: &str, tone: Tone) {
    if !text.is_empty() {
        entries.push(Entry {
            label: label.into(),
            text: text.into(),
            tone,
        });
    }
}

fn add_value(entries: &mut Vec<Entry>, label: &str, value: &Value, tone: Tone) {
    add(entries, label, &clip(&result_text(value), 4000), tone);
}
