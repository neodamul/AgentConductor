use serde_json::Value;

pub(crate) fn plan_configuration_available(snapshot: &Value) -> bool {
    let config = &snapshot["config"];
    config["goal"]
        .as_str()
        .is_some_and(|goal| !goal.trim().is_empty())
        && config["coordinator"].as_str().is_some_and(|coordinator| {
            config["profiles"]
                .as_array()
                .is_some_and(|profiles| profiles.iter().any(|profile| profile["id"] == coordinator))
        })
}
