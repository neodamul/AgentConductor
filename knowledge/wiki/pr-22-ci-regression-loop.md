# PR #22 CI Regression Loop

## Context

On 2026-11-02, Platform Validation run `41337149176` exposed several regressions on the root Rust migration branch. They were not a single platform failure: the run combined stale contracts from the removed JavaScript UI, locale-sensitive tests, missing localized copy, and two real gaps in the native run snapshot.

## Root causes and fixes

- The native snapshot response omitted `runner`, `stopping`, and `verification`, although the run screen and regression tests require them.
- The scheduler incremented `verification_attempt` in memory but did not persist it before launching integration tests. A concurrent read-only UI snapshot therefore could not name the active test.
- The snapshot test repeatedly opened the same Agent Room and released/reacquired its exclusive lease. It now holds one room for the observation and uses an impossible PID for deterministic absent-process behavior.
- Metadata tests asserted Korean copy regardless of the machine locale. They now assert the stable status suffix.
- Current manual and README screenshot contracts still referred to deleted/unsupported UI images; the Markdown and rendered bilingual HTML were aligned with the Rust screenshots.
- The Windows desktop test used UIA `ValuePattern.SetValue`, which does not verify keyboard entry. It now types Unicode through Win32 `SendInput` and verifies the UIA readback.
- `task_profile_layout` only inspected the deleted JavaScript `startup.css`; it was removed instead of preserving an obsolete implementation contract.

## Verification and limits

Commit `6d1762ca` is pushed to the existing PR #22. Focused regression suites, formatting, and diff hygiene pass locally. The all-target local run was interrupted and is not evidence of a pass. Exact-head Platform Validation run `41343162485` is still in progress; Windows package execution and screen-by-screen UI acceptance remain unverified.

## Process lesson

Keep issue #9 consolidated in its existing root PR, but close finite validation failures as small commits and checkpoint the PR/plan after each. Do not leave the PR in draft while unrelated work expands its scope, and do not describe interrupted or source-only checks as package acceptance.
