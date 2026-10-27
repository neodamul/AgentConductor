#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::appearance::State;
use std::time::SystemTime;

#[path = "support/env.rs"]
mod test_env;

#[test]
fn native_language_selection_updates_labels_and_survives_restart() {
    let root = std::env::temp_dir().join(format!(
        "ac-native-language-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let mut env = test_env::lock();
    env.set("HOME", &root);

    let mut state = State::default();
    assert_eq!(state.language(), "en");
    state.select_language("ko").unwrap();
    assert_eq!(state.text("startup.settings"), "설정");
    assert_eq!(state.text("startup.agent_roles"), "에이전트 역할");
    assert_eq!(State::default().language(), "ko");

    let context = eframe::egui::Context::default();
    context.enable_accesskit();
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::Probe::default().show(ui)
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
    for label in [
        "설정",
        "1. 워크스페이스 설정",
        "2. 에이전트 역할",
        "3. 계획 검토",
        "워크스페이스 이름",
        "프로젝트 폴더",
        "목표",
        "실행 격리",
        "공유 폴더",
        "Git worktree",
        "워크스페이스 저장",
        "프로젝트 폴더 선택",
    ] {
        assert!(
            labels.contains(&label),
            "{label}; actual labels: {labels:?}"
        );
    }

    let preferences =
        std::fs::read_to_string(root.join(".agentconductor/preferences.json")).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&preferences).unwrap()["language"],
        "ko"
    );
    std::fs::remove_dir_all(root).unwrap();
}
