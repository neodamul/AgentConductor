# Current local verification

## Current Windows/UTM status — 2026-10-24

The user confirms the existing `Z:` drive was accessible inside the Windows guest; host-side `utmctl` failing to read a logon-mapped drive did not mean the share was unavailable. After a previously approved VM reset, the user reports the VM no longer boots or displays. Earlier desktop and Z: observations are historical. No further UTM operations have been performed; guest UI verification is pending recovery.

Product source baseline: `3b367a28758248661a12d9897da6e72e25574900`. At that source commit, `cargo test --locked --offline --all-targets --features test-fixtures --no-fail-fast` completed with 989 passed, 14 ignored, 0 failed across 321 suites in 263.15 seconds. This host test evidence adds no packaged UI workflow acceptance: the count remains 20/25, and shipped/default Rust UI remains 0/25.

The repository-wide 300-line/source-test layout guard independently passes 2/2 at the same source commit. Tracked and working-tree scans found no authored `.py`, `.js`, `.jsx`, `.ts`, or `.tsx` source files. Evidence: `docs/evidence/issue9-source-layout-3b367a28-20261024.json`; this adds no packaged UI acceptance.

## Windows packaged default desktop acceptance

PR `9db304c4677ddf78ab971893171ef09f3a476239` extends the existing Windows UI Automation workflow to launch `agentconductor.exe` with no arguments from an isolated `HOME`, then save a workspace goal and verify its default startup file. In the Windows package job, `AGENTCONDUCTOR_DESKTOP_BINARY` points at the executable inside the package produced in that same job, so this check exercises the exact release candidate. Local macOS verification passed formatting, `git diff --check`, and the 300-line/source-test layout guard (2/2); `windows_desktop` runs zero tests on macOS because the test is Windows-only. Do not count this workflow until the Windows package job passes.

On exact head `23385cd02257f80ebfb68cbb635de7d3ed4521d3`, run `40083488603` passed macOS validation and Windows build/layout checks but failed the Windows all-target test only at `tests/windows_desktop.rs`: the UI Automation script invoked Save, but `startup.json` retained an empty goal. A first correction used `ValuePattern.SetValue`, but follow-up run `40085084340` still failed only in `windows_desktop`: PowerShell exited with no stdout/stderr while driving the UI. The current test activates the app window, uses ASCII `SendKeys`, isolates both HOME and USERPROFILE, and prints captured stdout on driver failure. A fresh exact-package Windows run is required; no packaged acceptance is claimed. Run `40086449655` then passed the Windows `windows_desktop` UIA test and all Windows validation checks, but macOS `cli_workflow` failed at fixture cleanup with `DirectoryNotEmpty`; the Windows package job was skipped. The local cleanup fix drops the Tokio runtime before removing its workspace tree; `cli_workflow` passed three consecutive local runs. A fresh exact-package workflow is required.
 Graphify refreshed the code map to 9,357 nodes / 15,780 edges; CocoIndex CLI is unavailable on this host.

The v0.1.37 arm64 package bundle has a valid `Info.plist`; `Contents/MacOS/agentconductor` exists, is executable, and is an arm64 Mach-O. A prior LaunchServices attempt returned `kLSNoExecutableErr`, and a temporary uniquely identified app copy aborted before exposing a UI window. The exact launch/window-binding failure remains unresolved; no package UI acceptance is claimed from those attempts.

Earlier, before the later approved VM reset, the UTM Windows guest was visibly booted to the desktop. File Explorer showed the mapped `Z:\AgentConductor` folder and its staged ZIPs; the user confirms Z: was accessible inside Windows. The exact-head Windows ZIP `agentconductor-x86_64-pc-windows-msvc-3b367a287582.zip` is present in the host share (SHA-256 `9d9b4ff65efb7a1a8aedd1da7bfdf4df2cebf93032768792830bcc157d7a7c49`). The prior statement that the guest did not boot or display is superseded. Workflow `40078615551` completed successfully for exact source `3b367a28758248661a12d9897da6e72e25574900`; both validation and package jobs passed on macOS and Windows, including Windows execution of the exact release binary in CI. Direct launch and visual interaction inside the UTM guest remain unverified: after rebinding CUA, input capture was initially off; enabling it still did not make clicks or keyboard input dismiss the Browse/Open dialog. No reset or reconfiguration had occurred at the time of this observation; a later approved reset was followed by the user reporting no boot/display.

Evidence: `docs/evidence/issue9-full-suite-3ae08e2-20261024.json`.

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
