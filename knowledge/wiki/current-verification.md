# Current local verification

## Workflow 20 source-level manual schedule interaction — 2026-10-24

Trace: R-18 → BG-07 → SF-09 → UC-09 → QAS-09 → DD-11. The isolated `native_ui_task_scheduling_interaction` regression reaches `waiting_availability` with the deterministic `quota-once` mock, enters an ISO-8601 timestamp with an explicit offset through the accessible text field, clicks Schedule retry through the accessible button bounds, dispatches the resulting action through the real Room dispatcher, and verifies persisted `available_at` plus `manual_schedule` provenance. It stops the watcher and removes the disposable fixture.

This is source-level interaction and persistence evidence only. It does not prove visual or behavioral parity in an exact package or the UTM guest, does not count toward the tracked 25 packaged UI workflows, and leaves aggregate acceptance at 20/25. The UI implementation and workflow protocol are unchanged. Exact command/results and graph refresh are in `docs/evidence/issue9-workflow20-schedule-interaction-20261024.json`. UTM was not operated; a later read-only desktop inspection showed the UTM app open with the Windows VM stopped. The user confirms Z: is accessible when Windows is running.

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
