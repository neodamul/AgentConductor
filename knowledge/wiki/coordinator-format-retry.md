# Coordinator format correction retry

When a coordinator process exits successfully but its output contains no usable JSON for the requested field, `coordinator::run_json_within_model` makes one additional call. The retry prompt includes the parse reason using the existing `prompt_form::with_previous_reply` helper. The first attempt's process directory is retained as `process-format-1` so both stdout and stderr remain available.

The correction path does not handle a timeout, explicit stop, nonzero process exit, provider error, or capacity response. Capacity retry behavior remains on its existing path. Two malformed replies fail after exactly two process executions; there is no unbounded loop.

The deterministic regression tests use a local fake CLI and cover one malformed reply followed by valid JSON, and two malformed replies. Focused evidence on 2026-10-29: 28 tests passed across the coordinator correction, coordinator, planner prompt, coordinator request, transient capacity, public docs, and source layout targets. The complete locked/offline all-target test command, `cargo fmt --check`, `git diff --check`, and warning-denied all-target Clippy passed at `09fa3482`. Exact package verification remains separate.

Architecture trace: R-186 → BG-141 → SF-174 → UC-177 → FR-178 → QAS-184 → DDR-42 → DD-198 (GitHub #235).
