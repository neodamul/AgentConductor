# Reliable Windows UI Automation Text Input

## Context

The packaged Windows desktop acceptance driver entered a workspace goal with `SendInput` Unicode key events. Even after pacing each character, the exact package sometimes persisted a truncated value (`Windows UIA goa`) or an empty value. A green source-level input test did not catch this because it could not run the Windows desktop.

## Decision

For deterministic packaged UI tests, address the accessible edit control through UI Automation's `ValuePattern` and call `SetValue` with the complete fixture text. Poll `Current.Value` for at most three seconds because the native UI event may settle asynchronously. Only activate Save after the returned value exactly matches the fixture. Keep actual keyboard input for actions that are keyboard-specific, such as pressing Enter to activate Save.

The application's locked UI stack supports this route: `egui 0.37.0` emits AccessKit `SetValue` for text edits, and `accesskit_windows 0.35.1` exposes `IValueProvider.SetValue` when the accessible node has a value. This is deterministic test automation; it does not change how users type into the application.

## Verification Boundary

The regression must assert the driver order (focus, `ValuePattern.SetValue`, value readback, Save) and the test must assert the persisted `startup.json` value. A macOS source-contract test verifies only the driver structure. Exact-head run `41347900792` disproved the earlier claim that the driver used `SetValue`: the Windows driver still used Unicode `SendInput`, focused the Goal field, and read an empty value. The source contract now requires `SetValue` plus bounded readback, but this correction still needs a fresh exact packaged Windows run. Count the Windows workflow only after that package test succeeds.

## References

- Windows package driver: `tests/windows_desktop.rs`
- Driver contract tests: `tests/windows_desktop_driver.rs`
- `egui` text edit accessibility action: `egui-0.37.0/src/widgets/text_edit/builder.rs`
- Windows provider pattern implementation: `accesskit_windows-0.35.1/src/node.rs`
