use super::{Probe, frame, labels};
use eframe::egui;
use std::{
    thread,
    time::{Duration, Instant},
};

pub(super) fn wait_for_ui(
    context: &egui::Context,
    probe: &mut Probe,
    ready: impl Fn(&[String]) -> bool,
) {
    wait_for_ui_timeout(context, probe, Duration::from_secs(5), ready);
}

pub(super) fn wait_for_ui_timeout(
    context: &egui::Context,
    probe: &mut Probe,
    timeout: Duration,
    ready: impl Fn(&[String]) -> bool,
) {
    let deadline = Instant::now() + timeout;
    loop {
        let mut output = frame(context, probe, Vec::new());
        let labels = labels(&output);
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        if ready(&labels) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "UI state not reached; labels: {labels:?}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}
