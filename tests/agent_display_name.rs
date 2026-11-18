//! GitHub #1071: a profile's display name round-trips and is omitted when unset.
use agentconductor::plan::AgentProfile;

#[test]
fn a_display_name_round_trips_and_is_omitted_when_unset() {
    let named: AgentProfile = serde_json::from_value(serde_json::json!({"id":"my-cli","executable":"my-cli","args":[],"parser":"custom","name":"My CLI"})).unwrap();
    assert_eq!(named.name.as_deref(), Some("My CLI"));
    assert_eq!(serde_json::to_value(&named).unwrap()["name"], "My CLI");
    let plain: AgentProfile = serde_json::from_value(
        serde_json::json!({"id":"my-cli","executable":"my-cli","args":[],"parser":"custom"}),
    )
    .unwrap();
    assert!(serde_json::to_value(&plain).unwrap().get("name").is_none());
}
