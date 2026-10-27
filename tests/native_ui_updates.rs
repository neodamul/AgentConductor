use agentconductor::native_ui_probe::updates::{Action, State, request};
use eframe::egui::{self, accesskit::Role};
use std::time::{Duration, Instant};

#[path = "support/native_ui_updates.rs"]
mod ui_interaction;
use ui_interaction::{click_update, render_update, update_button_center};

#[test]
fn update_actions_use_the_existing_rust_dispatcher() {
    for (action, kind) in [
        (Action::Status, "update_status"),
        (Action::Check, "check_update"),
        (Action::Apply, "apply_update"),
    ] {
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&request(action)).unwrap(),
            serde_json::json!({"kind":kind})
        );
    }
}

#[test]
fn update_panel_has_localized_actions_and_keeps_full_release_notes() {
    let status = serde_json::json!({
        "phase":"idle", "current":"0.1.24", "latest":"0.1.25",
        "newer":true, "installable":true,
        "page":"https://github.com/neodamul/AgentConductor/releases/tag/v0.1.25",
        "notes":"## Fixes\n\n- Fixed Windows paths.\n- Improved native accessibility."
    });
    for (language, check, apply, close, page) in [
        (
            "en",
            "Check for updates…",
            "Update and restart",
            "Close",
            "Release notes",
        ),
        (
            "ko",
            "업데이트 확인…",
            "업데이트 후 재시작",
            "닫기",
            "릴리스 노트",
        ),
    ] {
        let mut state = State::default();
        state.open = true;
        state.apply_status(status.clone());
        assert_eq!(state.status()["notes"], status["notes"]);
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut output = render_update(&context, &mut state, language, false, Vec::new());
        let tree = output.platform_output.accesskit_update.as_ref().unwrap();
        let labels = tree
            .nodes
            .iter()
            .filter(|(_, node)| node.role() == Role::Button)
            .filter_map(|(_, node)| node.label())
            .collect::<Vec<_>>();
        let release_link = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Link)
            .and_then(|(_, node)| node.label());
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        for label in [check, apply, close] {
            assert!(labels.contains(&label), "{label}: {labels:?}");
        }
        assert!(
            release_link.is_some_and(|label| label.starts_with(page)),
            "release notes should be a labeled link: {release_link:?}"
        );
    }
}

#[test]
fn release_notes_are_a_focusable_read_only_text_control_in_a_bounded_viewport() {
    let notes = (0..80)
        .map(|line| format!("Release detail line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut state = State::default();
    state.open = true;
    state.apply_status(serde_json::json!({"phase":"idle","notes":notes}));
    let context = egui::Context::default();
    let mut warmup = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show("en", false, ui);
        },
    );
    warmup.textures_delta.set.clear();
    warmup.textures_delta.free.clear();
    context.enable_accesskit();
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        },
        |ui| {
            state.show("en", false, ui);
        },
    );
    let note_node = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .find(|(_, node)| node.role() == Role::MultilineTextInput)
        .map(|(_, node)| (node.is_read_only(), node.value().map(str::to_owned)));
    let clipped_to_viewport = output.shapes.iter().any(|shape| {
        let egui::Shape::Text(text) = &shape.shape else {
            return false;
        };
        text.galley.job.text == notes
            && text.galley.rect.height() > 260.0
            && shape.clip_rect.height() <= 260.0
    });
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let (read_only, value) = note_node.expect("release notes should be keyboard-focusable");
    assert!(read_only);
    assert_eq!(value.as_deref(), Some(notes.as_str()));
    assert!(
        clipped_to_viewport,
        "release notes should stay in the scroll area"
    );
}

#[test]
fn background_update_check_runs_immediately_and_every_ten_minutes() {
    let mut state = State::default();
    let start = Instant::now() + Duration::from_secs(1);
    assert_eq!(state.background_action(start, false), Some(Action::Check));
    assert!(!state.open);
    assert_eq!(
        state.background_action(start + Duration::from_secs(599), false),
        None
    );
    assert_eq!(
        state.background_action(start + Duration::from_secs(600), false),
        Some(Action::Check)
    );
}

