# Coordinator triage for failed tasks

**Architecture trace:** R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173 (GitHub #135).

## Behavior

The existing coordinator repair loop can review a failed run when at least one task is failed. It does not triage out-of-scope changes or a run with a task waiting for account availability. The coordinator receives the retained task status, error, stdout, and stderr evidence through the existing review path. A `revise` verdict resets the affected task and dependents, then resumes the run. Missing coordinator configuration, a blocked or unusable verdict, and three exhausted review rounds leave the run for a person.

This is deliberately the same bounded review loop used for failed integration checks. It does not add an orchestrator, process, API key, or new retry budget. A quota wait itself is never auto-triaged. For an operator-requested review during a shared-account wait, the selected coordinator runs if its reset has passed; otherwise, the existing five-hour `fallback_due` policy gates its registered fallback chain. A fallback must use an unrestricted registered account. The configured fallback model is passed to its CLI, and the audit stores both the requested and effective coordinator IDs and model. Before eligibility, or when no fallback is available, review returns an actionable error before creating review artifacts.

## Verification

`tests/repair.rs` covers automatic recovery from a failed task, existing integration repair, retaining failure without a coordinator, and excluding scope failures and availability waits. `tests/review_quota_fallback.rs` verifies the deadline gate, fallback model argv, audit identities, and no process before the deadline. The current native branch passes the unfiltered locked/offline all-target suite: 935 passed, 14 ignored, 308 suites; warning-denied Clippy and rustfmt also pass.

## Implementation

- `src/workflow/repair.rs`: eligibility predicate and bounded orchestration loop.
- `src/workflow/review/orchestration.rs`: frozen evidence and coordinator verdict handling.
- `src/workflow/assignment.rs`: registered fallback-chain resolution shared with manual review.
- `src/coordinator/run.rs`: timeout-bounded, read-only review process with optional policy-selected model.
- `src/workflow/recovery/resume.rs`: reset and resume behavior after a revise verdict.
