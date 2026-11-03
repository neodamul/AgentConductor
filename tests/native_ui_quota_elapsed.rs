#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, accesskit::Role};
use serde_json::json;

#[test]
fn running_agent_quota_card_does_not_show_elapsed_query_text() {
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
    assert!(
        labels
            .iter()
            .all(|label| !label.contains("Quota query running")),
        "the refresh spinner already shows progress; do not add card text: {labels:?}"
    );
}

#[test]
fn failed_quota_card_exposes_the_timeout_stage_and_limit() {
    let error = "Claude quota prompt timed out before /usage after 60 s; no generation requested";
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
    assert!(labels.iter().any(|label| label.contains(error)));
}
