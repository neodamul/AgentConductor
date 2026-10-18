use crate::plan::ModelCatalog;
use serde_json::Value;

const MAX_CATALOG_INPUT_BYTES: usize = 256 * 1024;

pub fn parse_manual_catalog(input: &str, now: i64, language: &str) -> Result<ModelCatalog, String> {
    if input.len() > MAX_CATALOG_INPUT_BYTES {
        return Err(invalid(language));
    }
    let input = input.trim();
    if input.starts_with('{') {
        let catalog: ModelCatalog = serde_json::from_str(input).map_err(|_| invalid(language))?;
        return validate(catalog, language);
    }
    let mut models = Vec::new();
    for model in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if model.chars().count() > 256
            || model.starts_with('-')
            || model
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control())
        {
            return Err(invalid(language));
        }
        if !models.iter().any(|existing| existing == model) {
            models.push(model.to_owned());
        }
        if models.len() > 1024 {
            return Err(invalid(language));
        }
    }
    let catalog = ModelCatalog {
        models,
        source: crate::ui::messages::in_language(
            language,
            "startup.operator_configured_model_ids_cli_discovery_unavailable",
        )
        .into(),
        queried_at: now,
        cli_version: crate::ui::messages::in_language(language, "startup.not_exposed_by_cli")
            .into(),
    };
    validate(catalog, language)
}

fn validate(catalog: ModelCatalog, language: &str) -> Result<ModelCatalog, String> {
    if catalog.models.iter().any(|model| {
        model.chars().count() > 256
            || model
                .chars()
                .any(|ch| ch.is_whitespace() || ch.is_control())
    }) {
        return Err(invalid(language));
    }
    crate::ui::setup::validate_catalog(&catalog).map_err(|_| invalid(language))?;
    Ok(catalog)
}

fn invalid(language: &str) -> String {
    crate::ui::messages::in_language(language, "startup.enter_valid_model_ids_one_per_line").into()
}

pub fn catalog_request(revision: &str, agent: &str, catalog: &ModelCatalog) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "kind":"catalog", "revision":revision, "agent":agent, "catalog":catalog
    }))
    .expect("model catalog request is serializable")
}

pub(super) fn models(value: &Value) -> String {
    value["catalog"]["models"]
        .as_array()
        .map(|models| {
            models
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default()
}
