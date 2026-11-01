# Reliable Windows UI Automation Text Input

## Context

The packaged Windows desktop acceptance driver entered a workspace goal with `SendInput` Unicode key events. Even after pacing each character, the exact package sometimes persisted a truncated value (`Windows UIA goa`) or an empty value. A green source-level input test did not catch this because it could not run the Windows desktop.

## Decision

For deterministic packaged UI tests, address the accessible edit control through UI Automation's `ValuePattern`, call `SetValue` with the complete fixture text, and immediately read back `Current.Value`. Only activate Save after the returned value exactly matches the fixture. Keep actual keyboard input for actions that are keyboard-specific, such as pressing Enter to activate Save.

The application's locked UI stack supports this route: `egui 0.37.0` emits AccessKit `SetValue` for text edits, and `accesskit_windows 0.35.1` exposes `IValueProvider.SetValue` when the accessible node has a value. This is deterministic test automation; it does not change how users type into the application.

## Verification Boundary

The regression must assert the driver order (focus, `ValuePattern.SetValue`, value readback, Save) and the test must assert the persisted `startup.json` value. A macOS source-contract test verifies only the driver structure. Count the Windows workflow only after the exact packaged Windows test completes successfully.

## References

- Windows package driver: `tests/windows_desktop.rs`
- Driver contract tests: `tests/windows_desktop_driver.rs`
- `egui` text edit accessibility action: `egui-0.37.0/src/widgets/text_edit/builder.rs`
- Windows provider pattern implementation: `accesskit_windows-0.35.1/src/node.rs`
