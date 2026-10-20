# Reduced motion for native status indicators

Native Rust status indicators use egui's animation setting as the available motion preference. Running task and coordinator indicators remain steady green when animations are disabled. When animations are enabled, they retain the existing pulse and repaint cadence.

The review screen can continue refreshing elapsed time and observed output independently of the pulse. That refresh is not disabled with animation, so progress evidence remains current.

This behavior does not claim to read a platform-level reduced-motion preference. The app currently gates on egui's configured `animation_time`; wiring and validating the OS accessibility preference is a separate acceptance item.

Regression evidence: `tests/native_ui_pulse.rs` checks static color and ensures the animation-disabled task list does not request the pulse repaint; `tests/native_ui_activity.rs` checks static planning activity. Related issue: GitHub #59.
