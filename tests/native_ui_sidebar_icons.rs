#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::workspace::sidebar;
use eframe::egui;
use serde_json::json;

#[test]
fn installed_provider_icon_is_rendered_instead_of_the_monogram() {
    let icon = agentconductor::icons::png_data_url(2, 2, &[255; 16]).unwrap();
    let profiles = vec![json!({"id":"claude","parser":"claude"})];
    let snapshot = json!({
        "config":{"profiles":profiles},
        "icons":{"claude":icon}
    });
    let context = egui::Context::default();
    let mut output = context.run_ui(Default::default(), |ui| {
        sidebar::show(&profiles, &snapshot, "en", false, false, ui);
    });
    let image = output
        .shapes
        .iter()
        .any(|shape| matches!(shape.shape, egui::Shape::Mesh(_)));
    let monogram = output.shapes.iter().any(
        |shape| matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text == "CC"),
    );
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert!(
        image,
        "provider icon data must be painted in the agent card"
    );
    assert!(
        !monogram,
        "do not replace an installed icon with a monogram"
    );
}
