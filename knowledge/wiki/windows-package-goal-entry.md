# Windows Packaged Goal Entry

## Failure mode

In the Windows packaged default-desktop journey, UI Automation located the Goal edit, focused it, and obtained `ValuePattern`, but `ValuePattern.SetValue` left the value empty. A test must exercise the normal keyboard path for this egui field rather than relying on programmatic value mutation.

The test also must kill and wait for its spawned desktop process before asserting on the UI driver result. An assertion before cleanup can leave a GUI process alive when a UI automation check fails.

## Verified repair

Send Ctrl+A and the ASCII goal through `System.Windows.Forms.SendKeys.SendWait`, then poll the UI Automation value for the entered text. Capture stdout/stderr, kill and wait for the app, and only then assert. The TDD source-contract suites passed locally after this change.

## Current limit

A passing macOS package does not establish Windows package acceptance. Re-run the exact packaged Windows journey on the repaired head. The prior run 41396276498 failed this default-desktop journey; its chat worker-lifetime checks were skipped because the hosted Windows runner denied Job Object breakaway.

## Source

See `knowledge/raw/2026-11-02-issue9-windows-goal-entry.md` and `docs/09-completion-audit.md`.
