# Windows packaged chat test serialization

## Failure

Run `41356140816` passed both platform validation jobs and the exact macOS
package, but the Windows `Verify the packaged Windows default desktop` step
failed all three tests in `windows_coordinator_chat`. The failures included a
chat job never reaching running state and UI Automation reporting
`CHAT_TRANSCRIPT_INCOMPLETE`. The three tests run in one Rust test binary and
each controls the same interactive Windows desktop.

## Cause and correction

Cargo's default test-thread pool ran the Windows UI Automation tests
concurrently. They competed for desktop focus and typed into the wrong app
window. Keep these package tests in the existing integration test binary and
run that package UI test invocation with `--test-threads=1`. Do not serialize
unrelated platform validation tests or change the product UI to accommodate a
test-harness race.

`tests/windows_package_workflow.rs` locks this workflow contract. Its test
failed before the package command was serialized, then passed after adding the
single-thread flag.

## Verification boundary

The full local Rust suite passed, as did the workflow-contract, source-layout,
and public-doc suites (14 focused tests), warning-denied all-target Clippy,
rustfmt, and `git diff --check`. The Windows package run must pass again on an
exact pushed head before counting package acceptance. Local contract coverage
does not substitute for Windows UI Automation evidence.
