use super::*;

#[test]
fn background_metadata_is_quiet_but_errors_and_saves_remain_visible() {
    for pending in [Pending::Metadata, Pending::MetadataRefresh, Pending::Save] {
        let context = egui::Context::default();
        let mut probe = Probe {
            pending: Some(pending),
            error: Some("Action failed".into()),
            ..Default::default()
        };
        let mut output = context.run_ui(Default::default(), |ui| show(&probe, ui));
        let labels: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| {
                if let egui::Shape::Text(text) = &shape.shape {
                    Some(text.galley.text().to_owned())
                } else {
                    None
                }
            })
            .collect();
        assert!(labels.iter().any(|label| label == "Action failed"));
        assert!(
            !labels.iter().any(|label| label.contains("metadata")),
            "{labels:?}"
        );
        if matches!(pending, Pending::Save) {
            assert!(labels.iter().any(|label| label == "Saving workspace…"));
        }
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        probe.error = None;
        assert_eq!(visible(&probe), matches!(pending, Pending::Save));
    }
}
