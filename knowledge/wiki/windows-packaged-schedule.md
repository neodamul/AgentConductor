# Windows packaged schedule interaction

## Acceptance condition

For a waiting Agent Room task, the exact Windows release package must persist the requested timezone-qualified timestamp as `manual_schedule`. Accessibility labels, input-delivery markers, a non-throwing UIA `Invoke`, and visible controls do not prove success. The report state is the acceptance oracle.

## Failure evidence and test-driver guard

Actions run `40241618513` built the Windows package and passed the packaged briefing and desktop-save journeys. The schedule button was visible and enabled, but input produced no `coordinator-ui-*` action and the report stayed `manual_required` without `available_at`.

The schedule driver now activates the exact package window and checks both the foreground window process and the process under the schedule button's center using `GetForegroundWindow`, `WindowFromPoint`, and `GetWindowThreadProcessId`. A mismatch stops the test with an explicit diagnostic before UIA invocation. The source contract failed before this guard and passes afterward. This local check does not establish Windows PowerShell compilation or package behavior.

## Current verification state

- The combined local schedule dispatch/marker, native interaction, source-layout, and public-doc batch passes 22 tests across six suites.
- Warning-denied all-target Clippy, the test unsafe-comment gate, formatting, diff checks, and the 300-line source/test limit pass.
- Exact Windows package scheduling remains unverified; keep workflow acceptance at 20/25 until the report persists `manual_schedule=2099-01-01T09:00:00+09:00`.
- The current macOS host has no `pwsh` executable. Do not treat source contracts as Windows runtime evidence.
