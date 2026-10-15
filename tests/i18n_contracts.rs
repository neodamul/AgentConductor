use serde::{
    Deserialize, Deserializer,
    de::{Error, MapAccess, Visitor},
};
use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

const EN: &str = include_str!("../src/ui/messages/en.json");
const KO: &str = include_str!("../src/ui/messages/ko.json");

struct UniqueCatalog(BTreeMap<String, String>);

impl<'de> Deserialize<'de> for UniqueCatalog {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct CatalogVisitor;
        impl<'de> Visitor<'de> for CatalogVisitor {
            type Value = UniqueCatalog;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a message catalog with unique string IDs")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut entries = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, String>()? {
                    if entries.insert(key.clone(), value).is_some() {
                        return Err(A::Error::custom(format!("duplicate ID: {key}")));
                    }
                }
                Ok(UniqueCatalog(entries))
            }
        }
        deserializer.deserialize_map(CatalogVisitor)
    }
}

fn catalog(source: &str) -> BTreeMap<String, String> {
    serde_json::from_str::<UniqueCatalog>(source).unwrap().0
}

fn has_human_text(value: &str) -> bool {
    value
        .chars()
        .any(|c| c.is_ascii_alphabetic() || ('\u{ac00}'..='\u{d7a3}').contains(&c))
}

fn placeholders(value: &str) -> BTreeMap<String, ()> {
    let mut found = BTreeMap::new();
    let mut rest = value;
    while let Some(start) = rest.find('{') {
        let token = &rest[start + 1..];
        let Some(end) = token.find('}') else { break };
        let name = &token[..end];
        let mut chars = name.chars();
        if chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            found.insert(name.to_owned(), ());
        }
        rest = &token[end + 1..];
    }
    found
}

fn valid_id(id: &str) -> bool {
    let Some((namespace, key)) = id.split_once('.') else {
        return false;
    };
    let mut namespace_chars = namespace.chars();
    namespace_chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase())
        && namespace_chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.'))
}

fn quoted_value(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if quote != '\'' && quote != '"' {
        return None;
    }
    let end = value[1..].find(quote)? + 1;
    Some(&value[1..end])
}

fn check_source_refs(
    source: &str,
    path: &Path,
    ids: &BTreeMap<String, String>,
    errors: &mut Vec<String>,
) {
    for call in ["acMessage", "messages::text", "messages::render"] {
        let mut rest = source;
        while let Some(offset) = rest.find(call) {
            let tail = &rest[offset + call.len()..];
            let args = tail
                .trim_start()
                .strip_prefix('(')
                .unwrap_or("")
                .trim_start();
            if let Some(key) = quoted_value(args)
                && !ids.contains_key(key)
            {
                errors.push(format!("{}: {key}", path.display()));
            }
            rest = tail;
        }
    }
    for attr in [
        "data-i18n-title",
        "data-i18n-placeholder",
        "data-i18n-aria-label",
        "data-i18n",
    ] {
        let mut rest = source;
        while let Some(offset) = rest.find(attr) {
            let tail = &rest[offset + attr.len()..];
            let name_end = tail
                .chars()
                .next()
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'));
            if name_end {
                let value = tail
                    .trim_start()
                    .strip_prefix('=')
                    .unwrap_or("")
                    .trim_start();
                if let Some(key) = quoted_value(value)
                    && !ids.contains_key(key)
                {
                    errors.push(format!("{}: {key}", path.display()));
                }
            }
            rest = tail;
        }
    }
}

fn source_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            source_files(&path, files);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("rs" | "js" | "html")
        ) {
            files.push(path);
        }
    }
}

