# Native Workspace location status

## Behavior

The JavaScript Workspace keeps a status bar pinned to the bottom of the window. Before its first snapshot it shows `Reading local workspace…`; afterward it shows `Local workspace · <workspace root>`.

The Rust Workspace now renders the same localized states in an `egui::Panel::bottom` before the central panel. This reserves space for the status line while the central form scrolls independently.

## Verification

- `tests/native_ui_probe/workspace_footer.rs` renders the complete native Workspace and checks the loading message and bottom-pinned workspace path.
- The full locked/offline all-target suite passes (1,344 passed, 16 ignored across 411 suites). Footer regressions pass 2/2; native header/source-layout suites, warning-denied native-feature Clippy, formatting, and diff hygiene pass.
- The optional screenshot build could not resolve `eframe` from the offline registry index, so no window was opened. The full egui Workspace render test verifies the displayed text and bottom placement.

## Acceptance

This restores a visible Workspace context indicator but does not complete a distinct packaged workflow. Issue #9 remains 22/50 overall; shipped/default Rust UI parity remains 0/25 until full same-state screen review and exact package acceptance are complete.
