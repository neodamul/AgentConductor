# Reliable Windows UI Automation Text Input

## Context

The packaged Windows desktop acceptance driver entered a workspace goal with `SendInput` Unicode key events. Even after pacing each character, the exact package sometimes persisted a truncated value (`Windows UIA goa`) or an empty value. A green source-level input test did not catch this because it could not run the Windows desktop.

## Decision

For deterministic packaged UI tests, address the accessible edit control through UI Automation's `ValuePattern` and call `SetValue` with the complete fixture text. Poll `Current.Value` for at most three seconds because the native UI event may settle asynchronously. Only activate Save after the returned value exactly matches the fixture. Keep actual keyboard input for actions that are keyboard-specific, such as pressing Enter to activate Save.

The application's locked UI stack supports this route: `egui 0.37.0` emits AccessKit `SetValue` for text edits, and `accesskit_windows 0.35.1` exposes `IValueProvider.SetValue` when the accessible node has a value. This is deterministic test automation; it does not change how users type into the application.

## Verification Boundary

The regression asserts the driver order (foreground, edit `SetFocus`, `ValuePattern.SetValue`, bounded value readback, Save), requires the related lifecycle-driver contract to use the same input method, and checks the persisted `startup.json` value on Windows. The current local red test failed five assertions against the reverted Unicode `SendInput` implementation; after restoring `SetValue`, the Windows desktop-driver, lifecycle, and source-layout targets pass 19 tests with one ignored. Formatting and diff checks pass. The Windows-specific package journey was not run locally.

Exact package run `41520505534` failed because the Goal value remained empty and UI Automation focus had returned to the top-level window after keyboard input. Platform validation and the macOS package passed, but this Windows failure blocks package acceptance. Run `41545058701` was triggered on the pre-correction head `21eb66f4` and cannot validate the restored `SetValue` route. Count the Windows workflow only after a new exact-head package run passes and `startup.json` contains the full fixture goal.

## References

- Windows package driver: `tests/windows_desktop.rs`
- Driver contract tests: `tests/windows_desktop_driver.rs`
- `egui` text edit accessibility action: `egui-0.37.0/src/widgets/text_edit/builder.rs`
- Windows provider pattern implementation: `accesskit_windows-0.35.1/src/node.rs`
