//! GitHub #209: agent characteristics are settings. A profile can override the preset's session flags; what defines
//! full access, the plain argv and the folder rules stay fixed by the preset; invalid overrides are refused.
use agentconductor::{adapters, agent_settings, plan::AgentProfile, protocol::Provider};
use serde_json::json;

fn profile(parser: &str, args: Vec<String>, settings: Option<serde_json::Value>) -> AgentProfile {
    let mut value = json!({
        "id":"a","executable":"/bin/agent","args":args,"parser":parser,
        "catalog":{"source":"fixture","models":["m1"],"queried_at":0,"cli_version":"1"}
    });
    if let Some(settings) = settings {
        value["agent_settings"] = settings;
    }
    serde_json::from_value(value).unwrap()
}

#[test]
fn a_profile_override_changes_the_resume_arguments_and_nothing_else() {
    let plain = adapters::invocation(Provider::Claude, None, "t")
        .unwrap()
        .args;
    let base = profile("claude", plain.clone(), None);
    let custom = profile(
        "claude",
        plain.clone(),
        Some(
            json!({"invocation":{"session":{"resume":{"args":["--resume","{session_id}","--fork-session"]}}}}),
        ),
    );
    let before = adapters::session_args(&base, Some("m1"), Some("s-1"), "t").unwrap();
    let after = adapters::session_args(&custom, Some("m1"), Some("s-1"), "t").unwrap();
    assert_eq!(
        after[..after.len() - 3],
        before[..before.len() - 2],
        "model flags and the rest are unchanged"
    );
    assert_eq!(
        after[after.len() - 3..],
        ["--resume", "s-1", "--fork-session"]
    );
    assert_eq!(custom.validate_custom_configuration(), Ok(()));
}

#[test]
fn the_override_can_rename_the_model_flag() {
    let plain = adapters::invocation(Provider::OpenCode, None, "t")
        .unwrap()
        .args;
    let custom = profile(
        "opencode",
        plain,
        Some(
            json!({"invocation":{"session":{"model_args":["--use-model","{model}"],"model_aliases":[]}}}),
        ),
    );
    let args = adapters::session_args(&custom, Some("m1"), None, "t").unwrap();
    assert_eq!(args[args.len() - 2..], ["--use-model", "m1"]);
}

#[test]
fn what_defines_full_access_cannot_be_overridden_by_a_profile() {
    for patch in [
        json!({"invocation":{"full_access":{"mode":"append","args":["--yolo"]}}}),
        json!({"invocation":{"args":["run"]}}),
        json!({"invocation":{"direct_folder":null}}),
        json!({"invocation":{"executable":"other"}}),
    ] {
        let p = profile("codex", vec!["exec".into()], Some(patch.clone()));
        let error = p.validate_custom_configuration().unwrap_err();
        assert!(error.contains("fixed by the preset"), "{patch}: {error}");
    }
}

#[test]
fn an_invalid_override_is_refused_with_the_reason() {
    for (patch, needle) in [
        (
            json!({"invocation":{"session":{"resume":{"mode":"replace_argv"}}}}),
            "argv",
        ),
        (json!({"invocation":{"surprise":1}}), "not allowed"),
        (
            json!({"capabilities":{"control":{"user_message":"yes"}}}),
            "boolean",
        ),
    ] {
        let error = profile("claude", vec!["-p".into()], Some(patch.clone()))
            .validate_custom_configuration()
            .unwrap_err();
        assert!(error.contains(needle), "{patch}: {error}");
    }
}

#[test]
fn merge_patch_replaces_arrays_removes_nulls_and_merges_objects() {
    let mut base = json!({"a":{"b":1,"c":[1,2]},"d":1});
    agent_settings::merge(&mut base, &json!({"a":{"c":[9]},"d":null,"e":2}));
    assert_eq!(base, json!({"a":{"b":1,"c":[9]},"e":2}));
}

#[test]
fn presets_carry_the_new_characteristics_as_data() {
    let codex = agent_settings::preset(Provider::Codex).unwrap();
    assert_eq!(codex.capabilities["control"]["user_message"], false);
    assert_eq!(
        codex.capabilities["task_defaults"]["code"]["effort"],
        "high"
    );
    let claude = agent_settings::preset(Provider::Claude).unwrap();
    assert_eq!(claude.capabilities["control"]["tool_decision"], true);
    assert!(agent_settings::preset(Provider::Custom).is_none());
}
