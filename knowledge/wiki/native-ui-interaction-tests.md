# Native UI Interaction Tests

Native UI component tests should exercise the action a person triggers, not only verify that an accessible button is rendered. Enable AccessKit in the egui context, locate a button by its localized accessible label, and dispatch pointer-move, press, and release events at the button's reported bounds. Assert the resulting domain action, clipboard command, or dialog state.

When testing both English and Korean, use the real localized labels and run the same interaction in each language. For asynchronous panels, cover both accepted actions and the pending state: repeated work must remain disabled while safe navigation such as Close stays available.

Every synthetic egui frame returns a `FullOutput`. Tests that do not apply texture updates must clear both `textures_delta.set` and `textures_delta.free` before dropping the output; clear intermediate frames inside the click helper and the final frame in the caller. Otherwise egui panics on an unapplied texture delta and obscures the actual interaction result.

Issue #9 coverage added this pattern to the manual and updater panels. The component regressions verify copy/close in both languages, check/apply/close in both languages, and duplicate-action blocking while an update request is pending. These tests do not count as packaged end-to-end workflow acceptance.

## Popup menus that contain editable controls

egui `Popup::menu` defaults to `CloseOnClick`, which also closes the popup when a user clicks
inside it. A workspace menu contains a name field, so use `CloseOnClickOutside` and explicitly
close the popup after a workspace action, confirmation, guide, or manual opens. Test the full
pointer-to-text-to-Enter interaction; merely asserting that a text input is present misses this
failure. The Settings menu remains an ordinary `menu_button` because its items are selections,
not an editable form. Rendered geometry tests verify that opening either top-bar menu preserves
the underlying page position.

### Keep menu geometry tests locale-independent

Read the visible/accessibility label from the component's localization state instead of hard-coding
English or Korean. CI may run with a different default language than a developer machine; the
geometry assertion must still locate the same control. For example, use
`State::text("startup.settings")` for the Settings menu button.

### Match visual evidence to the executable

An egui geometry test proves behavior of the tested source, not of an already running packaged
application. Record the executable path and build time (or commit manifest), and ensure the running
process was built from the same source before claiming a screen-level fix. Keep a real workspace
window intact when it may contain unsaved state; use an isolated profile for visual verification.
