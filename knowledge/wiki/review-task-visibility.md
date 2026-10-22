# Native Review task-row visibility

## Reported behavior

An exact v0.1.34 package run showed three manual-plan tasks in the summary but appeared to show
only two editable rows. Adding a task made three rows visible, and a fresh plan rendered all three.
That observation was left open for reproduction rather than treated as a confirmed product defect.

## Current source verification

`tests/native_ui_probe/review_task_rows.rs` renders the full native Workspace screen at 1140×772
with three distinct manual task IDs. It resolves the task disclosure labels through the active
English or Korean locale and asserts all three controls have accessibility bounds within the
viewport. The regression passes on the current source. Its initial English-only assertion failed
under the Korean host locale; after using the active message catalog, the current screen showed all
three rows. No product behavior change was needed.

## Limits

This is rendered source-level evidence, not a packaged macOS/Windows runtime check, screenshot, or
end-to-end workflow. Packaged acceptance stays 8/25. See
[`issue9-review-three-task-rows.json`](../../docs/evidence/issue9-review-three-task-rows.json).
