#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::{Probe, status_color, status_color_with_motion};
use eframe::egui::{self, ViewportId};
use serde_json::json;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[test]
fn running_status_pulses_smoothly_and_repeats_after_one_point_six_seconds() {
    let red = |time| status_color("running", time).r();
    let low = red(0.0);
    let middle = red(0.4);
    let high = red(0.8);

    assert!(low < middle && middle < high);
    assert_eq!(low, red(1.6));
}

#[test]
fn running_status_does_not_schedule_pulse_repaints_when_motion_is_disabled() {
    let context = egui::Context::default();
    let repaint_delays = Arc::new(Mutex::new(Vec::new()));
    context.set_repaint_observer({
        let repaint_delays = Arc::clone(&repaint_delays);
        move |request| repaint_delays.lock().unwrap().push(request.delay)
    });
    let mut style = (*context.style_of(egui::Theme::Light)).clone();
    style.animation_time = 0.0;
    context.set_style_of(egui::Theme::Light, style.clone());
    context.set_style_of(egui::Theme::Dark, style);
    context.set_theme(egui::Theme::Light);
    let snapshot = json!({
        "report": {"plan": {"tasks": [{"id": "task-1", "agent": "claude"}]},
                     "tasks": [{"id": "task-1", "status": "running"}]}
    });
    let mut initial = context.run_ui(Default::default(), |_| {});
    initial.textures_delta.set.clear();
    initial.textures_delta.free.clear();
    let mut output = context.run_ui(Default::default(), |ui| {
        Probe::render_snapshot(&snapshot, ui)
    });
    let delay = output.viewport_output[&ViewportId::ROOT].repaint_delay;
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
    assert_ne!(delay, Duration::from_millis(500));
    assert!(
        !repaint_delays
            .lock()
            .unwrap()
            .contains(&Duration::from_millis(500)),
        "reduced motion still scheduled the pulse repaint"
    );
}

#[test]
fn running_status_is_static_green_when_motion_is_disabled() {
    let at = |time| status_color_with_motion("running", time, false);
    assert_eq!(at(0.0), at(0.8));
    assert_eq!(at(0.8), status_color("completed", 0.0));
}

#[test]
fn quota_observation_refresh_does_not_restart_the_running_pulse() {
    let context = egui::Context::default();
    let snapshot = |remaining| {
        json!({
            "quota_observations": [{"agent_id": "claude", "remaining_percent": remaining}],
            "report": {
                "state": "running",
                "plan": {"tasks": [{"id": "task-1", "agent": "claude"}]},
                "tasks": [{"id": "task-1", "status": "running"}]
            }
        })
    };
    let render = |time, remaining| {
        let data = snapshot(remaining);
        let mut output = context.run_ui(
            egui::RawInput {
                time: Some(time),
                ..Default::default()
            },
            |ui| Probe::render_snapshot(&data, ui),
        );
        let color = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) if text.galley.job.text == "●" => text
                    .galley
                    .job
                    .sections
                    .first()
                    .map(|section| section.format.color),
                _ => None,
            })
            .expect("running status pulse dot");
        output.textures_delta.set.clear();
        output.textures_delta.free.clear();
        color
    };

    let before_refresh = render(0.4, 64);
    let after_refresh = render(0.8, 63);

    assert_eq!(before_refresh, status_color("running", 0.4));
    assert_eq!(after_refresh, status_color("running", 0.8));
    assert_ne!(before_refresh, after_refresh);
}
