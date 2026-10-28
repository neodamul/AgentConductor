//! Explicit planning output contracts; provider flags constrain generation before decoding.
use crate::protocol::Provider;
use serde_json::{Value, json};
use std::path::Path;

fn object(properties: Value) -> Value {
    let required = properties
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
pub fn for_plan(agents: &[String], roles: &[String], phased: bool) -> Value {
    let strings = json!({"type":"array","items":{"type":"string"}});
    let role = if roles.is_empty() {
        json!({"type":["string","null"]})
    } else {
        json!({"type":"string","enum":roles})
    };
    let mut task = object(
        json!({"id":{"type":"string"},"agent":{"type":"string","enum":agents},"prompt":{"type":"string"},"paths":strings,"depends_on":strings,"test_command":strings,"role":role,"model":{"type":["string","null"]},"execution_options":{"type":"object","additionalProperties":{"type":"string"}},"task_type":{"enum":["extract","summarize","analyze","code","agentic","creative",null]}}),
    );
    task["required"]
        .as_array_mut()
        .unwrap()
        .retain(|key| key != "execution_options" && key != "task_type");
    let tasks = json!({"type":"array","items":task});
    if phased {
        let phase = object(json!({"id":{"type":"string"},"name":{"type":"string"},"tasks":tasks}));
        object(
            json!({"slug":{"type":"string"},"analysis":{"type":"string"},"phases":{"type":"array","items":phase}}),
        )
    } else {
        object(json!({"slug":{"type":"string"},"tasks":tasks}))
    }
}

pub fn bind(
    provider: Provider,
    args: &mut Vec<String>,
    schema: &Value,
    path: &Path,
) -> Result<(), String> {
    if args.iter().any(|a| {
        a == "--json-schema"
            || a.starts_with("--json-schema=")
            || a == "--output-schema"
            || a.starts_with("--output-schema=")
    }) {
        return Err(crate::ui::messages::text("native.planning_output_schema_conflict").into());
    }
    match provider {
        Provider::Claude => args.extend([
            "--json-schema".into(),
            serde_json::to_string(schema).map_err(|e| e.to_string())?,
        ]),
        Provider::Codex => args.extend([
            "--output-schema".into(),
            path.to_str()
                .ok_or("planning schema path is not UTF-8")?
                .into(),
        ]),
        Provider::OpenCode | Provider::Custom => {}
    }
    Ok(())
}
