use agentconductor::native_ui_probe::room;
use eframe::egui;
use serde_json::json;

#[test]
fn native_room_marks_the_current_workflow_stage_accessibly() {
    for (state, task_status, en, ko) in [
        (
            "prepared",
            "pending",
            "Current stage: 01 Plan",
            "현재 단계: 01 계획",
        ),
        (
            "running",
            "running",
            "Current stage: 02 Tasks",
            "현재 단계: 02 작업",
        ),
        (
            "waiting_approval",
            "completed",
            "Current stage: 03 Verify",
            "현재 단계: 03 검증",
        ),
        (
            "approved",
            "completed",
            "Current stage: 04 Merge",
            "현재 단계: 04 병합",
        ),
        (
            "completed",
            "completed",
            "Current stage: 05 Workspace",
            "현재 단계: 05 워크스페이스",
        ),
        (
            "failed",
            "failed",
            "Current stage: 02 Tasks",
            "현재 단계: 02 작업",
        ),
        (
            "failed",
            "completed",
            "Current stage: 03 Verify",
            "현재 단계: 03 검증",
        ),
    ] {
        let snapshot = json!({
            "report": {
                "state": state,
                "plan": {"goal":"Inspect progress", "tasks":[{}]},
                "tasks": [{"status": task_status}]
            }
        });
        for (language, current, steps) in [
            (
                "en",
                en,
                [
                    "01 Plan",
                    "02 Tasks",
                    "03 Verify",
                    "04 Merge",
                    "05 Workspace",
                ],
            ),
            (
                "ko",
                ko,
                [
                    "01 계획",
                    "02 작업",
                    "03 검증",
                    "04 병합",
                    "05 워크스페이스",
                ],
            ),
        ] {
            let context = egui::Context::default();
            context.enable_accesskit();
            let mut room_state = room::State::default();
            let mut output = context.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(792.0, 532.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    room::show(&mut room_state, Some(&snapshot), false, None, language, ui);
                },
            );
            let controls = output
                .platform_output
                .accesskit_update
                .as_ref()
                .unwrap()
                .nodes
                .iter()
                .filter_map(|(_, node)| Some((node.label()?.to_owned(), node.bounds()?)))
                .collect::<Vec<_>>();
            output.textures_delta.set.clear();
            output.textures_delta.free.clear();

            for label in steps {
                let expected = if current.ends_with(label) {
                    current
                } else {
                    label
                };
                let bounds = controls
                    .iter()
                    .find_map(|(text, bounds)| (text == expected).then_some(bounds))
                    .unwrap_or_else(|| panic!("{state} {language}: {controls:?}"));
                assert!(bounds.x0 >= 0.0 && bounds.x1 <= 792.0, "{bounds:?}");
                assert!(bounds.y0 >= 0.0 && bounds.y1 <= 532.0, "{bounds:?}");
            }
        }
    }
}
