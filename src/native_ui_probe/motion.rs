use eframe::egui::Context;
use std::cell::Cell;

#[derive(Default)]
pub(super) struct Policy {
    original_animation_time: Cell<Option<f32>>,
    reduced_motion: Cell<Option<bool>>,
}

impl Policy {
    pub(super) fn new(context: &Context) -> Self {
        Self {
            original_animation_time: Cell::new(Some(context.global_style().animation_time)),
            reduced_motion: Cell::new(None),
        }
    }

    pub(super) fn update(&self, context: &Context) {
        self.apply_detected(context, system_preference());
    }

    fn apply_detected(&self, context: &Context, preference: Option<bool>) {
        if self.original_animation_time.get().is_none() {
            self.original_animation_time
                .set(Some(context.global_style().animation_time));
        }
        if let Some(preference) = preference {
            self.reduced_motion.set(Some(preference));
        }
        let normal = self.original_animation_time.get().unwrap_or_default();
        let desired = if self.reduced_motion.get().unwrap_or(false) {
            0.0
        } else {
            normal
        };
        if context.global_style().animation_time != desired {
            context.global_style_mut(|style| style.animation_time = desired);
        }
    }
}

#[cfg(target_os = "macos")]
fn system_preference() -> Option<bool> {
    crate::icons::reduced_motion_preference()
}

#[cfg(target_os = "windows")]
fn system_preference() -> Option<bool> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SystemParametersInfoW,
    };

    let mut enabled = 0i32;
    // SAFETY: This query writes one BOOL result to the valid, writable local value.
    unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some((&mut enabled as *mut i32).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .ok()?;
    Some(enabled == 0)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn system_preference() -> Option<bool> {
    None
}

#[cfg(test)]
#[path = "../../tests/native_ui_probe/motion.rs"]
mod tests;
