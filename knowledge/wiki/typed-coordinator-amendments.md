# Typed coordinator amendments

The coordinator can request additional paths or dependencies for an existing task, or replace its test command when the recorded command cannot work in the project. Path and dependency amendments allow at most eight entries of at most 1,024 bytes each; test argv allows at most 32 arguments of at most 4,096 bytes each. The run validates these through the same `Plan::validate` rules used at plan creation, so unknown dependencies, cycles, path traversal, reserved paths and overlapping ownership fail closed. Empty, excessive or malformed fields are rejected at both review and application boundaries.

Paths that name a sensitive location or a whole top-level directory (including an explicit slash-terminated dotted directory such as `assets.v1/`) are recorded as escalations; they do not change the plan. Already-present dependencies are rejected instead of producing a no-op audit entry. Five applied amendments per run bound automated revision. Every application stores the replaced task paths, dependencies, and test argv in the existing SQLite audit stream. Replacement test commands must resolve in the repository or be a script the task owns. Undo restores those exact values, invalidates derived candidate/test approval, and leaves the run stopped for explicit resume. A later active amendment to the same task must be undone first.

The coordinator judgment list derives applied, escalated and undone rows from audit records. The native Agent Room offers Undo only for active applied amendments. The action carries the current run revision and uses the same local action process and CLI path as other human decisions.

## Charter and drift guard

Each new Report stores a charter derived from the exact Plan: the goal, SHA-256 of the effective planning rules, and ordered task IDs, owned paths, dependencies, and test argv. The rules themselves stay in the Plan and are omitted from serialized JSON when empty, preserving old plan digests. Phased plans persist the effective rules under `plan/plan-rules.md`; later phases reuse that snapshot.

Before workers are scheduled, a candidate is reviewed or approved, or a merge is applied, the Rust guardrail compares the Plan with its charter. Unrecorded changes block as `goal_drift` and appear in the existing coordinator judgment list. Legacy Reports without a charter remain readable and skip this comparison. Validated coordinator amendments and operator-approved scope recovery synchronize only the edited task; Undo restores the prior task contract and revokes candidate evidence.

The charter is structural: it protects the goal, effective-rule digest, task IDs, paths, dependencies and test argv. It does not judge whether a goal or prompt is semantically complete. Windows/package acceptance remains separate from local Rust regression evidence.

Validation: deterministic Rust coordinator fixtures cover safe application through re-verification and human approval, exact undo, sensitive/broad/overlapping rejection, budget exhaustion, and localized UI affordance visibility. Focused charter, amendment, phase, planner and decision tests pass. The unfiltered offline all-target suite passes 932 tests / 14 ignored / 307 suites; warning-denied all-target Clippy, source layout, formatting and the 300-line check pass. Graphify refreshed the local code graph (8,762 nodes / 15,015 edges). Windows/package acceptance remains open.

Historical checkpoint (2026-10-21): `cargo test --test charter_contract` passed all 5 tests. This suite verifies that a run saves and reloads its charter, rejects plan changes before approval, rejects changes after approval before merge, records drift decisions, and continues to deserialize legacy reports. At that checkpoint, exact packaged step-4 acceptance was still open and the packaged workflow count was 8/25.

The Rust Agent Room previously rendered a goal-drift judgment through the generic fallback (`Coordinator judgment: goal_drift blocked`). A red bilingual label test reproduced it. `room::judgments::label` now selects a dedicated localized explanation, and a rendered-view test confirms the English label and charter-change evidence are visible. The focused charter, coordinator-decision and native-judgment suites pass 16/16; source layout confirms the 300-line bound, warning-denied focused Clippy and formatting pass. Exact macOS package `aa8090405239` passes manifest validation and all four parallel fixture smoke cases. In the single packaged GUI window, I opened a disposable copy of the pending Issue 9 run and inserted a synthetic goal-drift audit event only in that copy. The Korean label “승인된 작업 기준이 변경되어 코디네이터가 실행을 차단했습니다” rendered with its synthetic evidence; the copied view showed 3/3 tasks and tests. The original candidate remained untouched and pending, and was restored in the single window after the display check. The temporary clone was removed. ZIP SHA-256 and package/UI details are in `docs/evidence/issue9-aa80904-macos-package.json`.

## Current issue #9 / #149 checkpoint — 2026-10-23

All four operator-approved #149 implementation steps are present on the Rust parity branch and main: visible decision records, deterministic launch gate, bounded reversible amendments, and the persisted charter/drift guard. Rust commit `43463f4b` passed 28 focused tests, source-layout 2/2, warning-denied Clippy, formatting, and the 300-line source/test check. The full all-target baseline passed 938 tests (14 ignored) at `084089fb`; commits through `9dc00bbe` after that point changed documentation only. The exact macOS v0.1.36 charter-drift refusal is recorded in `docs/evidence/issue9-aa80904-goal-drift-workflow.json`; Workflow 12 at the current package checkpoint brings packaged UI acceptance to 12/25 (48%).

The shipped/default Rust UI replacement remains 0/25. Windows package/runtime/UI acceptance is still unverified, so #149 stays open for that platform gate; #9 and draft PR #22 remain open. AgentMemory: `mem_mvkanf0j_cf143e9725cf`.

Main has since advanced to tagged v0.1.37 at `b7fa27a9`; this Rust migration branch is still on package version 0.1.36. The exact UI acceptance above is for the verified v0.1.36 package. Reconcile the release notes, manuals, manifest version, and PR conflicts when the final migration source is ready; do not treat the v0.1.36 package evidence as verification of v0.1.37.

## Source fingerprints (2026-10-23)

| Source | SHA-256 |
| --- | --- |
| `src/workflow/guardrail/charter.rs` | `af5f8244f4a906b9cd591edf9370bffb67b0e7653f18e814cf4daad014f4f291` |
| `src/workflow/guardrail.rs` | `10e757e7202fbb84e0e987ddfe451ccd9b09dc1e689fca730cdff2a2128e0baf` |
| `src/workflow/amendments.rs` | `11f9fd7e86596ac8d8ea03ebb528e7bb3a981de737c759440b74f95af757aa7e` |
| `src/workflow/scheduler.rs` | `35308fdd57305c56dfbbf4b6e9449a06e82ff3dc32c8f44c12297075275e7c64` |
| `src/workflow/approval.rs` | `d2354d01c296b4a3e325eba940db7c52319aef73ebcf458295b41c762d0bd274` |
| `src/workflow/scope_recovery.rs` | `0bb053e08c8aa7467d45e0d1013f59cde94b7a018ed6a8ae17f2cfe7f6c92bbf` |
| `src/planner.rs` | `a0775e0b47cf593a24c08ea84902f30d93c0c9f3edfe29615494c952a97eca0b` |
| `src/phases.rs` | `5f98e5598cc1b9d3e7e12b083e9b87ff19e3ed0e2c7fb0106e8e601db9b181e7` |
| `tests/charter_contract.rs` | `0a41c0e11bd821d3678a1bd93e3fd6c428b8b8caaf679ad3442d16ddf21bcace` |
