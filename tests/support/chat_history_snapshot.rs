use serde_json::Value;
use std::{fs, path::Path};

pub fn turns(root: &Path, saved: &Value) -> Option<Vec<Value>> {
    let result = saved["chat_result"].as_str()?;
    let id = result.strip_prefix("chat-")?.strip_suffix(".json")?;
    if id.is_empty()
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() || byte == b'-')
    {
        return None;
    }
    let mcp = root.join("mcp");
    let input: Value =
        serde_json::from_slice(&fs::read(mcp.join(format!("chat-input-{id}.json"))).ok()?).ok()?;
    let mut turns: Vec<Value> = serde_json::from_value(input["turns"].clone()).ok()?;
    let answer = fs::read(mcp.join(result))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .and_then(|value| value["answer"].as_str().map(str::to_owned));
    if let Some(answer) = answer {
        turns.push(serde_json::json!({"role":"assistant","text":answer}));
    }
    Some(turns)
}