#[test]
fn due_background_check_waits_for_the_existing_ui_request() {
    let mut state = State::default();
    let due = Instant::now() + Duration::from_secs(1);
    assert_eq!(state.background_action(due, true), None);
    assert_eq!(state.background_action(due, false), Some(Action::Check));
}

#[test]
fn an_active_update_is_polled_even_when_its_panel_is_closed() {
    let mut state = State::default();
    state.apply_status(serde_json::json!({"phase":"checking"}));
    let due = Instant::now() + Duration::from_secs(2);
    assert_eq!(state.background_action(due, false), Some(Action::Status));
    assert_eq!(state.background_action(due, false), None);
    assert!(!state.open);
}

#[test]
fn available_update_has_a_localized_titlebar_accessible_name() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = agentconductor::native_ui_probe::appearance::State::default();
    let mut initial = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::appearance::show(&mut state, ui);
    });
    let initially_available = initial
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter_map(|(_, node)| node.label())
        .any(|label| label.contains("0.1.25"));
    initial.textures_delta.set.clear();
    initial.textures_delta.free.clear();
    assert!(!initially_available);
    state.set_update_available(Some("0.1.25"));
    let mut output = context.run_ui(Default::default(), |ui| {
        agentconductor::native_ui_probe::appearance::show(&mut state, ui);
    });
    let buttons = output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == Role::Button)
        .filter_map(|(_, node)| node.label())
        .collect::<Vec<_>>();
    let glyphs = output
        .shapes
        .iter()
        .filter_map(|shape| match &shape.shape {
            egui::Shape::Text(text) => Some(text.galley.job.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    let expected = if state.language() == "ko" {
        "업데이트 있음: v0.1.25"
    } else {
        "Update available: v0.1.25"
    };
    assert!(buttons.contains(&expected), "{buttons:?}");
    assert!(glyphs.contains(&"⬇"), "{glyphs:?}");
    assert!(!glyphs.contains(&"↑"), "{glyphs:?}");
}

#[test]
fn localized_update_buttons_dispatch_check_apply_and_close_actions() {
    for (language, check, apply, close) in [
        ("en", "Check for updates…", "Update and restart", "Close"),
        ("ko", "업데이트 확인…", "업데이트 후 재시작", "닫기"),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut state = State::default();
        state.open = true;
        state.apply_status(serde_json::json!({
            "phase":"idle", "newer":true, "installable":true,
            "page":"https://github.com/neodamul/AgentConductor/releases/tag/v0.1.25",
            "notes":"Release details"
        }));
        let mut output = render_update(&context, &mut state, language, false, Vec::new());
        let check_point = update_button_center(&mut output, check);
        assert_eq!(
            click_update(&context, &mut state, language, false, check_point),
            Some(Action::Check)
        );

        let mut output = render_update(&context, &mut state, language, false, Vec::new());
        let apply_point = update_button_center(&mut output, apply);
        assert_eq!(
            click_update(&context, &mut state, language, false, apply_point),
            Some(Action::Apply)
        );

        let mut output = render_update(&context, &mut state, language, false, Vec::new());
        let close_point = update_button_center(&mut output, close);
        assert_eq!(
            click_update(&context, &mut state, language, false, close_point),
            None
        );
        assert!(!state.open, "Close should dismiss the update panel");
    }
}

#[test]
fn pending_update_keeps_close_available_and_blocks_duplicate_actions() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let mut state = State::default();
    state.open = true;
    state.apply_status(serde_json::json!({"phase":"idle","installable":true}));
    let mut output = render_update(&context, &mut state, "en", true, Vec::new());
    let check = update_button_center(&mut output, "Check for updates…");
    let apply = update_button_center(&mut output, "Update and restart");
    assert_eq!(click_update(&context, &mut state, "en", true, check), None);
    assert_eq!(click_update(&context, &mut state, "en", true, apply), None);
    let mut output = render_update(&context, &mut state, "en", true, Vec::new());
    let close = update_button_center(&mut output, "Close");
    assert_eq!(click_update(&context, &mut state, "en", true, close), None);
    assert!(
        !state.open,
        "Close should remain available while a request is pending"
    );
}
