# Windows packaged signal-settings input

## Observed behavior

The exact Windows release-package UI test intermittently retained only the first digit when entering the agentmemory port through `SendInput`. The failure appeared as `PORT_CHARACTER_NOT_APPLIED` (`expected=73 actual=7`); settings persistence was never reached.

## Cause and rule

The driver restored the port edit focus once before typing, but the native UI can rebuild the accessibility element after a keystroke. Reacquire the edit and assert its focus before each subsequent key. Keep real keyboard input in this test; replacing it with `ValuePattern.SetValue` would stop covering the user input path.

## Verification

The source-contract regression failed before the per-key focus assertion was added. After the fix, the Windows signal-settings contract suite passes 12 tests, source layout passes 2 tests, and `tests/windows_signal_settings.rs` remains at 300 lines. Commit `ef595d5c` has an exact macOS/Windows validation and package rerun pending as `40766484837`.

## First-run workspace path in the package fixture

The issue #226 Windows package journey starts with a blank profile, creates the required named workspace, then opens Settings. The name `Windows UIA workspace` produces the managed directory `windows-uia-workspace`. The test must wait for that exact path to appear in `.agentconductor/workspaces.json` after the UI save; reading the registry immediately can observe a valid empty list while the UI action is still settling. Poll for a bounded interval and fail with the last observed registry contents if the path never appears. Avoid filename-only matching, which can silently select a different workspace.

Run `40783168410` reached this assertion after the other Windows package journeys passed and observed `[]`; the macOS package and both platform validations passed. A deterministic delayed-registry regression now checks the wait behavior. Run `40784206494` failed earlier, before this assertion, because the shared port-edit UIA driver raised `ElementNotEnabledException` on `SetFocus`. That failure is tracked as a separate #9 test-driver correction and does not verify the registry polling change. Exact package verification of the polling fix remains pending; do not treat local source tests as package acceptance. See [`docs/evidence/issue226-windows-registry-poll-20261029.json`](../../docs/evidence/issue226-windows-registry-poll-20261029.json).
