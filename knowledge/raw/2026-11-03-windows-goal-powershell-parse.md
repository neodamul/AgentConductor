# Windows Goal diagnostic parse failure

## Evidence

Exact-head Platform Validation run `41617997008` passed macOS. Windows failed in `default_native_workspace_is_accessible_and_saves_through_ui_automation` before the UI journey entered text. PowerShell rejected the diagnostic interpolation `$goalX:` and `$goalY:` because a colon immediately after an unbraced variable is parsed as a scoped variable reference. The exact package job was skipped after validation failed.

## TDD correction

Added a regression to `tests/windows_desktop_driver.rs` requiring the diagnostic to use `GOAL_CLICK:${goalX}:${goalY}:`. The regression failed against the prior source. Updated `tests/windows_desktop.rs` to brace both variables. Local locked/offline verification passes 19 tests across `windows_desktop`, `windows_desktop_driver`, and `source_layout`; formatting, diff hygiene, and the 300-line check pass.

## Remaining verification

Commit `c241622ecb085baeec1371f0a10f319d312d1dbf` is pushed to the issue #9 Rust migration branch. Exact-head run `41622153226` must pass Windows validation and execute the packaged Goal save journey. The previous failed run provides no input or persistence acceptance.
