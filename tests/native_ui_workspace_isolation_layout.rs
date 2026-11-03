#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{Probe, appearance::State};
use eframe::egui::{self, Pos2, accesskit::Role};

#[test]
fn worktree_toggle_shares_the_project_picker_row() {
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
    let picker = bounds(Role::Button, locale.text("startup.browse"));
    let isolation = bounds(
        Role::CheckBox,
        locale.text("startup.isolate_tasks_in_git_worktrees"),
    );
    assert!(
        ((isolation.y0 + isolation.y1 - picker.y0 - picker.y1) / 2.0).abs() <= 2.0,
        "the latest main Workspace reference keeps execution isolation beside the project picker: picker={picker:?}, isolation={isolation:?}"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn project_picker_uses_the_legacy_browse_button_label() {
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
    let browse = locale.text("startup.browse");
    assert!(
        nodes
            .iter()
            .any(|(_, node)| node.role() == Role::Button && node.label() == Some(browse)),
        "the project picker must use the legacy Browse label `{browse}`"
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

#[test]
fn workspace_fields_match_legacy_control_geometry() {
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
    for (role, label) in [
        (Role::TextInput, locale.text("startup.workspace_name")),
        (Role::TextInput, locale.text("startup.project_folder")),
        (Role::Button, locale.text("startup.browse")),
    ] {
        let actual = bounds(role, label).y1 - bounds(role, label).y0;
        assert!(
            (actual - 40.0).abs() <= 2.0,
            "legacy Workspace control `{label}` is 40 px tall, got {actual}"
        );
    }
    let project_folder = bounds(Role::TextInput, locale.text("startup.project_folder"));
    let project_width = project_folder.x1 - project_folder.x0;
    assert!(
        (project_width - 380.0).abs() <= 2.0,
        "legacy project-folder input is 380 px wide, got {project_width}"
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
