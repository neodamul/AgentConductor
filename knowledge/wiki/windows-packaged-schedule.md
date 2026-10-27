# Windows packaged schedule interaction

## 2026-10-27 pointer fallback did not dispatch

Exact-package run [40496851499](https://github.com/neodamul/AgentConductor/actions/runs/40496851499) passed both platform validations, the macOS package, the Windows package build/smoke, and the packaged briefing/default-desktop journeys. The Windows schedule journey still failed: UIA button focus was refused, the foreground-/process-checked pointer fallback reported input sent, but the persisted run stayed `waiting_availability` / `manual_required` with no `available_at` and no coordinator UI action. Signal-settings was not reached. Windows artifact `12732733595`, SHA-256 `7fc2cc49b4204b29b679b87052fa6a293b6cb2f50e37e5126fa7916e12937689`; evidence: `docs/evidence/issue9-windows-schedule-40496851499-20261027.json`.

A test-first driver update now tries keyboard navigation from the schedule-time edit to the same-process enabled Schedule retry button and sends Enter only after confirming the focused target; the foreground-checked pointer remains the last fallback. Local focused tests pass, but this does not establish packaged Windows behavior. Keep acceptance at 22/25 until the exact package persists the requested schedule and then verifies the signal preference. AgentMemory: `mem_mvqn4p9k_43683bca7d5c`.

## 2026-10-27 checked fallback test driver

Commit `09383e9507807ef1669aacc06125216e6b79ce30` moves the long PowerShell interaction into `tests/support/windows_schedule_script.rs`, keeping all project-owned test/build sources in Rust. The driver tries UIA `Invoke` only when the Schedule button accepts keyboard focus; otherwise it sends a pointer click only after confirming the package window is foreground and the target point belongs to the package process. `windows_schedule_dispatch` asserts both paths and the test still accepts success only when the exact package persists the requested timestamp.

Local focused tests and warning-denied Clippy pass. The full local suite had one unrelated `cli_workflow` cleanup race (`DirectoryNotEmpty`); that test passed when rerun alone. Exact run [40496851499](https://github.com/neodamul/AgentConductor/actions/runs/40496851499) is still in platform validation; Windows is building and the exact package journey has not started. Do not claim schedule acceptance until the package report persists `manual_schedule=2099-01-01T09:00:00+09:00`. Signal settings remain unverified. AgentMemory: `mem_mvqlttn1_efca9941fe3b`, `mem_mvqm49ye_0ce781537be6`.

Run [40482504418](https://github.com/neodamul/AgentConductor/actions/runs/40482504418) passed Windows platform validation and the package's briefing/default-desktop journeys, then failed when the second UIA root `SetFocus()` threw `Target element cannot receive focus`. The driver had already confirmed the package HWND was foreground with Win32. Treat root UIA focus as best-effort after that native check: record an unavailable-focus diagnostic, reacquire the schedule control, and still require it enabled, on-window, and owned by the package process before dispatch. Local regression tests pass; exact package schedule persistence remains pending. Signal-settings was not reached. Evidence: `docs/evidence/issue9-windows-schedule-focus-40482504418-20261027.json`.

## 2026-10-27 refresh contention regression

Exact-head package run [40465150153](https://github.com/neodamul/AgentConductor/actions/runs/40465150153) passed the Windows package build, manual render, exact-binary execution, briefing, and default-desktop save. Scheduling failed after the valid timestamp was entered: the freshly reacquired Schedule retry button was disabled after focus, so neither dispatch nor persistence occurred; the signal-settings test did not run. The UI refresh loop treated `waiting_availability` as a one-second active state, while any pending refresh disables room controls. A test-first candidate now uses the existing five-second idle interval for this state. Focused tests pass, but the change is not accepted until a new exact Windows package persists both the manual schedule and signal preference. Acceptance remains 22/25. Failure artifact: `docs/evidence/issue9-windows-schedule-refresh-40465150153-20261027.json`.

## 2026-10-27 exact-package schedule persistence passed

Exact-head run [40431084361](https://github.com/neodamul/AgentConductor/actions/runs/40431084361) passed macOS/Windows platform validation and both exact-package jobs. The Windows package step `Package and execute the exact release binary` passed: its command ran `windows_briefing`, `windows_desktop`, and `windows_schedule`; the schedule suite passed 2/2 and the persistence oracle observed `manual_schedule=2099-01-01T09:00:00+09:00`. Distinct workflow 22, read-only team briefing, passed with all three tasks completed, approval pending, and report unchanged before/after. Windows artifact `12704291414`, SHA-256 `bb564126f63f712e8942cab00fc85127f91c657fb5c01a39ef8c7dec832bc146`; macOS artifact `12703725011`, SHA-256 `24ce1871651cbf42b7da25d6dd40aa8bc78a4162b373f00b302de649754f5272`. Packaged UI acceptance is 22/25, with three workflows remaining. AgentMemory: `mem_mvq5il1l_deb4cfec3101`. Evidence: `docs/evidence/issue9-windows-schedule-40431084361-20261027.json`.

## 2026-10-27 exact-package UIA dispatch failure

Exact-package run [40426253252](https://github.com/neodamul/AgentConductor/actions/runs/40426253252) passed platform validation, macOS package execution, and Windows build/smoke, desktop save, and briefing. UIA `InvokePattern.Invoke()` returned without error, but the task remained `waiting_availability` / `manual_required`, `available_at` was absent, and no `coordinator-ui-*` action directory appeared. Therefore no schedule persistence was proven and acceptance stays 20/25. Windows artifact `12701647324`, SHA-256 `d170dd5dd94508dbb2a48e090ae34b2749ef9824a0fdc893cfc130ca38c798c7`.

A test-first follow-up reasserts UIA window focus after Win32 foregrounding and logs post-invoke enabled, keyboard-focus, and offscreen state. The focused local suites pass, but this candidate still requires an exact packaged Windows run. AgentMemory: `mem_mvpzm7og_eff67d35c230`. Evidence: `docs/evidence/issue9-windows-schedule-40426253252-20261027.json`.

## 2026-10-27 exact-package UIA failure

Run [40415608903](https://github.com/neodamul/AgentConductor/actions/runs/40415608903) passed both platform validation jobs, both package builds and smoke runs, and the Windows default desktop save and briefing journeys. The exact packaged schedule action failed when `InvokePattern.Invoke()` raised `ElementNotEnabledException`; the required durable timestamp was not verified. After focus and scrolling, the test driver now reacquires the top-level window and schedule button from UI Automation, checks the fresh button's enabled state, and validates the process under its bounds before invoking. Local source-contract tests pass; a new exact-package Windows run is required. Keep acceptance at 20/25. AgentMemory: `mem_mvpwge8y_48466d3c942a`. Graphify refresh: 9,511 nodes / 15,983 edges. CocoIndex CLI is unavailable on this host. Evidence: `docs/evidence/issue9-windows-schedule-40415608903-20261027.json`.

## 2026-10-27 exact-package failure

Run [40400414629](https://github.com/neodamul/AgentConductor/actions/runs/40400414629) passed Windows platform validation, package build/smoke, desktop save, and briefing. The schedule test stopped before dispatch: button center `(884,867)` was below the window and `WindowFromPoint` resolved to PID `0`. The current local driver adds bounded inspector scrolling, a button/window geometry check, and packaged-process validation before clicking. The run predates these changes, so the revised driver still needs an exact Windows package run. Acceptance stays at 20/25 until the stored report contains `manual_schedule=2099-01-01T09:00:00+09:00`.

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
