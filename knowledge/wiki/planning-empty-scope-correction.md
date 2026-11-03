# Correcting empty task scopes — GitHub #251 / issue #9

## Context

The planner output schema accepted an empty `paths` array even though `Plan::validate` rejected it. That mismatch let a generated phased plan reach validation with no owned files, then fail with a combined message that did not identify the task or field. The Rust implementation also retried unsafe or overlapping scopes but not empty scopes.

## Decision

- Require at least one path and an executable test command in both flat and phased output schemas. Keep `depends_on` optional so independent work remains expressible.
- Report the invalid task and field. Use the shared `EMPTY_PATHS` diagnostic as the contract between validation and planner retry logic.
- Retry a phased empty scope once, just like an unsafe or overlapping scope. Keep flat planning's existing single attempt.
- Tell the planner that verification tasks also own paths; the corrective prompt names the validation error and asks for a complete corrected plan.
- Keep task validation in `src/plan/task_validation.rs` so `src/plan.rs` and `src/planner.rs` remain within the 300-line source limit.

This refines R-203 → BG-158 → SF-191 → UC-194 → FR-195 → QAS-201 → DD-215. Upstream #251 reused IDs already assigned to another Rust port, so this branch uses a unique trace.

## Verification

TDD first failed three checks against the pre-fix implementation: field-specific plan errors, `minItems: 1` for paths and test commands, and a deterministic phased-plan retry after an empty verification scope. Green verification passes `plan` (13 tests), `planning_schema` (5), `planning_empty_scope_retry` (1), and `planning_overlap_repair` (1). `src/plan.rs` is 287 lines, `src/plan/task_validation.rs` 38, `src/planner.rs` 298, `src/planner/retry.rs` 3, and every changed test file is below 300 lines.

Full all-target verification, Windows package acceptance, and the 300-line repository-wide layout suite remain separate checks; do not infer them from these focused tests.
