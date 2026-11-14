# #992 — Verify planned TDD Red outcomes

## Plan and trace

- Requirement / Business goal: a test-first Goal must continue through Red and Green without accepting missing deliverables.
- System feature: explicit expected-failure verification, defaulting to success for existing plans.
- Use case UC-992: a reviewer approves a Red phase whose check exits nonzero; the next phase runs the same check and must exit zero. An unexpected Red pass blocks approval.
- Functional analysis: preserve the expectation in planning, task files, run evidence, review, approval and UI verdicts. CLI worker failure is never a successful Red test.
- QAS-992: intentional test failures pass only when marked; timeout, stop, signal termination and launch failure remain failures. Existing plan serialization and hashes remain compatible.
- DD-992: one optional boolean marker, `test_expect_failure`, and a shared native outcome predicate. Preserve actual exit codes and logs; never rewrite a failure to exit zero. Publish the computed verdict for UI readers.
- Scope: Task/schema/planner and worker instructions; integration, review and approval gates; public evidence and existing result renderers. Keep existing rejection of vacuous checks such as `git status`.

## Checklist

- [x] Plan: inspect callers and ownership; create the issue-linked branch from main.
- [x] Task / Red: reproduce a Red phase stopping before Green.
- [x] Task / Green: apply expected outcome consistently.
- [x] Verify (local): Red → human approval → merge → Green in shared and worktree modes; unexpected pass, infrastructure failure, legacy serialization and UI verdicts. Platform merge gate is tracked on PR #993.
- [ ] Completed: final checks, PR merge, issue closure, memory/wiki and branch cleanup.

## Targeted evidence

The baseline rejected the marker. After adding the marker alone, Red still stopped with `integration test failed: one`, and an unexpected pass incorrectly reached approval. With the shared predicate, both shared and worktree fixtures passed Red, exposed a passing native UI verdict, passed approval, merged, supplied verified evidence to next-phase planning, and ran the same command successfully in Green. Light/dark UI checks also confirm that nonzero expected failures render as passing, while zero-exit unexpected passes show failure and recovery. A missing executable is rejected by the existing launch gate rather than treated as a Red result.

The final caller audit also reproduced loss of the marker when the operator edited and saved the plan. The editor now preserves it and includes it in the reviewed task identity; the expanded task and plan detail show the expected-failure label. The actual UI save regression first lost the marker, then passed in both themes. The initial full local batch was stopped before design checks to incorporate this caller fix; only the subsequent complete batch is final evidence.

The full local batch found the superseded #928 prompt assertion (`must not run those tests`). Its replacement retains the existing prompt ordering and vacuous-check guards and explicitly requires the new marker, rejection of an unexpected pass, and Red/Green phase separation. This is a fixture-only correction; the production source remains frozen. The old platform run was canceled and a single corrected batch is required before merge.

## Final local results

The final production-source batch passed all four design cells, all 120 top-level UI scripts and every native target except the superseded prompt assertion. Its 993 successful-target tests plus the corrected prompt target's 3 tests give 996 passing checks; 17 opt-in checks remain ignored. Python checks passed 47/47, formatting and Clippy (warnings denied) passed, touched code stayed within 300 lines, and an exact local credential-value scan found no matches. No production change was made after that full run; the follow-up changes only the obsolete fixture and these verification records. Exact-head macOS/Windows CI must still pass before merge.
