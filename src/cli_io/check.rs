//! A checker for the JSON Schema subset the acio schemas use (draft 2020-12 keywords: `type`, `enum`, `const`,
//! `required`, `properties`, `patternProperties`, `additionalProperties`, `propertyNames`, `items`, bounds,
//! `pattern`, `format: date-time`, `allOf`, `if`/`then` and local `$ref`). Other keywords are annotations.
//! Every violation is reported as `<json pointer>: <reason>`.
use super::{pattern::Pattern, timestamp};
use serde_json::Value;

pub(super) fn check(root: &Value, instance: &Value) -> Vec<String> {
    let mut errors = vec![];
    walk(root, root, instance, "", &mut errors);
    errors
}

fn fail(errors: &mut Vec<String>, path: &str, message: String) {
    errors.push(format!(
        "{}: {message}",
        if path.is_empty() { "/" } else { path }
    ));
}

fn is_type(instance: &Value, name: &str) -> bool {
    match name {
        "object" => instance.is_object(),
        "array" => instance.is_array(),
        "string" => instance.is_string(),
        "boolean" => instance.is_boolean(),
        "null" => instance.is_null(),
        "number" => instance.is_number(),
        "integer" => {
            instance.is_i64()
                || instance.is_u64()
                || instance.as_f64().is_some_and(|f| f.fract() == 0.0)
        }
        _ => false,
    }
}

fn valid(root: &Value, schema: &Value, instance: &Value) -> bool {
    let mut errors = vec![];
    walk(root, schema, instance, "", &mut errors);
    errors.is_empty()
}

