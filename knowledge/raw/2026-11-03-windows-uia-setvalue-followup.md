# Windows Goal UI Automation Follow-up

Date: 2026-11-03
Repository branch: `codex/issue-9-native-ui-parity`

## Red evidence

Platform Validation run `41520505534` built and launched the Windows package. The native package journey `default_native_workspace_is_accessible_and_saves_through_ui_automation` failed before Save. Its driver reported `GOAL_FOCUS_AFTER_INPUT:...ControlType.Window:AgentConductor`, `GOAL_TEXT_AFTER_INPUT:`, and `GOAL_VALUE_MISMATCH:`. The exact failure shows that successful Unicode `SendInput` did not update the Goal edit.

The existing LLM Wiki decision in `windows-uia-text-input.md` already specifies the UI Automation `ValuePattern.SetValue` route and bounded value readback. The implementation and source-contract tests had drifted back to keyboard injection.

## TDD correction

Changed `tests/windows_desktop_driver.rs` and `tests/windows_run_lifecycle.rs` first to require the order: foreground the window, focus the Goal edit, call `ValuePattern.SetValue`, poll the current UIA value, and then Save. The new source assertions failed five times against the current keyboard-injection driver.

The Windows desktop driver now calls `$goal.SetFocus()` and `$goalValue.SetValue('Windows UIA goal')`, then reacquires the edit while polling the value. It reacquires Save afterward and retains the mouse fallback for invoking Save. The unused Unicode keyboard-injection helper was removed.

## Local verification

`cargo test --locked --offline --test windows_desktop_driver --test windows_run_lifecycle --test source_layout` passed 19 tests with 1 ignored. `cargo fmt --all -- --check` and `git diff --check` passed. Changed Windows test files remain below 300 lines.

## Verification boundary

Run `41545058701` was triggered on pre-correction head `21eb66f4`, before this SetValue change, so it cannot verify the correction. A new exact-head Windows packaged run must persist the full Goal string in `startup.json` before package acceptance advances. macOS source-level verification does not substitute for that Windows run.
