use super::*;
use eframe::egui;
use serde_json::{Value, json};

fn run() -> Value {
    json!({
        "report": {
            "state": "running",
            "plan": {"tasks": [
                {"id":"design","agent":"claude"},
                {"id":"tests","agent":"codex"},
                {"id":"docs","agent":"local"},
                {"id":"release","agent":"codex"}
            ]},
            "tasks": [
                {"id":"release","status":"pending"},
                {"id":"docs","status":"failed"},
                {"id":"tests","status":"running"},
                {"id":"design","status":"completed"}
            ]
        },
        "activity": {"tests":{"quiet_ms":12000,"observed_at_ms":1000000}}
    })
}

#[test]
fn model_counts_mixed_states_and_pairs_each_task_with_its_agent_and_activity() {
    let snapshot = run();
    let board = model(&snapshot);
    assert_eq!(board.run_state, "running");
    assert_eq!(
        board.counts,
        Counts {
            total: 4,
            completed: 1,
            running: 1,
            failed: 1,
            pending: 1
        }
    );
    assert_eq!(board.rows[1].agent, "codex");
    assert_eq!(board.rows[1].status, "running");
    assert_eq!(output_age_ms(&board.rows[1], 1_012_000), Some(12_000));
    assert_eq!(output_age_ms(&board.rows[1], 1_020_000), Some(20_000));
    assert_eq!(board.rows[3].quiet_ms, None);
}

#[test]
fn coordinator_activity_renders_status_and_retains_judgments_without_dispatch() {
    let mut snapshot = run();
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    snapshot["activity"]["tests"]["observed_at_ms"] = json!(now_ms - 12_000);
    let data = json!({"history":[
        {"event":"workflow","summary":"repeated workflow save"},
        {"event":"decision","summary":"kept useful history"}
    ],"judgments":[{"kind":"review","judgment":"accept","detail":"Reviewed"}]});
    for language in ["en", "ko"] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut tab = crate::native_ui_probe::room::task_activity::Tab::Activity;
        let mut output = context.run_ui(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                assert_eq!(
                    crate::native_ui_probe::room::task_activity::coordinator(
                        &mut tab,
                        Some(&data),
                        &snapshot,
                        language,
                        ui,
                    ),
                    None
                );
            });
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
        for expected in [
            "design",
            "claude",
            "tests",
            "codex",
            "docs",
            "local",
            "release",
            "Reviewed",
            "kept useful history",
        ] {
            assert!(
                text.contains(expected),
                "missing {expected:?} in {language}: {text}"
            );
        }
        assert!(!text.contains("repeated workflow save"));
        assert_eq!(data["history"].as_array().unwrap().len(), 2);
        assert!(text.contains("Last output") || text.contains("마지막 출력"));
        assert!(
            labels.contains(&"tests"),
            "missing task accessible name in {language}: {labels:?}"
        );
        assert!(
            labels.iter().any(|label| label.ends_with("codex")),
            "missing agent accessible name in {language}: {labels:?}"
        );
    }
}
