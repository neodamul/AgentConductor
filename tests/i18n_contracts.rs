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
        } else if matches!(path.extension().and_then(|s| s.to_str()), Some("rs")) {
            files.push(path);
        }
    }
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
