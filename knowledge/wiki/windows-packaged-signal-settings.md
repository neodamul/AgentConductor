# Windows packaged signal-settings input

## Observed behavior

The exact Windows release-package UI test intermittently retained only the first digit when entering the agentmemory port through `SendInput`. The failure appeared as `PORT_CHARACTER_NOT_APPLIED` (`expected=73 actual=7`); settings persistence was never reached.

## Cause and rule

The driver restored the port edit focus once before typing, but the native UI can rebuild the accessibility element after a keystroke. Reacquire the edit and assert its focus before each subsequent key. Keep real keyboard input in this test; replacing it with `ValuePattern.SetValue` would stop covering the user input path.

## Verification

The source-contract regression failed before the per-key focus assertion was added. After the fix, the Windows signal-settings contract suite passes 12 tests, source layout passes 2 tests, and `tests/windows_signal_settings.rs` remains at 300 lines. Commit `ef595d5c` has an exact macOS/Windows validation and package rerun pending as `40766484837`.

## Workspace path in the package fixture

The signal-settings journey starts the exact binary with `ui-new <workspace-dir>`. That command opens the supplied directory and records that exact path in `.agentconductor/workspaces.json` before the UI starts. Resolve the registry entry by exact canonical path, then pass it to the signal-settings driver so the saved preference assertion targets the workspace the app actually owns. Registry reads remain bounded because test setup and UI startup can race; on timeout report the expected path and the last observed registry contents. Do not assume the current Workspace settings screen contains a separate `Create workspace` modal or infer a path from a workspace title.

Run `40783168410` used a now-stale modal setup and observed an empty registry immediately after that setup. Run `40784206494` failed earlier at the shared port-edit `SetFocus`; #9 fixed that driver in commit `28df8394`. On `40790683853`, the modal helper failed with `FIRST_WORKSPACE_PROMPT_MISSING`; the test was changed to use the explicit `ui-new` workspace path and bounded exact-path lookup. Run `40795726569` then reached the signal UI but hit the shared `PORT_FIELD_NOT_INTERACTABLE` gate, fixed on #9. After rebasing on `69887bab`, run `40800347115` saved the expected `agentmemory`/`7333` settings and passed macOS packaging, but the PowerShell driver failed afterward because `AC_SIGNAL_WORKSPACE` was null. Rust already owns the exact active workspace path and validates the saved JSON, so the redundant PowerShell `Join-Path` check was removed; local focused tests, Clippy, format, layout, and diff checks pass. Exact Windows packaging remains pending on this harness-only fix. Do not treat source tests as package acceptance. See [`docs/evidence/issue226-windows-registry-poll-20261029.json`](../../docs/evidence/issue226-windows-registry-poll-20261029.json).
