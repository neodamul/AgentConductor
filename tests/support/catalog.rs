//! Interface text is addressed by message IDs (src/ui/messages/*.json); tests that check
//! what a screen offers look the English text up and expect its ID in the markup or script.
#![allow(dead_code)]

const EN: &str = include_str!("../../src/ui/messages/en.json");

/// The catalog ID whose English text is exactly `text`.
pub fn id(text: &str) -> String {
    let catalog: serde_json::Value = serde_json::from_str(EN).unwrap();
    catalog
        .as_object()
        .unwrap()
        .iter()
        .find(|(_, v)| v.as_str() == Some(text))
        .map(|(k, _)| k.clone())
        .unwrap_or_else(|| panic!("no catalog message for {text:?}"))
}

/// True when `source` shows `text` literally or through its catalog ID.
pub fn shows(source: &str, text: &str) -> bool {
    source.contains(text) || source.contains(&format!("\"{}\"", id(text)))
}
