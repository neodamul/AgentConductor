use eframe::egui;

#[test]
fn workspace_content_remains_in_a_scrollable_accessible_viewport() {
    let context = egui::Context::default();
    context.enable_accesskit();
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(420.0, 220.0),
        )),
        ..Default::default()
    };
    let mut scroll = None;
    let mut output = context.run_ui(input, |ui| {
        scroll = Some(super::show(ui, |ui| {
            ui.label("Agent profile settings");
            ui.add_space(480.0);
            ui.button("Save agent roles")
                .accessible_name("Save agent roles");
        }));
    });
    let nodes = &output
        .platform_output
        .accesskit_update
        .as_ref()
        .unwrap()
        .nodes;
    assert!(
        nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Save agent roles"))
    );
    let scroll = scroll.unwrap();
    assert!(scroll.content_size.y > scroll.inner_rect.height());
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}
