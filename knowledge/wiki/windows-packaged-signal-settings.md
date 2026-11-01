# Windows packaged signal-settings input

## Observed behavior

The exact Windows release-package UI test intermittently retained only the first digit when entering the agentmemory port through `SendInput`. The failure appeared as `PORT_CHARACTER_NOT_APPLIED` (`expected=73 actual=7`); settings persistence was never reached.

## Cause and rule

The driver restored the port edit focus once before typing, but the native UI can rebuild the accessibility element after a keystroke. Reacquire the edit and assert its focus before each subsequent key. Keep real keyboard input in this test; replacing it with `ValuePattern.SetValue` would stop covering the user input path.

## Verification

The source-contract regression failed before the per-key focus assertion was added. After the fix, the Windows signal-settings contract suite passes 12 tests, source layout passes 2 tests, and `tests/windows_signal_settings.rs` remains at 300 lines. Commit `ef595d5c` has an exact macOS/Windows validation and package rerun pending as `40766484837`.
