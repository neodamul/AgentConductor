#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::Probe;
use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};
use std::{fs, sync::Arc, time::Duration};

#[test]
fn settings_dialogs_remain_accessible_across_sequential_open_close() {
    let root = std::env::temp_dir().join(format!(
        "ac-settings-journey-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let room = Arc::new(
        agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
    );
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::connect(room.clone(), context.clone()).unwrap();
    let output = until(&context, &mut probe, "startup.settings");
    let language = ["en", "ko"]
        .into_iter()
        .find(|language| labels(&output).contains(&message(language, "startup.settings")))
        .unwrap();
    drop(output);

    choose(&context, &mut probe, language, "mcp.settings_mcp_servers");
    let output = frame(&context, &mut probe, Vec::new());
    assert_present(&output, message(language, "mcp.register_mcp_servers"));
    assert_present(&output, message(language, "mcp.save_mcp_settings"));
    close_dialog(&context, &mut probe, language, &output);
    let output = frame(&context, &mut probe, Vec::new());
    assert_absent(&output, message(language, "mcp.save_mcp_settings"));

    choose(
        &context,
        &mut probe,
        language,
        "startup.cache_cleanup_accessible",
    );
    let output = frame(&context, &mut probe, Vec::new());
    assert_present(&output, message(language, "startup.cache_cleanup"));
    close_dialog(&context, &mut probe, language, &output);
    let output = frame(&context, &mut probe, Vec::new());
    assert_absent(&output, message(language, "startup.cache_cleanup"));

    choose(
        &context,
        &mut probe,
        language,
        "startup.workspace_settings_sharing",
    );
    let output = frame(&context, &mut probe, Vec::new());
    assert_present(
        &output,
        message(language, "startup.export_workspace_settings_accessible"),
    );
    assert_present(
        &output,
        message(language, "startup.import_workspace_settings_accessible"),
    );
    assert_present(&output, message(language, "startup.close"));
    close_dialog(&context, &mut probe, language, &output);
    let output = frame(&context, &mut probe, Vec::new());
    assert_absent(
        &output,
        message(language, "startup.export_workspace_settings_accessible"),
    );

    choose(
        &context,
        &mut probe,
        language,
        "startup.messenger_notifications",
    );
    let output = until(&context, &mut probe, "startup.messenger_notifications");
    assert_present(&output, message(language, "startup.close"));
    close_dialog(&context, &mut probe, language, &output);
    let output = frame(&context, &mut probe, Vec::new());
    assert_absent(
        &output,
        message(language, "startup.messenger_notifications"),
    );

    drop(output);
    drop(probe);
    drop(room);
    std::thread::sleep(Duration::from_millis(20));
    fs::remove_dir_all(root).unwrap();
}

fn choose(context: &egui::Context, probe: &mut Probe, language: &str, item: &'static str) {
    click_label(context, probe, message(language, "startup.settings"));
    click_label(context, probe, message(language, item));
}

fn close_dialog(
    context: &egui::Context,
    probe: &mut Probe,
    language: &str,
    output: &egui::FullOutput,
) {
    let target = message(language, "startup.close");
    click(context, probe, center(output, target));
}

fn click_label(context: &egui::Context, probe: &mut Probe, target: &str) {
    let output = frame(context, probe, Vec::new());
    let point = center(&output, target);
    assert_ne!(
        point,
        Pos2::ZERO,
        "missing accessible control {target}; {:?}",
        labels(&output)
    );
    click(context, probe, point);
}

fn click(context: &egui::Context, probe: &mut Probe, point: Pos2) {
    for pressed in [true, false] {
        let event = Event::PointerButton {
            pos: point,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        let mut output = frame(context, probe, vec![Event::PointerMoved(point), event]);
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
    }
}

fn until(context: &egui::Context, probe: &mut Probe, key: &'static str) -> egui::FullOutput {
    let target = message(&agentconductor::ui::language::load(), key);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let output = frame(context, probe, Vec::new());
        if labels(&output).contains(&target) {
            return output;
        }
        assert!(std::time::Instant::now() < deadline, "missing {target}");
        std::thread::yield_now();
    }
}

fn frame(context: &egui::Context, probe: &mut Probe, events: Vec<Event>) -> egui::FullOutput {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                Pos2::ZERO,
                egui::vec2(1140.0, 740.0),
            )),
            events,
            ..Default::default()
        },
        |ui| probe.update_ui(ui),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    output
}

fn message(language: &str, key: &'static str) -> &'static str {
    agentconductor::ui::messages::in_language(language, key)
}

fn labels(output: &egui::FullOutput) -> Vec<&str> {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .collect()
}

fn center(output: &egui::FullOutput, target: &str) -> Pos2 {
    output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.label() == Some(target))
        .and_then(|(_, node)| node.bounds())
        .map(|bounds| {
            Pos2::new(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            )
        })
        .unwrap_or(Pos2::ZERO)
}

fn assert_present(output: &egui::FullOutput, target: &str) {
    assert!(labels(output).contains(&target), "missing {target}");
}

fn assert_absent(output: &egui::FullOutput, target: &str) {
    assert!(!labels(output).contains(&target), "still visible: {target}");
}
