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

#[test]
fn long_foreground_request_names_the_operation_and_elapsed_time() {
    let probe = Probe {
        pending: Some(Pending::Save),
        pending_since: Some(std::time::Instant::now() - std::time::Duration::from_secs(65)),
        ..Default::default()
    };

    assert!(status(&probe).ends_with("Saving workspace… · 1:05"));
}

#[test]
fn run_action_keeps_its_specific_description_while_waiting() {
    let probe = Probe {
        pending: Some(Pending::Action("Stopping the run…")),
        pending_since: Some(std::time::Instant::now() - std::time::Duration::from_secs(2)),
        ..Default::default()
    };

    assert!(status(&probe).ends_with("Stopping the run… · 0:02"));
}

#[test]
fn foreground_request_shows_its_operation_before_one_second() {
    let probe = Probe {
        pending: Some(Pending::Save),
        pending_since: Some(std::time::Instant::now() - std::time::Duration::from_millis(100)),
        ..Default::default()
    };

    assert_eq!(status(&probe), "Saving workspace…");
}

#[test]
fn pending_age_restarts_for_new_work_and_stays_quiet_for_metadata() {
    let started = std::time::Instant::now();
    let mut probe = Probe {
        pending: Some(Pending::Save),
        ..Default::default()
    };
    track_pending(&mut probe, started);
    assert_eq!(probe.pending_since, Some(started));

    let next = started + std::time::Duration::from_secs(10);
    probe.pending = Some(Pending::Prepare);
    track_pending(&mut probe, next);
    assert_eq!(probe.pending_since, Some(next));

    probe.pending = Some(Pending::Metadata);
    track_pending(&mut probe, next);
    assert_eq!(probe.pending_since, None);

    probe.pending = Some(Pending::Refresh);
    track_pending(&mut probe, next);
    assert_eq!(probe.pending_since, None);
}
