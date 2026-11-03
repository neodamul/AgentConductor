#![cfg(feature = "native-ui-probe")]

use agentconductor::{manual::Block, native_ui_probe::manual_viewer};
use eframe::egui::{self, accesskit::Role};

#[test]
fn opening_manual_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let blocks = [Block::Paragraph("Manual body".into())];
    let before = manual_frame(&context, false, &blocks);
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    let after = manual_frame(&context, true, &blocks);
    let after_y = label_y(&after, "Workspace content");
    let visible = visible_text(&after);
    clear(after);
    assert_eq!(after_y, before_y, "manual must overlay workspace content");
    assert!(visible.iter().any(|line| line == "Manual body"));
}

#[test]
fn opening_agent_tools_guide_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let before = tools_frame(&context, false);
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    let after = tools_frame(&context, true);
    let after_y = label_y(&after, "Workspace content");
    let visible = visible_text(&after);
    assert_eq!(
        after_y, before_y,
        "tools guide must overlay workspace content"
    );
    assert!(visible.iter().any(|line| line.contains("agentmemory")));
}

#[test]
fn opening_workspace_confirmation_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::workspace::navigation::State::default();
    let before = frame(&context, |ui| {
        navigation(&mut state, false, ui);
    });
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    state.confirm_delete("/tmp/example", "Example");
    let after = frame(&context, |ui| {
        navigation(&mut state, true, ui);
    });
    let after_y = label_y(&after, "Workspace content");
    let visible = visible_text(&after);
    clear(after);
    assert_eq!(
        after_y, before_y,
        "confirmation must overlay workspace content"
    );
    assert!(visible.iter().any(|line| line.contains("Example")));
}

#[test]
fn opening_updates_does_not_push_workspace_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut updates = agentconductor::native_ui_probe::updates::State::default();
    let before = frame(&context, |ui| {
        updates.show("en", false, ui);
    });
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    updates.open = true;
    let after = frame(&context, |ui| {
        updates.show("en", false, ui);
    });
    let after_y = label_y(&after, "Workspace content");
    clear(after);
    assert_eq!(after_y, before_y, "updates must overlay workspace content");
}

#[test]
fn opening_execution_consent_does_not_push_review_content_down() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let before = execution_frame(&context, false);
    let before_y = label_y(&before, "Workspace content");
    clear(before);
    let after = execution_frame(&context, true);
    let after_y = label_y(&after, "Workspace content");
    let visible = visible_text(&after);
    assert_eq!(
        after_y, before_y,
        "execution consent must overlay page content"
    );
    assert!(visible.iter().any(|line| line.contains("Rotate tokens")));
    assert!(
        after
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .any(|(_, node)| {
                node.role() == Role::Button
                    && node.label() == Some("Confirm exact reviewed plan and start agent execution")
                    && node.bounds().is_some()
            })
    );
    clear(after);
}

fn navigation(
    state: &mut agentconductor::native_ui_probe::workspace::navigation::State,
    dirty: bool,
    ui: &mut egui::Ui,
) {
    use agentconductor::native_ui_probe::workspace::navigation::{
        NavigationContext, show_in_language,
    };
    show_in_language(
        state,
        NavigationContext {
            current_name: "Workspace",
            current_path: "/tmp/example",
            can_change: true,
            planning_running: false,
            dirty,
            show_cache: false,
            language: "en",
        },
        ui,
    );
}

fn execution_frame(context: &egui::Context, confirming: bool) -> egui::FullOutput {
    let snapshot = serde_json::json!({
        "revision":"revision-1",
        "config":{"repository":"/tmp/example"},
        "proposal":{"phase":"ready","digest":"digest-1","repository_base":"main",
            "plan":{"goal":"Rotate tokens","workspace_mode":"shared",
                "agents":[{"id":"codex","executable":"codex","args":[],
                    "permission_mode":"workspace-write","fallback":{"agent":"local"}}],
                "tasks":[{"id":"tests","agent":"codex","model":null,
                    "paths":["tests/auth.rs"],"test_command":["cargo","test"]}]}}
    });
    let mut confirmation = confirming.then(|| ("revision-1".into(), "digest-1".into()));
    frame(context, |ui| {
        agentconductor::native_ui_probe::workspace::start_controls_in_language(
            &snapshot,
            true,
            false,
            false,
            &mut confirmation,
            "en",
            ui,
        );
        agentconductor::native_ui_probe::workspace::show_execution_consent_in_language(
            &snapshot,
            true,
            false,
            &mut confirmation,
            "en",
            ui.ctx(),
        );
    })
}

fn manual_frame(context: &egui::Context, mut open: bool, blocks: &[Block]) -> egui::FullOutput {
    frame(context, |ui| {
        manual_viewer::show(&mut open, "en", blocks, ui);
    })
}

fn tools_frame(context: &egui::Context, mut open: bool) -> egui::FullOutput {
    frame(context, |ui| {
        agentconductor::native_ui_probe::workspace::navigation::tools_guide::show(
            &mut open, "en", ui,
        );
    })
}

fn frame(context: &egui::Context, mut show: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
    let mut output = None;
    for _ in 0..2 {
        let next = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000.0, 800.0),
                )),
                ..Default::default()
            },
            |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    show(ui);
                    ui.label("Workspace content")
                        .accessible_name("Workspace content");
                });
            },
        );
        if let Some(mut previous) = output.replace(next) {
            previous.textures_delta.set.clear();
            previous.textures_delta.free.clear();
        }
    }
    output.unwrap()
}

fn label_y(output: &egui::FullOutput, label: &str) -> f32 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(label))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| bounds.y0 as f32)
        .unwrap_or_else(|| panic!("missing {label:?}"))
}

fn visible_text(output: &egui::FullOutput) -> Vec<String> {
    output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.clone()),
            _ => None,
        })
        .collect()
}

fn clear(mut output: egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