fn static_markup_errors(source: &str) -> Vec<String> {
    let invariant = [
        "AgentConductor",
        "AC",
        "English",
        "한국어",
        "Claude Code",
        "Codex",
        "OpenCode",
        "Jira",
        "Confluence",
    ];
    let void = ["input", "img", "br", "hr", "meta", "link"];
    let ignored = ["code", "textarea", "script", "style", "title"];
    let mut stack: Vec<String> = Vec::new();
    let mut errors = Vec::new();
    let mut rest = source;
    while !rest.is_empty() {
        let Some(open) = rest.find('<') else { break };
        let text = rest[..open].trim();
        if !text.is_empty()
            && !stack.iter().any(|tag| ignored.contains(&tag.as_str()))
            && !invariant.contains(&text)
            && !text.starts_with("v{{APP_VERSION}}")
            && has_human_text(text)
        {
            errors.push(text.to_owned());
        }
        rest = &rest[open..];
        if rest.starts_with("<!--") {
            let Some(end) = rest.find("-->") else { break };
            rest = &rest[end + 3..];
            continue;
        }
        let Some(end) = rest.find('>') else { break };
        let raw = rest[1..end].trim();
        rest = &rest[end + 1..];
        let closing = raw.starts_with('/');
        let raw = raw.trim_start_matches('/').trim();
        let name_end = raw.find(char::is_whitespace).unwrap_or(raw.len());
        let name = raw[..name_end].trim_end_matches('/');
        if name.is_empty() {
            continue;
        }
        if closing {
            stack.pop();
            continue;
        }
        let attrs = raw[name_end..].trim_end_matches('/');
        for attr in ["title", "aria-label", "placeholder"] {
            if let Some(value) = attribute(attrs, attr)
                && has_human_text(value)
                && attribute(attrs, &format!("data-i18n-{attr}")).is_none()
            {
                errors.push(value.to_owned());
            }
        }
        if name == "option"
            && attrs
                .split_ascii_whitespace()
                .any(|part| part.starts_with("data-i18n-"))
        {
            errors.push("option label must bind its text".into());
        }
        if !void.contains(&name) && !raw.ends_with('/') {
            stack.push(name.to_owned());
        }
    }
    errors
}

fn attribute<'a>(source: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = source;
    while let Some(offset) = rest.find(name) {
        let tail = &rest[offset + name.len()..];
        if tail
            .chars()
            .next()
            .is_none_or(|c| c.is_ascii_whitespace() || c == '=')
        {
            let value = tail.trim_start().strip_prefix('=')?.trim_start();
            return quoted_value(value);
        }
        rest = tail;
    }
    None
}

#[test]
fn localization_contracts_are_owned_by_rust() {
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/test_i18n_catalogs.py")
            .exists()
    );
}

#[test]
fn catalog_ids_are_unique_valid_and_have_matching_translations_and_placeholders() {
    let en = catalog(EN);
    let ko = catalog(KO);
    assert!(serde_json::from_str::<UniqueCatalog>(r#"{"same":"a","same":"b"}"#).is_err());
    assert_eq!(en.keys().collect::<Vec<_>>(), ko.keys().collect::<Vec<_>>());
    for (id, english) in &en {
        assert!(valid_id(id), "invalid message ID: {id}");
        assert!(!ko[id].is_empty(), "empty Korean translation: {id}");
        assert_eq!(placeholders(english), placeholders(&ko[id]), "{id}");
    }
}

#[test]
fn every_explicit_source_reference_has_a_catalog_entry() {
    let ids = catalog(EN);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![root.join("src/ui.rs")];
    source_files(&root.join("src/ui"), &mut files);
    let mut errors = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path).unwrap();
        check_source_refs(&source, &path, &ids, &mut errors);
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
}

#[test]
fn visible_static_ui_text_has_localized_bindings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui");
    for page in ["startup.html", "room.html", "mcp.html"] {
        let source = fs::read_to_string(root.join(page)).unwrap();
        let errors = static_markup_errors(&source);
        assert!(errors.is_empty(), "{page}: {}", errors.join(" | "));
    }
}

#[test]
fn legacy_content_matching_helpers_are_gone() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui/i18n.js")).unwrap();
    assert!(!source.contains("acSwap"));
    assert!(!source.contains("acInvert"));
    assert!(
        !Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ui/i18n_ko.js")
            .exists()
    );
}
