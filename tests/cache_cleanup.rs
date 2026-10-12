use agentconductor::ui::setup::Startup;
use std::{fs, path::Path};
fn fixture() -> (std::path::PathBuf, Startup) {
    let root = std::env::temp_dir().join(format!(
        "ac-cache-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let startup = Startup::open(&root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    (root, startup)
}
fn query(root: &Path, id: &str) -> std::path::PathBuf {
    let dir = root.join(format!("quota-query-{id}"));
    fs::create_dir(&dir).unwrap();
    let pid = std::process::id();
    fs::write(dir.join("owner.json"),serde_json::to_vec(&serde_json::json!({"coordinator_pid":i32::MAX,"pid":pid,"process_group":pid,"phase":"terminal","exit_code":0,"stopped":false,"timed_out":false})).unwrap()).unwrap();
    fs::write(dir.join("stdout.jsonl"), "disposable metadata").unwrap();
    dir
}
#[test]
fn cache_preview_is_read_only_and_clear_is_bound_to_the_preview() {
    let (root, startup) = fixture();
    let cache = query(&root, "old");
    fs::write(root.join("plan.json"), "reviewed evidence").unwrap();
    let preview = startup.dispatch(br#"{"kind":"cache_preview"}"#).unwrap();
    assert_eq!(preview["items"].as_array().unwrap().len(), 1);
    assert!(preview["bytes"].as_u64().unwrap() > 0);
    assert!(cache.exists());
    assert!(
        startup
            .dispatch(br#"{"kind":"cache_clear","digest":"wrong"}"#)
            .is_err()
    );
    fs::write(cache.join("stdout.jsonl"), "changed after preview").unwrap();
    assert!(
        startup
            .dispatch(
                serde_json::json!({"kind":"cache_clear","digest":preview["digest"]})
                    .to_string()
                    .as_bytes()
            )
            .is_err()
    );
    let preview = startup.dispatch(br#"{"kind":"cache_preview"}"#).unwrap();
    let result = startup
        .dispatch(
            serde_json::json!({"kind":"cache_clear","digest":preview["digest"]})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    assert_eq!(result["removed"], 1);
    assert!(!cache.exists());
    assert_eq!(
        fs::read_to_string(root.join("plan.json")).unwrap(),
        "reviewed evidence"
    );
    assert!(root.join("startup.json").exists());
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn live_unknown_and_aliased_cache_candidates_are_retained() {
    let (root, startup) = fixture();
    let live = query(&root, "live");
    let mut owner: serde_json::Value =
        serde_json::from_slice(&fs::read(live.join("owner.json")).unwrap()).unwrap();
    owner["phase"] = "running".into();
    owner["exit_code"] = serde_json::Value::Null;
    fs::write(live.join("owner.json"), serde_json::to_vec(&owner).unwrap()).unwrap();
    let unknown = root.join("quota-query-unknown");
    fs::create_dir(&unknown).unwrap();
    fs::write(unknown.join("data"), "unknown ownership").unwrap();
    #[cfg(unix)]
    {
        let alias = query(&root, "alias");
        std::os::unix::fs::symlink(root.join("startup.json"), alias.join("outside")).unwrap();
    }
    let preview = startup.dispatch(br#"{"kind":"cache_preview"}"#).unwrap();
    assert!(preview["items"].as_array().unwrap().is_empty());
    assert!(live.exists() && unknown.exists());
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn completed_metadata_settlement_bounds_the_old_query_cache() {
    let (root, startup) = fixture();
    drop(startup);
    for i in 0..20 {
        let path = query(&root, &format!("old-{i}"));
        let mut owner: serde_json::Value =
            serde_json::from_slice(&fs::read(path.join("owner.json")).unwrap()).unwrap();
        owner["coordinator_pid"] = i32::MAX.into();
        fs::write(path.join("owner.json"), serde_json::to_vec(&owner).unwrap()).unwrap();
    }
    let logs = root.join("quota-jobs/coordinator-ui-done/process");
    fs::create_dir_all(&logs).unwrap();
    let pid = std::process::id();
    fs::write(logs.join("owner.json"),serde_json::to_vec(&serde_json::json!({"coordinator_pid":pid,"pid":pid,"process_group":pid,"phase":"terminal","exit_code":0,"stopped":false,"timed_out":false})).unwrap()).unwrap();
    let mut saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("startup.json")).unwrap()).unwrap();
    saved["quota_job"] = serde_json::json!({"id":"done","operation":"quota:claude","started_ms":1});
    saved["metadata_settled"] = serde_json::Value::Null;
    fs::write(
        root.join("startup.json"),
        serde_json::to_vec(&saved).unwrap(),
    )
    .unwrap();
    let startup = Startup::open(&root, Path::new(env!("CARGO_BIN_EXE_agentconductor"))).unwrap();
    startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let count = fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().starts_with("quota-query-"))
        .count();
    assert_eq!(count, 8);
    assert!(logs.join("owner.json").exists());
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_project_inside_a_cache_named_directory_is_never_a_cleanup_candidate() {
    let (root, startup) = fixture();
    let project = query(&root, "project");
    let state = startup.dispatch(br#"{"kind":"snapshot"}"#).unwrap();
    let mut config = state["config"].clone();
    config["repository"] = project.to_string_lossy().to_string().into();
    startup
        .dispatch(
            serde_json::json!({"kind":"save","revision":state["revision"],"config":config})
                .to_string()
                .as_bytes(),
        )
        .unwrap();
    let preview = startup.dispatch(br#"{"kind":"cache_preview"}"#).unwrap();
    assert!(preview["items"].as_array().unwrap().is_empty());
    assert!(project.exists());
    drop(startup);
    fs::remove_dir_all(root).unwrap();
}
