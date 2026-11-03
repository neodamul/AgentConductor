#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::{
    show_execution_consent_in_language, start_controls_in_language,
};
use eframe::egui;
use serde_json::json;

fn render(language: &str, confirming: bool) -> (Vec<String>, String) {
    let context = egui::Context::default();
    context.enable_accesskit();
    let snapshot = json!({
        "revision": "workspace-4",
        "config": {"repository": "/tmp/project"},
        "proposal": {
            "phase": "ready",
            "digest": "plan-9",
            "repository_base": "main",
            "plan": {
                "goal": "Rotate tokens",
                "workspace_mode": "shared",
                "tasks": [{
                    "id": "rotate",
                    "agent": "codex",
                    "model": "gpt-6",
                    "paths": ["src/auth.rs"],
                    "test_command": ["cargo", "test"]
                }],
                "agents": [{
                    "id": "codex",
                    "executable": "codex",
                    "args": ["exec", "--json"],
                    "permission_mode": "workspace-write",
                    "fallback": {"agent": "local-agent"}
                }]
            }
        }
    });
    let mut confirmation = confirming.then(|| ("workspace-4".into(), "plan-9".into()));
    let mut output = None;
    for _ in 0..2 {
        let next = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 700.0),
                )),
                ..Default::default()
            },
            |ui| {
                start_controls_in_language(
                    &snapshot,
                    true,
                    false,
                    false,
                    &mut confirmation,
                    language,
                    ui,
                );
                show_execution_consent_in_language(
                    &snapshot,
                    true,
                    false,
                    &mut confirmation,
                    language,
                    ui.ctx(),
                );
            },
        );
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    let mut output = output.unwrap();
    let labels = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label().map(str::to_owned))
        .collect();
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    (labels, visible)
}

#[test]
fn execution_consent_is_bilingual_and_keeps_reviewed_values() {
    let (korean_review, korean_review_visible) = render("ko", false);
    assert!(korean_review_visible.contains("실행 승인 검토"));
    assert!(
        korean_review
            .iter()
            .any(|value| value == "정확한 계획을 검토한 후 실행 승인")
    );
    let (korean, visible) = render("ko", true);
    for label in [
        "실행 동의 · 검토된 계획",
        "확인하면 등록된 CLI 에이전트를 시작합니다. 역할, 명령, 쓰기 가능 경로를 검토하세요.",
        "목표 · Rotate tokens",
        "저장소 · /tmp/project",
        "저장소 기준 · main",
        "워크스페이스 모드 · shared",
        "계획 다이제스트 · plan-9",
        "실행 파일 · codex",
        "권한 · workspace-write",
        "대체 에이전트 · local-agent",
        "쓰기 가능 경로 · src/auth.rs",
        "인자 · [\"exec\",\"--json\"]",
        "테스트 명령 · [\"cargo\",\"test\"]",
    ] {
        assert!(visible.contains(label), "{label}; {visible}");
    }
    for label in ["취소", "검토된 계획을 정확히 확인하고 에이전트 실행 시작"]
    {
        assert!(
            korean.iter().any(|value| value == label),
            "{label}; {korean:?}"
        );
    }

    let (english_review, english_review_visible) = render("en", false);
    assert!(english_review_visible.contains("Review execution and approve"));
    assert!(
        english_review
            .iter()
            .any(|value| value == "Review exact plan before approving execution")
    );
    let (english, visible_english) = render("en", true);
    for label in [
        "Execution consent · exact reviewed plan",
        "Approving starts the registered CLI agents. Review their assignments, commands, and writable paths.",
    ] {
        assert!(
            visible_english.contains(label),
            "{label}; {visible_english}"
        );
    }
    for label in [
        "Cancel",
        "Confirm exact reviewed plan and start agent execution",
    ] {
        assert!(
            english.iter().any(|value| value == label),
            "{label}; {english:?}"
        );
    }
}
