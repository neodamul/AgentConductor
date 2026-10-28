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

The exact Windows package run `40683626949` checked out head `829e58ee` but failed during Windows compilation before the package UI journey: two invalid boolean assertions wrapped `Repository::clean()`, which returns unit on success. The assertions are corrected to call `git.clean().unwrap()` directly. Focused source-contract/layout tests pass 4/4, formatting and diff checks pass, and the file is 97 lines. A new exact-head Windows run is required; workflow 24 and acceptance credit remain pending.

## Trace

Functional trace: R-07 → BG-03 → SF-03/04/05 → UC-04 → FR-09/NFR-01 → QAS-02/06 → DD-05/06. Package-verification context: R-24 → BG-10 → SF-20 → UC-17 → NFR-11 → QAS-18 → DDR-08 → DD-27. See [the implementation checklist](../../docs/07-implementation-plan.md) and [the Windows package workflow](../../.github/workflows/platform-validation.yml).
