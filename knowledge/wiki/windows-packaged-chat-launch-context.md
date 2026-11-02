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

## Harness limitation and reporting

`tests/support/windows_chat_launcher.rs` creates a scheduled task for the
current interactive user and starts the exact packaged executable from that
task. UI Automation targets the returned process ID. The scheduled task did not
escape the runner Job Object: exact package run `41373985619` reproduced the
same access-denied breakaway message in all three coordinator-chat lifecycle
journeys. Do not describe this launcher as being outside the runner job.

The package tests now skip only when the UI exposes all three exact markers:
Windows denied process breakaway, OS error 5, and `this app is inside a host
job`. The workflow writes each skip to its Step Summary and explains that chat
worker survival was not verified. Unknown or incomplete errors still fail the
tests. Product breakaway safeguards are unchanged.

Local contract tests (8/8), focused Clippy, rustfmt, diff hygiene, and the
300-line limit pass. The exact package chat lifecycle journeys remain
unverified until run on an interactive Windows host outside the restrictive
GitHub runner Job Object. Do not count a documented CI skip as package
acceptance; do not weaken production breakaway safeguards.
