use eframe::egui;

use super::Policy;

#[test]
fn operating_system_reduced_motion_disables_egui_animations() {
    let context = egui::Context::default();
    let policy = Policy::new(&context);

    policy.apply_detected(&context, Some(true));

    assert_eq!(context.global_style().animation_time, 0.0);
}

#[test]
fn clearing_operating_system_reduced_motion_restores_the_original_animation_time() {
    let context = egui::Context::default();
    let original = context.global_style().animation_time;
    let policy = Policy::new(&context);

    policy.apply_detected(&context, Some(true));
    policy.apply_detected(&context, Some(false));

    assert_eq!(context.global_style().animation_time, original);
}

#[test]
fn unavailable_operating_system_preference_preserves_existing_reduced_motion() {
    let context = egui::Context::default();
    context.global_style_mut(|style| style.animation_time = 0.0);
    let policy = Policy::new(&context);

    policy.apply_detected(&context, None);

    assert_eq!(context.global_style().animation_time, 0.0);
}

#[test]
fn temporary_preference_read_failure_keeps_the_last_known_setting() {
    let context = egui::Context::default();
    let policy = Policy::new(&context);

    policy.apply_detected(&context, Some(true));
    policy.apply_detected(&context, None);

    assert_eq!(context.global_style().animation_time, 0.0);
}

#[cfg(target_os = "macos")]
#[test]
fn macos_accessibility_preference_is_readable() {
    let context = egui::Context::default();
    let policy = Policy::new(&context);
    let reduced = super::system_preference().expect("macOS accessibility preference");

    policy.update(&context);

    assert_eq!(context.global_style().animation_time == 0.0, reduced);
}
