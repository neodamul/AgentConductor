//! Rust UI text is addressed by IDs; screen tests resolve expected text through these shards.
#![allow(dead_code)]

#[path = "../../src/ui/messages/catalog_sources.rs"]
mod catalog_sources;

fn merge(sources: &[&str]) -> std::collections::BTreeMap<String, String> {
    let mut messages = std::collections::BTreeMap::new();
    for source in sources {
        let shard: serde_json::Value = serde_json::from_str(source).unwrap();
        for (id, value) in shard.as_object().unwrap() {
            assert!(
                messages
                    .insert(id.clone(), value.as_str().unwrap().to_owned())
                    .is_none()
            );
        }
    }
    messages
}

pub fn english() -> std::collections::BTreeMap<String, String> {
    merge(catalog_sources::EN)
}

pub fn korean() -> std::collections::BTreeMap<String, String> {
    merge(catalog_sources::KO)
}

/// The catalog ID whose English text is exactly `text`.
pub fn id(text: &str) -> String {
    english()
        .iter()
        .find(|(_, value)| *value == text)
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| panic!("no catalog message for {text:?}"))
}

/// True when `source` shows `text` literally or through its catalog ID.
pub fn shows(source: &str, text: &str) -> bool {
    source.contains(text) || source.contains(&format!("\"{}\"", id(text)))
}
