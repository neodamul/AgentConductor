use agentconductor::custom_profile_registry::CustomProfileRegistry;
use serde_json::{Value, json};
use std::{fs, time::SystemTime};

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
