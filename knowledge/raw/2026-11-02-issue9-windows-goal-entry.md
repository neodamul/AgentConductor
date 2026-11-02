# Issue 9 Windows Goal Entry Failure

## Exact package evidence

Run `41396276498` passed macOS validation and package execution. Windows validation, release build, manual rendering, and package smoke passed. The packaged recovery, briefing, and candidate-review journeys passed. The three coordinator-chat worker-lifetime journeys were skipped because the hosted runner denied breakaway from its Job Object.

The packaged default-desktop UI journey failed at the Goal edit. UI Automation's ValuePattern.SetValue returned without updating the accessible value; the diagnostic reported a blank goal. The test asserted before killing its spawned app, and the package job was later cancelled at its timeout.

## Test-first repair

The source-contract tests failed against the old driver. Exact Windows run `41402275923` showed that keyboard input changed the UI Automation Goal value but did not persist it. Run `41405965619` also failed after the follow-up used Win32 `SetCursorPos`/`SendInput` to click the visible Save control. The current unverified attempt invokes the control through UI Automation `InvokePattern` first, with the checked pointer click as fallback. The driver still waits for `startup.json` only after successful activation and terminates the desktop process before assertions.

The 1,295-pass full local suite belongs to earlier source `76e0c39b`; it does not cover the current UIA Invoke or coordinator-port changes. Current coordinator-focused tests, source-layout check, and warning-denied all-target Clippy pass. Full local tests and exact packaged Windows validation remain pending.

## Remaining verification

The UIA Invoke source has not yet run on Windows. Do not treat either failed run as Windows package acceptance. Current Issue 9 acceptance remains 22/50 overall, with 22/25 packaged workflows and 0/25 shipped/default Rust UI.

## Coordinator profile and chat port — 2026-11-02

Merged main issues #325/#326 are being ported into the existing Rust #9 branch, not a child PR. New tests first failed, then passed for catalog-validated coordinator model selection, custom read-only argv, profile edit persistence, compatibility reasons, and blocked chat submission. The profile form exposes the model selector and readonly argv field. Chat now shows the coordinator's read-only incompatibility and disables sending. Focused tests and warning-denied all-target Clippy pass; full local verification and branch integration remain open.
