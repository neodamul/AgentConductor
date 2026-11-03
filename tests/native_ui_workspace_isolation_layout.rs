#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{Probe, appearance::State};
use eframe::egui::{self, Pos2, accesskit::Role};

#[test]
fn worktree_toggle_is_below_the_project_folder_row() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::default();
    let locale = State::load();
    let mut output = frame(&context, &mut probe);
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output = frame(&context, &mut probe);

    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    let bounds = |role, label: &str| {
        nodes
            .iter()
            .find(|(_, node)| node.role() == role && node.label() == Some(label))
            .and_then(|(_, node)| node.bounds())
            .unwrap_or_else(|| panic!("missing {label}"))
    };
    let picker = bounds(
        Role::Button,
        locale.text("startup.choose_your_project_folder"),
    );
    let isolation = bounds(
        Role::CheckBox,
        locale.text("startup.isolate_tasks_in_git_worktrees"),
    );
    assert!(
        isolation.y0 >= picker.y1,
        "execution isolation belongs on its own row below the project picker: picker={picker:?}, isolation={isolation:?}"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn frame(context: &egui::Context, probe: &mut Probe) -> egui::FullOutput {
    context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1280.0, 892.0),
            )),
            ..Default::default()
        },
        |ui| probe.show(ui),
    )
}
