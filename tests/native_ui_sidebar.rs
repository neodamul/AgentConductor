#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, accesskit::Role};
use serde_json::{Value, json};

#[test]
fn sidebar_keeps_agent_health_quota_and_edit_actions_visible() {
    let profiles = vec![
        json!({"id":"claude","parser":"claude","max_sessions":3,"catalog":{"models":["sonnet","opus"]}}),
        json!({"id":"codex","parser":"codex","max_sessions":2,"catalog":{"models":["gpt-6"]}}),
        json!({"id":"aipro","parser":"custom","max_sessions":1}),
    ];
    let snapshot = json!({
        "config":{"coordinator":"codex","profiles":profiles},
        "job":{"kind":"plan","state":"running"},
        "quotas":{"claude":{"status":"stale"},"codex":{"status":"available"}},
        "quota_badges":{"claude":{"text":"64%/100%","title":"5h: 64% remaining; weekly: 100% remaining"}},
        "metadata_job":{"kind":"quota:claude","state":"running"}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = context.run_ui(Default::default(), |ui| {
        assert_eq!(sidebar::show(&profiles, &snapshot, "en", true, ui), None);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        assert_eq!(sidebar::show(&profiles, &snapshot, "en", true, ui), None);
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let text = nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let buttons = nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    for label in [
        "CLI agents",
        "claude",
        "codex",
        "aipro",
        "64%/100%: 5h: 64% remaining; weekly: 100% remaining",
        "claude · 3 sessions",
        "codex · 2 sessions",
        "2 models",
        "1 models",
    ] {
        assert!(text.contains(&label), "missing {label:?}: {text:?}");
    }
    assert!(buttons.contains(&"Register CLI agent"));
    assert!(buttons.contains(&"Refresh codex quota"));
    assert!(text.iter().any(|label| label.contains("Running")));
    assert!(text.iter().any(|label| label.contains("Ready")));
}

#[test]
fn sidebar_localizes_agent_roster_and_preserves_quota_details() {
    let profiles = vec![json!({"id":"codex","parser":"codex","max_sessions":2})];
    let snapshot: Value = json!({
        "config":{"profiles":profiles,"coordinator":"codex"},
        "job":{"kind":"plan","state":"failed"},
        "quotas":{"codex":{"status":"available"}},
        "quota_badges":{"codex":{"text":"72%/36%","title":"Enterprise: 72%/36%"}}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut first = context.run_ui(Default::default(), |ui| {
        assert_eq!(sidebar::show(&profiles, &snapshot, "ko", true, ui), None);
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        assert_eq!(sidebar::show(&profiles, &snapshot, "ko", true, ui), None);
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
    assert!(labels.contains(&agentconductor::ui::messages::in_language(
        "ko",
        "startup.cli_agents"
    )));
    assert!(labels.iter().any(|label| label.contains("72%/36%")));
    assert!(labels.iter().any(|label| label.contains("codex")));
    let failed = agentconductor::ui::messages::in_language("ko", "room.failed_031a8f");
    assert!(labels.iter().any(|label| label.contains(failed)));
}
