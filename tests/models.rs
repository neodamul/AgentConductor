#![cfg(unix)]
#[path = "support/workflow.rs"]
mod support;
use agentconductor::{models, protocol::Provider};

#[tokio::test]
async fn native_catalogs_are_queried_without_prompting_a_model_or_api_key() {
    for provider in [Provider::Claude, Provider::Codex, Provider::OpenCode] {
        let (root, plan) = support::setup("shared");
        let mut profile = plan.agents[0].clone();
        profile.parser = provider;
        let catalog = models::discover(&profile, &root.join("catalog"))
            .await
            .unwrap();
        assert_eq!(catalog.models, ["local/one", "local/two"]);
        assert_eq!(catalog.cli_version, "mock 0.1");
        assert!(catalog.queried_at > 0);
        assert!(!catalog.source.contains("api_key"));
        std::fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn malformed_or_empty_catalog_is_rejected_and_model_names_are_not_invented() {
    assert!(models::parse_catalog(Provider::Claude, &serde_json::json!({"response":{}})).is_err());
    assert!(
        models::parse_catalog(
            Provider::Codex,
            &serde_json::json!({"result":{"data":[{"model":"--unsafe"}]}})
        )
        .is_err()
    );
}
