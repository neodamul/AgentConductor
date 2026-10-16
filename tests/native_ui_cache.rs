use agentconductor::native_ui_probe::cache::{clear_request, preview_request};
use eframe::egui::{self, accesskit::Role};

#[test]
fn cache_preview_request_uses_the_existing_startup_boundary() {
    let request: serde_json::Value = serde_json::from_slice(preview_request()).unwrap();
    assert_eq!(request, serde_json::json!({"kind":"cache_preview"}));
}

#[test]
fn cache_clear_requires_items_and_uses_the_exact_preview_digest() {
    let preview = serde_json::json!({
        "digest":"reviewed-digest",
        "items":[{"path":"/tmp/quota-query-old","bytes":128}],
        "bytes":128,
        "skipped":2
    });
    let request: serde_json::Value =
        serde_json::from_slice(&clear_request(&preview).unwrap()).unwrap();
    assert_eq!(
        request,
        serde_json::json!({
            "kind":"cache_clear",
            "digest":"reviewed-digest"
        })
    );
}

#[test]
fn cache_clear_refuses_missing_or_empty_previews() {
    assert!(clear_request(&serde_json::json!({"items":[{}]})).is_err());
    assert!(clear_request(&serde_json::json!({"digest":"d","items":[]})).is_err());
}

#[test]
fn cache_preview_lists_exact_paths_and_requires_review_before_clear() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::cache::State {
        open: true,
        preview: Some(serde_json::json!({
            "digest":"reviewed-digest",
            "items":[{"path":"/tmp/quota-query-old","bytes":128}],
            "bytes":128,
            "skipped":2
        })),
        result: None,
    };
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::cache::show(&mut state, ui, false);
    });
    let text = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(text.contains("Clear reviewed cache"), "{text}");
    assert!(text.contains("/tmp/quota-query-old"));
}

#[test]
fn native_worker_previews_and_clears_only_the_reviewed_cache() {
    use std::{fs, sync::Arc, time::Duration};

    let root = std::env::temp_dir().join(format!(
        "ac-native-cache-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    let cache = root.join("quota-query-stale");
    fs::create_dir(&cache).unwrap();
    let pid = std::process::id();
    fs::write(
        cache.join("owner.json"),
        serde_json::to_vec(&serde_json::json!({
            "coordinator_pid":i32::MAX,
            "pid":pid,
            "process_group":pid,
            "phase":"terminal",
            "exit_code":0,
            "stopped":false,
            "timed_out":false
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(cache.join("stdout.jsonl"), "disposable metadata").unwrap();

    let room = Arc::new(
        agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
    );
    let worker =
        agentconductor::native_ui_probe::RequestWorker::new(room.clone(), egui::Context::default())
            .unwrap();
    worker.send(preview_request()).unwrap();
    let preview = worker
        .replies
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap();
    assert_eq!(preview["items"].as_array().unwrap().len(), 1);
    assert_eq!(
        preview["items"][0]["path"],
        cache.canonicalize().unwrap().to_string_lossy().to_string()
    );

    worker.send(&clear_request(&preview).unwrap()).unwrap();
    let result = worker
        .replies
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap();
    assert_eq!(result["removed"], 1);
    assert!(!cache.exists());
    assert!(root.join("startup.json").exists());
    drop(worker);
    drop(room);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_workspace_exposes_accessible_cache_maintenance() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::default().show(ui);
    });
    let buttons = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(buttons.contains(&"Preview and clear local cache"));
}