fn walk(root: &Value, schema: &Value, instance: &Value, path: &str, errors: &mut Vec<String>) {
    let Some(schema) = schema.as_object() else {
        if schema == &Value::Bool(false) {
            fail(errors, path, "no value is allowed here".into());
        }
        return;
    };
    if let Some(target) = schema.get("$ref").and_then(Value::as_str) {
        match target.strip_prefix('#').and_then(|p| root.pointer(p)) {
            Some(found) => walk(root, found, instance, path, errors),
            None => fail(errors, path, format!("unresolved reference {target}")),
        }
    }
    if let Some(t) = schema.get("type") {
        let names: Vec<&str> = match t {
            Value::String(s) => vec![s],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        if !names.iter().any(|n| is_type(instance, n)) {
            fail(
                errors,
                path,
                format!("expected type {}", names.join(" or ")),
            );
            return;
        }
    }
    if let Some(Value::Array(options)) = schema.get("enum")
        && !options.contains(instance)
    {
        let shown: Vec<String> = options.iter().map(Value::to_string).collect();
        fail(
            errors,
            path,
            format!("{instance} is not one of {}", shown.join(", ")),
        );
    }
    if let Some(expected) = schema.get("const")
        && expected != instance
    {
        fail(
            errors,
            path,
            format!("{expected} was expected, got {instance}"),
        );
    }
    match instance {
        Value::String(text) => string(schema, text, path, errors),
        Value::Number(n) => {
            let value = n.as_f64().unwrap_or(f64::NAN);
            if let Some(min) = schema.get("minimum").and_then(Value::as_f64)
                && value < min
            {
                fail(errors, path, format!("{value} is below the minimum {min}"));
            }
            if let Some(max) = schema.get("maximum").and_then(Value::as_f64)
                && value > max
            {
                fail(errors, path, format!("{value} is above the maximum {max}"));
            }
        }
        Value::Array(items) => array(root, schema, items, path, errors),
        Value::Object(map) => object(root, schema, map, path, errors),
        _ => {}
    }
    if let Some(Value::Array(all)) = schema.get("allOf") {
        for part in all {
            walk(root, part, instance, path, errors);
        }
    }
    if let (Some(condition), Some(then)) = (schema.get("if"), schema.get("then"))
        && valid(root, condition, instance)
    {
        walk(root, then, instance, path, errors);
    }
}

fn string(
    schema: &serde_json::Map<String, Value>,
    text: &str,
    path: &str,
    errors: &mut Vec<String>,
) {
    let length = text.chars().count();
    if let Some(min) = schema.get("minLength").and_then(Value::as_u64)
        && (length as u64) < min
    {
        fail(
            errors,
            path,
            format!("shorter than the minimum length {min}"),
        );
    }
    if let Some(max) = schema.get("maxLength").and_then(Value::as_u64)
        && (length as u64) > max
    {
        fail(
            errors,
            path,
            format!("longer than the maximum length {max}"),
        );
    }
    if let Some(source) = schema.get("pattern").and_then(Value::as_str) {
        match Pattern::new(source) {
            Ok(pattern) if !pattern.is_match(text) => {
                fail(errors, path, format!("{text:?} does not match {source}"));
            }
            Err(e) => fail(errors, path, format!("unsupported pattern {source}: {e}")),
            Ok(_) => {}
        }
    }
    if schema.get("format").and_then(Value::as_str) == Some("date-time")
        && timestamp::parse(text).is_none()
    {
        fail(
            errors,
            path,
            format!("{text:?} is not an RFC 3339 date-time"),
        );
    }
}

fn array(
    root: &Value,
    schema: &serde_json::Map<String, Value>,
    items: &[Value],
    path: &str,
    errors: &mut Vec<String>,
) {
    if let Some(min) = schema.get("minItems").and_then(Value::as_u64)
        && (items.len() as u64) < min
    {
        fail(errors, path, format!("fewer than the minimum {min} items"));
    }
    if let Some(max) = schema.get("maxItems").and_then(Value::as_u64)
        && (items.len() as u64) > max
    {
        fail(errors, path, format!("more than the maximum {max} items"));
    }
    if schema.get("uniqueItems") == Some(&Value::Bool(true))
        && items
            .iter()
            .enumerate()
            .any(|(i, a)| items[..i].contains(a))
    {
        fail(errors, path, "items must be unique".into());
    }
    if let Some(item) = schema.get("items") {
        for (i, value) in items.iter().enumerate() {
            walk(root, item, value, &format!("{path}/{i}"), errors);
        }
    }
}

fn object(
    root: &Value,
    schema: &serde_json::Map<String, Value>,
    map: &serde_json::Map<String, Value>,
    path: &str,
    errors: &mut Vec<String>,
) {
    if let Some(Value::Array(required)) = schema.get("required") {
        for name in required.iter().filter_map(Value::as_str) {
            if !map.contains_key(name) {
                fail(errors, path, format!("{name:?} is a required property"));
            }
        }
    }
    if let Some(min) = schema.get("minProperties").and_then(Value::as_u64)
        && (map.len() as u64) < min
    {
        fail(
            errors,
            path,
            format!("fewer than the minimum {min} properties"),
        );
    }
    let properties = schema.get("properties").and_then(Value::as_object);
    let patterns: Vec<(Pattern, &Value)> = schema
        .get("patternProperties")
        .and_then(Value::as_object)
        .map(|p| {
            p.iter()
                .filter_map(|(k, v)| Some((Pattern::new(k).ok()?, v)))
                .collect()
        })
        .unwrap_or_default();
    for (key, value) in map {
        let child = format!("{path}/{key}");
        if let Some(names) = schema.get("propertyNames")
            && !valid(root, names, &Value::String(key.clone()))
        {
            fail(errors, &child, format!("the name {key:?} is not allowed"));
        }
        let mut known = false;
        if let Some(sub) = properties.and_then(|p| p.get(key)) {
            known = true;
            walk(root, sub, value, &child, errors);
        }
        for (pattern, sub) in &patterns {
            if pattern.is_match(key) {
                known = true;
                walk(root, sub, value, &child, errors);
            }
        }
        if !known {
            match schema.get("additionalProperties") {
                Some(Value::Bool(false)) => {
                    let hint = if patterns.is_empty() {
                        ""
                    } else {
                        " (extensions start with x-)"
                    };
                    fail(errors, &child, format!("{key:?} is not allowed here{hint}"));
                }
                Some(sub @ Value::Object(_)) => walk(root, sub, value, &child, errors),
                _ => {}
            }
        }
    }
}
