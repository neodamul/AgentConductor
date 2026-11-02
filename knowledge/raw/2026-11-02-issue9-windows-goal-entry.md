# Issue 9 Windows Goal Entry Failure

## Exact package evidence

Run `41396276498` passed macOS validation and package execution. Windows validation, release build, manual rendering, and package smoke passed. The packaged recovery, briefing, and candidate-review journeys passed. The three coordinator-chat worker-lifetime journeys were skipped because the hosted runner denied breakaway from its Job Object.

The packaged default-desktop UI journey failed at the Goal edit. UI Automation's ValuePattern.SetValue returned without updating the accessible value; the diagnostic reported a blank goal. The test asserted before killing its spawned app, and the package job was later cancelled at its timeout.

## Test-first repair

The source-contract tests were changed first and failed against the old driver. The driver now focuses the Goal edit, sends Ctrl+A and the goal with Windows Forms SendKeys, waits for and reads back the UI Automation ValuePattern value, and kills/waits for the desktop process before asserting on driver status.

Local verification on the repaired source passed 29 tests with 1 ignored across four suites, warning-denied all-target Clippy, rustfmt, diff hygiene, and the 300-line layout guard.

## Remaining verification

The repaired Windows package journey has not yet run on an exact new source head. Do not treat the failed run as Windows package acceptance. Current Issue 9 acceptance remains 22/50 overall, with 22/25 packaged workflows and 0/25 shipped/default Rust UI.
