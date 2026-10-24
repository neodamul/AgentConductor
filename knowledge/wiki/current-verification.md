# Current local verification

At issue #9 branch head `3ae08e2db6cbf0c02c74ac390fdbc9e3218ae889`, `cargo test --locked --offline` completed with 963 passed, 14 ignored, 0 failed across 321 test binaries in 245.25 seconds. This is host/default-feature test evidence only. It adds no packaged UI workflow acceptance: the count remains 20/25, and shipped/default Rust UI remains 0/25.

The v0.1.37 arm64 package bundle has a valid `Info.plist`; `Contents/MacOS/agentconductor` exists, is executable, and is an arm64 Mach-O. A prior LaunchServices attempt returned `kLSNoExecutableErr`, and a temporary uniquely identified app copy aborted before exposing a UI window. The exact launch/window-binding failure remains unresolved; no package UI acceptance is claimed from those attempts.

The user confirmed that the existing mapped Z: drive is accessible; direct inspection now shows `Z:\AgentConductor` open in Windows File Explorer. The older ZIP targets source commit `70db73e` and was excluded. Exact-head Windows ZIP `agentconductor-x86_64-pc-windows-msvc-3b367a287582.zip` is staged in the share (SHA-256 `9d9b4ff65efb7a1a8aedd1da7bfdf4df2cebf93032768792830bcc157d7a7c49`). Workflow `40078615551` completed successfully for exact head `3b367a28758248661a12d9897da6e72e25574900`; both validation and package jobs passed on macOS and Windows, including Windows execution of the exact release binary in CI. Direct launch and visual interaction inside the UTM guest remain unverified because current CUA input did not activate archive extraction or launch. The UTM VM was not reset or reconfigured.

Evidence: `docs/evidence/issue9-full-suite-3ae08e2-20261024.json`.

## macOS native launch failure

Two separate package bundle IDs (the temporary Workflow20 bundle and the standard `dev.agentconductor.desktop` bundle) produced the same host crash: SIGABRT in `___RegisterApplication_block_invoke` during AppKit `NSApplication` registration, with LaunchServices `_LSCopyLocalDatabase` on a background thread. The application does not reach the eframe window callback. Static package checks pass (valid plist, executable path/name, mode 0755, arm64 Mach-O). The OS-level failure cause is still unknown; no product code or LaunchServices database was changed. See `docs/evidence/issue9-macos-ui-launch-crash-20261024.json`.

## Rust Workspace back/discard actions

R-33 / DD-39 titlebar behavior is now present in the Rust Workspace view. The header exposes Back to run only when a saved run is available and Discard changes while unsaved edits exist. Confirmation names the affected areas; confirming restores workspace fields and clears local profile/task drafts without sending save, plan, edit, or start requests. Back navigation reuses `open_run`. Source tests and the full native UI feature suite pass (967 passed, 14 ignored across 321 suites); CI-equivalent warning-denied Clippy, formatting, and the 300-line/source-test layout check pass. This is not packaged UI acceptance; current score remains unchanged. Evidence: `docs/evidence/issue9-r33-rust-header-20261024.json`.

An earlier user report said the Windows UTM guest could not boot and had no display. The latest direct observation is newer: UTM is running and the Windows desktop is visible. CocoIndex is not installed on this host (`rtk cocoindex --version` returned no such file).

An older read-only check reported the VM as stopped and UTM absent from the running-app inventory. This state was superseded by the direct 2026-10-24 screenshot showing the Windows desktop. The new observation does not validate Windows package execution or Z: file contents.

## Native navigation pointer tests and current Windows display

The navigation test module now sends egui pointer events to the localized Keep editing, Discard unsaved changes, and Back to run controls, then checks cancellation/restoration or event routing. `native-ui-probe` library tests pass (47), source-layout guard passes (2), strict all-target Clippy and formatting pass. Test file remains under the 300-line limit. Evidence: `docs/evidence/issue9-navigation-pointer-tests-and-utm-20261024.json`. Source-level interaction evidence does not count as packaged UI acceptance.

## Full local validation at issue #9 head 3b367a28

`cargo test --locked --offline --all-targets --features test-fixtures --no-fail-fast` passes 989 tests with 14 ignored across 321 suites (263.15 seconds). Current-head workflow dispatch `40078615551` completed successfully for commit `3b367a28758248661a12d9897da6e72e25574900`; macOS/Windows validation and native package jobs all passed, including exact release binary execution in CI. The exact-head Windows ZIP was verified and staged in the accessible Z: share. The older `70db73e` ZIP was excluded. Direct UTM guest launch and visual interaction are still pending and are not inferred from CI smoke evidence.

## Exact macOS package at d89b3c37

The pushed source `d89b3c37bab80b65570bee358a78c6487f8f6e3f` builds as the v0.1.37 arm64 macOS package. `validate-package`, ZIP integrity, and smoke passed for shared/worktree × Git/folder (four configurations, three parallel tasks and three tests each; approved merge verified, unapproved merge refused). Binary SHA-256: `34f42d7cd406c6bbe234dce323a16068685bcff02841e4282dccb0663b712579`; archive SHA-256: `c6251f91411ee52043fa0e009a280b3e2802d14aaa248545128a288a89f990d9`. Exact package evidence: `docs/evidence/issue9-d89b3c37-macos-package-smoke-20261024.json`. No app window was opened, so this does not verify the titlebar interactions or increase UI acceptance. The older local 6ca19965 package and disposable smoke output were removed.
