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
    unsafe { std::env::set_var("HOME", root.join("home")) };
    let room = Arc::new(
        agentconductor::ui::Room::startup(&root, &std::env::current_exe().unwrap()).unwrap(),
    );
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut probe = Probe::connect(room.clone(), context.clone()).unwrap();
    let output = until(
        &context,
        &mut probe,
        message(&agentconductor::ui::language::load(), "startup.settings"),
    );
    let language = ["en", "ko"]
        .into_iter()
        .find(|language| labels(&output).contains(&message(language, "startup.settings")))
        .unwrap();
    drop(output);

    choose(&context, &mut probe, language, "mcp.settings_mcp_servers");
    let output = until(
        &context,
        &mut probe,
        message(language, "mcp.mcp_server_registration"),
    );
    assert_present(&output, message(language, "mcp.mcp_server_registration"));
    assert_rendered_text(&output, message(language, "mcp.settings_mcp_servers"));
    assert_present(&output, message(language, "mcp.save_mcp_settings"));
    close_dialog(&context, &mut probe, language, &output);
    until_absent(
        &context,
        &mut probe,
        message(language, "mcp.save_mcp_settings"),
    );

    choose(
        &context,
        &mut probe,
        language,
        "startup.cache_cleanup_accessible",
    );
    let opening = until(
        &context,
        &mut probe,
        message(language, "startup.cache_cleanup"),
    );
    assert_present(&opening, message(language, "startup.cache_cleanup"));
    drop(opening);
    // The asynchronous preview changes dialog layout; sample Close after it settles.
    until_absent(
        &context,
        &mut probe,
        message(language, "startup.cache_previewing"),
    );
    let output = frame(&context, &mut probe, Vec::new());
    close_dialog(&context, &mut probe, language, &output);
    until_absent(
        &context,
        &mut probe,
        message(language, "startup.cache_cleanup"),
    );

    choose(
        &context,
        &mut probe,
        language,
        "startup.workspace_settings_sharing",
    );
    let output = until(
        &context,
        &mut probe,
        message(language, "startup.export_workspace_settings_accessible"),
    );
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
    until_absent(
        &context,
        &mut probe,
        message(language, "startup.export_workspace_settings_accessible"),
    );

    choose(
        &context,
        &mut probe,
        language,
        "startup.messenger_notifications",
    );
    let output = until(
        &context,
        &mut probe,
        message(language, "startup.messenger_notifications"),
    );
    assert_present(&output, message(language, "startup.close"));
    close_dialog(&context, &mut probe, language, &output);
    until_absent(
        &context,
        &mut probe,
        message(language, "startup.messenger_notifications"),
    );

    choose(&context, &mut probe, language, "startup.update_repository");
    let output = until(
        &context,
        &mut probe,
        message(language, "startup.update_repository_input"),
    );
    assert_present(&output, message(language, "startup.save"));
    assert_present(&output, message(language, "startup.cancel"));
    drop(output);
    click_label(
        &context,
        &mut probe,
        message(language, "startup.update_repository_input"),
    );
    frame(&context, &mut probe, vec![Event::Text("acme/tool".into())]);
    click_label(&context, &mut probe, message(language, "startup.save"));
    until_absent(
        &context,
        &mut probe,
        message(language, "startup.update_repository_input"),
    );
    let settings = room.dispatch(br#"{"kind":"update_settings"}"#).unwrap();
    assert_eq!(settings["repository"], "acme/tool");
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

fn until(context: &egui::Context, probe: &mut Probe, target: &str) -> egui::FullOutput {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let output = frame(context, probe, Vec::new());
        if labels(&output).contains(&target) {
            return output;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "missing {target}; current labels: {:?}",
            labels(&output)
        );
        std::thread::yield_now();
    }
}

fn until_absent(context: &egui::Context, probe: &mut Probe, target: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        let output = frame(context, probe, Vec::new());
        if !labels(&output).contains(&target) {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "still visible: {target}; current labels: {:?}",
            labels(&output)
        );
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
    let labels = labels(output);
    assert!(labels.contains(&target), "missing {target}: {labels:?}");
}

fn assert_rendered_text(output: &egui::FullOutput, target: &str) {
    let visible = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::epaint::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ");
    assert!(visible.contains(target), "missing {target}: {visible}");
}
