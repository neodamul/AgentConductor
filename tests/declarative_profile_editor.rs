#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::Editor;
use serde_json::{Value, json};

fn profile() -> Value {
    json!({
        "id":"aipro", "parser":"custom", "executable":"aipro", "args":["run"],
        "catalog":{"models":["small","large"],"source":"operator","queried_at":0,
            "cli_version":"manual","origin":"manual"}
    })
}

#[test]
fn native_profile_editor_round_trips_declarative_cli_rules() {
    let mut profile = profile();
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.set_extensions(&serde_json::to_string(&json!({
        "catalog":profile["catalog"],
        "model_discovery":{"args":["models","--json"],"models_pointer":"/items"},
        "execution_options":[
            {"name":"model","args":["--model","{value}"],"default":"small"},
            {"name":"reasoning","args":["--reasoning","{value}"],"values":["low"],"model_values":{"small":["low"]}}
        ],
        "result_pointer":"/data",
        "error_condition":{"pointer":"/status","equals":"error"}
    })).unwrap());

    editor.apply(&mut profile).unwrap();
    assert_eq!(profile["model_discovery"]["models_pointer"], "/items");
    assert_eq!(
        profile["execution_options"][1]["model_values"]["small"][0],
        "low"
    );
    assert_eq!(profile["result_pointer"], "/data");
    assert_eq!(profile["error_condition"]["equals"], "error");
}

#[test]
fn invalid_profile_rules_do_not_partially_update_the_profile() {
    let mut profile = profile();
    let original = profile.clone();
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.set_extensions("{ invalid");

    assert!(editor.apply(&mut profile).is_err());
    assert_eq!(profile, original);
}
