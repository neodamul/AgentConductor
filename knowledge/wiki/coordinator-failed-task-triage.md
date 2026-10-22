# Coordinator triage for failed tasks

**Architecture trace:** R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173 (GitHub #135).

## Behavior

The existing coordinator repair loop can review a failed run when at least one task is failed. It does not triage out-of-scope changes or a run with a task waiting for account availability. The coordinator receives the retained task status, error, stdout, and stderr evidence through the existing review path. A `revise` verdict resets the affected task and dependents, then resumes the run. Missing coordinator configuration, a blocked or unusable verdict, and three exhausted review rounds leave the run for a person.

This is deliberately the same bounded review loop used for failed integration checks. It does not add an orchestrator, process, API key, or new retry budget. Reassignment of quota-waiting work is a follow-up under #135.

## Verification

`tests/repair.rs` covers automatic recovery from a failed task, existing integration repair, retaining failure without a coordinator, and excluding scope failures and availability waits. Run `cargo test --locked --test repair` for this contract.

## Implementation

- `src/workflow/repair.rs`: eligibility predicate and bounded orchestration loop.
- `src/workflow/review/orchestration.rs`: frozen evidence and coordinator verdict handling.
- `src/workflow/recovery/resume.rs`: reset and resume behavior after a revise verdict.
