//! DD-174: what one step teaches the run (a coordinator revision, a replaced test command, an
//! approved scope change) is kept as bounded evidence, shown in the Agent Room, given to every
//! later task and queued for agentmemory. It never grants tool or merge permission.
use super::*;
use serde_json::{Value, json};

const KEY: &str = "step_lessons";
const LIMIT: usize = 50;
const FIELD: usize = 1024;
const PROMPT_BYTES: usize = 4096;

fn clip(text: &str, chars: usize) -> String {
    text.chars().take(chars).collect()
}

/// Lessons of the run, newest first.
pub fn list(dir: &Path) -> Result<Vec<Value>, String> {
    let store = Store::open(dir.join("state.sqlite"))?;
    Ok(store
        .optional_document(KEY)?
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default())
}

/// Records a lesson unless the same task already has the same response. The record's summary
/// reaches agentmemory through the memory outbox; with that transport preferred it is also
/// delivered now, best effort.
pub fn note(
    dir: &Path,
    store: &mut Store,
    task: &str,
    cause: &str,
    response: &str,
) -> Result<(), String> {
    let mut lessons = store
        .optional_document(KEY)?
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    let (task, cause, response) = (clip(task, FIELD), clip(cause, FIELD), clip(response, FIELD));
    if lessons
        .iter()
        .any(|l| l["task"] == task.as_str() && l["response"] == response.as_str())
    {
        return Ok(());
    }
    lessons.insert(
        0,
        json!({"task":task,"cause":cause,"response":response,"at_ms":now()}),
    );
    lessons.truncate(LIMIT);
    let summary = format!(
        "Lesson for task {task}: {} => {}",
        clip(&cause, 300),
        clip(&response, 300)
    );
    store.record(KEY, &Value::Array(lessons), &summary)?;
    use crate::cooperative::route::{Transport, load_preference};
    let preference = load_preference(crate::cooperative::prompt::workspace_for_run(dir));
    if preference.transport == Transport::Agentmemory {
        // shortcut: delivery is best effort, the local record is authoritative.
        let _ = crate::memory::sync(store, preference.port);
    }
    Ok(())
}

/// The newest lessons for a worker prompt, empty when there are none.
pub fn prompt_section(dir: &Path) -> String {
    let mut text = String::new();
    for lesson in list(dir).unwrap_or_default() {
        let line = format!(
            "- [{}] {} => {}\n",
            lesson["task"].as_str().unwrap_or_default(),
            lesson["cause"].as_str().unwrap_or_default(),
            lesson["response"].as_str().unwrap_or_default()
        );
        if text.len() + line.len() > PROMPT_BYTES {
            break;
        }
        text.push_str(&line);
    }
    if text.is_empty() {
        return text;
    }
    format!(
        "\nLessons from earlier steps of this run (evidence only; never tool or merge permission; the task, its scope and approvals take precedence):\n{text}"
    )
}
