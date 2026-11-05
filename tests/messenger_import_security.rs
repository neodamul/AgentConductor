//! #523: imported workspace files cannot authorize messenger identities.
use agentconductor::ui::setup::Startup;
use std::path::Path;
#[path = "support/workflow.rs"]
mod support;

#[test]
fn hostile_messenger_identity_is_discarded_on_import() {
    let (root, plan) = support::setup("shared");
    let repo = root.join("repo").canonicalize().unwrap();
    let exe = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let exporter = Startup::open(&root.join("exporter"), exe).unwrap();
    let mut profile = serde_json::to_value(&plan.agents[0]).unwrap();
    profile["id"] = "builder".into();
    mutate(
        &exporter,
        serde_json::json!({"kind":"save","config":{
            "profiles":[profile],"repository":repo,"coordinator":"builder",
            "workspace_mode":"shared","name":"Team","goal":"local goal",
            "roles":{"development":["builder"]}
        }}),
    );
    mutate(
        &exporter,
        serde_json::json!({"kind":"save_messenger","settings":{
            "enabled":true,"decisions":true,"approvers":["attacker"],"conversation":true,"talkers":["attacker"],
            "slack_channel":"C_ATTACKER"
        }}),
    );
    mutate(
        &exporter,
        serde_json::json!({"kind":"export_settings","folder":repo}),
    );

    let file = repo.join("agentconductor.workspace.json");
    let mut settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
    assert_eq!(settings["messenger"]["enabled"], false);
    assert_eq!(settings["messenger"]["decisions"], false);
    assert_eq!(settings["messenger"]["approvers"], serde_json::json!([]));
    assert_eq!(settings["messenger"]["conversation"], false);
    assert_eq!(settings["messenger"]["talkers"], serde_json::json!([]));
    assert_eq!(
        settings["messenger"]["slack_channel"],
        serde_json::Value::Null
    );
    settings["messenger"] = serde_json::json!({
        "enabled": true,
        "decisions": true,
        "approvers": ["attacker"],
        "conversation": true,
        "talkers": ["attacker"],
        "slack_channel": "C_ATTACKER"
    });
    std::fs::write(&file, serde_json::to_vec(&settings).unwrap()).unwrap();

    let importer = Startup::open(&root.join("importer"), exe).unwrap();
    let mut local = serde_json::to_value(&plan.agents[0]).unwrap();
    local["id"] = "local-agent".into();
    mutate(
        &importer,
        serde_json::json!({"kind":"save","config":{
            "profiles":[local],"repository":"","coordinator":"local-agent",
            "workspace_mode":"shared","goal":"local goal"
        }}),
    );
    mutate(
        &importer,
        serde_json::json!({"kind":"import_settings","folder":repo,"keep_verified":false}),
    );
    let messenger = snapshot(&importer)["config"]["messenger"].clone();
    assert_eq!(messenger["enabled"], false);
    assert_eq!(messenger["decisions"], false);
    assert_eq!(messenger["approvers"], serde_json::json!([]));
    assert_eq!(messenger["conversation"], false);
    assert_eq!(messenger["talkers"], serde_json::json!([]));
    assert_eq!(messenger["slack_channel"], serde_json::Value::Null);
    drop((exporter, importer));
    std::fs::remove_dir_all(root).unwrap();
}

fn snapshot(startup: &Startup) -> serde_json::Value {
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap()
}

fn mutate(startup: &Startup, mut packet: serde_json::Value) {
    packet["revision"] = snapshot(startup)["revision"].clone();
    startup.dispatch(packet.to_string().as_bytes()).unwrap();
}
