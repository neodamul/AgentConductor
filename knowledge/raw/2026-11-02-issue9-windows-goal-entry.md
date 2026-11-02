# Issue 9 Windows Goal Entry Failure

## Exact package evidence

Run `41396276498` passed macOS validation and package execution. Windows validation, release build, manual rendering, and package smoke passed. The packaged recovery, briefing, and candidate-review journeys passed. The three coordinator-chat worker-lifetime journeys were skipped because the hosted runner denied breakaway from its Job Object.

The packaged default-desktop UI journey failed at the Goal edit. UI Automation's ValuePattern.SetValue returned without updating the accessible value; the diagnostic reported a blank goal. The test asserted before killing its spawned app, and the package job was later cancelled at its timeout.

## Test-first repair

The source-contract tests failed against the old driver. Exact Windows run `41402275923` showed that keyboard input changed the UI Automation Goal value but did not persist it. Run `41405965619` also failed after the follow-up used Win32 `SetCursorPos`/`SendInput` to click the visible Save control. The current unverified attempt invokes the control through UI Automation `InvokePattern` first, with the checked pointer click as fallback. The driver still waits for `startup.json` only after successful activation and terminates the desktop process before assertions.

The final local verification on 2026-11-02 passes the locked/offline all-target suite, warning-denied all-target Clippy, rustfmt, diff hygiene, and source-layout guard on the main-integrated source. These macOS checks do not exercise Windows UI Automation at runtime.

## Remaining verification

The UIA Invoke source has not yet run on Windows. Do not treat either failed run as Windows package acceptance. Current Issue 9 acceptance remains 22/50 overall, with 22/25 packaged workflows and 0/25 shipped/default Rust UI.

## Coordinator profile and chat port — 2026-11-02

Merged main issues #325/#326 are integrated into the existing Rust #9 branch, not a child PR. Tests cover catalog-validated coordinator model selection, custom read-only argv, profile edit persistence, compatibility reasons, and blocked chat submission. The profile form exposes the model selector and read-only argv field. Chat shows the coordinator's read-only incompatibility and disables sending. The full local all-target suite, warning-denied Clippy, rustfmt, diff hygiene, and source-layout guard pass; exact Windows packaged acceptance remains open.
