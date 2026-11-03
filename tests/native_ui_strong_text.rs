#![cfg(feature = "native-ui-probe")]
//! egui draws `RichText::strong()` with `Visuals::strong_text_color()`, which is the text colour
//! of the *active* widget state. The theme must keep it readable on every surface in both themes.

use agentconductor::native_ui_probe as native;
use eframe::egui::{self, Color32};

fn luminance(color: Color32) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

fn contrast(a: Color32, b: Color32) -> f64 {
    let (high, low) = (
        luminance(a).max(luminance(b)),
        luminance(a).min(luminance(b)),
    );
    (high + 0.05) / (low + 0.05)
}

#[test]
fn strong_text_is_readable_on_every_surface_in_both_themes() {
    for theme in ["light", "dark"] {
        let context = egui::Context::default();
        native::appearance::apply(theme, &context);
        let visuals = context.global_style().visuals.clone();
        let strong = visuals.strong_text_color();
        for (name, surface) in [
            ("panel", visuals.panel_fill),
            ("window", visuals.window_fill),
            ("rail", visuals.faint_bg_color),
            ("input", visuals.extreme_bg_color),
            ("button", visuals.widgets.inactive.bg_fill),
            ("hovered button", visuals.widgets.hovered.bg_fill),
        ] {
            assert!(
                contrast(strong, surface) >= 4.5,
                "{theme}: strong text {strong:?} on {name} {surface:?} is {:.1}:1",
                contrast(strong, surface)
            );
        }
    }
}

#[test]
fn pressed_widgets_keep_readable_text_in_both_themes() {
    for theme in ["light", "dark"] {
        let context = egui::Context::default();
        native::appearance::apply(theme, &context);
        let active = context.global_style().visuals.widgets.active;
        assert!(
            contrast(active.fg_stroke.color, active.bg_fill) >= 4.5,
            "{theme}: pressed text {:?} on {:?} is {:.1}:1",
            active.fg_stroke.color,
            active.bg_fill,
            contrast(active.fg_stroke.color, active.bg_fill)
        );
    }
}
