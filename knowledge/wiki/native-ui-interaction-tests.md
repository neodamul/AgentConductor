# Native UI Interaction Tests

Native UI component tests should exercise the action a person triggers, not only verify that an accessible button is rendered. Enable AccessKit in the egui context, locate a button by its localized accessible label, and dispatch pointer-move, press, and release events at the button's reported bounds. Assert the resulting domain action, clipboard command, or dialog state.

When testing both English and Korean, use the real localized labels and run the same interaction in each language. For asynchronous panels, cover both accepted actions and the pending state: repeated work must remain disabled while safe navigation such as Close stays available.

Every synthetic egui frame returns a `FullOutput`. Tests that do not apply texture updates must clear both `textures_delta.set` and `textures_delta.free` before dropping the output; clear intermediate frames inside the click helper and the final frame in the caller. Otherwise egui panics on an unapplied texture delta and obscures the actual interaction result.

Issue #9 coverage added this pattern to the manual and updater panels. The component regressions verify copy/close in both languages, check/apply/close in both languages, and duplicate-action blocking while an update request is pending. These tests do not count as packaged end-to-end workflow acceptance.
