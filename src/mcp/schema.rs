use serde_json::Value;
/// Validate the discovered read-tool arguments before transmitting them.
pub(super) fn validate(v: &Value, s: &Value, depth: usize) -> Result<(), String> {
    if depth > 16 || !s.is_object() {
        return Err("unsupported MCP input schema".into());
    }
    if let Some(t) = s["type"].as_str() {
        let valid = match t {
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "integer" => v.is_i64() || v.is_u64(),
            "number" => v.is_number(),
            "boolean" => v.is_boolean(),
            "null" => v.is_null(),
            _ => false,
        };
        if !valid {
            return Err("MCP tool argument type does not match discovered schema".into());
        }
    }
    if let Some(values) = s["enum"].as_array()
        && !values.contains(v)
    {
        return Err("MCP argument is not in discovered enum".into());
    }
    if let Some(m) = v.as_object() {
        if s["required"].as_array().is_some_and(|keys| {
            keys.iter()
                .any(|k| k.as_str().is_none_or(|k| !m.contains_key(k)))
        }) {
            return Err("MCP tool required argument missing".into());
        }
        for (k, v) in m {
            if let Some(schema) = s["properties"].get(k) {
                validate(v, schema, depth + 1)?;
            } else if s["additionalProperties"] == false {
                return Err("unexpected MCP tool argument".into());
            }
        }
    }
    if let Some(a) = v.as_array()
        && let Some(items) = s.get("items")
    {
        for v in a {
            validate(v, items, depth + 1)?;
        }
    }
    Ok(())
}
