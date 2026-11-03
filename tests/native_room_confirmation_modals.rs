#![cfg(feature = "native-ui-probe")]
use agentconductor::native_ui_probe::room::{self, CandidateDecision, State};
use eframe::egui;
use serde_json::json;

#[test]
fn opening_confirmation_preserves_initiating_controls_in_place() {
    for (mode, run_state, button) in [
        ("stop", "running", "Stop run"),
        ("recover", "running", "Recover interrupted run"),
        ("approve", "waiting_approval", "Approve verified candidate"),
        ("reject", "waiting_approval", "Reject candidate"),
        ("merge", "approved", "Merge approved candidate"),
    ] {
        let context = egui::Context::default();
        let snapshot = json!({"revision":"rev", "job":{"state":"idle"},
            "report":{"state":run_state,"candidate":{"commit":"sha"}}});
        let mut state = State::default();
        state.evidence = Some(json!({"revision":"rev","report":{"candidate":{"commit":"sha"}}}));
        let render = |state: &mut State| {
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1280.0, 892.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    room::show(state, Some(&snapshot), false, None, "en", ui);
                },
            );
            let position = output.shapes.iter().find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.job.text == button => Some(text.pos),
                _ => None,
            });
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();
            position
        };
        render(&mut state);
        let before = render(&mut state).expect(button);
        match mode {
            "stop" => state.confirming_stop = true,
            "recover" => state.confirming_recover = true,
            "approve" => state.confirming_decision = Some(CandidateDecision::Approve),
            "reject" => state.confirming_decision = Some(CandidateDecision::Reject),
            _ => state.confirming_decision = Some(CandidateDecision::Merge),
        }
        render(&mut state);
        assert_eq!(
            render(&mut state),
            Some(before),
            "{mode} must overlay existing controls"
        );
    }
}

#[test]
fn modal_buttons_preserve_explicit_confirmation_and_reason_payloads() {
    for mode in ["stop", "recover", "approve", "reject", "merge"] {
        for confirm in [false, true] {
            let context = egui::Context::default();
            context.enable_accesskit();
            let run_state = match mode {
                "approve" | "reject" => "waiting_approval",
                "merge" => "approved",
                _ => "running",
            };
            let snapshot = json!({"revision":"rev", "job":{"state":"idle"},
                "report":{"state":run_state,"candidate":{"commit":"sha"}}});
            let mut state = State::default();
            state.evidence =
                Some(json!({"revision":"rev","report":{"candidate":{"commit":"sha"}}}));
            match mode {
                "stop" => state.confirming_stop = true,
                "recover" => state.confirming_recover = true,
                "approve" => state.confirming_decision = Some(CandidateDecision::Approve),
                "reject" => state.confirming_decision = Some(CandidateDecision::Reject),
                _ => state.confirming_decision = Some(CandidateDecision::Merge),
            }
            let button = match (mode, confirm) {
                ("stop", true) => "Confirm stop",
                ("stop", false) => "Cancel stop",
                ("recover", true) => "Confirm recovery",
                ("recover", false) => "Cancel recovery",
                (_, true) => "Confirm candidate decision",
                (_, false) => "Cancel candidate decision",
            };
            let mut render = |events| {
                let mut event = None;
                let mut output = context.run_ui(
                    egui::RawInput {
                        events,
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1280.0, 892.0),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        event = room::show(&mut state, Some(&snapshot), false, None, "en", ui);
                    },
                );
                let bounds = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| node.label() == Some(button))
                    .and_then(|(_, node)| node.bounds());
                let input = output
                    .platform_output
                    .accesskit_update
                    .as_ref()
                    .unwrap()
                    .nodes
                    .iter()
                    .find(|(_, node)| {
                        node.role() == egui::accesskit::Role::TextInput
                            && matches!(node.label(), Some("Stop reason" | "Rejection reason"))
                    })
                    .and_then(|(_, node)| node.bounds());
                output.textures_delta.set.clear();
                output.textures_delta.free.clear();
                (bounds, event, input)
            };
            render(vec![]);
            let initial = render(vec![]);
            if let Some(input) = initial.2 {
                let position = egui::pos2(
                    ((input.x0 + input.x1) / 2.0) as f32,
                    ((input.y0 + input.y1) / 2.0) as f32,
                );
                for pressed in [true, false] {
                    render(vec![
                        egui::Event::PointerMoved(position),
                        egui::Event::PointerButton {
                            pos: position,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ]);
                }
                render(vec![egui::Event::Text(
                    if mode == "stop" {
                        "test stop"
                    } else {
                        "test rejection"
                    }
                    .into(),
                )]);
            }
            let bounds = render(vec![]).0.expect(button);
            let position = egui::pos2(
                ((bounds.x0 + bounds.x1) / 2.0) as f32,
                ((bounds.y0 + bounds.y1) / 2.0) as f32,
            );
            render(vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::default(),
                },
            ]);
            let event = render(vec![egui::Event::PointerButton {
                pos: position,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }])
            .1;
            if confirm {
                assert!(
                    match (mode, event) {
                        ("stop", Some(room::Event::Stop(reason))) => reason == "test stop",
                        ("recover", Some(room::Event::Recover)) => true,
                        ("approve", Some(room::Event::Decision(CandidateDecision::Approve, _))) =>
                            true,
                        (
                            "reject",
                            Some(room::Event::Decision(CandidateDecision::Reject, reason)),
                        ) => reason == "test rejection",
                        ("merge", Some(room::Event::Decision(CandidateDecision::Merge, _))) => true,
                        _ => false,
                    },
                    "{mode} must preserve its event payload"
                );
            } else {
                assert!(!matches!(
                    event,
                    Some(room::Event::Stop(_) | room::Event::Recover | room::Event::Decision(_, _))
                ));
            }
            assert!(
                !state.confirming_stop
                    && !state.confirming_recover
                    && state.confirming_decision.is_none()
            );
        }
    }
}
