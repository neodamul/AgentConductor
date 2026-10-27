use agentconductor::custom_profile_registry::CustomProfileRegistry;
use agentconductor::ui::setup::Startup;
use serde_json::{Value, json};
use std::{fs, path::Path, time::SystemTime};

#[path = "support/env.rs"]
mod test_env;

fn registry(profiles: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({"version":1,"profiles":profiles})).unwrap()
}

fn profile(id: &str) -> Value {
    json!({
        "id":id,"executable":"aipro","args":["run","--jsonl"],
        "parser":"custom","input_mode":"stdin","permission_mode":"unverified",
        "max_sessions":3,"quota_refresh_minutes":5
    })
}

#[test]
fn registry_accepts_only_unique_custom_cli_definitions() {
    let loaded = CustomProfileRegistry::parse(&registry(vec![profile("aipro")])).unwrap();
    assert_eq!(loaded.profiles.len(), 1);
    assert_eq!(loaded.profiles[0].id, "aipro");
    assert!(
        CustomProfileRegistry::parse(&registry(vec![profile("aipro"), profile("aipro")])).is_err()
    );
    let mut with_environment = profile("environment-agent");
    with_environment["environment"] = json!({"OPENAI_API_KEY":"secret"});
    assert!(CustomProfileRegistry::parse(&registry(vec![with_environment])).is_err());
    let mut built_in = profile("claude");
    built_in["parser"] = json!("claude");
    assert!(CustomProfileRegistry::parse(&registry(vec![built_in])).is_err());
}

#[test]
fn registry_rejects_unsupported_versions_unknown_fields_and_shell_wrappers() {
    let mut value: Value = serde_json::from_slice(&registry(vec![profile("aipro")])).unwrap();
    value["version"] = json!(2);
    assert!(CustomProfileRegistry::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    value["version"] = json!(1);
    value["profiles"][0]["unknown"] = json!(true);
    assert!(CustomProfileRegistry::parse(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut shell = profile("shell");
    shell["executable"] = json!("bash");
    assert!(CustomProfileRegistry::parse(&registry(vec![shell])).is_err());
}

#[test]
fn registry_enforces_input_and_profile_count_bounds() {
    assert!(CustomProfileRegistry::parse(&vec![b' '; 65_537]).is_err());
    let profiles = (0..65).map(|i| profile(&format!("agent-{i}"))).collect();
    assert!(CustomProfileRegistry::parse(&registry(profiles)).is_err());
}

#[cfg(unix)]
#[test]
fn registry_load_refuses_symlink_files() {
    let root = std::env::temp_dir().join(format!(
        "ac-profile-registry-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let target = root.join("profiles.json");
    fs::write(&target, registry(vec![profile("aipro")])).unwrap();
    let link = root.join("linked.json");
    std::os::unix::fs::symlink(&target, &link).unwrap();
    assert!(CustomProfileRegistry::load(&target).is_ok());
    assert!(CustomProfileRegistry::load(&link).is_err());
    assert!(CustomProfileRegistry::load(&root).is_err());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn workspace_snapshot_includes_machine_registered_custom_templates() {
    let root = std::env::temp_dir().join(format!(
        "ac-profile-snapshot-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let home = root.join("home");
    fs::create_dir_all(&home).unwrap();
    let mut env = test_env::lock();
    env.set("HOME", &home);
    let registry_path = CustomProfileRegistry::global_path().unwrap();
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    let profile = profile("aipro");
    fs::write(&registry_path, registry(vec![profile])).unwrap();

    let workspace = root.join("workspace");
    let executable = Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let startup = Startup::open(&workspace, executable).unwrap();
    let snapshot = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let template = snapshot["templates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|profile| profile["id"] == "aipro")
        .expect("custom CLI profile should be available in the native template picker");
    assert_eq!(template["executable"], "aipro");
    assert!(
        template.get("environment").is_none(),
        "secrets stay private"
    );
    drop(startup);
    fs::write(&registry_path, b"{").unwrap();
    let invalid_startup =
        Startup::open(root.join("invalid-workspace").as_path(), executable).unwrap();
    let invalid_snapshot = invalid_startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    assert_eq!(invalid_snapshot["custom_registry_invalid"], true);
    assert!(
        invalid_snapshot["templates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|profile| { profile["id"] != "aipro" })
    );
    drop(invalid_startup);
    fs::remove_dir_all(root).unwrap();
}
