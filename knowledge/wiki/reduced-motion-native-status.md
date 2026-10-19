# Reduced-motion native status indicators

The native UI uses `egui::Style::animation_time` as its local animation policy. A value of zero keeps the `running` state readable with its steady green status color and prevents animation-only pulse repaint requests. When animation is enabled, the existing status pulse remains.

Task rows and the workspace planning indicator share the motion-aware status color helper. Planning still refreshes elapsed time and coordinator output while running; those updates carry information and are not pulse-only animation. The Agent Room continues its independent data polling cadence.

This behavior verifies egui animation suppression. It does not prove that macOS or Windows reduced-motion preferences are detected or mapped into the egui style. Keep OS-level claims pending platform-specific verification.

Trace: `R-17` → `NFR-10` → `QAS-21` → `DD-17`. GitHub #59.
