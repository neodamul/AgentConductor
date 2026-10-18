#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui::{self, accesskit::Role};
use serde_json::{Value, json};

#[test]
fn edit_and_remove_icons_are_vector_shapes_instead_of_font_glyphs() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        sidebar::show(
            &[json!({"id":"company","parser":"custom"})],
            &json!({}),
            "en",
            true,
            false,
            ui,
        );
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        !output.shapes.iter().any(|shape| matches!(&shape.shape,
        egui::Shape::Text(text) if matches!(text.galley.job.text.as_str(), "✎" | "🗑"))),
        "action icons must not depend on font coverage"
    );
    for label in ["Edit agent company", "Remove agent company"] {
        let bounds = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap();
        assert!(
            output.shapes.iter().any(|shape| match &shape.shape {
                egui::Shape::LineSegment { points, .. } => points.iter().all(|p| f64::from(p.x)
                    >= bounds.x0
                    && f64::from(p.x) <= bounds.x1
                    && f64::from(p.y) >= bounds.y0
                    && f64::from(p.y) <= bounds.y1),
                _ => false,
            }),
            "{label} needs visible vector strokes"
        );
    }
}

#[test]
fn sidebar_keeps_agent_health_quota_and_edit_actions_visible() {
    let profiles = vec![
        json!({"id":"claude","parser":"claude","executable":"claude","permission_mode":"full_access","max_sessions":3,"fallback":{"agent":"aipro","model":"company-default"},"catalog":{"models":["sonnet","opus"]}}),
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
        assert_eq!(
            sidebar::show(&profiles, &snapshot, "en", true, false, ui),
            None
        );
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        assert_eq!(
            sidebar::show(&profiles, &snapshot, "en", true, false, ui),
            None
        );
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
    for label in [
        "Refresh models for claude",
        "Remove agent claude",
        "Edit agent claude",
    ] {
        assert!(
            buttons.contains(&label),
            "missing button {label:?}: {buttons:?}"
        );
    }
    assert!(
        text.iter()
            .any(|label| label.contains("claude · full_access"))
    );
    assert!(text.iter().any(|label| label.contains("fallback: aipro")));
    assert!(text.iter().any(|label| label.contains("Running")));
    assert!(text.iter().any(|label| label.contains("Ready")));
}

#[test]
fn sidebar_reports_an_agent_with_a_running_task_as_active() {
    let profiles = vec![
        json!({"id":"claude","parser":"claude","max_sessions":1}),
        json!({"id":"codex","parser":"codex","max_sessions":1}),
    ];
    let snapshot = json!({
        "config":{"profiles":profiles,"coordinator":"codex"},
        "job":{"kind":"idle","state":"completed"},
        "agent_activity":{"claude":"running"},
        "quotas":{"claude":{"status":"available"},"codex":{"status":"available"}}
    });
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        assert_eq!(
            sidebar::show(&profiles, &snapshot, "en", true, false, ui),
            None
        );
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
    assert!(labels.contains(&"claude: Running"), "{labels:?}");
    assert!(labels.contains(&"codex: Ready"), "{labels:?}");
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
        assert_eq!(
            sidebar::show(&profiles, &snapshot, "ko", true, false, ui),
            None
        );
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        assert_eq!(
            sidebar::show(&profiles, &snapshot, "ko", true, false, ui),
            None
        );
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
    assert!(labels.contains(&"codex 에이전트 아이콘"));
    assert!(!labels.iter().any(|label| label.contains("agent icon")));
    let failed = agentconductor::ui::messages::in_language("ko", "room.failed_031a8f");
    assert!(labels.iter().any(|label| label.contains(failed)));
}
