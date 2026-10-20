//! GitHub #84: the documented `Bearer {env:TOKEN}` header must validate.
use agentconductor::mcp::Settings;
use serde_json::json;

fn remote(header: &str) -> String {
    json!({"mcp":{"x":{"type":"remote","url":"https://example.org/mcp","headers":{"Authorization":header}}}})
        .to_string()
}

#[test]
fn bearer_and_basic_references_validate() {
    for header in ["{env:TOKEN}", "Bearer {env:TOKEN}", "Basic {env:TOKEN}"] {
        assert!(Settings::parse(&remote(header)).is_ok(), "{header}");
    }
}

#[test]
fn literal_secrets_and_odd_prefixes_stay_rejected() {
    for header in [
        "Bearer secret",
        "Token {env:TOKEN}",
        "Bearer {env:}",
        "Bearer",
    ] {
        assert!(Settings::parse(&remote(header)).is_err(), "{header}");
    }
}

#[test]
fn local_environment_values_stay_bare_references() {
    let local = |value: &str| {
        json!({"mcp":{"x":{"type":"local","command":["mcp"],"environment":{"T":value}}}})
            .to_string()
    };
    assert!(Settings::parse(&local("{env:TOKEN}")).is_ok());
    assert!(Settings::parse(&local("Bearer {env:TOKEN}")).is_err());
}
