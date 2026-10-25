## Latest Windows schedule dispatch retest — restore UIA window focus / 2026-10-25

Exact-head run [40170014908](https://github.com/neodamul/AgentConductor/actions/runs/40170014908) at `631390043d6499da9a7e634fca684e203a3c7f9f` passed all macOS validation but failed the Windows schedule test; packaging was skipped. Three input strategies emitted an input marker, but the saved report remained `waiting_availability` / `manual_required` and no coordinator action was created. Comparing the last passing package run [40119945464](https://github.com/neodamul/AgentConductor/actions/runs/40119945464) identified a difference: its UIA driver called `window.SetFocus()` after locating the controls. A test now requires that focus call before timestamp entry; it failed before restoring the call and passes locally afterward. Focused schedule/retry/layout suites pass 7 tests across 4 suites; formatting and diff checks pass; touched test files remain under 300 lines. Graphify refreshed to 9,451 nodes / 14,703 edges; CocoIndex CLI is unavailable. AgentMemory: `mem_mvnhc0uk_c92190c682a4`. The updated exact-head Windows rerun is pending. Acceptance remains 21/25. Evidence: `docs/evidence/issue9-windows-schedule-focus-40170014908-20261025.json`.

# Current local verification

## Windows schedule success-marker contract — 2026-10-25

The schedule UI test driver compared `SCHEDULE_CLICK_SENT`, but its PowerShell script emitted `SCHEDULE_ACTION_INPUT_SENT`. A small source regression test failed while that mismatch was restored and passed after aligning the expected marker. Local focused schedule, briefing, interaction, layout, marker tests and formatting pass. This does not prove Windows UI behavior. Run [40160742328](https://github.com/neodamul/AgentConductor/actions/runs/40160742328) was cancelled during the Windows build before Windows tests ran. Exact-head run [40161383352](https://github.com/neodamul/AgentConductor/actions/runs/40161383352) on `07b988cd3cdc2b285dc3d49e993f68ec576745ab` passed all macOS gates and Windows build/layout, but Windows `windows_desktop` failed after `SendKeys` saved `Windows UIA go`. Windows briefing and schedule (including the exact schedule action) passed; package validation was skipped. The driver now uses UIA `ValuePattern` and verifies the complete goal before Save; three local suites pass. Exact-head Windows re-run is pending; package count remains 21/25. Graphify: 9,433 nodes / 15,895 edges; CocoIndex CLI is unavailable. Graphify: 9,433 nodes / 15,895 edges. CocoIndex CLI is unavailable on this host.

## Prior exact-head Windows schedule validation — 2026-10-25

Run [40155931134](https://github.com/neodamul/AgentConductor/actions/runs/40155931134) confirmed the viewport is now valid: Schedule retry was at `748,703,99,20` and UIA's clickable point was `798,713`, inside the desktop Pane `0,0,1024,768`. The click still produced no coordinator action. The next driver revision sends an explicit no-distance mouse-move event and waits before down/up so eframe can process the pointer location. Persisted `manual_schedule` remains the acceptance condition; Windows package acceptance remains 21/25. AgentMemory: `mem_mvn96tb0_e5a517ef8287`; Graphify: 9,425 nodes / 14,681 edges / 702 communities. CocoIndex CLI is not installed on this host.

## Prior exact-package Windows schedule result — 2026-10-25

Platform Validation run `40119945464` passed on exact head `fc5a1f8b97a343a5e0b7f7a2f038cbee9062e5b0`. Both Windows and macOS validation jobs passed all-target tests, native UI checks, formatting, warning-denied Clippy, and the unsafe-block gate. The macOS native-package job passed release build, Rust-rendered bilingual manuals, exact package execution, and artifact upload.

The Windows native-package job built the release binary, rendered the manuals, and passed package execution. Its final desktop step set `AGENTCONDUCTOR_DESKTOP_BINARY` to the executable inside the package and ran `tests/windows_desktop.rs` plus `tests/windows_schedule.rs`: the default desktop save test passed 1/1; the schedule suite passed 2/2, including `exact_windows_package_schedules_a_waiting_agent_room_task`. That test confirmed the timezone-qualified timestamp was persisted with `manual_schedule` provenance. Windows artifact `12572455474` has uploaded SHA-256 `5536970a8cfca8fb6d7084cbc1ab6e85ba717c62d619dbfc519a6657110b6e0f`; macOS artifact `12571846892` has uploaded SHA-256 `89297cadced4be867d89aaf2b9cd88679ad30371879095c56de0ea8718653fb4`.

This exact-package rerun resolves the previous packaged schedule acceptance failure; that earlier failure was not reproduced, and no product-code fix is claimed. Packaged UI acceptance advances from 20/25 to 21/25. Evidence: `docs/evidence/issue9-windows-package-schedule-40119945464-20261025.json`. Graphify refreshed to 9,413 nodes / 15,872 edges. CocoIndex CLI is unavailable. UTM guest inspection remains separate; its current screen is locked and no guest files were changed.

## Prior exact-package Windows schedule failure — not reproduced

Platform Validation run `40112817061` at source `a829db4f9b6c81be54674bad10d64c64f3bce9d4` passed macOS validation/package and Windows validation. The Windows package built and `windows_desktop` passed, but `windows_schedule` failed after UI Automation invoked Schedule retry: the workflow stayed `waiting_availability` with `manual_required`, no scheduled time, and no `coordinator-ui-*` action directory. The failure was not reproduced on exact head `fc5a1f8b97a343a5e0b7f7a2f038cbee9062e5b0` in run `40119945464`; the package-level schedule test passed there and packaged acceptance advanced to 21/25. This earlier failure was reported in issue #9 and AgentMemory `mem_mvmoqxzr_2ab12248fd73`. Run: https://github.com/neodamul/AgentConductor/actions/runs/40112817061.

## Workflow 20 exact Windows package UI test — passed / 2026-10-25

The Windows native-package job launches the exact packaged executable with a deterministic quota-waiting run, finds the accessible schedule timestamp and retry controls with Windows UI Automation, submits an offset-qualified timestamp, and verifies persisted `manual_schedule` provenance. Exact-head run `40119945464` passed `windows_desktop` 1/1 and `windows_schedule` 2/2 with `AGENTCONDUCTOR_DESKTOP_BINARY` set to the package executable. The tracked aggregate advances from 20/25 to 21/25. The wrapper runs zero tests on macOS; GitHub Windows package Actions is the authoritative host UI check. Graphify: 9,413 nodes / 15,872 edges. CocoIndex CLI remains unavailable.

## Workflow 20 source-level manual schedule interaction — 2026-10-24

Trace: R-18 → BG-07 → SF-09 → UC-09 → QAS-09 → DD-11. The isolated `native_ui_task_scheduling_interaction` regression reaches `waiting_availability` with the deterministic `quota-once` mock, enters an ISO-8601 timestamp with an explicit offset through the accessible text field, clicks Schedule retry through the accessible button bounds, dispatches the resulting action through the real Room dispatcher, and verifies persisted `available_at` plus `manual_schedule` provenance. It stops the watcher and removes the disposable fixture.

This is source-level interaction and persistence evidence only. At this checkpoint it did not prove visual or behavioral parity in an exact package or the UTM guest, and did not count toward the tracked 25 packaged UI workflows; aggregate acceptance was 20/25. Run `40119945464` later supplied exact-package Windows evidence and raised the count to 21/25. The UI implementation and workflow protocol are unchanged. Exact command/results and graph refresh are in `docs/evidence/issue9-workflow20-schedule-interaction-20261024.json`. UTM inspection remains separate. The user confirms Z: is accessible when Windows is running.

At pushed head `fd42dd8d773bed1fc31129ad46316e3ef2a5bd91`, the complete locked/offline all-target suite with `test-fixtures` passed **990 tests, 14 ignored, 0 failures across 322 suites** in 262.49 seconds. AgentMemory record: `mem_mvmdit22_defe3d9ebf91`. This is source regression evidence and does not change the packaged UI acceptance count.

## Current Windows/UTM status — 2026-10-24

The user confirms the existing `Z:` drive is accessible inside the Windows guest when it is running; host-side `utmctl` failing to read a logon-mapped drive does not mean the share is unavailable. The latest read-only UTM window inspection showed the app open and the Windows VM stopped. A stopped VM has no guest display; this does not establish a boot failure. No VM start, reset, power, share, or configuration operation was performed. Direct guest UI verification remains pending.

Product source baseline: `3b367a28758248661a12d9897da6e72e25574900`. At that source commit, `cargo test --locked --offline --all-targets --features test-fixtures --no-fail-fast` completed with 989 passed, 14 ignored, 0 failed across 321 suites in 263.15 seconds. This host test evidence adds no packaged UI workflow acceptance: the count remains 20/25, and shipped/default Rust UI remains 0/25.

The repository-wide 300-line/source-test layout guard independently passes 2/2 at the same source commit. Tracked and working-tree scans found no authored `.py`, `.js`, `.jsx`, `.ts`, or `.tsx` source files. Evidence: `docs/evidence/issue9-source-layout-3b367a28-20261024.json`; this adds no packaged UI acceptance.

## Windows packaged default desktop acceptance

Exact PR head `95fe85dde121ba0fb1fdc1020c301e53baba103d` passed workflow `40089756382` on macOS and Windows. Both validation jobs passed native build, source/test layout, all-target tests, native UI checks, formatting, Clippy, and unsafe-comment gates.

The Windows native-package job built the exact release executable, rendered both manuals with the Rust binary, and ran `tests/windows_desktop.rs` with `AGENTCONDUCTOR_DESKTOP_BINARY` set to the executable inside that package. The test `default_native_workspace_is_accessible_and_saves_through_ui_automation` passed (1 passed, 0 failed); it saved the expected ASCII goal in the isolated default workspace startup file. Windows package archive SHA-256: `84b60521ae534a69349776763c2e1e9db0e14401f59f0f73cdff3b48d279bdc1`. Artifact `12560105056`, uploaded artifact SHA-256 `916871c50c4332f62002bbd7575a810aa0ee3d27aef4e14b4e3bbc8497db4582`.

The macOS package job also passed exact package execution and uploaded its package evidence. Archive SHA-256: `cef8d2001c1bf87188f4f86b2da96587b7a18ec4262386575bd8eb5857e0907e`. Artifact `12558817898`, uploaded artifact SHA-256 `bef12d7e6646c1589732aa4daaef6435133f923f7dff4ecc4046efa9f5add38d`.

This verifies the exact packaged Windows default-workspace save path, not direct visual interaction inside the user's UTM guest or full UI parity. The guest's mapped `Z:` share is accessible according to the user; the last read-only UTM window showed the VM stopped. No UTM power/configuration action was taken. Graphify was refreshed after this checkpoint (9,362 nodes, 15,785 edges). CocoIndex CLI is unavailable in this environment. Aggregate packaged UI acceptance is unchanged until this workflow is mapped to the tracked 25-workflow inventory. Issue #9 and PR #22 remain open.

## macOS native launch failure

Two separate package bundle IDs (the temporary Workflow20 bundle and the standard `dev.agentconductor.desktop` bundle) produced the same host crash: SIGABRT in `___RegisterApplication_block_invoke` during AppKit `NSApplication` registration, with LaunchServices `_LSCopyLocalDatabase` on a background thread. The application does not reach the eframe window callback. Static package checks pass (valid plist, executable path/name, mode 0755, arm64 Mach-O). The OS-level failure cause is still unknown; no product code or LaunchServices database was changed. See `docs/evidence/issue9-macos-ui-launch-crash-20261024.json`.

## Rust Workspace back/discard actions

R-33 / DD-39 titlebar behavior is now present in the Rust Workspace view. The header exposes Back to run only when a saved run is available and Discard changes while unsaved edits exist. Confirmation names the affected areas; confirming restores workspace fields and clears local profile/task drafts without sending save, plan, edit, or start requests. Back navigation reuses `open_run`. Source tests and the full native UI feature suite pass (967 passed, 14 ignored across 321 suites); CI-equivalent warning-denied Clippy, formatting, and the 300-line/source-test layout check pass. This is not packaged UI acceptance; current score remains unchanged. Evidence: `docs/evidence/issue9-r33-rust-header-20261024.json`.

An earlier user report said the Windows UTM guest could not boot and had no display. The later direct observation confirms the Windows desktop is visible and Explorer can browse the mapped Z: share. This does not prove the packaged application has run in the guest. CocoIndex is not installed on this host (`rtk cocoindex --version` returned no such file).

An older read-only check reported the VM as stopped and UTM absent from the running-app inventory. This state was superseded by the direct 2026-10-24 screenshot showing the Windows desktop. The new observation confirms the Z: folder and package filenames are visible, but does not validate Windows package execution.

## Native navigation pointer tests and current Windows display

The navigation test module now sends egui pointer events to the localized Keep editing, Discard unsaved changes, and Back to run controls, then checks cancellation/restoration or event routing. `native-ui-probe` library tests pass (47), source-layout guard passes (2), strict all-target Clippy and formatting pass. Test file remains under the 300-line limit. Evidence: `docs/evidence/issue9-navigation-pointer-tests-and-utm-20261024.json`. Source-level interaction evidence does not count as packaged UI acceptance.

## Full local validation at issue #9 head 3b367a28

`cargo test --locked --offline --all-targets --features test-fixtures --no-fail-fast` passes 989 tests with 14 ignored across 321 suites (263.15 seconds). Current-head workflow dispatch `40078615551` completed successfully for commit `3b367a28758248661a12d9897da6e72e25574900`; macOS/Windows validation and native package jobs all passed, including exact release binary execution in CI. The exact-head Windows ZIP was verified and staged in the accessible Z: share. The older `70db73e` ZIP was excluded. Direct UTM guest launch and visual interaction are still pending and are not inferred from CI smoke evidence.

## Exact macOS package at d89b3c37

The pushed source `d89b3c37bab80b65570bee358a78c6487f8f6e3f` builds as the v0.1.37 arm64 macOS package. `validate-package`, ZIP integrity, and smoke passed for shared/worktree × Git/folder (four configurations, three parallel tasks and three tests each; approved merge verified, unapproved merge refused). Binary SHA-256: `34f42d7cd406c6bbe234dce323a16068685bcff02841e4282dccb0663b712579`; archive SHA-256: `c6251f91411ee52043fa0e009a280b3e2802d14aaa248545128a288a89f990d9`. Exact package evidence: `docs/evidence/issue9-d89b3c37-macos-package-smoke-20261024.json`. No app window was opened, so this does not verify the titlebar interactions or increase UI acceptance. The older local 6ca19965 package and disposable smoke output were removed.

## Windows validation failures and diagnostic correction — issue #9 / 2026-10-25

Exact-head Actions run `40117648402` on `31663dcf8bd3d1051917250d1d1f8f965d819d2d` passed macOS validation and the Windows build/source-layout gates. Windows all-target tests failed in two places: `exact_windows_package_schedules_a_waiting_agent_room_task` left the task at `manual_required` with no scheduled time, and `settings_dialogs_remain_accessible_across_sequential_open_close` found the cache dialog still visible after closing.

The schedule test had a diagnostics gap: its PowerShell driver successfully returned the post-click accessibility snapshot, but the later persistence assertion discarded that stdout. A test now requires that snapshot to be included in schedule failure diagnostics. The settings journey now waits for the asynchronous cache preview to finish, then reacquires the close button bounds from the settled layout before clicking. Focused local verification passes 4 tests across the schedule, settings journey, and source-layout suites. The full local suite passes 973 tests with 14 ignored across 323 suites; formatting, all-target warning-denied Clippy, unsafe-block gate, and the 300-line source-layout guard pass. Graphify refreshed to 9,412 nodes / 15,871 edges. These are test/diagnostic changes; no product root cause is claimed yet and packaged UI acceptance remains 20/25.

Next verification: push the tested checkpoint and run exact-head Windows validation and package UI tests. The post-click UIA names from the rerun must guide any product fix. Evidence: `docs/evidence/issue9-windows-validation-40117648402-20261025.json`.
