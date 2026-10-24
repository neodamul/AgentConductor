//! Embedded locale catalogs for Rust UI messages and user-facing diagnostics.
use std::{collections::BTreeMap, sync::OnceLock};

mod catalog_sources;

fn merge(sources: &[&str]) -> BTreeMap<String, String> {
    let mut merged = BTreeMap::new();
    for source in sources {
        let shard: BTreeMap<String, String> =
            serde_json::from_str(source).expect("valid embedded message shard");
        for (id, value) in shard {
            assert!(
                merged.insert(id.clone(), value).is_none(),
                "duplicate embedded message ID: {id}"
            );
        }
    }
    merged
}

fn catalog(language: &str) -> &'static BTreeMap<String, String> {
    static EN: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    static KO: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    let (cell, sources) = if language == "ko" {
        (&KO, catalog_sources::KO)
    } else {
        (&EN, catalog_sources::EN)
    };
    cell.get_or_init(|| merge(sources))
}

/// Unsupported languages and missing translations fall back to English.
pub fn in_language(language: &str, id: &'static str) -> &'static str {
    catalog(language)
        .get(id)
        .or_else(|| catalog("en").get(id))
        .map(String::as_str)
        .unwrap_or(id)
}

pub fn text(id: &'static str) -> &'static str {
    in_language(&super::language::load(), id)
}

/// Substitute only tokens in the template, never tokens inside supplied values.
pub fn render(id: &'static str, params: &[(&str, &str)]) -> String {
    interpolate(text(id), params)
}

pub fn interpolate(template: &str, params: &[(&str, &str)]) -> String {
    let mut output = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        output.push_str(&rest[..start]);
        let token = &rest[start..];
        let Some(end) = token.find('}') else {
            output.push_str(token);
            return output;
        };
        let name = &token[1..end];
        if let Some((_, value)) = params.iter().find(|(key, _)| *key == name) {
            output.push_str(value);
        } else {
            output.push_str(&token[..=end]);
        }
        rest = &token[end + 1..];
    }
    output.push_str(rest);
    output
}
