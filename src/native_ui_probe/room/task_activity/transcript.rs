use serde_json::Value;

#[path = "transcript/entry.rs"]
mod entry;
#[path = "transcript/format.rs"]
mod format;
use self::entry::{add, add_value, unknown};
pub(super) use self::format::tool_line;
use self::format::{clip_lines_in_language, suffix};

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
    event_entries_in_language(provider, event, "en")
}
pub fn event_entries_in_language(provider: &str, event: &Value, language: &str) -> Vec<Entry> {
    let mut entries = transcript_in_language(provider, &event["payload"], language);
    if event["kind"] == "warning" {
        for entry in &mut entries {
            entry.label = "room.warning".into();
            entry.tone = Tone::Warning;
        }
    }
    entries
}
pub fn transcript(provider: &str, raw: &Value) -> Vec<Entry> {
    transcript_in_language(provider, raw, "en")
}
pub fn transcript_in_language(provider: &str, raw: &Value, language: &str) -> Vec<Entry> {
    let mut entries = Vec::new();
    let (kind, item) = (raw["type"].as_str().unwrap_or_default(), &raw["item"]);
    match provider {
        "claude" => match kind {
            "system" => add(
                &mut entries,
                "room.session",
                crate::ui::messages::in_language(language, "room.session_started"),
                Tone::Muted,
            ),
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
                            &clip_lines_in_language(&content["thinking"], 4, language),
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
                            &clip_lines_in_language(&content["content"], 8, language),
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
            "thread.started" => add(
                &mut entries,
                "room.session",
                crate::ui::messages::in_language(language, "room.session_started"),
                Tone::Muted,
            ),
            "turn.completed" => {
                let usage = &raw["usage"];
                if usage.is_object() {
                    add(
                        &mut entries,
                        "room.usage",
                        &crate::ui::messages::interpolate(
                            crate::ui::messages::in_language(language, "room.tokens_in_out"),
                            &[
                                (
                                    "value1",
                                    &usage["input_tokens"].as_u64().unwrap_or(0).to_string(),
                                ),
                                (
                                    "value2",
                                    &usage["output_tokens"].as_u64().unwrap_or(0).to_string(),
                                ),
                            ],
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
                        &clip_lines_in_language(&item["text"], 4, language),
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
                                "{}{}",
                                crate::ui::messages::interpolate(
                                    crate::ui::messages::in_language(language, "room.exit_e1fe73"),
                                    &[(
                                        "value1",
                                        &item["exit_code"]
                                            .as_i64()
                                            .map_or("?".into(), |code| code.to_string())
                                    )],
                                ),
                                suffix(&item["aggregated_output"])
                            ),
                            if error { Tone::Error } else { Tone::Result },
                        );
                    }
                    "file_change" if kind == "item.completed" => add(
                        &mut entries,
                        "room.tool",
                        &crate::ui::messages::interpolate(
                            crate::ui::messages::in_language(language, "room.files_changed_count"),
                            &[(
                                "value1",
                                &item["changes"].as_array().map_or(0, Vec::len).to_string(),
                            )],
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
                    "todo_list" => add(
                        &mut entries,
                        "room.plan",
                        crate::ui::messages::in_language(language, "room.task_list_updated"),
                        Tone::Muted,
                    ),
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
                        &clip_lines_in_language(&state["output"], 8, language),
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
