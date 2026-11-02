# Windows Packaged Goal Entry

## Failure mode

In the Windows packaged default-desktop journey, UI Automation located the Goal edit, focused it, and obtained `ValuePattern`, but `ValuePattern.SetValue` left the value empty. A test must exercise the normal keyboard path for this egui field rather than relying on programmatic value mutation.

The test also must kill and wait for its spawned desktop process before asserting on the UI driver result. An assertion before cleanup can leave a GUI process alive when a UI automation check fails.

## Verified repair

Send Ctrl+A and the ASCII goal through `System.Windows.Forms.SendKeys.SendWait`, then poll the UI Automation value for the entered text. On successful driver exit, keep the app alive until `startup.json` confirms the saved goal; on driver failure, skip that wait. In both cases, kill and wait for the app before asserting so the save is not lost and failed tests do not orphan a GUI process. The TDD source-contract suites pass locally.

## Current limit

A passing macOS package does not establish Windows package acceptance. Re-run the exact packaged Windows journey on the repaired head. The prior run 41396276498 failed this default-desktop journey; its chat worker-lifetime checks were skipped because the hosted Windows runner denied Job Object breakaway.

## Save activation follow-up — 2026-11-02

Exact Windows run `41402275923` showed SendKeys changed the accessible Goal value but did not persist it. Run `41405965619` also failed after the driver clicked Save through Win32 `SetCursorPos`/`SendInput`. Current source tries UI Automation `InvokePattern` before the checked pointer fallback; this has not yet been verified on Windows. The final local verification passes the locked/offline all-target suite, warning-denied all-target Clippy, rustfmt, diff hygiene, and source-layout guard. Those macOS checks do not exercise Windows UI Automation at runtime.

## Coordinator profile and chat port — 2026-11-02

The #325/#326 Rust port is integrated in the existing #9 branch. Agent profile editing exposes an observed-catalog coordinator model and separate custom read-only argv. Invalid compatibility fails closed; the chat panel displays the reason and remediation and disables sending. Mainline snapshot blockers and actionable refusal messages are included. Full local verification passes; exact Windows packaged acceptance remains pending.

## Source

See `knowledge/raw/2026-11-02-issue9-windows-goal-entry.md` and `docs/09-completion-audit.md`.
