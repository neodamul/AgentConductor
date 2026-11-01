#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn only_the_running_agents_quota_card_shows_elapsed_time() {
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let profiles = [
        json!({"id":"claude","parser":"claude"}),
        json!({"id":"codex","parser":"codex"}),
    ];
    let snapshot = json!({
        "config":{"profiles":[{"id":"claude"},{"id":"codex"}]},
        "quota_badges":{
            "claude":{"text":"73%/100%","title":"Weekly quota"},
            "codex":{"text":"70%/90%","title":"Enterprise quota"}},
        "metadata_job":{"kind":"quota:claude","state":"running","started_ms":now_ms - 12_000},
        "metadata_queue":[]
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        sidebar::show(&profiles, &snapshot, "en", true, false, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Label)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_eq!(
        labels
            .iter()
            .filter(|label| label.contains("Quota query running for 00:1"))
            .count(),
        1,
        "elapsed time must appear once, on Claude's card: {labels:?}"
    );
}

#[test]
fn failed_quota_card_exposes_the_timeout_stage_and_limit() {
    let error = "Claude quota prompt timed out after 60 s before /usage; no generation requested";
    let profiles = [json!({"id":"claude","parser":"claude"})];
    let snapshot = json!({
        "config":{"profiles":[{"id":"claude"}]},
        "quota_badges":{"claude":{"text":"—","title":"Quota unavailable"}},
        "metadata_job":{"kind":"quota:claude","state":"failed","error":error},
        "metadata_queue":[]
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        sidebar::show(&profiles, &snapshot, "en", true, false, ui);
    });
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        labels.contains(&error),
        "timeout reason is not exposed: {labels:?}"
    );
}
