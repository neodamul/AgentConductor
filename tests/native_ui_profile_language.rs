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
        "catalog":{"models":[]}
    });
    let mut profiles = vec![profile.clone()];
    let mut editor = Editor::default();
    editor.open(&profile);
    editor.request_remove("claude");
    let snapshot = serde_json::json!({
        "config":{"profiles":[profile]},
        "templates":[{"id":"claude","parser":"claude"}],
        "metadata_job":{}, "metadata_queue":[], "quota_badges":{}
    });
    let mut coordinator = "claude".to_owned();
    let mut roles = Default::default();
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        show_in_language(
            &mut profiles,
            &mut coordinator,
            &mut roles,
            &mut editor,
            RenderContext {
                snapshot: &snapshot,
                editable: true,
                language: "ko",
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
    let mut inputs = nodes
        .iter()
        .filter(|(_, node)| matches!(node.role(), Role::TextInput | Role::MultilineTextInput))
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    inputs.sort();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();

    for label in [
        "등록된 에이전트",
        "claude 에이전트 편집",
        "claude 에이전트 삭제",
        "claude 에이전트 삭제 확인",
        "claude 에이전트 삭제 취소",
        "모델 0개",
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
    assert!(inputs.contains(&"실행 파일"));
    assert!(inputs.contains(&"인자 · JSON 배열"));
}
