#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::profile_tools::{
    Editor, RenderContext, show_in_language,
};
use eframe::egui::{self, accesskit::Role};

#[test]
fn agent_profile_list_and_editor_localize_actions_and_accessible_inputs() {
    let profile = serde_json::json!({
        "id":"claude", "parser":"claude", "executable":"claude", "args":["-p"],
        "input_mode":"stdin", "max_sessions":1, "quota_refresh_minutes":5,
        "fallback":{"agent":"codex","model":"gpt-5-codex"},
        "catalog":{"models":[]}
    });
    let codex = serde_json::json!({
        "id":"codex", "parser":"codex", "executable":"codex", "args":["exec"],
        "input_mode":"stdin", "max_sessions":1, "quota_refresh_minutes":5,
        "catalog":{"models":["gpt-5-codex"]}
    });
    let mut profiles = vec![profile.clone(), codex.clone()];
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.set_save_error("ko", "fallback account uses the same quota group");
    editor.request_remove("claude");
    let snapshot = serde_json::json!({
        "config":{"profiles":[profile, codex]},
        "templates":[{"id":"claude","parser":"claude"},{"id":"codex","parser":"codex"}],
        "metadata_job":{}, "metadata_queue":[], "quota_badges":{}
    });
    let mut coordinator = "claude".to_owned();
    let mut roles = Default::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 450.0),
        )),
        ..Default::default()
    };
    let mut first = context.run_ui(input.clone(), |ui| {
        show_in_language(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &mut editor,
            RenderContext {
                snapshot: &snapshot,
                editable: true,
                language: "ko",
                show_registry: true,
            },
            ui,
        );
    });
    first.textures_delta.set.clear();
    first.textures_delta.free.clear();
    let mut small_output = context.run_ui(input.clone(), |ui| {
        show_in_language(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &mut editor,
            RenderContext {
                snapshot: &snapshot,
                editable: true,
                language: "ko",
                show_registry: true,
            },
            ui,
        );
    });
    let first_nodes = &small_output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let first_actions = first_nodes
        .iter()
        .filter_map(|(_, node)| {
            let label = node.label()?;
            if matches!(label, "프로필 저장" | "취소") {
                Some((label.to_owned(), node.bounds()?))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(first_actions.len(), 2, "{first_actions:?}");
    for (label, bounds) in first_actions {
        assert!(
            bounds.x0 >= 0.0 && bounds.x1 <= 900.0 && bounds.y0 >= 0.0 && bounds.y1 <= 450.0,
            "{label} is below or outside the short profile dialog viewport: {bounds:?}"
        );
    }
    small_output.textures_delta.set.clear();
    small_output.textures_delta.free.clear();
    let full_height_input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(900.0, 740.0),
        )),
        ..Default::default()
    };
    let mut output = context.run_ui(full_height_input, |ui| {
        show_in_language(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &mut editor,
            RenderContext {
                snapshot: &snapshot,
                editable: true,
                language: "ko",
                show_registry: true,
            },
            ui,
        );
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let labels = nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let action_bounds = nodes
        .iter()
        .filter_map(|(_, node)| {
            let label = node.label()?;
            if matches!(label, "프로필 저장" | "취소") {
                Some((label.to_owned(), node.bounds()?))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut inputs = nodes
        .iter()
        .filter(|(_, node)| matches!(node.role(), Role::TextInput | Role::MultilineTextInput))
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    inputs.sort();
    let visible_shapes = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some((
                text.galley.job.text.as_str().to_owned(),
                text.galley.size().x,
            )),
            _ => None,
        })
        .collect::<Vec<_>>();
    let visible_text = visible_shapes
        .iter()
        .map(|(text, _)| text.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    for label in [
        "등록된 에이전트",
        "claude 에이전트 편집",
        "에이전트 ID",
        "대체 에이전트",
        "대체 모델",
        "에이전트 프로필을 저장하지 못했습니다: fallback account uses the same quota group",
        "claude 에이전트 삭제",
        "claude 에이전트 삭제 확인",
        "claude 에이전트 삭제 취소",
        "모델 미조회",
        "claude 모델 새로고침",
        "claude 사용량 새로고침",
        "claude 에이전트 편집",
        "세션 수",
        "사용량 갱신 · 분",
        "프로필 저장",
        "취소",
    ] {
        assert!(labels.contains(&label), "{label}: {labels:?}");
    }
    assert_eq!(action_bounds.len(), 2, "{action_bounds:?}");
    for (label, bounds) in action_bounds {
        assert!(
            bounds.x0 >= 0.0 && bounds.x1 <= 900.0 && bounds.y0 >= 0.0 && bounds.y1 <= 740.0,
            "{label} is below or outside the profile dialog viewport: {bounds:?}"
        );
    }
    assert!(inputs.contains(&"실행 파일"));
    assert!(inputs.contains(&"에이전트 ID"));
    assert!(inputs.contains(&"인자 · JSON 배열"));
    for label in ["에이전트 ID", "실행 파일", "인자 · JSON 배열"] {
        assert!(visible_text.contains(label), "{label}: {visible_text}");
    }
    for (label, minimum_width) in [
        ("제공자", 27.0),
        ("입력 방식", 44.0),
        ("대체 에이전트", 63.0),
        ("대체 모델", 45.0),
    ] {
        let width = visible_shapes
            .iter()
            .filter(|(text, _)| text == label)
            .map(|(_, width)| width)
            .copied()
            .fold(0.0_f32, f32::max);
        assert!(
            width >= minimum_width,
            "{label} wrapped into {width}px: {visible_shapes:?}"
        );
    }
    assert!(visible_text.contains("권한은 실행파일과 인자에 따라 정해집니다."));
}
