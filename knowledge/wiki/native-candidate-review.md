# Packaged Rust candidate review

## Purpose

Workflow 24 of issue #9 verifies that an operator can inspect the exact candidate evidence and make an explicit approve, reject, and merge decision in the packaged Windows Rust UI.

## Verification contract

- The review screen shows the candidate SHA, the actual `+implemented` diff line, and passing test evidence.
- Approve and merge require separate confirmations and remain bound to the reviewed candidate.
- A rejected candidate leaves the repository HEAD at its original base.
- A merged candidate moves HEAD to the reviewed SHA and leaves the repository clean.

## Current implementation

`tests/windows_candidate_review.rs` launches the release executable selected by `AGENTCONDUCTOR_DESKTOP_BINARY` against disposable Git worktree fixtures. The embedded PowerShell UI Automation driver reads accessible UI text through ValuePattern and TextPattern, checks the exact added diff line, and invokes named controls belonging to the packaged app process. `platform-validation.yml` includes the test in the exact Windows package run.

The exact Windows package run `40683626949` checked out head `829e58ee` but failed during Windows compilation before the package UI journey: two invalid boolean assertions wrapped `Repository::clean()`, which returns unit on success. That compile error was fixed and run `40684818525` passed macOS and Windows source validation plus the macOS package. Its packaged Windows UI driver clicked approve, merge, and confirmation, but the Rust test then observed report state `approved` instead of `completed`. The driver's `Wait-Text 'Completed'` was not authoritative because completed task rows already contain that word; it let the test close the app before the asynchronously persisted run merge finished. The Rust test now keeps the packaged app alive and polls `report.json` for the expected final decision state (20-second bound). Local source-layout and Windows desktop-driver tests pass 4/4, rustfmt and diff checks pass, and the changed candidate test is 112 lines. A new exact-head Windows package run is required; workflow 24 and acceptance credit remain pending.

Run `40692521680` then passed macOS/Windows source validation and the exact macOS package, but the Windows package review journey timed out waiting for the candidate SHA. The captured UIA tree still showed `Waiting approval` and the `Review candidate` button, with no SHA or diff evidence. The driver cached the first top-level `AutomationElement` and reused its descendants after invoking the review action. A test-first source contract now requires the driver to reacquire the app window for each descendant enumeration; the local contract/layout suites pass 5/5 with rustfmt and diff checks. Evidence: `docs/evidence/issue9-windows-candidate-review-stale-uia-tree-40692521680-20261028.json`. This is a test-driver correction, not proof that the exact package review journey passes; keep workflow 24 uncredited until the Windows package proves both merge and rejection paths and their Git/report assertions.

## Trace

Functional trace: R-07 → BG-03 → SF-03/04/05 → UC-04 → FR-09/NFR-01 → QAS-02/06 → DD-05/06. Package-verification context: R-24 → BG-10 → SF-20 → UC-17 → NFR-11 → QAS-18 → DDR-08 → DD-27. See [the implementation checklist](../../docs/07-implementation-plan.md) and [the Windows package workflow](../../.github/workflows/platform-validation.yml).
