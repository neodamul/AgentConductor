#![cfg(feature = "native-ui-probe")]

use agentconductor::native_ui_probe::status_color;

#[test]
fn running_status_pulses_smoothly_and_repeats_after_one_point_six_seconds() {
    let red = |time| status_color("running", time).r();
    let low = red(0.0);
    let middle = red(0.4);
    let high = red(0.8);

    assert!(low < middle && middle < high);
    assert_eq!(low, red(1.6));
}
