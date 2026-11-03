# Codex planning requires strict structured-output schemas

## Decision

All schema objects declare every property as required and closed. Optional task fields use nullable values. The schema limits `execution_options` to the supported `effort` field; null effort values are omitted when decoding into the runtime map. Task paths and test commands retain their non-empty array constraints.

Coordinator failure details include the bounded stderr and stdout tails. When both streams contain output they are labeled separately; a single stream is returned unchanged.

## Reason

Codex rejected the previous plan schema with `invalid_json_schema: Missing 'task_type'`. The prior schema omitted optional properties and allowed arbitrary execution-option keys. The CLI also wrote a skill-loading warning to stderr and the actual rejection to stdout, while failure reporting preferred stderr and hid the cause.

## Evidence

PR #338 passed macOS and Windows validation and package checks and was squash-merged to `main` as `75456ae4`. The native Rust migration branch preserves its modular source structure while incorporating the schema and diagnostic fixes. Local verification passes 17 focused tests and 1,299 all-target tests with 16 ignored across 403 suites; warning-denied Clippy, rustfmt, and file-length checks pass. No live Codex planning request was run.
