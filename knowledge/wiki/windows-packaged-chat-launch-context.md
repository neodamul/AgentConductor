# Windows packaged chat launch context

## Observed boundary

The exact Windows package ran in GitHub Actions run `41360959648`. All three
coordinator-chat UI journeys failed even when run sequentially. The package log
reported that Windows denied `CREATE_BREAKAWAY_FROM_JOB` because the app was
inside the host Job Object. The product refused to start a detached chat worker
because it could not guarantee that the worker would survive app closure.

This is a test launch-context problem, not evidence that the product safety
check should be removed. `--test-threads=1` remains necessary because the tests
share one desktop, but it does not remove Job Object membership.

## Harness change

`tests/support/windows_chat_launcher.rs` creates a scheduled task for the
current interactive user and starts the exact packaged executable from that
task. UI Automation then targets the returned process ID. Cleanup terminates
only the desktop app process so the coordinator worker can outlive it in the
resume scenarios; the task is removed after the app exits.

## Verification status

The portable workflow contract and rustfmt pass locally. GitHub must confirm
that its Windows runner permits the interactive task to start in the UI
session, outside the Cargo runner job. The exact package tests must pass before
Windows chat acceptance is counted. If Task Scheduler cannot provide that
context, record the environment limitation and run the same exact package on a
real interactive Windows host; do not weaken production breakaway safeguards.
