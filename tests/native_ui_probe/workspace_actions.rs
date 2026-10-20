use super::super::Probe;
use eframe::egui;

#[test]
fn workspace_and_role_save_actions_are_aligned_to_the_card_end() {
    let locale = super::super::appearance::State::default();
    for (step, key) in [
        (0, "startup.save_workspace"),
        (1, "startup.save_agent_roles"),
    ] {
        let context = egui::Context::default();
        context.enable_accesskit();
        let mut probe = Probe {
            workspace_step: step,
            ..Default::default()
        };
        let mut output = context.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1140.0, 740.0),
                )),
                ..Default::default()
            },
            |ui| super::show(&mut probe, ui),
        );
        let label = locale.text(key);
        let rect = output
            .platform_output
            .accesskit_update
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .find(|(_, node)| {
                node.role() == egui::accesskit::Role::Button && node.label() == Some(label)
            })
            .and_then(|(_, node)| node.bounds());
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
        let rect = rect.unwrap_or_else(|| panic!("{label} button is rendered; labels={labels:?}"));
        assert!(rect.x0 > 800.0, "{label} should be right aligned: {rect:?}");
    }
}

#[test]
fn primary_action_text_contrasts_with_its_fill_in_both_themes() {
    for dark in [false, true] {
        let context = egui::Context::default();
        context.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        let mut output = context.run_ui(Default::default(), |ui| {
            crate::native_ui_probe::appearance::primary_button(ui, true, "Continue");
        });
        let fill = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Rect(rect) if rect.fill != egui::Color32::TRANSPARENT => Some(rect.fill),
            _ => None,
        });
        let ink = output.shapes.iter().find_map(|shape| match &shape.shape {
            egui::Shape::Text(text) if text.galley.job.text == "Continue" => text
                .galley
                .job
                .sections
                .first()
                .map(|section| section.format.color),
            _ => None,
        });
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        let ratio = contrast(fill.unwrap(), ink.unwrap());
        assert!(ratio >= 4.5, "dark={dark} contrast is {ratio:.2}:1");
    }
}

fn contrast(a: egui::Color32, b: egui::Color32) -> f32 {
    let luminance = |color: egui::Color32| {
        let channel = |value: u8| {
            let value = f32::from(value) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    };
    let (light, dark) = match luminance(a).total_cmp(&luminance(b)) {
        std::cmp::Ordering::Less => (luminance(b), luminance(a)),
        _ => (luminance(a), luminance(b)),
    };
    (light + 0.05) / (dark + 0.05)
}
