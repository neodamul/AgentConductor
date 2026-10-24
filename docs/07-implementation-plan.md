### Fix the Windows packaged default UI automation failure — issue #9 / 2026-10-24

- [x] Red: exact-head workflow `40083488603` passed macOS validation and Windows build/layout checks, then failed only in `tests/windows_desktop.rs`; UI Automation invoked Save but the saved goal remained empty.
- [x] Diagnose: the original non-ASCII `SendKeys` run invoked Save but left the persisted goal empty. A `ValuePattern.SetValue` retry exited the PowerShell UIA driver with no stdout/stderr, so that pattern is not relied on.
- [x] Task: activate the app window before setting focus, use an ASCII goal with `SendKeys`, isolate both `HOME` and `USERPROFILE`, and include driver stdout in failure diagnostics.
- [x] Local check: formatting, `git diff --check`, and the 300-line bound pass.
- [ ] Verify: rerun the packaged Windows acceptance on the updated exact commit; the macOS host cannot execute this Windows-only test. Do not count the package workflow until the exact binary passes UI Automation.

### Latest Windows/UTM status — issue #9 / 2026-10-24

- [x] User clarification: Windows could access the existing `Z:` drive. Host-side `utmctl` not reading the guest's mapped drive was not evidence that the share was unavailable.
- [x] Current status: after the previously approved VM reset, the user reports Windows does not boot or display. The earlier desktop screenshot and Z: browsing are pre-reset evidence, not current state.
- [x] Safety: no further UTM operations have been performed since the user reported the boot/display failure; direct package UI verification remains pending.

### Run Windows UI Automation against the exact release package — issue #9 / 2026-10-24

- [x] Plan: extend the existing isolated Windows Workspace UI Automation check to launch the default no-argument app from a disposable `HOME`.
- [x] Task: make the test accept an explicit binary path and pass the exact packaged `dist/agentconductor-x86_64-pc-windows-msvc-<commit>/agentconductor.exe` from the Windows package job.
- [ ] Verify: run the UI Automation save workflow against that package binary; require the saved goal to appear under the isolated default workspace root.
- [x] Verify locally: formatting, `git diff --check`, and `source_layout` pass (2/2). The focused `windows_desktop` target is compiled with zero tests on macOS due to `#![cfg(windows)]`; Windows UI Automation remains pending.
- [ ] Complete: count one packaged/default workflow only after the Windows UI Automation check passes; retain issue #9 and PR #22 as open until all remaining gates pass.

### Reconcile live UTM Windows display and Z: access — issue #9 / 2026-10-24

- [x] Verify: the UTM window currently shows a booted Windows desktop and File Explorer at `Z:\AgentConductor`; the staged Windows ZIPs are visible.
- [x] Verify host-side share identity: the exact-head package ZIP `agentconductor-x86_64-pc-windows-msvc-3b367a287582.zip` is present and its SHA-256 is `9d9b4ff65efb7a1a8aedd1da7bfdf4df2cebf93032768792830bcc157d7a7c49`.
- [x] Correct stale status: the earlier report that the guest did not boot or display has been superseded by this live screenshot. Z: accessibility is confirmed.
- [ ] Verify: launch the packaged binary and inspect its UI in Windows. After rebinding UTM, input capture was off; enabling it still did not make guest clicks or keyboard input dismiss the Browse/Open dialog. Share visibility and CI smoke do not count as guest UI acceptance.
- [x] Safety: no VM reset, power action, or configuration change was made.

### Confirm the existing UTM Z: share and stage current-head Windows package — issue #9 / 2026-10-24

- [x] Plan: use the already-mapped Z: drive; do not change UTM settings or VM power state.
- [x] Task: open `Z:\AgentConductor` in Windows File Explorer. The older ZIP targets `70db73e` and is excluded; the exact-head ZIP is also visible in the share.
- [x] Verify: workflow `40078615551` completed successfully for exact head `3b367a28758248661a12d9897da6e72e25574900`; macOS/Windows validation and native package jobs all passed, including execution of the exact release binary in CI.
- [x] Verify: inspect the exact-head Windows package manifest and checksum; stage only `agentconductor-x86_64-pc-windows-msvc-3b367a287582.zip` in the existing Z: share. ZIP SHA-256 `9d9b4ff65efb7a1a8aedd1da7bfdf4df2cebf93032768792830bcc157d7a7c49`; executable SHA-256 `c02438f3639b5a6eb85211f0276cf15c3bfda3fad4977260d337c4b2fe40e5f4`.
- [ ] Verify: direct launch and visual interaction in the UTM Windows guest. The staged package is visible, but current CUA input did not activate Explorer extraction/launch. A Windows file-picker dialog remains open after Escape/close attempts; do not count CI execution as guest UI acceptance.
- [ ] Complete: issue #9 remains open until remaining native UI acceptance gates pass. UTM configuration was unchanged at the time of this staging check; a later approved VM reset superseded that power-state observation.

### Fix macOS release bundle localization metadata — issue #9 / 2026-10-24

- [x] Plan: map the LaunchServices bundle-registration bug to R-24 → BG-10 → SF-20 → UC-17 → NFR-11/QAS-18 → DDR-08/DD-27.
- [x] Red: extend `tests/macos_bundle.rs` to require the English development region, `en`/`ko` localizations, and corresponding `InfoPlist.strings`; tests exposed missing plist metadata and then missing localized resources.
- [x] Task / Green: generate the metadata and both localized strings files in the Rust macOS package assembler.
- [x] Verify: `macos_bundle` and `release_package` suites pass 6/6; release executable builds offline; a disposable app bundle and all three plist-format files pass `plutil -lint`; `cargo fmt --check --all` and `git diff --check` pass.
- [x] Record evidence and LLM Wiki entry; save the launch-isolation and UTM safety findings to AgentMemory.
- [x] Verify isolated runtime: a source-built bundle with a unique test bundle ID, disposable `HOME`, and no CLI profiles launched by the default no-argument entrypoint; CUA showed the empty Workspace screen and “Register an agent to get started.” The test app was stopped and its entire temporary root removed.
- [x] Boundary: this proves the current Rust default startup renders from a packaged app bundle without loading the user's profiles; no controls were used and no quota CLI was configured. The test bundle had a temporary unique identifier, so this is not acceptance of the production bundle identity or a packaged user workflow.
- [x] Commit/push code and test fix as `99cb31aa`.
- [ ] Remaining: verify the production bundle identity without binding to the older app entry; Windows runtime/UI acceptance is blocked while the UTM guest does not boot or display. Do not touch UTM.

### Diagnose exact-package macOS UI launch abort — issue #9 / 2026-10-24

- [x] Plan: inspect the two existing crash reports from separate package-bundle launch attempts; do not launch another window or modify host services.
- [x] Task: verify both reports are arm64 v0.1.37 and terminate with `SIGABRT` in `___RegisterApplication_block_invoke` during AppKit `NSApplication` registration; both show LaunchServices `_LSCopyLocalDatabase` on a background thread.
- [x] Verify: package `Info.plist` parses, names `agentconductor`, and the executable exists with mode `0755` as arm64 Mach-O. The crash occurs before the eframe app callback/window rendering.
- [x] Assetize: record exact bundle IDs, stack frames, scope, and unresolved cause in `docs/evidence/issue9-macos-ui-launch-crash-20261024.json` and `knowledge/wiki/current-verification.md`.
- [x] Decision: no product-code change is justified by these host-level crash reports; no LaunchServices database repair or reset was attempted.
- [ ] Remaining: verify the packaged workflow on a macOS host that successfully registers native apps; resolve remaining UI workflow acceptance and ship integration. Windows UTM guest still does not boot or display and remains untouched.

### Full locked/offline verification — issue #9 / 2026-10-24

- [x] Plan: run the current complete package test set locally at clean head `3ae08e2`; avoid slow remote CI for this non-release checkpoint.
- [x] Task: `cargo test --locked --offline` completed at the existing default host feature configuration.
- [x] Verify: **963 passed, 14 ignored, 0 failed across 321 test binaries**; exit code 0; 245.25 seconds.
- [x] Assetize: record exact command, commit, scope, and remaining boundaries in `docs/evidence/issue9-full-suite-3ae08e2-20261024.json` and `knowledge/wiki/current-verification.md`.
- [x] Scope: this confirms the local Rust test suite only. Packaged UI acceptance remains **20/25 (80%)** and shipped/default Rust UI remains **0/25**.
- [ ] Remaining: fix reliable macOS package launch/window binding for workflow 20; Windows UTM guest currently does not boot or display, so Windows UI/runtime remains unverified; finish remaining UI/platform, accessibility/IME, resource, license, and release gates.

### Exact-package manual scheduling attempt — workflow 20 / 2026-10-24

- [x] Plan: use the existing v0.1.37 macOS arm64 package from source `6ca19965`, a disposable Git repository, and the Rust `quota-once` fixture; only schedule a retry far in the future if the exact Agent Room is visible.
- [x] Task: the package CLI created a one-task run that reached `waiting_availability` with `manual_required`; the fixture used no company/provider CLI.
- [x] Verify source: `native_ui_task_scheduling` passes 3/3, including timestamp-with-timezone validation and persistence through the Room dispatcher.
- [x] Safety/cleanup: launching `ui <fixture-run>` returned an AgentConductor window, but CUA bound by display name to a Workspace showing preconfigured profiles and saved workspace data instead of the requested Agent Room. No UI control, schedule, setting, workspace, approval, or merge was changed. The run was stopped and all files under `/private/tmp/ac-issue9-workflow20-1d07b5d0` were removed. The app inventory shows no running AgentConductor instance.
- [x] Evidence: `docs/evidence/issue9-workflow20-package-ui-attempt-20261024.json`. The displayed Workspace exposed quota refresh timestamps; whether automatic profile quota queries ran is unverified.
- [x] Scope: CLI and source tests do not count as packaged UI acceptance; keep **20/25 (80%)**, shipped/default Rust UI **0/25**.
- [x] Follow-up isolation attempt: a temporary package copy with a unique bundle ID was launched directly; the native UI process aborted with signal 6 (exit 134) before CUA exposed a window. The mock run was stopped, the temporary copy/root removed, and the app inventory shows no running AgentConductor process. No UI action occurred.
- [ ] Remaining: resolve a reliable package launch/window-binding path before retrying Workflow 20. Windows remains unverified; do not touch UTM while its boot/display is unavailable.

### Verify native manual text exposure in AccessKit — issue #9 / 2026-10-24

- [x] Plan: validate the existing manual headings, paragraphs, command-table cells, code and image alternative text through the native accessibility tree; do not claim screen-reader support from rendered pixels alone.
- [x] Red/diagnosis: the first assertion looked only for node labels and failed. Inspecting the AccessKit update showed that ordinary manual text is represented by `TextRun.value`, while the screenshot has an accessible image name.
- [x] Verify: the corrected regression passes against the unchanged renderer. Manual-related tests pass 23/23 across seven suites; `public_docs` and `source_layout` pass 13/13; `cargo fmt --check --all` and `git diff --check` pass. Changed test is 68 lines; manual renderer is 270 lines.
- [x] Assetize: record the native-package observation, AccessKit representation, test result and verification limits in the evidence file and offline-manual Wiki.
- [x] Scope: this is source-level accessibility evidence only; packaged UI acceptance remains **20/25 (80%)** and shipped/default Rust UI remains **0/25**.
- [ ] Remaining: verify actual macOS VoiceOver and Windows Narrator behavior; the macOS CUA tree output did not expose the manual's TextRun values as visible child labels.

### Exact-package task activity, raw logs, and lessons — workflow 21 / 2026-10-24

- [x] Plan: reuse R-104 → BG-59 → SF-91 → UC-94 → FR-89 → QAS-100 → DD-112; keep normalized Activity separate from raw JSONL and display reviewed task lessons.
- [x] Package UI: exact v0.1.37 macOS arm64 package displayed task activity, JSONL stdout/stderr in Raw logs, and the task-specific cause/action in Lessons. The fixture remained at `waiting_approval`; no approval, reassignment, feedback, or merge was sent.
- [x] Verify: `native_ui_task_activity`, `native_step_lessons`, and `native_ui_task_activity_layout` passed 9/9 tests. The exact package binary SHA-256 matches the workflow 19 package evidence.
- [x] Assetize: record observations, package identity, fixture scope, and limitations in `docs/evidence/issue9-workflow21-task-activity-20261024.json` and the LLM Wiki.
- [x] Acceptance: distinct packaged UI workflows are **20/25 (80%)**; shipped/default Rust UI remains **0/25**.
- [ ] Remaining: Windows guest boot/display and package execution, Korean IME/accessibility, retained screenshots, full-suite completion, shipped/default integration, idle-resource measurements, final license review, and stable release.

### Revalidate the repository-wide 300-line source/test bound — issue #9 / 2026-10-24

- [x] Verify product source `3b367a28758248661a12d9897da6e72e25574900` with `cargo test --locked --offline --test source_layout -- --nocapture`: both tests pass, including the recursive 300-line and source/test separation guard over `src/`, `tests/`, `scripts/`, `examples/`, and `build.rs`.
- [x] Scan tracked and working-tree files for authored `.py`, `.js`, `.jsx`, `.ts`, and `.tsx` sources; none remain. The older Python/JavaScript counts below are historical. Evidence: `docs/evidence/issue9-source-layout-3b367a28-20261024.json`. This check adds no packaged UI acceptance credit.
- [ ] Remaining #9 acceptance: exact-source UI workflows, shipped/default Rust UI, Windows guest/runtime and accessibility/IME, resource measurements, final license review, and stable release.

### Keep Agent Room controls responsive during task execution — issue #9 / QAS-158 / 2026-10-24

- [x] Plan: trace the disabled live-task feedback and pending-task assignment/model controls to R-162, QAS-158 and DD-172.
- [x] Red: reproduce a live snapshot exceeding its 2-second bound and, in the exact package, a disabled running-task feedback editor; regressions fail before their fixes.
- [x] Task: derive the availability watcher from the loaded report instead of waiting on coordinator-lock reconciliation for every snapshot; use read-only SQLite snapshots and enable feedback editing only for the running task.
- [x] Verify: 25 `gates`/`ui`/`source_layout`/`public_docs` tests, availability regression (1), native task controls and probe (17), AccessKit feedback-editor regression (1), all-target Clippy, formatting, and the 300-line source/test limit pass locally.
- [x] Assetize: record cause, red/green evidence, the refreshed Graphify graph (9,278 nodes / 15,677 edges), CocoIndex's missing `mcp.server.fastmcp` runtime dependency, and exact-package results in the evidence file and LLM Wiki.
- [x] Package UI: exact package from `6aa687a3` showed the active-task feedback editor enabled and the pending-task editor disabled; choosing the alternate agent enabled Assign. The run was stopped without sending fixture feedback or applying reassignment.
- [x] Acceptance: add this distinct packaged workflow; packaged UI acceptance is 18/25 (72%), while shipped/default Rust UI remains 0/25.

### Exact-package task session and model selection — workflow 19 / 2026-10-24

- [x] Plan: reuse R-20 → BG-09 → SF-11 → UC-11 → FR-02 → QAS-11 → DD-13; use only a disposable shared-Git fixture and a deterministic local CLI.
- [x] Task: build and validate the exact v0.1.37 package at source commit `6ca19965`; package manifest validation and ZIP integrity pass.
- [x] Verify UI: the exact package displayed fixture session `fixture-session-19` and its observed model options. Selecting `fixture-large` enabled Set; after saving, the UI showed that model and the read-only CLI `sessions` report confirmed both model and session ID. Model change invalidated the failed task evidence as designed; the run was stopped.
- [x] Verify regression: all 4 `native_ui_task_controls` tests pass, including revision-bound model changes and observed-catalog validation.
- [x] Assetize: `docs/evidence/issue9-workflow19-session-model-20261024.json` records package hashes, UI observations, and fixture boundaries. No provider CLI or external service ran.
- [x] Acceptance: distinct packaged UI workflows are **19/25 (76%)**; shipped/default Rust UI remains **0/25**.
- [ ] Full all-target attempt: interrupted with exit 130 after RTK reported 825 passed, 14 ignored across 260 suites; do not treat this as a green full-suite result.
- [ ] Remaining acceptance: six packaged workflows, shipped/default Rust UI, Windows guest/runtime and accessibility/IME, resource measurements, final license review, and stable release.

### Reconcile QAS-29 planning gate against current source — issue #9 / 2026-10-24

- [x] Plan: compare the historical packaged report of a missing-coordinator error with the current R-37 / UC-28 / QAS-29 / DD-45 implementation.
- [x] Verify: the current product source requires a non-empty goal and a coordinator present in registered profiles before enabling Generate plan; `plan_generation_button_is_disabled_without_a_registered_coordinator` passes (1/1) at product source `db568c3f`.
- [x] Assetize: `docs/evidence/issue9-qas29-plan-gate-source-20261024.json` records the trace and the older package/source distinction.
- [x] Package UI: exact v0.1.37 Actions package, isolated HOME, disposable repository. Generate plan stayed disabled without a registered coordinator; Write plan manually created three empty tasks immediately; confirmation of that incomplete draft was rejected before any CLI invocation. Evidence: `docs/evidence/issue9-qas29-plan-gate-source-20261024.json`.
- [x] Assetize: record the package workflow and verification boundary in AgentMemory `mem_mvlwz2up_715f5973b7b8`; no source change or workflow-count increase (QAS-29 was already represented in packaged acceptance).

### Verify UC-29 folder-picker source contracts — issue #9 / 2026-10-24

- [x] Plan: verify the project-folder picker selects a path, preserves the prior path on cancellation, and stays visible with the coordinator drawer open.
- [x] Task / Verify: folder-picker worker and cancellation tests pass (2/2); the native probe layout test passes (1/1). The current product source is unchanged from the tested CI source db568c3f.
- [x] Graphify: resolved the picker worker test to tests/native_ui_probe.rs:203 and its containing test module.
- [x] Assetize: evidence docs/evidence/issue9-uc29-source-verification-20261024.json; AgentMemory entry saved for this continuation.
- [x] Package UI: exact v0.1.37 Actions executable was bound to an isolated app window. The native picker selected a disposable project path; cancel preserved it. No workspace was saved and no task was started.
- [x] Acceptance: this adds one distinct packaged workflow; packaged acceptance is 17/25 (68%) and shipped/default Rust UI remains 0/25. CocoIndex CLI is unavailable on this host.

### Recheck UTM Windows after the approved reset — issue #9 / 2026-10-24

- [x] Plan: execute the reset after the user confirmed that unsaved Windows changes could be discarded; leave the existing Z: share settings untouched.
- [x] Task: stop and restart the Windows VM; UTM reports started.
- [x] Historical verification: `utmctl ip-address` returned 192.168.64.6 and a read-only pull of `C:\Windows\win.ini` succeeded immediately after the prior reset.
- [ ] Current state: the user reports the Windows guest does not boot and the display is unavailable; the earlier partial-recovery evidence is stale. Do not reset, power-cycle, alter shares, or change settings. Windows app execution and Z: visibility remain unverified pending read-only diagnosis.
- [x] Assetize: evidence docs/evidence/issue9-utm-windows-state-20261024.json; AgentMemory mem_mvlemf8b_37de1d774936; issue #9 progress comment 6373771571. The zero-byte failed host transfer was removed.

### Repair the Windows unsafe-comment gate — issue #9 / 2026-10-24

- [x] Plan: preserve the Win32 reduced-motion query and add only the safety rationale required by the project's unsafe-block lint gate.
- [x] Red: Actions run `40008320185` passed Windows build, source/test layout, all-target tests, native UI validation, formatting, and warning-denied Clippy; its final safety-comment gate failed at `src/native_ui_probe/motion.rs:54`.
- [x] Task: document that `SPI_GETCLIENTAREAANIMATION` writes one BOOL result to valid writable local storage; behavior is unchanged. Commit `c6c6052`.
- [x] Verify locally: formatting, host unsafe-comment Clippy, `source_layout` (2/2), and `git diff --check` pass. macOS cannot compile the Windows-only block, so remote Windows verification is required.
- [x] Verify remotely: pushed `c6c6052` / `db568c3`; Platform Validation run `40011350882` passed every macOS and Windows validation gate, including the Windows safety-comment gate.
- [x] Package: both platform release executables built, Rust rendered both manuals, exact-package execution/smoke passed, and artifacts were retained. Evidence: `docs/evidence/issue9-platform-packages-40011350882-20261024.json`.
- [x] Assetize: record the diagnostic and local verification in `docs/evidence/issue9-windows-unsafe-comment-fix-20261024.json` and the LLM Wiki. UI acceptance remains 16/25 packaged and 0/25 shipped/default Rust UI.

### Verify exact-package Workflow 16 — issue #9 / 2026-10-24

- [x] Plan: check that a plain non-Git folder can be saved as a workspace and that an inactive workspace can be archived with explicit confirmation.
- [x] Task: open the exact v0.1.37 macOS arm64 package from source `278ad6de` with an isolated HOME; save a workspace for a disposable folder, create a replacement, and confirm archiving the first workspace.
- [x] Verify: the workspace menu lists only the replacement; archive metadata records `confirmed_archive`; during the check the disposable project folder remained intact and had no `.git`. No agent, plan, task, merge, or user workspace was accessed.
- [x] Cleanup: close the single package process and remove `/private/tmp/agentconductor-issue9-workspace-lifecycle-20261024`.
- [x] Assetize: evidence `docs/evidence/issue9-workspace-create-archive-278ad6de-20261024.json`; packaged acceptance advances to **16/25 (64%)** and shipped/default Rust UI remains **0/25**.
- [ ] Complete: retain Windows guest validation, remaining packaged UI workflows, shipped/default Rust UI parity, and release review as open gates.

### Audit the legacy browser asset cleanup — issue #9 / 2026-10-24

- [x] Inventory: tracked application/build/test source contains no authored JavaScript, Python, CSS, or HTML UI files. The only tracked HTML outputs are the English and Korean offline manuals generated by Rust and retained in release packages.
- [x] Verify: `source_layout` passes both tests, enforcing Rust-only authored source under `src/`, `tests/`, `scripts/`, and `examples/`, the 300-line bound, and no inline source tests. `tests/native_ui_assets.rs` remains as a useful negative regression asserting removed browser assets stay absent.
- [x] Correct the stale completion item under “Remove the legacy browser document”: no legacy app assets or asset-inspection tests remain to delete; the manual HTML is generated documentation, not a UI runtime. Evidence: `docs/evidence/issue9-rust-source-inventory-20261024.json`.
- [x] Assetize: update the LLM Wiki and AgentMemory. This inventory correction does not increase packaged workflow acceptance or shipped/default Rust UI acceptance.

### Recheck UTM Windows guest state — issue #9 / 2026-10-24

- [x] Plan: retain the user's existing `Z:` mapping to `/Users/neodamul/Downloads/UTM`; do not change UTM share or power settings.
- [x] Verify: UTM power controls and `utmctl list` report Windows as running/`started`; `utmctl ip-address Windows` returns `192.168.64.6`.
- [x] Task: cancelled the Windows restart-apps confirmation. No UTM stop, pause, or restart control was used.
- [x] Verify and assetize: `public_docs` and `source_layout` pass 13 tests; evidence JSON and `git diff --check` pass. AgentMemory `mem_mvl63tpj_d85ad6415cc8` records the guest-verification boundary. Graphify reports 9,180 nodes / 14,377 edges, unchanged on the second refresh. CocoIndex CLI is unavailable (`No such file or directory`).
- [x] Task: after the user approved the explicit data-loss warning, triggered UTM's VM reset. No share setting was changed.
- [x] Verify: UTM still reports `started`; UEFI boot briefly appeared, then the guest display reported `Display output is not active`. `utmctl ip-address` now returns IPv4 `192.168.64.6` plus IPv6, but the display remains inactive. One read-only pull of `C:\Windows\win.ini` terminated with signal 6 (exit 134).
- [x] Recheck: UTM 5.0.6 (125) still shows `Display output is not active`. A read-only `cmd.exe /c ver` request returned OSStatus -2700 / child-process-not-found with no guest output; no VM power or share setting was changed.
- [ ] Remaining: restore the guest display and a reliable guest I/O path before Windows package verification. macOS also denies listing the provided host share path. A temporary guest marker's cleanup could not be verified after the same signal-6 failure. The then-current desktop observation later confirmed `Z:` access, as the user clarified; after the subsequent approved VM reset, the user reported no boot/display. That newer state supersedes this historical check. Evidence: `docs/evidence/issue9-utm-windows-state-20261024.json`.

### Recheck the exact-package named-workspace lifecycle — issue #9 / UC-30 / UC-31 / 2026-10-24

- [x] Plan: repeat the already accepted named-workspace flow on exact package source `f23381cf`; use the exact `.app` bundle, temporary `HOME`, and app-managed disposable workspace folders.
- [x] Task: create two named workspaces, save separate folder/goal values, switch between them, remove the inactive workspace from the recent list, and reopen it from the retained saved-workspace list.
- [x] Verify: the exact package displays the empty isolated workspace; both names appear in the menu; switching restores each saved name, folder, and goal; the removed entry's folder remains and it opens with the same values. No CLI, MCP, external service, plan, task, archive, or user workspace was touched.
- [x] Cleanup: quit the single package process and remove `/private/tmp/ac-issue9-exact-f23381` after validation.
- [x] Assetize: evidence `docs/evidence/issue9-f23381cf-named-workspaces-20261024.json`; this rechecks existing UC-30/31 coverage and adds no workflow credit. Acceptance remains 15/25 packaged and 0/25 shipped/default Rust UI.

### Restore the Settings gear control — R-70 / issue #9 / 2026-10-24

- [x] Plan: retain the existing Settings menu actions and restore the established icon-only control; keep its localized name available to assistive technology and as a hover tooltip.
- [x] Red: the rendering regression failed because the Settings button had no visible gear glyph.
- [x] Task: render a 20-point `⚙` menu button, expose the localized Settings label as its accessible name and tooltip, and preserve the existing menu action behavior.
- [x] Verify: 22 tests pass across appearance, Workspace menu layout, updates, bilingual UI, settings journey, and source layout. Warning-denied targeted Clippy, formatting, and diff checks pass. Both changed Rust files remain below 300 lines (262 and 288).
- [x] Package: exact pushed source `f23381cfb8cf4165a9207a0a67805c9c89de4d8a` builds as an aarch64 macOS archive. `validate-package`, ZIP integrity, and all four shared/worktree × Git/folder smoke cases pass; each case runs three parallel tasks/tests, approves the expected merge, and rejects unapproved merge. Archive SHA-256 `8bb8dcc511e422ce3b7822e221d125f00b56b501ef10eecd6ec4d76d18b270ee`; binary SHA-256 `735d9fc3f0ba6695be47280783fe823cc25bce55eb57b80388687bd5156f79ad`. Evidence: `docs/evidence/issue9-settings-gear-f23381cf-20261024.json`.
- [x] Assetize: the R-70 trace and visual-verification limits are recorded in the evidence and LLM Wiki; AgentMemory and Graphify updated.
- [x] Visual spot-check: opened the exact macOS package and confirmed the gear-only control in light and dark themes, then opened Settings and saw its accessible name. No screenshot was retained; tooltip hover was not verified.
- [x] Visual check: bound the already-running exact `f23381cfb8cf4165a9207a0a67805c9c89de4d8a` app by full bundle path; the app identified as v0.1.37. Its AX tree names the gear-only button `설정`; screenshot showed the gear and opening the menu exposed language/theme, MCP, workspace sharing, messenger, cache, updates, and update-repository actions. Escape dismissed the menu. The Settings renderer and regression test have no source diff from this package commit to the current branch.
- [x] Assetize: refreshed Graphify to 9,223 nodes / 14,423 edges / 697 communities; CocoIndex CLI is not installed on this host.
- [x] Read-only screen review: opened Agent Roles and Plan Review, then returned to Workspace. The role matrix has six rows with primary/fallback agent and model columns. Plan Review showed no reviewed plan; save and run controls were disabled. No plan was generated and no workspace data changed.
- [x] Current-source verification: focused appearance, roles, plan cards/layout, workspaces, and planning-stop tests pass 23/23 across six suites; no desktop process was started.
- [ ] Isolation/acceptance: did not launch another app or use a temporary HOME because this existing process displayed saved workspace/profile data. No workspace fields or settings were changed; no screenshot asset was retained because it contains private workspace context. This spot-check earns no workflow acceptance. An isolated exact-current-package check and Windows behavior remain open; acceptance stays 15/25 and shipped/default Rust UI stays 0/25.

### Build and verify the current macOS Rust package — issue #9 / 2026-10-24

- [x] Plan: build the exact pushed PR head offline with the existing Rust package command; use disposable fixture workspaces and do not open another desktop window.
- [x] Task: `cargo run --offline --locked --release -- package-release` produced the aarch64 package from `c10121cee4a94e4d0af701779459a34cc3a684f4`.
- [x] Verify: `validate-package`, ZIP integrity, and `smoke-package` passed. All four shared/worktree × Git/folder cases ran three tasks and three tests each; approved merge was verified and unapproved merge refused. The binary is 18,858,304 bytes (SHA-256 `3fa44f63193cdb47ec6d06db450a090804b1241eddb4938e3c2da4451beeaa65`); the archive is 21,672,397 bytes (SHA-256 `6ec2aed9dece072a390906416bd6a4288d40e8f698af404c961fb5c71b031f94`). Evidence: `docs/evidence/issue9-local-macos-package-c10121ce-20261024.json`.
- [x] License/package contents: the manifest reports 242 dependencies; the archive contains the SBOM, dependency-license bundle, notices, and project license.
- [x] Complete this macOS package-build/validation unit; AgentMemory and the LLM Wiki now contain the result. The global #9 gates stay open: this is not a GitHub Release, Windows verification, or UI workflow acceptance. Packaged UI acceptance remains 15/25 and shipped/default Rust UI remains 0/25.

### Verify current-head macOS and Windows release packages — issue #9 / 2026-10-23

- [x] Plan: keep the run on the exact pushed PR head until validation and both platform packages finish; do not interrupt the active build.
- [x] Task: run `39968582271` completed successfully for macOS and Windows validation and release-package jobs. Both builds rendered the English/Korean manuals and passed exact-package execution smoke.
- [x] Verify: artifact ZIP checksums match each embedded `SHA256SUMS`; both ZIP integrity checks pass. macOS archive/binary SHA-256: `dbd756c547d9a014dc0a0edf87d32bfae18a25e1d7b355e758898fb05477f7fb` / `c725e96e44fc949abe1fb75711bcd3b03663c82fa6bc765860fdf87603d6c652`. Windows archive/binary SHA-256: `b9540618139f279d22cbfe7515618ad0d5db04f3b67da8be3c9838ef4835c281` / `b0b7f1bf7fa71bf5f9a0537316d07ff10abf1121e27c3ed057db2d7c55c1a633`. Both packages include bilingual HTML manuals, dependency license notices, and SBOMs. Evidence: `docs/evidence/issue9-ci-packages-39968582271-20261023.json`.
- [ ] Remaining: these are CI package smoke results, not a GitHub release or exact visual workflow acceptance. The user-confirmed interactive Windows `Z:` guest UI check, complete packaged workflow parity, shipped-main integration, and final release gates remain open.

### Stage current source for the existing UTM Z: share — issue #9 / 2026-10-24

- [x] Plan: use the existing share target `/Users/neodamul/Downloads/UTM`; leave UTM share configuration and VM power state unchanged.
- [x] Task: archived PR head `6fd78fed3ec2139a2b7395a23d0902e3f7f1a957` and copied it to `Z:\AgentConductor\AgentConductor-6fd78fe.tar.gz`.
- [x] Verify host transfer: source and destination SHA-256 match (`8c4a2f788de724dc67ee1bde52ea371ccea3c3e17811bd7e39039a45a287e9b1`); archive size is 5,984,948 bytes. Evidence: `docs/evidence/issue9-utm-z-share-6fd78fe-20261024.json`.
- [x] Package: copied the verified Windows x64 package from run `39968582271` to `Z:\AgentConductor\AgentConductor-Windows-x64-PR22-8abbb6df.zip`. Its SHA-256 matches the run's embedded checksum (`b9540618139f279d22cbfe7515618ad0d5db04f3b67da8be3c9838ef4835c281`); ZIP integrity passes. The package source commit has no product-source delta through current PR head `1db9a5f2`.
- [x] Verify and assetize: `public_docs` and `source_layout` pass 13 tests; rustfmt, evidence JSON validation and `git diff --check` pass. Wiki and AgentMemory `mem_mvl3z9xr_c6a0c6383123`, `mem_mvl474ff_dae792f111ce` record the package/checksum and guest-verification boundary. Graphify refreshed without clustering (9,172 nodes / 15,561 edges). CocoIndex is unavailable in this runtime.
- [x] Guest support: after attempting the previously authorized restart of the three listed guest apps and dismissing the stuck prompt, `utmctl list` still reports `started` and `utmctl ip-address` returns `192.168.64.6`; input capture is restored off. No UTM VM restart or share-setting change was made.
- [ ] Verify guest: the current screen shows the restart-apps confirmation for UTM Guest Tools Installer, Network File Explorer, and Task Manager. Guest file pull still aborts with signal 6 and command output is not observable. `Z:` file visibility and Windows package/UI execution remain unverified.

### Verify local PR #22 head — issue #9 / 2026-10-24

- [x] Plan: run the full locked/offline release test suite and warning-denied Clippy locally on exact PR head `7806b18e2ca6a3b8ce493e81664a3e267df4679d`.
- [x] Verify: all 959 tests pass, 14 are ignored, across 319 test binaries; all-target release Clippy with `-D warnings`, rustfmt, `public_docs` and `source_layout` (13 tests), and `git diff --check` pass.
- [x] Knowledge: saved AgentMemory `mem_mvl4tx5n_79ab32014652`; Graphify refreshed the code graph without clustering (9,173 nodes / 15,562 edges).
- [ ] Verify remotely: Platform Validation run `39977328948` was cancelled after the documentation push; current-head run `39979909480` is validating macOS and Windows and must finish before closing this gate.
- [ ] Verify Windows guest: UTM still reports `started`; the Windows restart-apps confirmation remains visible. Input capture is off. The user's existing `Z:` share maps to `/Users/neodamul/Downloads/UTM`; no share setting or VM power control was changed. Guest access and Windows runtime remain unverified.
- [ ] Acceptance: local source/test validation does not increase packaged UI workflow acceptance (15/25) or shipped/default Rust UI acceptance (0/25).

### Build the current macOS Rust executable — issue #9 / 2026-10-24

- [x] Task: run `cargo run --offline --locked --release -- --help` on pushed head `c67425bfa897e8f7a30efe989c7ce132ec28e4e8`; the default Rust CLI builds and prints its command help without opening a desktop window.
- [x] Verify: `target/release/agentconductor` is 18.0 MiB, SHA-256 `3fa44f63193cdb47ec6d06db450a090804b1241eddb4938e3c2da4451beeaa65`.
- [ ] Complete: this is a local executable, not a distributable package or GitHub Release. Keep macOS/Windows packaging, Windows guest execution, exact UI workflows, and issue #9 acceptance gates open.

### Build and verify the current Rust macOS package — issue #9 / 2026-10-23

- [x] Plan: build the exact issue-branch source with the existing Rust package command; retain only the current verified package locally.
- [x] Task: `cargo run --offline --locked --release -- package-release` built source commit `91393d7b82c92b0c3e4891b4730fc8475aa250fd` as the aarch64 macOS package.
- [x] Verify: `validate-package` passed; the manifest records one 18,858,304-byte native executable and 242 dependencies. Package-release smoke succeeded; ZIP integrity passed. Archive size is 21,672,395 bytes, SHA-256 `9c068e85d2610f3736b935f3bcdc49dfeb7194a34c73141c568b8ae831c369c6`; binary SHA-256 `3fa44f63193cdb47ec6d06db450a090804b1241eddb4938e3c2da4451beeaa65`. Evidence: `docs/evidence/issue9-rust-macos-package-91393d7-20261023.json`.
- [x] Audit and cleanup: the current branch's default feature launches the Rust native Workspace; tracked-source scan found 0 `.js`, `.jsx`, `.ts`, `.tsx`, or `.py` files. Removed superseded generated package artifacts from `dist`, reducing it from 886 MB to 63 MB; kept the current package and checksum. The 24-GB debug cache was also removed while preserving the 4.3-GB release cache.
- [ ] Remaining: this package was not uploaded as a GitHub release and does not count toward exact UI workflow acceptance. Windows package/build/UI, shipped-main integration, full parity, and release gates remain open. Packaged UI acceptance remains 15/25; shipped/default-main Rust acceptance remains 0/25.

### Stage current Windows test head through the existing UTM share — issue #9 / 2026-10-23

- [x] Plan: use the user's existing Z: mapping backed by `/Users/neodamul/Downloads/UTM`; do not alter UTM share settings or VM power state.
- [x] Task: archive pushed head `82c1b309eb21212a88fbb765659772d16fd03a66` as `AgentConductor-82c1b309.tar.gz` and copy it to the host share folder.
- [x] Verify host transfer: source and destination SHA-256 match (`23726a374a9bb4aac42cd5382c012ea20f68c6edc4fc815999c0272cef7e9e68`); archive size is 5.7 MB. Evidence: `docs/evidence/issue9-utm-z-share-82c1b309-20261023.json`.
- [x] Record boundary: UTM reports Windows started. A `utmctl exec` PowerShell probe did not create a host-visible Z: marker; `utmctl exec --debug` returns two OSStatus `-10004` errors without guest output, and `utmctl file push` aborts with signal 6. These guest-agent failures do not disprove the interactive user's Z: mapping. No share setting or VM power state changed.
- [x] Verify checkpoint: run `39967301046` was superseded by the documentation push and cancelled. Its macOS validation steps passed; the Windows build step was cancelled, so it is not a green cross-platform result.
- [ ] Verify Windows: both validation jobs in run `39968582271` passed, including Windows build, 300-line source/test layout, all native tests, opt-in Rust UI launch, formatting, and Clippy. The macOS/Windows release-package jobs are now running; still use the interactive guest Z: mapping for a Windows UI check when available.
- [x] Reconcile PR metadata: GitHub reports PR #22 OPEN/DRAFT, head `d7b8989d7476f3fd99dc6ed5f8ba2f0eb996bb63`, base `602ec1182145c59eac402c39558af05af6ebe1e1`, and `MERGEABLE`; `UNSTABLE` reflects unfinished checks. The issue body's previous `e7ec233` head is stale.
- [ ] Complete: keep the Windows gate open until the current-head Windows tests and a Windows package/runtime check pass.

### Fix Windows test teardown and native goal input — issue #9 / 2026-10-23

- [x] Plan: inspect Windows failures in updated-head Actions run `39962957846`; preserve real-user keyboard entry and remove test-owned files cleanly.
- [x] Red: Windows `charter_contract` failed deleting two run folders while each SQLite `Store` was open (OS error 32); `windows_desktop` invoked UIA `ValuePattern` on a multiline goal and the saved goal stayed empty.
- [x] Task: explicitly drop both SQLite stores before folder cleanup; focus the goal editor and type via `SendKeys`; make PowerShell errors fail the driver test.
- [x] Verify locally: `charter_contract` 5/5, source layout 2/2, all-target warning-denied Clippy, rustfmt, and `git diff --check` pass. Both changed test files remain under 300 lines.
- [x] Assetize: AgentMemory `mem_mvl1qlyp_14f1bd97e9fb`; Graphify refreshed to 9,164 nodes / 15,553 edges; updated `knowledge/wiki/packaged-workflow.md`. CocoIndex is unavailable because `mcp.server.fastmcp` is missing.
- [ ] Verify on Windows: rerun the committed fix on a Windows runner; the UIA test is Windows-only and cannot be executed on this macOS host.
- [ ] Complete: close this repair unit only after Windows tests, UI launch, formatting, and Clippy pass. Issue #9 stays open; packaged UI acceptance remains 15/25 and shipped/default Rust UI remains 0/25.

### Normalize Windows manual-test line endings — issue #9 / 2026-10-23

- [x] Plan: inspect Windows job `120433273032`; preserve exact HTML content comparison while normalizing checkout line endings.
- [x] Red: run `39959344746` showed rendered LF output against CRLF checked-in HTML; macOS passed and Windows failed the Korean equality assertion.
- [x] Task: normalize CRLF to LF on both rendered and checked-in English/Korean HTML comparisons in `tests/workspace_screenshot_assets.rs`.
- [x] Verify locally: screenshot/manual/source-layout suites pass 8/8; all-target warning-denied Clippy, rustfmt and `git diff --check` pass. Test file is 50 lines.
- [x] Assetize: AgentMemory `mem_mvl0omak_31a305954875`; Graphify refreshed to 9,162 nodes / 15,551 edges. Updated `knowledge/wiki/packaged-workflow.md`. CocoIndex CLI fails to import missing `mcp.server.fastmcp`.
- [x] Verify remotely: in run `39962957846`, `workspace_screenshot_assets` passed on Windows and the full macOS job passed. The broader Windows job exposed unrelated `charter_contract` and `windows_desktop` test harness failures, tracked in the next checklist.
- [x] Complete: close this assertion-fix unit; keep the overall Windows platform gate open until all tests, UI launch, formatting, and Clippy pass. UI acceptance remains 15/25 and shipped/default Rust UI remains 0/25.

### Stage the exact PR head through the existing UTM Z: share — issue #9 / 2026-10-23

- [x] Plan: use the existing Z: share and user-specified host folder; leave UTM share configuration unchanged.
- [x] Task: archive pushed source `091aa9b4576a863d05cdb400a8ace25de7b138c9` as `AgentConductor-091aa9b4.tar.gz` and copy to `/Users/neodamul/Downloads/UTM`.
- [x] Verify host transfer: source and destination SHA-256 both equal `7860127ec648972b2e108bd97fb66a13e71882499d1b4c5723abca58166144eb`; `issue9_current_status` and `source_layout` pass 3 tests; JSON, formatting and diff checks pass. Evidence: `docs/evidence/issue9-utm-z-share-091aa9b4-20261023.json`; AgentMemory `mem_mvl0rxia_37a709730336`.
- [ ] Verify: Windows guest Z: visibility and exact-source build/tests remain pending; the restart-apps prompt is still visible. `utmctl exec` and `file pull` both terminate with signal 6 on host VM UUID `34FD796F-726E-4F66-8809-8EE37F810B6A`. UTM remains started and input capture is off.
- [ ] Complete: close this gate only after guest read, Windows build and test results are recorded.

### Fix the macOS PR lint failure — issue #9 / 2026-10-23

- [x] Plan: inspect failing job `120423814068` in Actions run `39956497286`; preserve the current test behavior and avoid adding lint suppressions.
- [x] Red: macOS build, native tests and formatting passed, but Rust 1.99 warning-denied Clippy rejected `labels.iter().any(|label| *label == result)` as `manual_contains`.
- [x] Task: replace the equality scan with `labels.contains(&result)` in `tests/native_ui_task_activity.rs`.
- [x] Verify locally on Rust 1.99: focused test 7/7, all-target warning-denied Clippy, rustfmt, source-layout 2/2, and `git diff --check` pass.
- [x] Assetize: evidence is `docs/evidence/issue9-pr-clippy-fix-20261023.json`; AgentMemory `mem_mvkzmzgs_5e9246d8aa0a` records the CI failure and correction; Graphify refreshed without clustering (9,159 nodes / 15,548 edges). CocoIndex's CLI and Python module are unavailable here.
- [ ] Verify remotely: the pre-fix Windows job was still running at 2026-10-23 13:11 UTC. It does not validate this source change; recheck Actions on the pushed fix head.
- [ ] Complete: commit and push the fix, then require updated-head macOS and Windows validation before closing this unit. Issue #9 remains open; packaged acceptance stays 15/25 and shipped/default Rust UI stays 0/25.

### Recheck the live UTM Windows screen — issue #9 / 2026-10-23

- [x] Inspect: UTM reports the Windows VM as started; the Windows restart-apps confirmation is visible and lists Guest Tools Installer, Network File Explorer, and Task Manager.
- [x] Preserve: no Z: mapping/share setting or VM power state was changed. Input capture was temporarily enabled for a Cancel attempt and restored off; the prompt did not dismiss.
- [ ] Verify: guest-side Z: access and exact-source Windows build/tests remain unverified. Do not restart the guest without the user's explicit go-ahead because the prompt says three apps will close.
- [ ] Complete: close the Windows verification gate only after guest archive access and Windows build/test evidence. Evidence: `docs/evidence/issue9-utm-screen-recheck-20261023.json`.

### Transfer the current #9 PR head through the existing UTM Z: share — 2026-10-23

- [x] Plan: use the user's existing `Z:` mapping, backed by `/Users/neodamul/Downloads/UTM`; do not alter drive settings or VM power state.
- [x] Red: extend `issue9_current_status` to require the exact branch archive checksum and preserve the unverified guest-build boundary; it failed while the host-copy checksum was pending.
- [x] Task: archive branch head `d1e1b3746b3ea08cbaa09c1ac167e0c569c9810d` as `AgentConductor-d1e1b374.tar.gz`, copy it to the share, and verify matching source/destination SHA-256 `6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd`.
- [x] Assetize: LLM-Wiki/log and AgentMemory `mem_mvky5hgz_cdc3472b5691` record the verified host transfer and unverified guest boundary. Graphify refreshed locally to 9,150 nodes / 15,539 edges without clustering; CocoIndex is unavailable in this runtime.
- [ ] Verify: open this archive from guest `Z:` and build/test on Windows. The VM still shows the Windows restart-apps confirmation; no restart/shutdown or Windows build was performed.
- [ ] Complete: close the transfer/build gate only after guest-side archive access and local Windows verification. Evidence: `docs/evidence/issue9-utm-z-share-head-20261023.json`.

### Transfer earlier #9 source through the existing UTM Z: share — 2026-10-23

- [x] Plan: use the user's existing `Z:` mapping backed by `/Users/neodamul/Downloads/UTM`; leave Guest Tools and VM power controls unchanged.
- [x] Red: extend `issue9_current_status` to require evidence for the exact branch archive, host share path, guest-access boundary, and no VM restart; it failed while the evidence file was absent.
- [x] Task: create `AgentConductor-c4cbd2a.tar.gz` from branch head `c4cbd2a6c13b5ab59f50b1efa37842a54a9c7a9c` and copy it into the host share. Finder shows the copied archive at the requested path.
- [ ] Verify: open the archive from guest `Z:` and build/test on Windows. The VM still shows a Windows restart-apps confirmation; do not select Restart. No Windows build, package, or UI result is claimed.
- [ ] Complete: only mark the Windows transfer/build gate complete after guest-side archive access and local Windows verification. Evidence: `docs/evidence/issue9-utm-z-share-transfer-20261023.json`.

### Reconcile latest main feature changes with Rust — issue #9 / 2026-10-23

- [x] Plan: compare the ten main-only commits with the Rust implementation before resolving PR #22 conflicts; preserve the Rust-only shipped UI boundary.
- [x] Verify: `amendment_policy`, `charter_contract`, `coordinator_amendment_contract`, `coordinator_decisions`, and `coordinator_request` plus quota display passed 42 tests across ten suites total (32 across seven suites, 7 in quota display, and 3 in process/worker readiness) with locked/offline release-profile tests.
- [x] Evidence: `docs/evidence/issue9-main-feature-parity-20261023.json`; current source `e7ec2337d2f2e86c8aa406cc3ff8e3b931552834`, upstream main `b7fa27a9f033b91df5787e05833c30db471645e5`.
- [x] Assetize: the Rust equivalents for coordinator feedback, bounded plan amendments/undo, and charter goal-drift gates are recorded in `knowledge/wiki/coordinator-directed-operator-feedback.md`.
- [x] Checkpoint: at this pre-merge audit, PR #22 was open/draft and `DIRTY`, with 39 conflict paths; the source archive targeted the old `8b50d0bd` head. The following checklist records the completed integration work and supersedes that state.

### Integrate latest main and verify Rust parity — issue #9 / 2026-10-23

- [x] Plan: merge the 39 current-main conflict paths in the existing #9 issue branch, retaining the Rust-only UI and source/test split.
- [x] Red: migration regressions exposed stale Charter APIs, a duplicate fixture, a mismatched room-action test packet, and a missing `amendment-undo` child-process allowlist entry.
- [x] Task: port the immutable-charter cumulative path limit (8 per run), reset the charter after an operator-approved scope recovery, and route the native Undo action through the broker.
- [x] Verify: 69 tests passed across 15 focused suites; `cargo fmt -- --check`, diff checks, conflict-marker scan, and the 300-line source/test file limit pass.
- [x] Knowledge: preserve the discovery and test result in the coordinator feedback Wiki and AgentMemory.
- [x] Complete: integration commit `d1f5071fb5c47217b09c5283f1b0dd1731483d83` is pushed to the issue-linked branch. PR #22 is open/draft with two checks pending. The UTM Windows VM remains started; use its existing Z: mapped share for transfer first. No Windows build/package/UI result is claimed. Packaged acceptance remains 15/25 and shipped/default Rust UI 0/25.

# Implementation plan and TDD

### Refresh the PR and release-version audit — issue #9 / 2026-10-23

- [x] Plan: record remote state after the policy-port push and prevent stale `DIRTY`/v0.1.36 claims from guiding integration.
- [x] Red: the updated `issue9_current_status` assertion failed against the old `DIRTY` audit text.
- [x] Task: verify live PR/main/release metadata; update the completion audit and current checklist with the actual state and the version-alignment gate.
- [x] Verify: `issue9_current_status`, `public_docs`, and `source_layout` pass 14 tests; rustfmt and diff checks pass. UI acceptance is unchanged.
- [x] Complete: the latest PR query, published v0.1.37, branch v0.1.36, and unmerged state are recorded. The final next version must be rechecked at release time.
- [x] Assetize: AgentMemory `mem_mvkv768g_ad00e2a409b0` records the PR/release state and version gate; Wiki `native-rust-ui-migration.md` carries the current checkpoint.

### Port the Google Rust review policy to the Rust migration branch — GitHub #169

- [x] Plan: preserve the selected review rules, explicit exclusions, baseline, and unchanged source-layout limits from `main` without adding executable lints or rewriting existing code.
- [x] Red: `rust_review_policy` failed because `docs/13-rust-review-policy.md` was absent from this branch.
- [x] Task: port the policy and its R-174 → DD-185 architecture trace, docs index, changelog, and LLM-Wiki article.
- [x] Verify: 14 tests pass across `rust_review_policy`, `public_docs`, and `source_layout`; `cargo fmt --all -- --check`, `git diff --check`, and the 300-line source-layout guard pass. No product source changed.
- [x] Complete: #169's selected review guidance is documented for this Rust line. No new lint or broad cleanup is included; executable checks remain the current formatter, warning-denied Clippy, and 300-line guard.
- [x] Assetize: `knowledge/wiki/rust-review-policy.md` summarizes the rules and traceability; AgentMemory `mem_mvkv1xt2_6edd95931d5e` records the policy and evidence. Graphify's existing 9,035-node graph surfaces `retry_review` and source-layout concepts but predates this policy, so no new edge is claimed. CocoIndex is not installed on this host.

### Build and validate current-source macOS manual package — issue #9 / 2026-10-23

- [x] Plan: package the exact current branch source and compare bilingual manual and Agent Room screenshot hashes with the repository assets.
- [x] Task: `cargo run --offline --locked --release -- package-release` built v0.1.36 from `575fef74d160d6050a878e89d3bafa7f831432f5`; package smoke completed successfully.
- [x] Verify: `validate-package` passed for the aarch64 macOS package (242 dependencies). The archive checksum is `640742c6cd06143ffe828463e078bd722e8d5e31132eaab07b9e1ae3a892c36e`; bilingual HTML and Agent Room PNG hashes match repository files.
- [x] Complete: the exact v0.1.36 package opened its User manual dialog under an isolated `HOME` and workspace; scrolling displayed the dark Korean Workspace and three-task English Agent Room screenshots. No task, approval, or merge was performed. This verification adds no workflow acceptance credit.
- [x] Knowledge: update the offline-manual Wiki, index, and log; AgentMemory `mem_mvksk77o_52dd00979543`, `mem_mvkszuvs_4bffa3562d06`, and `mem_mvktbdx0_3c9cde41ab07`. Graphify read-only path found no edge from the asset test to the manual page. CocoIndex is not initialized, so no graph/index coverage is claimed.
- [x] Evidence: `docs/evidence/issue9-current-source-package-manual-20261023.json`.


### Capture a real current Workspace screen for the README — issue #9 / 2026-10-23

- [x] Plan: replace the README's no-image guard only with a single verified UI screenshot; preserve public-facing product copy and exclude mock task data.
- [x] Red: the README screenshot test failed because there was no image. The Rust manual-capture path also required a loaded Agent Room snapshot, so it could not export the empty Workspace.
- [x] Task: reuse the opt-in eframe framebuffer capture for the first rendered screen on any surface; keep the taller capture size specific to Agent Room. Capture the empty Workspace from an isolated HOME and workspace, then replace the existing Workspace asset and render the offline HTML manual.
- [x] Verify: the captured v0.1.36 screen is 1140×740, shows zero registered agents and blank workspace fields, and contains no task fixture or user data. No provider CLI or MCP server was started. Screenshot SHA-256 `824044b44a29920ded08631b649cde53d4634ef6ed3606be1c2c5649a9d34abf`; evidence `docs/evidence/issue9-readme-workspace-screenshot-20261023.json`.
- [x] Tests: focused capture, README asset, issue-status, public-doc and source-layout suites pass 17/17 across five suites; Rust formatting and diff checks pass. Files remain below 300 lines.
- [x] Assetize: AgentMemory `mem_mvkqixpj_4be5b5a6852c` records the no-demo-data screenshot rule. A scoped Graphify graph linked the README test, plan and provenance (554 nodes, 557 edges; 141 query-relevant nodes). CocoIndex CLI is unavailable on this host; no index result is claimed.
- [x] Complete: README and English manual use the current source-built Workspace capture. This docs/capture improvement adds no packaged workflow credit; acceptance remains 15/25 and shipped/default Rust UI remains 0/25.
- [x] Task: recapture Korean light/dark Workspace screens from the exact current source build in isolated preferences and an empty HOME; visually inspect both images and regenerate both offline HTML manuals.
- [x] Verify: both images are 1140×740 and contain no agents or task data. Regression test confirms each language's Markdown and HTML manuals stay synchronized with their embedded images. Evidence: `docs/evidence/issue9-korean-workspace-screenshots-20261023.json`.
- [x] Assetize: AgentMemory and screenshot provenance Wiki record source/binary and image hashes. Scoped Graphify extraction contains 612 nodes/618 edges; its code-only query resolves the new screenshot regression test but does not claim document-level graph links. Packaged workflow acceptance remains 15/25.
- [x] Complete: current English and Korean Agent Room images now show the same isolated 3/3 task, 3/3 passing-test approval-pending state; both language manuals and in-app viewer use the matching image. They add no package acceptance credit. Evidence: `docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.
- [ ] Remaining: exact-package/manual review, Windows, accessibility/IME, and final-package gates remain open.

### Wait for a fully loaded Agent Room before capturing its manual image — issue #9 / 2026-10-23

- [x] Red: reproduce a Korean Agent Room screenshot with the first-frame capture and assert that the reviewed result label is localized. The source renderer exposed raw `result` for custom CLI output, and the capture path saved the initial “loading task details” frame.
- [x] Task: map the common `result` event label through the existing bilingual message catalog; defer framebuffer capture until the initial snapshot is loaded and, in Agent Room, selected-task details have arrived.
- [x] Verify: the focused manual, task-activity, docs, layout, and issue-status checks pass 32 tests across 8 suites; Clippy (`-D warnings`), formatting, and diff checks pass. Changed files remain below 300 lines.
- [x] Complete: rebuilt the release-profile capture probe from current source; both visually inspected 1140×960 screenshots show loaded task details, and Korean shows the translated `결과` event label. Replaced the stale shared JPG with separate English/Korean PNGs, regenerated both HTML manuals, and updated the native viewer's embedded assets.
- [x] Assetize: AgentMemory `mem_mvkrrsp1_e957210822e7` and Wiki provenance updated; a source-only Graphify index linked the capture readiness/localization tests (74 nodes, 113 edges). CocoIndex was unavailable. No provider CLI, MCP, approval, or merge occurred. This docs/UI capture fix does not change packaged acceptance (15/25).

### Retain bilingual Agent Room PNGs in release packages — issue #9 / 2026-10-23

- [x] Plan: verify the release assembler still copies the new English/Korean Agent Room images into both macOS and Windows packages.
- [x] Red: update the package test to require the current PNG names; the existing JPG fixture failed at the English PNG assertion.
- [x] Green: use the exact current manual asset names in the release-package fixture and assert both are included in the generated package.
- [x] Verify: `release_package` passes all three assembly tests for macOS and Windows targets; `workspace_screenshot_assets` passes. Clippy with warnings denied, rustfmt, diff checks, and 300-line bounds pass.
- [x] Knowledge: updated the packaged-workflow Wiki and AgentMemory `mem_mvkryc3i_dc49548e5d58`; scoped Graphify query covers package asset collection and the regression test (124 nodes, 233 edges). CocoIndex was unavailable.
- [x] Complete: this fixture-level package regression covers the screenshot path transition but is not a built exact-source package or operator UI check; acceptance remains 15/25 and shipped/default Rust UI 0/25.

### Keep the current audit separate from the last PR metadata result — issue #9 / 2026-10-23

- [x] Plan: distinguish the latest pushed product/test checkpoint from the last successfully queried GitHub PR state; do not present a stale connector response as current.
- [x] Red: after the audit wording changed, `issue9_current_status` failed because its assertion still required the older `PR #22 was pushed through ...` sentence.
- [x] Green: assert the latest product/test SHA, the last successful PR metadata SHA, the unconfirmed-state caveat, and the acceptance boundary independently.
- [x] Verify: `issue9_current_status` and `source_layout` pass; rustfmt and diff checks pass. Test file remains below 300 lines.
- [x] Knowledge: AgentMemory `mem_mvks2ftf_3aeef5889b24` records the status-boundary rule and `mem_mvks5qc0_21426db21ed7` records the live-state refresh; a GitHub CLI query confirmed PR #22 at `bafa5335220bf1ea0330bc39662cf9d937d265cc`, open/draft and conflicting with `main`.
- [x] Complete: no product behavior or workflow acceptance changed. Keep issue #9 open at 15/25 packaged and 0/25 shipped Rust UI.


### Synchronize the issue #9 audit with the current PR head — 2026-10-23

- [x] Plan: compare the audit checkpoint with the exact GitHub PR #22 head and preserve the 14/25 packaged-acceptance boundary.
- [x] Red: the full local native-feature run failed because the status guard still expected an older PR head.
- [x] Task: update the guard and completion audit to the confirmed `47974b34b317e28930178dd04ac3ef186700c5b4` head.
- [x] Verify: focused status/docs/layout suites pass 14/14; full offline locked native-feature suite passes 943, with 14 ignored and 0 failed across 312 suites. `git diff --check` passes.
- [x] Complete: no product code or workflow acceptance changed; issue #9 remains open at 14/25 packaged workflows and 0/25 shipped Rust UI replacement.

### Route safe native manual links to the operating system — issue #9 / R-144 / UC-134 / QAS-140 / DD-153

- [x] Plan: retain the shared HTTP(S) allowlist and egui link controls; ensure the native eframe/winit backend actually handles their `OpenUrl` requests.
- [x] Red: a focused regression failed because the shared eframe dependency omitted its `links` feature; the upstream backend logs that browser opening is disabled without it.
- [x] Task: enable eframe's existing `links` feature and lock the resolved dependency graph; add a regression guarding the native feature configuration.
- [x] Verify: native manual, renderer, source-layout and feature-configuration suites pass 14/14; focused release Clippy, rustfmt, diff checks, and macOS arm64 release build pass. `webbrowser` is MIT OR Apache-2.0, already allowed by package policy.
- [x] Package: exact source `d539e86f1d099279e7a80fe8f9753b9065621631` builds as macOS arm64 v0.1.36; `package-release` passed its four-case smoke matrix and `validate-package` reports 242 dependencies with the new license files included. Binary SHA-256 `574851db0b8db321cc26d9a307b2cc34cd4e37e0bc66c703d8196d85ee661f91`; archive SHA-256 `c6d74baf015881d95a8f97c1f881383bcd35f2cf0a2b27972c7a8d2fa057ef4a`. Do not count this as a new operator workflow.
- [x] Knowledge: updated the native UI LLM-Wiki and AgentMemory (`mem_mvko0x1r_a190208e21c9`). Graphify's fresh extraction introduced 1,271 dangling edges, so keep the reviewed `graphify-out/2026-10-23` snapshot active (9,035 nodes, 14,220 edges, zero dangling). CocoIndex CLI is unavailable on this host.
- [x] Isolation follow-up: direct launch of the exact d539e86f1d09 package with temporary HOME and `ui-new` opened an empty workspace (zero profiles, no saved workspaces); the bundled manual opened.
- [x] CUA diagnosis: re-resolving the app by name or bundle path launched a second default-profile instance without the temporary arguments/environment and displayed the user's saved workspace plus refreshed quota metadata. No workspace fields were changed or saved; no task, approval, update, or merge was run. The instances and temporary files were removed; an automatic quota refresh may have occurred. AgentMemory: `mem_mvkohxhn_1d474190b4f1`.
- [x] UI follow-up: the exact v0.1.36 macOS arm64 package built from documentation head `6b9641a1f6e47c9601618747a2b842b6e1b96da9` rendered the bundled Korean manual. Clicking its visible “공식 설정 문서” HTTPS link opened `learn.chatgpt.com/docs/config-file/config-reference` in Safari. Archive SHA-256: `7572e511897ee73f01bcbfb53c5c19fc9ff423697611099b1a36bcb89206cb72`.
- [x] Cleanup and boundary: closed the app and the test-created Safari tab without saving or changing the displayed workspace. App startup may have refreshed quota metadata. This is manual-link component evidence only; it adds no operator workflow credit and acceptance stays 14/25. The user workspace was visible, so its screenshot was not retained; unsafe-scheme behavior remains source-test-only.
- [x] Progress: issue [#9](https://github.com/neodamul/AgentConductor/issues/9) records the diagnostic and remaining test gate.

### Verify exact-package Workflow 15 — issue #9 / 2026-10-23

- [x] Plan: verify the packaged offline manual, embedded Agent Room image, and release-update notification without starting a task or installing an update.
- [x] Verify package identity: source `b282fca47792890bbe85169652164f4ee56c7e42`, v0.1.36 macOS arm64; binary SHA-256 `574851db0b8db321cc26d9a307b2cc34cd4e37e0bc66c703d8196d85ee661f91`; archive SHA-256 `d988bf064fc1252e94a4212dfef65085d75dfc3aaf93bc461f2e376a518feb61`; 242 dependencies.
- [x] Observe: launched the exact package executable directly with a disposable `HOME` and explicit `ui-new` root. The app menu exposed only `/private/tmp/agentconductor-issue9-isolation-b282fca-20261023/.agentconductor/desktop`; the initial screen showed zero registered agents. The bundled offline manual displayed its Agent Room image. The update panel reported installed v0.1.36/latest v0.1.37 and its release notes scrolled to the final GitHub #156 entry. No settings were saved, task started, approval made, update installed, or merge performed.
- [x] Isolation and cleanup: direct package path was bound in CUA; no browser tab was opened in this isolated run. The app, isolated HOME, and this test's package directory/archive were removed. No agent CLI quota query was started.
- [x] Acceptance: Workflow 15 is accepted; packaged native UI progress is **15/25 (60%)**, with 10 distinct workflows remaining. Shipped/default Rust UI replacement remains 0/25. Evidence: `docs/evidence/issue9-workflow15-ui-isolated-20261023.json`; failed earlier attempt retained separately in `docs/evidence/issue9-workflow15-ui-isolation-20261023.json`.

### Fix wrapped table layout in the native manual — issue #9 / R-144 / UC-134 / QAS-140 / DD-153

- [x] Plan: preserve readable offline manual tables in the native UI without adding a renderer dependency or changing the standalone HTML path.
- [x] Red: the first layout test measured only 48 px between two table columns; after widening columns, a long-cell regression showed a 77 px row overlap.
- [x] Green: allocate columns from the available window width (at least 220 px) and render cell segments vertically so wrapped height advances the table row.
- [x] Verify: native manual, manual renderer and source-layout suites pass 13/13; focused Clippy (`-D warnings`), rustfmt and `git diff --check` pass. `manual_viewer.rs` is 267 lines; `native_ui_manual.rs` is exactly 300 lines.
- [x] Package: exact source `d051c8d4d7cdcb583b6591a8aec0f37db214b246` builds a validated v0.1.36 macOS arm64 package. Binary SHA-256 `7efb41e3c20001b60282772fff4f7ccc372a44eea4b7e62451030cc70df7a36d`; archive SHA-256 `678b9fa141bc8a63fbbbf6e182fcb1c6d99f40ffe6b904fe443556b16ed79f6d`. All four deterministic smoke modes passed: shared/worktree × Git/folder, three parallel tasks and three tests per case, with unapproved merges refused and approved merges verified.
- [x] UI: the exact package opened the built-in manual in one isolated workspace; the command table fills the window width and wrapped rows no longer overlap. No task or external link was run; screenshot bytes were not retained.
- [x] Acceptance: this manual rendering fix adds no distinct workflow credit; remain at 14/25. The manual screenshot refresh is complete from the current Rust source build; Windows, accessibility/IME, final-source package and other #9 gates remain open.


### Revalidate exact packaged review and approval UI — issue #9 / 2026-10-23

- [x] Plan: run the exact v0.1.36 macOS arm64 package from source `c92e2b49658b26da2153cc743fb99b23e0aa7e28` against a disposable three-task worktree fixture.
- [x] Task: execute three independent mock CLI tasks with non-overlapping files; keep the fixture outside the repository and the candidate unapproved.
- [x] Verify: all 3 tasks completed and all 3 tests exited 0 without timeout. The assembled candidate is `cb180bde36e446ab70ad41ecab242cb74c2d3464` and state is `waiting_approval`.
- [x] UI: exact packaged Agent Room loaded review evidence showing the candidate hash, each test exit code/stdout and scoped diff. Approve was available; merge remained disabled before approval. No approval, rejection or merge was performed.
- [x] Complete: evidence is recorded in `docs/evidence/issue9-c92-package-approval-review-20261023.json` and Wiki `knowledge/wiki/native-inspector-and-judgments.md`. This fixture is not a distinct acceptance journey; keep packaged UI acceptance at 14/25 and shipped/default Rust UI at 0/25.
- [x] Remaining: the repository screenshot is retained by the 2026-10-23 manual refresh checkpoint below. Windows verification and the other #9 gates remain open.

### Package the native manual safe-link fix — issue #9 / R-144 / 2026-10-23

- [x] Plan: build from the exact pushed source head and use the repository's checksum validator and isolated package smoke suite; do not disturb the pending approval fixture or the user's workspace.
- [x] Task: create the local macOS arm64 v0.1.36 package from source `8c8a08a8dfbe52598cf13e3e7be2e2bd6e82f65e`.
- [x] Verify: package checksum validation passed; all 214 SBOM components have a license expression, and the package includes `licenses/`, `DEPENDENCY-LICENSES.md`, and `THIRD-PARTY-NOTICES.md`. Four isolated smoke cases passed (shared/worktree × Git/folder), each with three parallel tasks and tests; approved merges were verified and unapproved merges refused.
- [x] Evidence: `docs/evidence/issue9-native-manual-links-package-20261023.json`; package binary SHA-256 `60c1295a7cae1d4341ceb0370e6654a64ff7144eef12533c74e394d5e56b111d`; archive SHA-256 `0b05c48941b7e5de6910b8506932d1c459046ec8136a140dbb1cc61716b4bea2`.
- [x] UI: exact v0.1.36 macOS arm64 package link interaction is verified in `docs/evidence/issue9-package-manual-link-ui-20261023.json`; the bundled manual rendered and its official HTTPS link opened in Safari. This adds no workflow acceptance; Windows and other release gates remain open.

### Restore safe links in the in-app manual — issue #9 / R-144 / 2026-10-23

- [x] Plan: compare the app-menu manual behavior with R-144 and UC-134; keep the static bilingual HTML manual generator unchanged.
- [x] Red: add an interaction regression for a safe external link and a `javascript:` link; the safe-link test failed because the native viewer rendered both as plain text.
- [x] Task: reuse the manual renderer's URL allowlist and make links clickable in native paragraphs, lists, table headers, and cells; unsafe targets stay plain text.
- [x] Verify: interaction coverage clicks all four safe-link contexts and checks the unsafe-scheme path. 11 tests pass across `native_ui_manual`, `manual_renderer`, and `source_layout`; warning-denied focused Clippy, rustfmt, and `git diff --check` pass. All changed Rust/test files remain below 300 lines.
- [x] Complete: Wiki and focused evidence are updated. CUA capture now works, but no screenshot was retained from the user's saved workspace. The exact-package safe-link interaction is recorded in the package follow-up above; it adds no workflow acceptance.

### Synchronize the issue #9 PR-head checkpoint — 2026-10-23

- [x] Plan: compare the live issue and PR head with the current branch and the checkpoint regression.
- [x] Red: update the regression to require live PR head `cba882af115dacbefadb8a137802bb57f957e78a`; it failed while the audit still named `b6b44f7c`.
- [x] Task: synchronize the audit and Wiki checkpoint with the live PR head; keep acceptance at 14/25 and shipped/default Rust UI at 0/25.
- [x] Verify: 14 tests pass across `issue9_current_status`, `public_docs`, and `source_layout`; rustfmt and `git diff --check` pass.
- [x] Complete: PR #22 description now reports the verified head/tests. GitHub returned no check runs/statuses, so this is local verification only. Evidence: `docs/evidence/issue9-pr-head-status-cba882af-20261023.json`.
- [ ] Continue: no packaged UI workflow credit; Windows, screenshot, accessibility/IME, resource, merge, and release gates remain open.

### Synchronize issue #9 with the current PR head — 2026-10-23

- [x] Plan: compare the GitHub issue body, PR #22 body, checked-out branch and live PR-check response.
- [x] Red: add a public-doc regression requiring the current PR SHA and keep the 14/25 packaged, 0/25 shipped boundary explicit; the focused test fails before the audit update.
- [x] Task: correct the stale PR SHA in issue #9 and record the verified head in the completion audit and native UI migration Wiki.
- [x] Verify: run the focused checkpoint test, public-doc/source-layout suites, formatting and diff checks.
- [x] Complete: AgentMemory `mem_mvkiybqb_8dae78213b95`; Graphify refreshed to 9,013 nodes / 15,367 edges. CocoIndex is not installed in this runtime. The PR-check endpoint returned no checks or statuses for this head; that is not a pass. Packaged workflow acceptance remains 14/25.

### Full native-feature regression after Claude quota fix — issue #9 / 2026-10-23

- [x] Plan: run every locked/offline all-target test at the latest Rust product-code commit `f43882095aa0308dc2870bd34269e0ea240bab8d`; use release artifacts to avoid rebuilding the removed debug cache.
- [x] Verify: `cargo test --offline --locked --release --all-targets --features native-ui-probe -- --test-threads=1` passes **938 tests, 14 ignored, 0 failed across 310 test binaries** in 351.22 seconds. Exact checked-out documentation head: `8a46c8a0181a5d27440a309d9f68e9420037c858`.
- [x] Evidence: `docs/evidence/issue9-full-regression-f438820-release-20261023.json`.
- [ ] Acceptance: this source regression does not add packaged UI workflow credit; acceptance remains 14/25. Windows, manual screenshots, accessibility/IME, resource, and release gates remain open.

### Resolve Claude quota refresh from the installed CLI path — issue #9 / 2026-10-23

- [x] Plan: reproduce the exact-package Claude quota error (`No such file or directory`) with a deterministic CLI fixture; do not query a real account or send a generation request.
- [x] Red: on macOS, use a bare executable name present only in the application PATH while the CLI's own PATH omits it. Before the fix, the collector closes before reporting quota.
- [x] Task: resolve the configured Claude executable with the shared filesystem-only resolver before passing it to `/usr/bin/script`; preserve the configured name as fallback.
- [x] Verify: the focused `claude_prompt` regression passes after failing before the change. `cargo fmt --all -- --check`, `git diff --check`, warning-denied focused Clippy, and the 300-line source/test limit pass.
- [x] Package: pushed source `f43882095aa0308dc2870bd34269e0ea240bab8d` builds as a macOS arm64 v0.1.36 candidate. Package validation passes; the 214-component manifest has no missing license expressions. Built-in package smoke passes four isolated shared/worktree × Git/folder cases with three parallel mock tasks/tests each and refuses unapproved merges.
- [x] Runtime: the packaged binary's matching SHA-256 queried the signed-in Claude CLI with the app PATH reduced to `/usr/bin:/bin`; exit 0, two quota windows persisted, and no usage values were recorded. No generation or tools ran.
- [x] Knowledge: AgentMemory `mem_mvkgfbu4_c8be2c1ac746`; Wiki entry `knowledge/wiki/claude-quota.md`; package evidence `docs/evidence/issue9-claude-quota-f438820-macos-package-20261023.json`.
- [x] UI: in the exact rebuilt v0.1.36 macOS arm64 package, clicked Claude's refresh button in a disposable workspace. The busy indicator appeared, then cleared; the displayed observation time advanced from 13:21:10 to 13:22:30 and the control became available again. The UI showed the refreshed quota summary. No screenshot was retained because it exposed account usage and a local project path.
- [x] Scope: this verifies the quota-refresh control only; it is not a distinct #9 workflow, so packaged workflow acceptance remains 14/25. Historical manual screenshots, Windows, and remaining product acceptance gates remain open.

### Diagnose PR #22 mergeability against latest main — issue #9 / 2026-10-23

- [x] Plan: fetch latest `origin/main` (`b7fa27a9`) and compare it with Rust head `fe34aa0d` without changing the working tree.
- [x] Verify: shared base is `e9d1f1ff`; the Rust branch is 656 commits ahead and 15 behind. `git merge-tree --write-tree origin/main HEAD` reports conflicts in requirements/design/plan docs, manuals/Wiki, Rust coordinator/UI/quota/workflow paths, tests, and main-edited legacy UI files deleted or split in the Rust tree.
- [x] Preserve: the merge-tree inspection left the worktree clean; no merge/rebase was attempted. Latest-main behavior parity is separately verified by 24 focused tests.
- [x] Evidence: `docs/evidence/issue9-main-mergeability-b7fa-20261023.json`.
- [ ] Complete: keep PR #22 draft until conflict resolution preserves Rust-only modular sources and reconciles the release version; do not merge main wholesale.

### Verify native UI accessibility, keyboard and contrast regressions — issue #9 / 2026-10-23

- [x] Plan: run the existing focused Rust UI regression suites at source head `f5a7ed6d` without opening or changing an application window.
- [x] Verify: `native_ui_tasks`, `native_ui_mcp_keyboard`, and `native_ui_profile_contrast` pass 15 tests total. Coverage includes localized task-editor names, overlay layout, MCP keyboard save behavior, and 4.5:1 profile/quota contrast in light and dark themes.
- [x] Evidence: `docs/evidence/issue9-native-ui-accessibility-keyboard-20261023.json`.
- [ ] Acceptance: automated source tests do not establish native screen-reader behavior or Korean IME behavior; the packaged workflow count remains 14/25.

### Verify current PR-head documentation and source layout — issue #9 / 2026-10-23

- [x] Plan: verify the checked-out and pushed head `608eb741` after the macOS package/license checkpoint.
- [x] Verify: `cargo test --offline --locked --release --test public_docs --test source_layout` passes 13 tests; `cargo fmt --all -- --check` and `git diff --check` pass.
- [x] Evidence: `docs/evidence/issue9-current-head-doc-layout-20261023.json`.
- [ ] Scope: this is documentation/layout verification only; it does not close packaged UI, Windows, accessibility, Korean IME, resource, or release gates.

### Build and verify the current macOS package and license bundle — issue #9 / 2026-10-23

- [x] Plan: build from the exact pushed branch head `88e372e95a8c2507e15c1c8b24743bf7d1f9b38a`; product-code change since `9845db79` is `eb0bfce` (manual-image packaging).
- [x] Task: run `package-release` for `aarch64-apple-darwin`; package version remains 0.1.36.
- [x] Verify: package validation succeeds; the SBOM contains 214 components with no missing license expressions. The `self_cell` Apache-2.0 option and both upstream license texts are bundled.
- [x] Verify: exact-package smoke passes four isolated combinations (shared/worktree × Git/folder), each with three parallel tasks and three passing tests; approved merge succeeds and unapproved merge is refused in disposable repositories.
- [x] Evidence: `docs/evidence/issue9-current-head-macos-package-license-20261023.json`; archive SHA-256 `6019a0ff883c5713e519c9932153c14bc1e793918265a80aa1c773efc5868894`.
- [ ] Release gate: do not publish this package. It is v0.1.36, contains historical manual screenshots, and has no Windows package/UI/license verification.

### Re-verify current Rust source layout and public docs — issue #9 / 2026-10-23

- [x] Plan: verify the exact current branch head after the Windows-access documentation commit.
- [x] Verify: `cargo test --offline --locked --release --test public_docs --test source_layout` passes 13 tests across both suites; `cargo fmt --all -- --check` passes.
- [x] Evidence: `docs/evidence/issue9-current-head-layout-doc-tests-20261023.json` records source head `acf2c322f0087a684703263114902bb7e07a49f7`.
- [x] Complete: the docs and source/test layout guards pass at the current head. This does not change packaged UI acceptance (14/25), shipped/default Rust UI acceptance (0/25), or any Windows, screenshot, accessibility, IME, resource, or final-license gate.

### Probe local Windows VM access — issue #9 / 2026-10-23

- [x] Plan: check the existing UTM Windows ARM64 VM without entering credentials or touching project files.
- [x] Task: boot the stopped VM to its lock screen; stop the probe when no authenticated desktop was available.
- [x] Verify: UTM returned to the stopped state with Start enabled and Stop disabled; no build, package, or UI test ran. Evidence: `docs/evidence/issue9-windows-vm-access-20261023.json`.
- [ ] Complete: Windows package/runtime/UI verification still needs an authorized guest login or another Windows build host. Keep the VM stopped to return its configured 10 GB RAM to the host.
- [ ] Acceptance: Windows gates remain open; packaged native UI acceptance remains 14/25.

### Resume Windows verification in UTM — issue #9 / 2026-10-23

- [x] Plan: use the existing UTM Windows ARM64 VM after the selected Windows App device `neodamul-PC` failed with `0x104`; do not enter account credentials.
- [x] Task: resume the paused VM. Windows recovery reported a network failure and `SrtTrail.txt`, then reached the Windows sign-in screen. No password was entered.
- [x] Verify: the exact Windows project build and UI remain unverified. UTM warned that Stop could corrupt guest data; cancel Stop and pause the VM instead. Evidence: `docs/evidence/issue9-windows-utm-login-20261023.json`.
- [ ] Next: after the operator signs in to the same VM, run exact-source Windows build, package, runtime, and UI checks. Do not claim Windows acceptance before those checks.
- [ ] Resource note: the paused VM may retain its configured 10 GB of host memory.

### Replace the stale Agent Room screenshot in the packaged manuals — issue #9 / 2026-10-23

- [x] Plan: inspect the built-in manual in the exact v0.1.36 macOS package without saving workspace changes.
- [x] Verify defect: both English and Korean manuals reference `docs/assets/ui-agent-room-native.jpg`, which visibly identifies itself as v0.1.24. Existing `ui-agent-room-current.png` and `ui-agent-room-dark-current.png` are v0.1.21; the other inspected screenshots are older or design mockups.
- [x] Preserve fixtures: do not alter the existing in-progress `Issue 9 Mock Workflow` or historical package-review fixtures; restore the user's original workspace after read-only inspection.
- [x] Task: complete a separate disposable three-task run in the exact package, inspect its completed Agent Room, review its three-file diff and passing checks, and approve/merge only into the disposable repository. Candidate `1e263295ee0a1e2b0c4ae17690613bd45d3b831d` was merged into base `3e71b09b799eb9c962e150619ec0945efa3f76e6`; the fixture and temporary repository were removed and the user's original workspace restored. This duplicates an existing workflow and adds no acceptance credit.
- [x] Verify current head: package source `6200e49b2d9f7cef5f11c56e96adeb6c840ed234` completed a fresh isolated three-task/three-test fixture; the exact v0.1.36 Agent Room visibly showed all tasks and tests passing, with the candidate still waiting approval. No approval or merge occurred. CUA showed the image; even the correct 1140×772 CGWindow ID failed with `screencapture: could not create image from window` (`kCGWindowSharingState` 0). Evidence: `docs/evidence/issue9-6200e49b-current-agent-room-20261023.json`.
- [x] Task: capture the current Rust Agent Room from source commit `25556157` with the opt-in `manual-screenshot-capture` feature after the disposable three-task run snapshot loaded. The screenshot shows 3/3 tasks complete, 3/3 task tests passing, and the candidate waiting for approval; no approval or merge occurred.
- [x] Verify: replace the shared screenshot in both Markdown manuals, regenerate both HTML manuals, and validate the embedded image and checksum. Evidence: `docs/evidence/issue9-native-agent-room-manual-screenshot-20261023.json`.
- [x] Scope: this development-build screenshot is documentation evidence only; it does not add packaged UI acceptance. Keep packaged UI at 14/25.

### Measure exact-package idle resource use — issue #9 / NFR-05 / 2026-10-23

- [x] Plan: observe the already-running single v0.1.36 macOS arm64 package process; do not start a second window or invoke an agent.
- [x] Verify: four native `top` samples at five-second intervals report 79 MiB RSS and 0.0% CPU each. The executable hash matches the retained exact-package manifest; one AgentConductor process exists and it has no child process.
- [x] Evidence: `docs/evidence/issue9-9845db79-idle-resources-20261023.json`.
- [ ] Acceptance: this 15-second idle macOS sample meets NFR-05's <100 MiB RSS / <1% CPU thresholds for this state only. Startup, longer idle duration, active work, Windows, and the final release source remain unverified.

### Remove the idle generated debug cache — issue #9 / 2026-10-23

- [x] Plan: inspect current build activity and keep the release profile and packaged binary intact.
- [x] Task: after confirming no Cargo/Rust build process was active, remove only the regenerable `target/debug` tree (9.0 GB); retain `target/release` and the extracted macOS release package.
- [x] Verify: `target/` fell from 9.7 GB to 731 MB. The retained exact-package executable remains present and `--help` succeeds. No source or user data was removed.
- [x] Evidence: `docs/evidence/issue9-target-debug-cache-cleanup-20261023.json`.
- [x] Assetize: Graphify refreshed the local code graph to 8,970 nodes, 14,140 edges, and 692 communities. CocoIndex is not installed in this runtime; no index result is claimed.
- [ ] Risk: the next debug test/build will recompile dependencies and take longer; this cleanup does not affect release artifacts or verification claims.

### Complete the current-head native-feature regression — issue #9 / 2026-10-23

- [x] Plan: rerun the full locked, offline native-feature suite on the current branch head; do not treat the earlier interrupted run as completion evidence.
- [x] Verify: `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` passes **938 tests, 14 ignored, 0 failed across 310 test binaries** (363.89 seconds). Warning-denied all-target Clippy, `cargo fmt --all -- --check`, and `git diff --check` also pass.
- [x] Evidence: `docs/evidence/issue9-full-regression-20261023.json`; tested docs head `12d6d808`, product-code head `9845db79af260b631bde7ade2104bae2400cc649`.
- [ ] Acceptance: this is source-level regression evidence only. Packaged UI remains 14/25; shipped/default Rust UI remains 0/25. Windows package/runtime/UI, accessibility and Korean IME, resource measurements, current manual screenshots, and final-source license review remain open.

### Review the settings overlays in the exact macOS package — issue #9 / 2026-10-23

- [x] Plan: inspect the existing single v0.1.36 package window without changing workspace data or invoking provider tools.
- [x] Verify: at 1140×768, the workspace menu overlays the page without moving it; Settings → MCP registration and workspace sharing show no visible text overlap in Korean dark theme. No action or setting was saved.
- [x] Knowledge: AgentMemory `mem_mvkd4i5n_771f4210cdac`; `knowledge/wiki/native-settings-dialogs.md` records the component contract.
- [ ] Acceptance: this is component-level visual evidence only. No screenshot file was retained and no packaged workflow was added; current manual screenshots, Windows UI, accessibility/IME and remaining issue #9 gates stay open.

### Review the agent-tools installation guide in the exact macOS package — issue #9 / 2026-10-23

- [x] Plan: inspect the existing v0.1.36 package window without switching workspaces or copying commands.
- [x] Verify: the workspace menu exposes Agent tools separately; its dialog lists AgentMemory, Ponytail, RTK, Graphify, and CocoIndex commands. At 1140×768, the explanatory text, all commands, and both close affordances are visible without clipping or overlap.
- [x] Preserve state: close the dialog and leave the saved `헬스체크 기능 개발` workspace and settings unchanged.
- [ ] Acceptance: this is component evidence only. No screenshot was retained and no packaged workflow was added; acceptance remains 14/25. Windows UI, accessibility/IME, resources, current manual screenshots, and final-source license review remain open.

### Reconcile the LLM Wiki packaged-workflow checkpoint — issue #9 / 2026-10-23

- [x] Plan: compare the Wiki's current acceptance count and latest package links with the implementation plan and retained workflow evidence.
- [x] Task: update the Wiki checkpoint from stale 12/25 to 14/25 and add the exact Workflow 13 and 14 evidence references.
- [x] Verify: public-doc/source-layout tests pass 13/13; rustfmt and `git diff --check` pass. Graphify confirms the packaged-workflow page and workflow 13 nodes, though the retained graph predates this documentation update.
- [x] Assetize: save the discrepancy and evidence boundary in AgentMemory; no workflow count is added by this documentation correction.

### Reconcile current UI metrics across the completion audit and migration Wiki — issue #9 / 2026-10-23

- [x] Plan: compare the current package, full-suite evidence, packaged-workflow Wiki, and R-12/R-13/R-17 audit rows.
- [x] Task: advance the current audit rows to package source `9845db79`, test head `12d6d808`, and 14/25 packaged UI workflows; replace the stale Workflow 13 “not yet rerun” statement with its exact-package evidence.
- [x] Verify: `public_docs` and `source_layout` pass 13 tests; `git diff --check` passes.
- [x] Complete: no product code or acceptance count changed; AgentMemory records this consistency repair.

### Preserve the default company-webhook template in the native editor — issue #9 / workflow 13

- [x] Plan: preserve `messenger::payload`'s documented default body when an unset template is first rendered in Settings → Messenger.
- [x] Red: exact v0.1.36 package `2ef5c3b` reproduced `message body is not JSON: EOF while parsing a value` after selecting the company webhook and sending a test message; the injected local transport received no request.
- [x] Task: initialize the editor with the existing `{"text":"{{text}}"}` default instead of converting `None` to an empty template; retain the current payload renderer and settings contract.
- [x] Verify: the new rendered-value regression failed before the fix and passes after it; the production payload parser accepts the rendered default as a JSON object. Messenger 2/2, source-layout 2/2, public-docs 11/11, warning-denied all-target Clippy, rustfmt, and diff checks pass. Changed files remain below 300 lines.
- [x] Knowledge: Wiki and AgentMemory record the regression and the no-network reproduction boundary.
- [x] Verify package: exact source `9845db79af260b631bde7ade2104bae2400cc649` builds and packages for `aarch64-apple-darwin`; built-in package smoke, `validate-package`, packaged `--help`, ZIP integrity, and archive SHA-256 pass. Binary SHA-256: `0c4e1fa888cc18d13da40f81683032686829a79ea5411250a2b4ab59c8397a23`; archive SHA-256: `6bba2a52980147a01402949749789d6d49a615406a9399b5bdd9b449724b1570`.
- [x] Verify UI: run the exact `9845db79` package in one isolated app window and isolated HOME; select Company messenger, confirm the JSON default, and click Send test message. The UI reports “Test message sent.” The injected fake curl captured `https://127.0.0.1/ac-issue9-webhook` and JSON body `{"text":"AgentConductor · : test message"}`; no network request or settings save occurred. CUA screenshot was visually inspected but not archived as a file.
- [x] Complete workflow 13: packaged UI acceptance is 13/25 (52%); 12 remain. Shipped/default Rust UI replacement is still 0/25, and Windows/platform gates remain open.

### Exact macOS package cache cleanup UI workflow — issue #9 / workflow 14

- [x] Plan: verify the native cache maintenance flow in the exact v0.1.36 package, with disposable metadata only and an isolated HOME.
- [x] Task: preview a synthetic terminal quota-query cache entry, inspect its exact path/count and protected-entry summary, then clear it only through the reviewed UI action.
- [x] Verify: the exact package showed one eligible fixture and zero protected/uncertain entries, then reported “Removed 1 cache entries · 0.0 MiB.” The fixture directory was absent afterward and the isolated `startup.json` remained present (938 bytes).
- [x] Evidence: `docs/evidence/issue9-9845db79-cache-cleanup-ui.json`; exact app source `9845db79af260b631bde7ade2104bae2400cc649`, binary SHA-256 `0c4e1fa888cc18d13da40f81683032686829a79ea5411250a2b4ab59c8397a23`, archive SHA-256 `6bba2a52980147a01402949749789d6d49a615406a9399b5bdd9b449724b1570`.
- [x] Complete workflow 14: packaged UI acceptance is 14/25 (56%); 11 remain. Shipped/default Rust UI replacement remains 0/25; Windows/platform gates remain open.

### Reconcile upstream main v0.1.37 with the Rust migration branch — 2026-10-23

- [x] Plan: compare the fetched `origin/main` (`b7fa27a9`, v0.1.37) with the issue-9 branch and verify relevant code changes before porting or merging.
- [x] Task: confirm the Rust equivalents for #149, #156, #162, and #165 remain present; retain the modular Rust-only architecture and 300-line source/test guard instead of merging the upstream refactor wholesale.
- [x] Verify: focused coordinator, quota-display, documentation, and source-layout tests pass 38/38 across seven suites; formatting and diff checks pass. The Rust branch Cargo version remains 0.1.36 while main is 0.1.37.
- [x] Knowledge: LLM Wiki and AgentMemory `mem_mvkatgcf_4f73ba8f24bd` record the release-version gap and merge boundary. Graphify integrity check found 8,916 nodes, 14,081 edges, and no dangling endpoints; the queried code-to-Wiki path was absent. CocoIndex is unavailable because its runtime lacks `mcp.server.fastmcp`.
- [ ] Merge/release preparation: reconcile v0.1.37 requirements/manual/changelog changes and resolve PR #22 conflicts after packaged workflow and platform acceptance gates; keep PR #22 draft.

### Reconcile coordinator guardrail #149 completion evidence — 2026-10-23

- [x] Plan: verify all four operator-approved steps against the Rust branch, main merges, exact-package evidence, and the issue comments before changing completion status.
- [x] Task: confirm decision visibility, deterministic launch gate, bounded/reversible amendments, and persisted charter/drift checks are implemented; assetize the final contract in `knowledge/wiki/typed-coordinator-amendments.md`.
- [x] Verify: the current branch includes the #149 Rust commits; focused amendment/charter suites passed 28 tests at `43463f4b`, source layout is 2/2, warning-denied Clippy, formatting, and the 300-line guard passed. macOS exact-package goal-drift refusal is recorded in `docs/evidence/issue9-aa80904-goal-drift-workflow.json`.
- [x] Verify this documentation update: 36 tests across eight focused suites pass, including `public_docs` and `source_layout`; `cargo fmt --check` and `git diff --check` pass.
- [x] Knowledge: AgentMemory `mem_mvkanf0j_cf143e9725cf` records the completed code path and acceptance boundary; the Wiki is synchronized; sync #9, #149, and PR #22 after this documentation commit.
- [ ] Final acceptance: Windows package/runtime and UI verification remain open. Keep #149 open until that supported-platform gate is verified; issue #9 remains 12/25 packaged workflows and 0/25 shipped/default Rust UI replacement.

### Reconcile issue #9 evidence and check local Windows verification options — 2026-10-23

- [x] Plan: compare the issue, PR, package evidence, test output, and current host capabilities before claiming cross-platform acceptance.
- [x] Task: update the current R-12–R-17 and R-23–R-24 audit rows to the latest exact macOS package, 12/25 UI acceptance, and 938-test result; record the local Windows toolchain check in `docs/evidence/issue9-windows-local-toolchain-20261023.json`.
- [x] Verify: the exact branch and origin head matched at `084089fb`; `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` passed 938, 14 ignored, 0 failed across 310 suites. Warning-denied Clippy, rustfmt, diff checks, and the focused public-doc/source-layout suites passed locally.
- [x] Knowledge: Graphify query connected the current quota-fallback evidence to the quota Wiki and completion audit. CocoIndex is unavailable on this host; no index result is claimed.
- [ ] Windows: local `cargo check --target x86_64-pc-windows-gnu` cannot find `core`/`std`; `rustup` and MinGW/Zig are unavailable, and the UTM Windows VM is stopped. Verify the exact source on a Windows build host before claiming Windows acceptance; no remote CI was run.
- [ ] Complete: Windows package/runtime/UI, accessibility and IME, resource measurement, current manual screenshots, final-source license closure, and full-product acceptance remain open. Keep #9 open and PR #22 in draft.

### Exact macOS package quota-fallback UI workflow — issue #9 / 2026-10-23

- [x] Plan: exercise the exact ca9495b2 macOS package with three independent tasks, an explicit restricted primary profile, a configured local fallback, scoped file ownership, tests, and human review.
- [x] Task: in a disposable shared Git repository, every quota-agent attempt emitted usage_limit_reached with a reset time; all three tasks transferred to mock on attempt 2 and wrote only their owned files.
- [x] Verify: the packaged UI showed 3/3 completed tasks, all three quota-agent → mock handoffs, 3/3 passing git diff --check results, and candidate 9ca548243128d007e8678d64cf4df6654d1a9230. The inspected diff contained only one.txt, two.txt, and three.txt.
- [x] Complete: approved the exact candidate in the UI, refreshed approval-bound evidence, merged in the UI, and verified the run completed at stage 05 with a clean disposable repository. Exact package source and archive hash are recorded in docs/evidence/issue9-ca9495b2-quota-fallback-ui.json.
- [x] Acceptance: packaged native UI operator acceptance advances from 11/25 (44%) to 12/25 (48%); 13 remain. No real provider CLI or external service ran.
- [ ] Remaining gates: Windows UI, Korean IME, accessibility parity, resource measurements, current manual screenshots, and final-source license review.

### Revalidate source-layout gate and reclaim idle build cache — issue #9 / 2026-10-23

- [x] Plan: recheck the Rust-only source/test boundary and 300-line limit after the exact macOS package and Windows VM probe; keep GUI/package acceptance separate from headless checks.
- [x] Verify: `cargo test --offline --locked --test source_layout -- --test-threads=1` passes 2/2; Rust source/test inventory has no file above 300 lines. `public_docs` passes 11/11.
- [x] Cleanup: with no active Cargo/Rust compiler, remove only `target/debug/incremental` and `target/debug/tmp`; the measured target falls from 11 GB to 8.7 GB. Preserve debug dependencies, release output, the running v0.1.36 app, and packaged artifacts.
- [x] Knowledge: AgentMemory `mem_mvk94ev1_7dfbd3b09cfb` records the measured sizes, preserved artifacts, and rebuild cost.
- [ ] Acceptance: cleanup and source-layout verification add no packaged UI workflow; acceptance remains 11/25 (44%). Exact-head GUI workflow, Windows UI, accessibility/IME, resource, and final license checks remain open.

### MCP registration dialog component parity — issue #9

- [x] Plan: compare the native Settings → MCP servers window with the existing MCP dialog; retain its localized section eyebrow and registration heading while keeping the native window title distinct.
- [x] Red: an English/Korean rendered-text regression failed because the section eyebrow was absent and Korean repeated “MCP server registration” as both window and content title.
- [x] Task: render the shared Settings · MCP servers eyebrow and distinguish the window title as MCP server settings; preserve the existing editor, discovery, save, and close behavior.
- [x] Verify: 11 tests pass across `native_ui_mcp` and `native_ui_settings_journey`; `cargo clippy --locked --offline --features native-ui-probe --all-targets -- -D warnings` and `cargo fmt --check` pass. Changed Rust/test files remain below 300 lines.
- [x] Exact-package visual confirmation: source `cff95dd02677519ebbb7c359f5ca9e3da0c504ed`, v0.1.36 macOS arm64 archive SHA-256 `dafc056361c6e68a1165f30db1f3bd8a0cf99d24b59876d7d6eb7d2aa1ad44d8`; inspected Korean/dark, English/dark, and English/light. The saved user workspace was not changed, and the pending candidate was not approved or merged. Evidence: `docs/evidence/issue9-cff95dd-mcp-dialog.json`.
- [ ] Screenshot asset was inspected in-session but not retained. This presentation fix adds no packaged workflow; acceptance remains 11/25 (44%), with 14 workflows remaining.
- [x] Knowledge: AgentMemory `mem_mvk7geva_5b15ec7aa847`; Graphify confirmed the MCP editor state/event module. CocoIndex is unavailable in this environment.
- [x] Knowledge: `knowledge/wiki/native-settings-dialogs.md` records the component contract. AgentMemory: `mem_mvk73wbj_5f33afab5ffd`. Graphify inspected the editor node and its module edges; read-only graph diagnosis reports 8,916 nodes, 14,081 edges, and no dangling endpoints. CocoIndex is not installed in this environment.

### Remove stale README screenshots while preserving offline manual images — issue #9 / 2026-10-23

- [x] Plan: public README screenshots showed v0.1.21/v0.1.24 screens or a prepared Agent Room demo, not the current exact package. Keep the README text-only until current captures can be retained.
- [x] Red: the README image regression failed before removal. The release-package fixture then failed because `referenced_assets` discovered manual images only through README links.
- [x] Task: remove README image embeds; independently collect and path-validate local assets linked by the English and Korean manuals so offline HTML/package images remain present.
- [x] Verify: release-package, public-docs and source-layout suites pass 16 tests; warning-denied all-target Clippy, rustfmt, diff checks and the 300-line guard pass. `src/release_package.rs` is 260 lines and changed test files are below 300 lines.
- [x] Knowledge: AgentMemory `mem_mvk6p1gq_83dadc591c2f`; screenshot provenance Wiki updated. Graphify query of `referenced_assets` confirmed its caller is `assemble_with_binary`; retained the validated curated graph. Manual assets remain in the package despite no README image links. CocoIndex is unavailable; no index result is claimed.
- [ ] Current screenshots: v0.1.21/v0.1.24 manual assets still need replacement by retained captures from the exact Rust package. This documentation correction adds no workflow; packaged UI acceptance remains 11/25 (44%).

### Exact v0.1.36 Agent Room light-theme visual check — issue #9 / 2026-10-23

- [x] Verify: in the existing single packaged macOS window, inspect the Korean Agent Room in light theme. The approval banner, three completed task rows, candidate SHA, and three passing test exit codes are visible.
- [x] Safety: candidate `3db8840b02ce28558701698c327a5820b7dc7311` remains `waiting_approval`; no approve, reject, or merge action occurred. Restore dark theme and return to the saved user workspace; its name, goal, and project path remain intact.
- [x] Evidence: `docs/evidence/issue9-f1602e7-agent-room-light-theme.json`. This component visual check does not add a distinct workflow; acceptance remains 11/25 (44%). Screenshot was inspected in-session but not retained as a file.
- [x] Knowledge: AgentMemory `mem_mvk6fhw8_8eff3cddc6fc` records the visual check; follow-up `mem_mvk6ikgc_355d84efc620` records the Graphify integrity finding. The no-cluster extraction had 1,271 dangling edges, so restore the curated snapshot; its diagnostic passes with 8,916 nodes, 14,081 edges, and zero dangling endpoints. CocoIndex CLI is unavailable; no index result is claimed.
- [ ] Windows Agent Room visual verification remains open.

### Remove duplicated goal from the Agent Room task list — issue #9 / R-17 / DD-137 / 2026-10-23

- [x] Plan: the exact v0.1.36 Rust screenshot repeats the full goal in the wide page header and narrow task-summary column; retain one prominent goal and keep the task list, verification and human-review controls.
- [x] Red: the rendered AccessKit bounds showed goal rows at y=60 and y=220 before the fix.
- [x] Task: suppress root/goal/run-summary context only in the Room's sidebar task list; shared Workspace snapshot rendering remains unchanged.
- [x] Verify focused: Room layout/render and shared-snapshot/source-layout suites pass 17 tests; rustfmt, diff checks and the 300-line guard pass.
- [x] Verify full: locked/offline serial all-target suite passes 929 tests, 14 ignored, 0 failed across 310 suites; warning-denied Clippy, rustfmt, diff check, 300-line guard and optimized release build pass.
- [x] Verify exact package: source `f1602e7d32022fac377a168bd2bfb6686593db02`; manifest validation and package smoke pass. In one v0.1.36 window, the goal appears once in the wide Agent Room header while the left column retains task/status and verification details. Candidate `3db8840b` remains `waiting_approval` with 3/3 tasks and tests; no approval, rejection or merge action occurred.
- [x] Knowledge: Graphify refreshed to 8,893 nodes / 14,043 edges / 683 communities. CocoIndex CLI is unavailable in this environment; no index result is claimed. AgentMemory `mem_mvk4r2u5_d3f75533fb77` records the decision and verification.
- [x] Complete: package evidence, full-product audit and LLM Wiki updated; AgentMemory saved; issue #9 and PR #22 updated; commit and push completed. This component regression does not add a packaged workflow.

### Stretch Agent Room activity cards across the review column — issue #9 / 2026-10-23

- [x] Plan: exact v0.1.36 screenshots showed activity cards wrapping to their text width even though the center review column had unused space.
- [x] Red: add `native_ui_task_activity_layout`; measure the rendered group-frame rectangle and require at least 540 px in a 620 px test window. The original renderer fails at 257 px.
- [x] Task: set each activity entry group's minimum width from its containing review UI.
- [x] Verify focused: activity, review, room-layout and source-layout suites pass (26 tests); formatting and diff checks pass. All changed Rust/test files remain below 300 lines.
- [x] Full-suite correction: `native_ui_targets` found that the new integration test lacked the `native-ui-probe` feature gate. Add the gate and verify both the no-default-feature guard and the feature-enabled activity-layout test.
- [x] Verify full: locked/offline serial all-target suite passes 929 tests, 14 ignored, 0 failed across 310 suites; warning-denied all-target Clippy, rustfmt, and optimized release build pass.
- [x] Graphify updated: 8,886 nodes, 14,036 edges, 689 communities. CocoIndex CLI/MCP is unavailable in this environment.
- [x] Verify exact package: source `1f152729e0c3babd0beb2b6338536b667ea1671c`; one v0.1.36 window shows full-width activity cards, the review column, and a bounded inspector. Candidate remains `waiting_approval` with 3/3 tasks and tests; no approval, rejection, or merge action occurred.
- [x] Complete: record package evidence in `docs/evidence/issue9-1f152729-agent-room-layout.json`, add the LLM Wiki entry, save AgentMemory, update #9/PR #22, and commit/push this record.

### Restore Agent Room review and inspector proportions — issue #9 / 2026-10-23

- [x] Plan: compare the exact Rust package at 1140 px with the shipped Agent Room screenshot. Rust gave the review column about 428 px while the inspector consumed over 420 px; the shipped screen gives review the wider center column and keeps the inspector near 300 px.
- [x] Red: require the review span to be at least 550 px and the inspector's merge control to stay within 330 px in `native_room_layout`; the previous layout fails at 428 px.
- [x] Task: reduce the summary to 18% (180–240 px) and the wide-window inspector to 24% (280–320 px). Preserve narrow-window sizing and all workflow controls.
- [x] Verify focused: room layout, task selection, language, and source-layout suites pass (9 tests); all-target warning-denied Clippy, rustfmt, and the release build pass.
- [x] Verify full: locked/offline serial all-target suite passes 928 tests, 14 ignored, 0 failed across 309 suites. Graphify now reports 8,883 nodes, 14,033 edges, and 680 communities.
- [x] Verify exact package: source `1f152729e0c3babd0beb2b6338536b667ea1671c`; one v0.1.36 window confirms the center review column and 280–320 px inspector balance at 1140 px. The candidate remains `waiting_approval`; no approval, rejection, or merge action occurred.
- [x] Complete: record exact-package evidence in `docs/evidence/issue9-1f152729-agent-room-layout.json`, the LLM Wiki and AgentMemory; update issue #9/PR #22; commit and push this record.

### Exact-package native language and status workflow — issue #9 / Workflow 11 / 2026-10-23

- [x] Plan: verify language switching in the exact v0.1.36 macOS package and confirm that Agent Room phase, candidate, task, test, and approval states remain stable.
- [x] Task: in the single packaged window, switch Settings from Korean to English, inspect the visible Agent Room labels and task status, then restore Korean.
- [x] Verify: English and Korean labels update across the shared controls, phase/status tags, review evidence, and candidate actions. The same candidate remains `waiting_approval`, 3/3 tasks and 3/3 tests; no approve, reject, or merge action occurred.
- [x] Verify current Rust source: locked/offline all-target tests pass 948 (14 ignored) across 310 suites; warning-denied all-target Clippy, rustfmt, `public_docs`, `source_layout` (13 tests), and diff checks pass.
- [x] Evidence: `docs/evidence/issue9-6f1111c-language-status-workflow.json`; exact package archive SHA-256 `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`.
- [x] Knowledge: Graphify refreshed to 8,879 nodes / 15,210 edges. CocoIndex has no installed CLI or MCP tool in this environment.
- [x] AgentMemory: recorded the package interaction and full-source verification as `mem_mvk21hgy_bfdbd9eda1d9`.
- [x] Acceptance: distinct packaged UI workflow 11/25 (44%); 14 remain. Windows, Korean IME, accessibility parity, and shipped/default UI replacement remain open.

### Reconcile latest upstream main without restoring non-Rust code — issue #9 / 2026-10-23

- [x] Plan: compare current `origin/main` at `b7fa27a9` with shared base `e9d1f1ff`; 15 upstream commits now follow the base, four since the prior `c9c575a7` audit.
- [x] Compare: the only behavior change after the prior audit is `4195f64c`, a 24-second bounded retry for transient coordinator lease conflicts. The Rust equivalent is already in `f7dccadb`. Other new files update the v0.1.37 release metadata/manuals.
- [x] Release version: keep the Rust branch at 0.1.36 until migration acceptance; select the next stable release version at final package preparation rather than importing main's release-only version bump early.
- [x] Preserve constraints: do not merge upstream's broad refactor; `origin/main` contains 47 project-owned JS/CJS/Python files and consolidates Rust modules, while this branch contains zero such files and enforces the 300-line split.
- [x] Preserve unique architecture trace: retain R-172 for #156 and R-173 for #165. Upstream's #165 renumber to R-172 collides with the already-traced #156 requirement in this branch.
- [x] Verify: `coordinator_amendment_contract`, `charter_contract`, `coordinator_request`, `native_coordinator_feedback`, `quota_display`, and `source_layout` pass 24 tests across six suites at Rust head `2fed27cc`.
- [x] Evidence: `docs/evidence/issue9-main-parity-b7fa-20261023.json`.
- [x] Complete: keep the Rust-only modular branch; no source behavior or packaged UI acceptance count changed. #149 remains open until the Rust PR is accepted.

### Remove duplicate application branding from the Agent Room — issue #9 / FR-138 / DD-160

- [x] Plan: the exact v0.1.36 screen showed the shared shell and Agent Room both rendering the AC mark, app name, and version; preserve the Room's workspace, phase, authentication, and action controls.
- [x] Red: update `native_ui_room_header` to reject repeated shared branding; both rendered regressions fail before the fix.
- [x] Task: remove only the duplicate AC mark, application name, and version from the Room context bar.
- [x] Verify: focused Room-header/titlebar tests pass (3); formatting and diff checks pass; optimized release binary builds.
- [x] Package: exact commit `6f1111c5ae61c91933e03a20fb04a4df513193de` was packaged and launched from its extracted archive. Archive SHA-256: `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`. CUA captured the exact v0.1.36 Agent Room: shared app branding appears once; the candidate review shows SHA `3db8840b02ce28558701698c327a5820b7dc7311`, all three file diffs, and three tests with exit code 0. The disposable fixture remains at approval pending. Approval was enabled after evidence review but was not clicked; no merge action was taken.
- [x] Complete source/evidence update: LLM Wiki, Graphify, AgentMemory, #9/PR #22, and commits `6f1111c` / `495d2cf` are updated and pushed. This component visual check does not add a workflow or change packaged acceptance from 10/25 (40%).

### Port coordinator-directed operator feedback — GitHub #165 / issue #9 / R-173 / DD-184

- [x] Plan: inspect current `main` at `c9c575a7`; its #165 workflow is not in the Rust branch. Preserve native eframe UI, the existing coordinator CLI/review worker, charter validation, and audit-derived judgment model.
- [x] Requirements: trace R-173 → BG-128 → SF-161 → UC-164 → FR-159/NFR-69 → QAS-170 → DDR-33 → DD-184; selecting the coordinator must route independently from task feedback, keep separate drafts, expose all lifecycle states, avoid unrelated task restarts, and preserve exact human approval/merge.
- [x] Red: add workflow and rendered native regressions for bounded operator-request propagation, lifecycle audit, recipient display, and completed-task attempt preservation; observed the missing review-export compile failure before implementing it.
- [x] Task: implement the coordinator-request CLI operation and native action using the existing review process, audit stream, and validator.
- [x] Verify: all 920 local tests passed (14 ignored, 310 suites, 369.64 seconds); warning-denied Clippy, formatting, and the 300-line source-layout checks passed. The one UI settings test that failed in parallel passed alone and in the serial full run.
- [x] Regression follow-up: reproduce the room briefly reacquiring its lease between the preflight check and review; retry only `lease is held` within the original request deadline, and fail fast for unrelated review errors.
- [x] Latest-main parity: `4195f64c` caps post-preflight lease-race retries at 240 × 100 ms. Keep the shared request deadline and enforce the same cap in Rust so a long configured timeout cannot hold the request for 30 minutes on repeated lease conflicts.
- [x] Red: the bounded-retry regression failed to compile because `retry_review` had no retry-budget contract.
- [x] Task: cap only transient `lease is held` retries at 240; retain fail-fast handling for unrelated errors and the request-wide deadline.
- [x] Verify focused regressions: retry unit tests (3), coordinator workflow tests (3), source-layout tests (2), and CLI workflow tests (2) pass; warning-denied all-target Clippy, rustfmt, and `git diff --check` pass.
- [x] Verify full suite: `cargo test --locked --offline -- --test-threads=1` passed 923 tests, 14 ignored, 0 failed across 310 suites in 367.20 seconds.
- [x] Verify latest-main retry-budget regression and the completed package-license fixture: full native-feature suite passed 938 tests, 14 ignored, 0 failed across 310 suites; warning-denied all-target Clippy passed.
- [x] Package UI: build and validate exact source commit `b298ae79774e`; inspect the coordinator recipient/draft in the native app, confirm send is disabled when empty and enabled with a draft, clear the draft, and confirm the three-task candidate remains `waiting_approval` with no coordinator judgment, approval, or merge.
- [x] Verify packaged dispatch: copy the disposable run and SQLite state, configure a deterministic read-only Rust reviewer, and observe request `rq-7c80bcc153` transition queued → processing → completed with an accepted, evidence-backed response; the copied run remained `waiting_approval`, with 3/3 tasks/tests complete and no approval or merge. Three earlier fixture-only setup attempts failed before the successful request.
- [x] Render regression: verify the packaged UI's coordinator panel displays the latest request status for queued, processing, completed, and failed states; the Activity view remains the history surface. The focused render and coordinator-judgment suites pass (10 tests).
- [x] Verify dispatched lifecycle in the exact packaged UI: open the copied run in a single v0.1.36 window, send request `rq-6f7f030551` through the coordinator panel, and observe queued → processing → completed plus the evidence-backed response. The copied run stayed at `waiting_approval` with 3/3 tasks/tests complete; no approval or merge was created, and the original run remains untouched. Evidence: `docs/evidence/issue9-b298ae7-packaged-ui-coordinator-request.json`.
- [x] Acceptance: this is distinct packaged UI workflow 10/25 (40%); coordinator-directed operator feedback is a new Agent Room interaction, beyond the existing plan-to-merge and charter-drift workflows. 15 workflows remain.
- [ ] Windows UI verification remains a separate platform gate.
- [x] Complete: evidence is recorded in the LLM Wiki and AgentMemory (`mem_mvjx7viw_19e9423e6053`); commit `f7dccad` is pushed and #9/PR #22 progress is updated.

### Build exact-source macOS package — issue #9 / 2026-10-23

- [x] Plan: build the committed Rust source `ca9495b21c77886df2f75b1672f965dfe75d53fc` with the native release profile and repository package pipeline.
- [x] Task: create the arm64 app bundle and archive without launching another GUI process.
- [x] Verify: release build, `package-release`, `validate-package`, packaged `--help`, four approval-gated mock smoke cases, ZIP integrity, and checksum validation passed. The executable is 18,606,512 bytes and the package manifest has 214 dependency records.
- [x] Evidence: `docs/evidence/issue9-ca9495b2-macos-package.json`; binary SHA-256 `c057aab0ea15b9a486ee05dbcdd88e541d4f8743f1ef9001eb642969739fbe27`; archive SHA-256 `1f9b49bfcae1be68d6d816fcb87ca30af573092c809c53f04addb37ef6347eac`.
- [x] Limitations: this is package/CLI evidence, not GUI acceptance. It adds no workflow; acceptance remains 11/25. The bundled manual screenshots remain historical, and local Windows cross-compilation is unavailable because its Rust standard-library target is not installed.
- [x] Complete: update the release-package LLM Wiki and AgentMemory (`mem_mvk8ttc6_a3a62f3d24c4`); reconcile `docs/09-completion-audit.md` with the current source/package. Follow-up AgentMemory: `mem_mvk90rug_3e145f507328`. Retain the verified package for local review and remove only this run's transient smoke logs.

### Probe local Windows verification VM — issue #9 / 2026-10-23

- [x] Plan: test the existing UTM Windows VM as a local cross-platform verification route without modifying its persistent disk or opening another AgentConductor instance.
- [x] Task: start the VM with hidden disposable mode and query Windows through UTM's guest execution channel.
- [x] Verify result: UTM reports the VM started, but its QEMU guest agent is unavailable; both IP discovery and guest command execution fail with OSStatus -2700. `x86_64-pc-windows-gnu` Rust `core`/`std` are also absent from the host toolchain.
- [x] Cleanup: stop the disposable VM, verify it is stopped, and close the VM window created by this probe. No Windows build or runtime pass is claimed.
- [x] Record: AgentMemory `mem_mvk8ydbb_7d7019419082`; LLM Wiki records the exact OSStatus and cleanup. Next action: install/configure a guest agent in an authorized disposable Windows VM or use an exact-source Windows runner for the release gate. Keep Windows package/UI acceptance open.

### Claude macOS quota readiness — issue #9, DD-69

- [x] Plan: isolate the pre-prompt timeout in the actual packaged collector. Same executable/cwd/login succeeds with terminal environment supplied; TERM alone also succeeds without an interactive terminal. Captured API time and token use are zero.
- [x] Red: actual PTY fixture requires a usable TERM and exits before readiness under `dumb`; existing prompt collection fails with `Claude closed before reporting quota`.
- [x] Task: set a child-only xterm-256color fallback for missing/empty/dumb TERM, preserving usable profile or inherited settings and all readiness/ownership/timeout safeguards.
- [x] Verify: actual PTY prompt variants, quota parser/ownership regressions, source-layout and warning-denied Clippy after the final change.
- [x] Package: build exact source, replace only the idle prior window and verify authenticated automatic Claude and Codex collection from the GUI launch.
- [x] Complete: update evidence, LLM Wiki/AgentMemory, commit/push; preserve the user's approved pending workflow fixture. Windows verification remains open.

### Synchronize main #146/#149 without restoring the WebView — issue #9 / 2026-10-22

- [x] Plan: merge main `e9d1f1ff` into the issue-linked Rust branch; preserve native source paths and both sides of the requirements/design history. Main's lessons entry is already represented by native R-167; assign the newly ported fold/charter/judgment/launch requirements R-168–R-171 and DD-179–DD-182 to avoid ID collisions.
- [x] Red: rendered native tests fail because `Fold task details` is absent and a task's coordinator review does not appear in Activity. Preserve the real repaired-run backend test, replacing JavaScript source assertions with native rendered assertions.
- [x] Task: port the launch-gate and judgment backend additions into their existing split Rust modules; keep removed WebView/JavaScript files deleted. Add the native fold preference/decision reducer, accessible toggle/warning, candidate-first controls, task judgment entries and an asynchronously loaded coordinator row/view with a snapshot count. Match the existing coordinator selection after task rows and reuse the central Activity/Review/Logs/Lessons pane.
- [x] Verify focused: 22 tests pass across seven suites, including decision-key changes, preference restoration, bilingual expanded-state controls, steady reduced-motion warning, repaired-run judgment delivery, pre-launch rejection, layout and source separation.
- [x] Verify broader checkpoint: Rust rendered both HTML manuals; full native/test-fixture all-target execution passed 914 tests, 14 ignored, across 303 suites. A prior full run caught the inspector's unwanted automatic tab change; keep the operator's Activity/Review selection while opening the inspector.
- [x] Verify final follow-up: a combined task response exposed hidden original tool inputs when judgments were present; remove that filter and retain both facts. The final one-line fix passes all 10 affected activity/review/layout tests and warning-denied all-target Clippy, rustfmt and diff checks. Count/list consistency also passes concurrent audit-writer coverage through one read-only SQLite snapshot.
- [x] Verify exact integrated head `ece1ead4c21c`: full local run passes 914 tests, 14 ignored, 303 suites; exact macOS package validation and its four smoke cases pass. Single-window startup retains saved workspace fields. Inspector interaction and Windows acceptance remain separate.
- [x] Record and publish local checkpoint: update LLM Wiki/AgentMemory and evidence, commit/push through `e872992b`, and remove 19.8 GiB of completed development build cache while preserving the active app and fixture.
- [x] Port the coordinator's bounded typed path/dependency amendments to Rust: legacy review records remain readable, the validator reuses plan overlap/cycle rules, risky paths/budget exhaustion block for a person, audit records support one-at-a-time undo, and the Agent Room exposes Undo for active changes.
- [x] Verify exact macOS package `01f6443b804918ee23b4bd12d84e9e3e64674a16`: package manifest/binary checksum, ZIP integrity, and packaged `--help` pass; launched the single packaged app and inspected the workspace overview, role/model/fallback assignments, quota summaries, and empty plan-review state. This is component inspection, not a distinct plan-to-merge workflow; acceptance remains 8/25.
- [x] Inspect exact-package Agent Room controls in the saved disposable fixture: all three tasks and tests are complete, candidate diff and passing test evidence render, per-task session/log details are visible, coordinator judgments show zero entries, and the read-only briefing reports absent processes and stale last results. Candidate `3db8840b02ce28558701698c327a5820b7dc7311` remains pending; no approval or merge was performed. This re-inspection does not add a workflow; acceptance remains 8/25.
- [x] Verify step 4 source contract: `cargo test --test charter_contract` passes 5/5, including persisted charter round-trip, refusal after unrecorded goal drift at approval, and refusal after drift following human approval before merge. Legacy reports without a charter remain readable.
- [x] Close the source-level drift-label gap: the red native label regression reproduced the generic untranslated fallback; the dedicated English/Korean label now renders in the judgments view. Verify 16 tests across charter, coordinator decisions and native judgments, 2 source-layout/300-line checks, warning-denied focused Clippy, formatting and diff checks.
- [x] Build and verify exact macOS arm64 package from `aa8090405239`: package manifest/binary digest, ZIP integrity, and four fixture smoke cases pass. Each case ran three parallel tasks and tests, refused merge before approval, and verified merge after approval. Evidence: `docs/evidence/issue9-aa80904-macos-package.json`.
- [x] Launch exact package `aa8090405239` in one window with a disposable copy of the pending Issue 9 Mock Workflow. A synthetic `goal_drift` event in the copied database renders the Korean coordinator label “승인된 작업 기준이 변경되어 코디네이터가 실행을 차단했습니다”; the copied screen shows 3/3 tasks and 3/3 tests. The original approval candidate remains untouched and pending; no approval or merge was performed.
- [x] Acceptance: the distinct post-approval drift-refusal journey advances packaged native UI acceptance to **9/25 (36%)**; 16 workflows remain. The earlier isolated label-only display check is not counted separately.
- [ ] Platform gate: Windows exact-package UI and full cross-platform workflow behavior remain open.

### Synchronize native-worker test teardown on Windows — issue #9 / 2026-10-22

- [x] Plan/Red: inspect exact Windows run `39748528606`, job `119723469899`: cache-clear response exceeded a five-second test wait; Agent Room fixture deletion failed with OS error 32 while the worker still owned its Room.
- [x] Task: replace guessed 20ms teardown sleeps with a bounded wait for the real worker-owned Arc to release; then drop the Room before removing its fixture. Apply the same helper to the new metadata renderer regression. Allow 30 seconds for the cache integration test's reply under loaded Windows CI; application deadlines remain unchanged.
- [x] Verify local: both affected suites, all library tests (including the rendered metadata regression), and source layout pass 51 tests across four suites; focused warning-denied Clippy, rustfmt and diff checks pass. Test files remain 290/261/236/67/17 lines.
- [x] Knowledge: AgentMemory `mem_mvjfvbx1_c1915808b3a8`, LLM Wiki `asynchronous-metadata.md`, and exact failure evidence record the distinction between test lifetime/CI waits and application deadlines.
- [x] Complete local fix: commit/push `a6b7fe5`; retain source evidence and shared test support.
- [x] Current-head revalidation: `cooperative_worker`, `process_timeout`, and `windows_containment` pass on Rust source `43463f4b` with locked/offline, serial tests. This is macOS source-level evidence only.
- [ ] Complete platform verification: synchronize latest main and verify Windows. Retain the Windows gate until actual CI passes; local macOS success is not a substitute.

### Settle shared-sidebar metadata on every workspace step — issue #9 / 2026-10-22

- [x] Plan: trace existing R-63 → SF-53 → UC-53 → DD-69/DD-73; the packaged Workspace settings screen retains a Claude refresh spinner after the child has exited because native polling is restricted to the Team step.
- [x] Red: rendered `update_ui` regression fails on step 0 with `step 0 must poll the shared sidebar metadata job`; the test uses the real asynchronous request worker and disposable startup state, without invoking an agent.
- [x] Task: remove the Team-only guard from shared metadata polling and retain the existing active-only scheduling and request worker.
- [x] Verify: all three steps observe metadata completion without overwriting unsaved workspace name/goal or changing the selected step; idle metadata does not fast-poll. Focused local validation passes 39 tests across six suites, warning-denied Clippy, rustfmt and diff checks. Changed Rust files are 149 and 64 lines.
- [x] Knowledge: AgentMemory `mem_mvjayz0d_02d7203a86d2` and LLM Wiki `asynchronous-metadata.md` record the cause and fix. Graphify refreshed to 8,498 nodes / 14,595 edges; CocoIndex remains unavailable because its installed environment lacks `mcp.server.fastmcp`.
- [x] Complete local fix: commit/push `6a1f69e`; update issue #9/PR #22 and remove failed-test temporary folders. Distinct packaged workflow acceptance remains 8/25.
- [x] Verify exact macOS package: source `6a1f69e`, manifest validation and four approval-gated parallel smoke cases pass; archive SHA-256 `951b23ddb90fd236ffff36469c68407313bbe567d77b04469569889f82b7c6fd`. In the single native app window, Workspace settings stayed selected while the actual Claude refresh settled to `92%/69%`; Codex also refreshed to `80%/18%`. No generation or candidate merge was requested. Evidence: `docs/evidence/issue9-sidebar-metadata-polling.json`.
- [ ] Windows acceptance: previous run `39748528606` passed macOS but failed Windows `native_ui_cache` (reply timeout) and `native_ui_probe` (fixture removal before async worker released file handles); fix and revalidate these distinct test failures. This macOS component check does not close issue #9.

### Audit v0.1.36 CI package licenses — issue #9 / 2026-10-22

- [x] Plan: audit the paired v0.1.36 macOS and Windows packages produced by exact PR-validation run `39740270740`.
- [x] Task: compare each package manifest with its SBOM, verify package and executable checksums, and inspect dependency notices and license files.
- [x] Verify: source is GitHub merge commit `eff9f448` (PR head `cd9935de`, base `3fecd21d`); macOS has 214 components and Windows 201, with no missing license expressions. Both package and binary checksums match their embedded checksum files. `self_cell 1.3.0` is the only GPL-optional dependency (`Apache-2.0 OR GPL-2.0-only`); both packages include both license texts, and the permissive Apache option is available.
- [x] Knowledge: update the LLM Wiki and AgentMemory record `mem_mvja8ga0_477e45de583d`; Graphify refreshed to 8,496 nodes / 13,475 edges / 645 communities. CocoIndex is unavailable because its CLI fails to import `mcp.server.fastmcp`.
- [ ] Complete: repeat dependency-closure and license review against the final release source after issue #9 acceptance. This artifact audit does not close issue #9 or authorize release.

### Recheck the three-row manual plan in the complete Review screen — issue #9 / 2026-10-22

- [x] Plan: reproduce the v0.1.34 package observation that a three-task summary appeared to have only two editable rows.
- [x] Red: the first assertion assumed English labels and failed under the host's Korean locale; this was a test assumption, not evidence of a product defect.
- [x] Task: add a rendered regression through the full native Workspace screen, using the active localized label and three distinct task IDs.
- [x] Verify: the current Rust screen exposes all three expand/edit controls, with each control's bounds inside the 1140×772 viewport. Focused test 1/1, source-layout 2/2, public-doc tests 11/11, warning-denied Clippy, rustfmt, and diff checks pass. Both changed Rust files are under 300 lines.
- [x] Knowledge: AgentMemory `mem_mvj9ir18_d2d731bb14ab`; Graphify refreshed to 8,492 nodes / 13,471 edges / 649 communities. CocoIndex CLI is unavailable in this environment.
- [x] Complete this reproduction: no product behavior change was justified, and no workflow acceptance is added. Evidence: `docs/evidence/issue9-review-three-task-rows.json`.
- [ ] Verify exact package and Windows rendering: current-head packaged visual acceptance remains open; Actions run `39744289800` predates this test and is still running.

### Synchronize main worktree cleanup into issue #9 — GitHub #143 / 2026-10-22

- [x] Plan: merge main `3fecd21d` so run-owned worktrees are released after merge/reject and before cache clearing/workspace archival; preserve unrelated worktrees and run evidence.
- [x] Red: the incoming `Repository::remove_worktrees_under` method made `src/git.rs` 329 lines; `source_layout` failed the 300-line guard.
- [x] Task: move the method to `src/git/worktrees.rs` and keep the existing public `Repository` method and callers unchanged.
- [x] Verify: focused cleanup and source-layout suites pass 6/6; the full native-feature all-target suite passes 873 tests, 14 ignored, 0 failed across 298 suites; combined native-ui-probe/test-fixtures Clippy with `-D warnings`, rustfmt, and diff checks pass.
- [x] Knowledge: update this plan and Wiki/log; Graphify refresh reports 8,479 nodes / 14,574 edges; AgentMemory checkpoint `mem_mvj800lg_5c940c9a959d`.
- [x] Local package: exact pushed-head macOS archive validated and fixture smoke passed; see `docs/evidence/issue9-cd9935de-macos-package.json`.
- [x] Complete integration: exact-head GitHub run `39740270740` passed validation and package execution on macOS and Windows; artifact digests are recorded in the package evidence.
- [ ] UI acceptance remains 8/25; this backend cleanup does not add a GUI workflow.

### Verify integrated worktree cleanup on the exact pushed head — issue #9 / 2026-10-22

- [x] Plan/Task: merge main `3fecd21d`; preserve the public cleanup API while moving the method out of the 300-line `src/git.rs` limit.
- [x] Verify code: focused cleanup/layout 6/6; full native-feature all-target 873 passed, 14 ignored, 0 failed across 298 suites; warning-denied Clippy, rustfmt, and diff checks pass.
- [x] Verify package: Rust rendered English/Korean manuals; `package-release`, `validate-package`, and exact package smoke pass. Four fixture cases (shared/worktree × Git/folder) each complete 3 parallel tasks and tests; pre-approval merge is refused and approved merge succeeds. No real CLI was launched.
- [x] Assetize: Graphify refreshed to 8,479 nodes / 14,574 edges; AgentMemory checkpoint `mem_mvj800lg_5c940c9a959d`.
- [x] Cleanup: `cargo clean` removed 38,456 files / 10.8 GiB; removed superseded `ead895e` macOS package pair (~63 MiB). Retain the exact `cd9935de` package and evidence; `dist` is 63 MiB.
- [x] Verify remote: exact-head run `39740270740` passed macOS and Windows builds, source layout, all native tests, native UI launch path, rustfmt, and warning-denied Clippy. Both macOS and Windows release-package jobs rendered manuals, packaged the executable, ran the exact-package smoke, and retained artifacts.
- [x] Record artifacts: macOS `12409469483`, SHA-256 `c1a1b6efda2c88180815aafb38f07b05c94dbd6f9298dbf61236083f4104724f`; Windows `12409824890`, SHA-256 `5d2e0cf637a586cf1b3a05cbed2d4c93a509917e00074533d1515b7a357d18ed`.
- [ ] GUI workflow acceptance remains 8/25.

### Verify the exact ead895e package and PR #22 head — issue #9 / 2026-10-22

- [x] Plan: package the exact pushed PR #22 source without modifying product code or invoking real agent CLIs.
- [x] Verify local package: manifest, packaged `--help`, binary SHA-256, and ZIP CRC pass. Evidence: `docs/evidence/issue9-ead895e-macos-package.json`.
- [x] Cleanup: `rtk cargo clean` removed 1,537 generated files / 647.3 MiB; removed twelve superseded ignored package builds (~750 MiB), retain the exact `ead895e` package, checksum, and source evidence. `dist` is 63 MiB.
- [x] Verify exact pushed PR #22 head `287e9f2608bbacb9dd9d7d1983e00dba2d7ef8d2`: Actions run [39735141070](https://github.com/neodamul/AgentConductor/actions/runs/39735141070) passed macOS and Windows native validation, Rust manual rendering, exact-package execution, and artifact retention. Evidence: `docs/evidence/issue9-287e9f2-exact-head-ci.json`.
- [x] Knowledge: update the LLM Wiki and AgentMemory; Graphify no-cluster refresh reports 8,456 nodes / 14,546 edges. CocoIndex Code could not start because `mcp.server.fastmcp` is missing.
- [ ] Verify: exact-package GUI flow remains open. The current app displayed an unsaved-workspace warning, so no package replacement or workspace discard was performed.
- [ ] Acceptance: cross-platform package validation does not complete UC-28 or add a distinct packaged UI workflow; issue #9 remains at 8/25 (32%) with 17 workflows open.

### Share step lessons in the native Agent Room — GitHub #138 / issue #9 / 2026-10-22

- [x] Plan: trace R-167 → BG-123 → SF-155 → UC-158 → FR-153/NFR-63 → QAS-164 → DDR-27 → DD-178, keeping lessons as bounded evidence with no permission authority.
- [x] Red: add a rendered Agent Room Lessons regression that supplies a step lesson and requires its task, cause, and response to be visible.
- [x] Task: expose the existing run lesson document through native task details and render it alongside task-scoped reviewed lessons with English/Korean labels.
- [x] Verify: full `cargo test --locked --all-targets --features native-ui-probe` passes; warning-denied all-target Clippy, rustfmt, `git diff --check`, and source-layout/public-doc/lesson targets pass (16 targeted tests across 4 suites); every changed Rust file is under 300 lines.
- [x] Complete: LLM Wiki page and log updated; AgentMemory `mem_mvj31knz_6c8c9dd887b8`; Graphify refreshed to 8,446 nodes / 14,536 edges. CocoIndex Code remains unavailable because its launcher cannot import `mcp.server.fastmcp`. Packaged workflow acceptance remains 8/25 pending a distinct exact-package end-to-end journey.

### Match the update indicator to the v0.1.35 downward-arrow requirement — GitHub #127 / issue #9 / 2026-10-22

- [x] Plan: `origin/main` v0.1.35 requires a downward update arrow; the current Rust title bar still renders `↑` in `src/native_ui_probe/appearance.rs`.
- [x] Red: the rendered native update UI test failed with the actual glyph `↑`, while the localized accessible name passed.
- [x] Task: change only the native indicator glyph to `⬇`.
- [x] Verify: `cargo test --locked --features native-ui-probe --test native_ui_updates --test source_layout` passes 11 tests across 2 suites; rustfmt, warning-denied focused Clippy, and diff checks pass. Both changed Rust files remain below 300 lines.
- [x] Complete: update the LLM Wiki and AgentMemory; Graphify refreshes to 8,348 nodes / 14,396 edges; CocoIndex Code remains unavailable because `mcp.server.fastmcp` is missing; keep issue #9 workflow acceptance unchanged.

### Port configurable update source to the Rust Settings UI — GitHub #130 / issue #9 / 2026-10-22

- [x] Plan: synchronize v0.1.35 `main` into the issue branch; preserve Rust-only file deletions, port its validated update-source backend, and expose the setting in the Rust UI.
- [x] Red: the rendered Settings journey failed because `startup.update_repository` was absent; extend it to enter and save a repository through the async worker.
- [x] Task: add the localized native editor using the existing async request worker; retain owner/name and HTTPS-only validation, default-repository behavior, and the fixed download location.
- [x] Verify: 17 tests pass across update-source, native Settings journey/error rendering, source-layout and public-doc suites; library Clippy passes with `-D warnings`; rustfmt and staged/unstaged diff checks pass; every touched Rust source/test file is below 300 lines.
- [x] Complete: regenerate both HTML manuals with the Rust renderer; add the LLM Wiki page, index and log; save AgentMemory `mem_mvj0x49s_0bfdc162e4cb`; refresh Graphify to 8,387 nodes / 14,461 edges; push merge commit `84a101f` to the issue #9 branch. Issue #9 acceptance remains 8/25.

The broader native-feature run reported 573 passing and 14 ignored tests across 192 suites before it was interrupted to avoid competing with Claude's active build; it is not counted as a complete full-suite pass.

### Revalidate exact v0.1.34 packaged native plan-to-merge UI — issue #9 / 2026-10-22

- [x] Plan: open the package built from exact source `76d8082b` in one app window; run only deterministic mock tasks in a disposable Git repository with worktree isolation.
- [x] Fixture diagnosis: a custom JSONL profile requires `type=result`; Codex `turn.completed` is not a terminal result for the custom parser. The corrected Rust fixture emits the declared result event, and `cargo test --locked --test protocol` passes 9 tests.
- [x] Task: save and review a manual three-task plan with disjoint paths, explicit test argv, and a human execution-consent gate.
- [x] Verify: all three worktree tasks completed; 3/3 tests passed; the exact diff was loaded and reviewed; candidate approval and merge required explicit confirmation; the run completed with the disposable project clean.
- [x] Evidence: `docs/evidence/issue9-76d8082-macos-package-ui-workflow.json`; binary SHA-256 `6076725213cde19f24b29cc5111da9ac6e885cb70c78e377a122376e290c0e05`.
- [x] Assetize: AgentMemory `mem_mvj06ed6_a5514176f478`; update `knowledge/wiki/packaged-workflow.md` and the append-only log.
- [x] Acceptance: this repeats existing packaged plan-to-merge workflows 4/5; keep the total at **8/25**, with 17 remaining.
- [ ] Remaining #9 gates: Windows package/UI, authentic provider CLI sessions, full visual/accessibility parity, Korean IME, and resource measurements.

### Keep the task editor open through nested selectors — issue #9 / 2026-10-22

- [x] Plan: reproduce the task-editor dismissal when selecting an agent from its nested ComboBox.
- [x] Red: the rendered interaction test failed because egui's single-popup state replaced the parent editor while opening the selector.
- [x] Task: keep the editor's open state independently in egui temporary data and render it as an accessible foreground Area; close on Escape or an outside click, while allowing an open child selector to consume its click.
- [x] Verify: `cargo test --locked --test native_ui_tasks --test source_layout` passes 14 tests across 2 suites; warning-denied focused Clippy, rustfmt and diff checks pass. The source-layout test enforces the 300-line limit.
- [x] Broad local regression: `cargo test --locked` was interrupted after 818 passed / 14 ignored across 267 suites; it reported no failing test before interruption, but the run is incomplete and is not counted as a pass.
- [x] Assetize: Graphify refresh reports 8,336 nodes / 14,385 edges. CocoIndex Code remains unavailable because `cocoindex-code --help` cannot import `mcp.server.fastmcp`.
- [x] Complete this component regression and record it in AgentMemory/LLM Wiki; packaged workflow acceptance remains 8/25.
- [ ] Remaining #9 acceptance: full native UI workflow parity and exact packaged macOS/Windows runtime validation remain open.

### Build and validate the exact-head macOS package — issue #9 / 2026-10-22

- [x] Plan: build only from the clean issue #9 head `9443a666096bb68e590ad757dbb08a852de4b75c`; keep the already-running older app process untouched.
- [x] Task: build the native release package and run the package's built-in isolated workflow smoke suite.
- [x] Verify: `package-release`, `validate-package`, and packaged `--help` pass. The manifest records one native executable (18,356,880 bytes), 214 dependency records, and SHA-256 `d215f6e2e01e000451de6f0390ac6408e5262081de651292d90357103a0b4970`; archive SHA-256 is `38669871eab1b7bba361f26276df96371752f1e1b12810f17a31e3a8f8c9bbe6`.
- [x] Verify smoke: shared-folder/worktree × Git/folder cases each run three parallel tasks and tests, refuse an unapproved merge, and verify the approved merge. The focused manual-plan and native workspace-action targets pass 3 tests across 2 suites.
- [x] Assetize: Graphify re-extraction reports 8,320 nodes / 13,229 edges. CocoIndex was retried; its launcher still fails to import `mcp.server.fastmcp`.
- [x] Cleanup: copy the smoke result into the compact tracked evidence file and remove only this run's raw temporary log directory; retain the exact package/archive for a later UI review.
- [x] Native UI inspection: opened the exact package in the existing single app slot, created an isolated workspace, saved a three-task manual plan with disjoint scopes, and reviewed its exact digest and per-agent permissions. The UI refused an overlapping path and an empty task prompt. The final consent dialog was canceled before any CLI started; the temporary workspace was archived, its folder removed, and the original workspace restored.
- [ ] Acceptance: do not count UC-28 yet. Its final confirmation starts the configured real CLI agents; this inspection stopped before execution. Packaged native UI acceptance remains **8/25**.
- [ ] Platform: exact-head Windows package/runtime and native UI workflow verification remain open.
- [x] Evidence: `docs/evidence/issue9-9443a66-macos-package.json`; package smoke details are retained there. This is a macOS package/runtime checkpoint, not a release or an acceptance of UC-28.

### Localize generated Agent Room failure feedback — issue #9 / R-70 / UC-58

- [x] Plan: trace R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76; keep generated guidance in the selected UI language while preserving captured provider/test output verbatim.
- [x] Red: the Korean rendered regression failed first, showing the generated instruction and `Verification failed` fallback in English.
- [x] Task: pass the visible Room language into the existing failure-feedback helper and reuse bilingual message catalogs for its generated wrapper and fallback; preserve actual failure output as captured.
- [x] Verify: Room status, i18n catalog parity, source-layout (2), warning-denied all-target Clippy, rustfmt, public-docs, and diff checks pass. The broader native all-target test run passed 258 tests (4 ignored) across 79 suites before reaching the long-running coordinator suite; it was stopped before completion, so no full-suite pass is claimed.
- [x] Complete: update the LLM Wiki/AgentMemory and issue #9/PR #22 evidence; commit and push this component-only slice without changing the packaged workflow count. Commit `175f97f6`; packaged workflow acceptance remains 8/25.

### Integrate the latest upstream release and recovery fixes — issue #9 / 2026-10-22

- [x] Plan: `origin/main` is 7 commits ahead; inspect the merge tree before changing files. The delta includes v0.1.34 docs/version, scope-recovery fixes, shared controller/dispatch edits, and restored WebView assets that conflict with the Rust-only migration.
- [x] Red: the incoming Room integration tests exposed the upstream/WebView request schema mismatch, duplicate native request variants after auto-merge, missing direct-Resume recovery guidance, and conflicting CLI argument order.
- [x] Task: merge v0.1.34 release metadata and Markdown manuals; regenerate both HTML manuals with Rust; preserve the native UI request/action modules and hardened scope restoration; remove WebView source/assets and CJS tests; add the Room→CLI recovery test, canonicalize its packet schema, and refuse direct resume with bilingual recovery guidance.
- [x] Verify focused: 12 tests across native recovery UI, workflow recovery, and Room/controller integration pass; source-layout (2), public-docs (11), formatting, diff checks, and the 300-line source/test limit pass. The Room fixture now removes its temporary workspace on assertion failure.
- [x] Verify full: native-feature all-target regression passes **857 tests, 14 ignored, 0 failed across 293 suites**; warning-denied Clippy, formatting, source-layout/public-doc checks, and Graphify refresh (**8,312 nodes / 14,316 edges**) pass. CocoIndex is unavailable in this environment.
- [x] Package: `package-release` produced a macOS app archive with the English/Korean offline manuals, dependency license bundle, and SBOM; the packaged binary opens its CLI help.
- [x] Complete: verify the committed macOS app bundle and embedded bilingual manuals; update issue #9 and its LLM Wiki, push the integration, and remove the merged continuation branch plus generated test fixtures.

### Verify native scope-recovery approval interactions — issue #9 / R-165 / UC-156 / 2026-10-22

- [x] Plan: close the interaction-test gap left by the reviewed scope-recovery dialog; verify both proposal preview and explicit approval dispatch exact task, paths, test argv, and proposal digest.
- [x] Red: the first pointer test did not emit the preview event because its harness skipped egui's hover frame; use the same hover → press → release sequence as the other native UI interaction tests.
- [x] Task: add a separate focused test target so each Rust test file remains below 300 lines; exercise the preview and approval buttons against the real rendered room widget.
- [x] Verify: `native_scope_recovery_actions` passes (1), `native_scope_recovery` passes (2), source-layout (2), and public-doc checks (11) pass; formatting, all-target Clippy, and authored Rust files below 300 lines are verified.
- [x] Full local regression: `cargo test --locked --all-targets --features native-ui-probe` passes **855 tests, 14 ignored, 0 failed across 292 suites** (223.63s).
- [x] Knowledge: update the scope-failure recovery wiki with the exact click-path evidence; no new end-to-end packaged workflow is claimed and acceptance stays 8/25.
- [x] Assetize: Graphify code refresh reports **8,301 nodes / 14,302 edges**. CocoIndex was not run because its CLI/runtime is unavailable in this environment; no installation or API-backed substitute was used.
- [x] Complete this interaction-test unit; issue #9 remains open and no packaged workflow acceptance is added.
- [ ] Platform: exact-head Windows/macOS package and end-to-end native workflow acceptance remain open.

### Previous native feature verification — issue #9 / 2026-10-22

- [x] Plan: verify the exact issue branch after the Settings journey stabilization and feature-gate repair; source code remains at `214e359`, with documentation commits after it.
- [x] Verify: `cargo test --locked --all-targets --features native-ui-probe` passes **845 tests, 14 ignored, 0 failed across 289 suites** (224.71s).
- [x] Assetize: Graphify local no-cluster code refresh reports **8,170 nodes / 14,029 edges**; no graph structure was committed.
- [ ] Complete: packaged macOS/Windows UI parity remains unverified; workflow acceptance remains **8/25** and this test run adds none.

### Clean completed worktrees and test artifacts — issue #9 / 2026-10-22

- [x] Plan: compare local worktrees and branches with merged pull requests; preserve open #9/#126 work and branches with unmerged commits.
- [x] Task: remove 11 clean worktrees and their local branches for merged issues #11, #14, #15, #16, #50, #56, and #62; remove two stale local aliases already contained in their tracked upstream heads.
- [x] Verify: the full local all-target suite passes **865 tests, 14 ignored, 0 failed across 290 suites** (226.03s); the active issue #9 worktree remains clean.
- [x] Cleanup: after tests exited and no open handles remained, remove 3,017 stale project-test `ac-*` artifacts from the task temp directory (about 1.8 GB). Preserve smoke evidence and the active #9/#126 worktrees.
- [x] Complete this cleanup unit; issue #9 remains open with packaged native UI acceptance at 8/25 (32%).

### Gate the Team Briefing layout test on the native UI feature — issue #9 / 2026-10-22

- [x] Plan: Actions run `39634545852` failed on macOS and Windows because Cargo auto-discovered `tests/native_room_briefing_layout.rs` without `native-ui-probe` gating.
- [x] Red: `native_ui_targets::auto_discovered_native_ui_tests_are_feature_gated` reproduced the missing-gate condition on both platforms.
- [x] Green: add the same crate-level `cfg(feature = "native-ui-probe")` used by other auto-discovered UI tests.
- [x] Verify focused: `cargo test --no-default-features --test native_ui_targets`, the Team Briefing popup regression, rustfmt, and diff checks pass.
- [x] Verify: the workflow-equivalent all-target run completed and passed all other targets; one `cli_workflow` test failed once with a temporary-directory `DirectoryNotEmpty` cleanup error, then passed when rerun alone. The feature-gate audit, focused popup test, source layout, docs, formatting, and diff checks pass.
- [x] Verify exact feature gating locally: the no-default-feature integration target compiles with 0 tests, the feature-enabled popup regression passes (1 test), and `public_docs` passes (11 tests).
- [x] CI follow-up: PR merge-result run `39687975008` passed Windows; macOS exposed a sequential Settings dialog dismissal race (`Cache cleanup` remained visible). Source commit `214e359` stabilized dismissal polling; its focused journey test passes locally at current head.
- [ ] Platform: no exact-current-head macOS/Windows result is available after `214e359`; keep this platform gate open and do not count UI acceptance.

### Keep workspace navigation from pushing content down — issue #9 / R-39 / QAS-160 / 2026-10-21

- [x] Plan: an older Rust app bundle (built at 16:29, before the popup source update at 18:06) showed Saved Workspaces pushing the screen down; the current source uses an anchored popup. The project picker also clipped when the coordinator drawer narrowed the center column. Audit the other top-bar menu too.
- [x] Requirements: trace R-39 → BG-119 → SF-151 → UC-154 → FR-149/NFR-59 → QAS-160 → DD-174.
- [x] Red: rendered egui tests failed when menu opening moved workspace content from y=33 to y=154 and when the project picker ended at x=847 beyond the coordinator divider at x=753.
- [x] Task: render workspace actions in an anchored popup with a bounded scroll area; wrap the project-folder controls in the existing center column.
- [x] Verify focused: workspace and Settings popup geometry, creation by Enter, navigation actions, accessibility, and coordinator-open project picker tests pass locally (16 tests).
- [x] Follow-up: mirrored the production outer frame, agent roster, and central panel, and populated one saved workspace in the switcher regression. Opening it leaves both roster and workspace y-coordinates unchanged while exposing the expected Open/Archive actions (3 focused tests pass); warning-denied all-target Clippy, formatting, and the 300-line source-layout check pass. The visible effect is the opaque popup covering content beneath it, not a layout shift.
- [x] Audit follow-up: the user reported another menu that appeared to push content. The first phase-menu test omitted the production scroll area, so it was strengthened to use the real workspace scroll container, a 480px viewport, eight phases, and a long detail popup. Opening the last phase leaves the leading content anchor's y-coordinate fixed while displaying the details; no layout code change was needed.
- [x] Audit follow-up: apply the same production-scroll check to the long Coordinator analysis popup. Its content is bounded to the viewport and opening it leaves the workspace anchor fixed; no layout code change was needed.
- [x] Audit follow-up: render the Team Briefing popup with a populated session; it paints the details while the Stop control keeps the same y-coordinate. Add a rendered regression in `tests/native_room_briefing_layout.rs` so the remaining menu path is checked for layout shifts. The broader popup audit passes 21 tests across six suites.
- [x] Audit follow-up: verify the two remaining popup editors that had no layout assertion: Profile Rules JSON keeps the Save action fixed, and opening a task detail popup keeps the next task fixed. Both rendered regressions pass; popup geometry checks are now wired into the macOS/Windows validation job.
- [x] CI coverage: register both menu regression suites in the macOS/Windows native UI test and Clippy commands. Add a small workflow assertion so future menu-layout tests cannot silently pass locally while being omitted from the platform job.
- [x] Verify: full native-feature all-target tests (831 passed, 14 ignored across 281 suites), Settings-menu geometry regression, Clippy, 300-line source layout, and docs checks.
- [x] CI red/fix: exact-head macOS run `39576003076` exposed a locale-hard-coded Settings label (`설정` vs `Settings`); resolve it from the active localization state in commit `6faeedb`.
- [x] Re-verify: focused menu geometry (2 tests) and native UI launch path (12 tests) pass locally; fresh offline release builds of `agentconductor` and `agentconductor-native-ui-probe` succeed; rustfmt and diff checks pass.
- [x] Progress: update issue #9 with the stale-binary finding and exact-head run `39577297730`; record the CI finding and fix in AgentMemory.
- [x] CI red/fix: run `39577590534` found a second Korean-only assertion (`한국어` vs `Korean`); use the active localized label `startup.language_korean`.
- [x] Re-verify the menu geometry suite (2/2) under both the local language and an isolated English-default HOME; formatting and diff checks pass.
- [x] Exact-head CI: run `39613489337` for commit `dbf893a1` passed all four Windows/macOS validation and native-package jobs, including tests, source layout, formatting, Clippy, binary packaging, manuals, and package execution.
- [x] Verify UI: inspect the exact-source v0.1.33 app at `/private/tmp/AgentConductor-1f75758.app`; Workspace, Settings, and coordinator-selection popups overlay the page without moving it. Preserve this as component evidence only.
- [x] Local completion evidence: AgentMemory and LLM Wiki record popup and locale-test lessons; implementation commit `a4918f7`, test fixes `6faeedb` and current follow-up.
- [ ] Complete: packaged native UI acceptance remains 8/25 (32%); the stable shipped UI is still WebView. Complete the remaining distinct workflows and final release audit before closing #9.
- [x] Audit follow-up: inspect every `egui::Window` and menu path after the operator reported another layout regression; workspace/Settings popups and source-reference rows already have geometry coverage. The execution-consent window was still created from inside the page scroll area.
- [x] Red: the start journey showed the consent window and accessible Confirm button, but the button did not receive a click while its first-frame layout was changing.
- [x] Green: render consent after the page scroll closes; keep the confirmation window at a stable 560 × 580 size and scroll only its reviewed details.
- [x] Verify focused: consent overlay, bilingual reviewed values, stable approval click and full start journey pass; update panel/link accessibility tests pass after warming the window before collecting its AccessKit tree.
- [x] Windows UI Automation: run `39606419496` passed `native_workspace_is_accessible_and_saves_through_ui_automation` (11.57s), persisting `Windows UIA goal 윈도우`; the older `39597791275` failure was not reproduced at head `27da38fb`.

### Integrate latest main into the Rust migration branch — issue #9

- [x] Plan: `origin/main` is 9 commits ahead, including #109, #112, #115, #117, and v0.1.33 documentation. A merge preview shows Rust-vs-WebView modify/delete conflicts plus overlapping coordinator-chat changes.
- [x] Resolution rule: preserve the native Rust implementation where it overlaps; retain upstream release/manual updates and non-conflicting backend fixes; remove upstream WebView UI assets and CJS tests from the Rust-only result.
- [x] Red: post-merge checks exposed duplicate chat modules, missing Rust-embedded locale data, stale generated HTML, a v0.1.32 room-header assertion, and README links missing manual screenshot assets.
- [x] Task: merge current main, remove WebView assets and Node/CJS tests, keep required Rust locale data, regenerate both HTML manuals with Rust, make the room-header test use `CARGO_PKG_VERSION`, and restore README links to the manual's shipped screenshots.
- [x] Verify focused: native room-header (2), manual renderer (4), source-layout (2), chat (14) tests and Rust formatting pass.
- [x] Verify docs: `public_docs` passes 11 tests after restoring all referenced screenshot links in the README.
- [x] Verify full source/tests: all-target suite passes 828 tests, 14 ignored across 278 suites; all-target Clippy has no warnings; source-layout passes; no JavaScript/CSS/HTML application files or JS/CJS/Python tests remain.
- [x] Cleanup: remove only the confirmed 148 MB of generated test temporaries from `target/debug/tmp`; preserve build cache, package evidence, Graphify output, and external workspaces.
- [x] Verify chat result: remove the duplicate unconditional snapshot read; `startup_chat` and `chat_ask` pass (2 tests), formatting and diff checks pass.
- [x] Verify macOS package: exact merge source `9be8e880ad96d01d83d826c6ac843e195706861e` built and passed `validate-package`, packaged `--help`, ZIP CRC, and four smoke cases across shared/worktree × Git/folder. Every case ran three parallel mock tasks/tests; approved merge succeeded and unapproved merge was refused. Binary SHA-256 `86c72bf263fa32b4affe652b38b7d4d80d8125d754372252ae84509b63993d30`; 21 MB archive SHA-256 `e82f2a852215413a946ef54829237725e8c2b5b47779171d5ac301f1b2c16916`; smoke evidence in `docs/evidence/issue9-9be8e88-macos-smoke/run-36772-1792567832764762000-0`.
- [ ] Verify Windows package: exact-source Windows build and package execution remain pending.
- [ ] Complete: push the resolved merge and update issue/PR status; keep packaged workflow acceptance separate from component parity.

### Bound stop-during-backoff timing — main #112 / issue #9

- [x] Plan: main raised the retry delay to 5 seconds after the prior fixed window failed under load; the Rust branch still used a 1-second retry and 500ms stop bound.
- [x] Red/evidence: the native test's 500ms scheduling window was narrower than the observed host-load failure; no production behavior change is needed.
- [x] Task: align the retry delay and stop-response bound with main's 5-second and 2-second values.
- [x] Verify: focused `stop_during_timeout_backoff_prevents_automatic_relaunch` passed (1 test, 5.67s); `tests/workflow.rs` remains below 300 lines.
- [x] Acceptance: test reliability only; packaged native UI acceptance remains 8/25 (32%).
- [x] Complete: full workflow suite passed (8 tests); commit and push this unit.

### Keep native Agent Room activity in a fixed slot — main #117 / issue #9

- [x] Plan: main reserves one 18px activity row and hides it with visibility; native `live::show` currently returns early or renders up to three labels, shifting subsequent task content.
- [x] Red: render the same task details with empty and populated live activity; before the fix the following content moved from y=111 to y=168.
- [x] Task: combine live facts into one clipped, localized 18px row that is always reserved, including when the task has no activity.
- [x] Verify: native task-activity suite (7 tests), source-layout (2 tests), warning-denied native-feature Clippy, rustfmt, and diff checks pass; touched files are 44 and 269 lines.
- [x] Complete local slice: AgentMemory and the LLM Wiki record the code change and evidence; no packaged workflow is counted.
- [ ] Package: verify the activity line in exact macOS and Windows UI packages; packaged workflow acceptance remains 8/25 until a distinct complete journey is reviewed.

### Recover disk space and verify the current native package — issue #9 / 2026-10-21

- [x] Plan: inspect task-owned temporary paths and Rust build-cache size before deleting anything; verify no local Cargo or Rust compiler is active.
- [x] Task: remove only regenerable `target/debug/deps` and `target/debug/incremental` caches (about 59 GB); preserve source, release dependencies, package artifacts, and both existing executable files.
- [x] Verify: `target/` fell from 61 GB to 2.2 GB; exact branch status is clean; isolated UI HOME/workspace and review processes are removed.
- [x] Package: rebuild source `de032e0`; `package-release`, package validation, and the deterministic four-mode release smoke pass. Binary SHA-256: `36dd3e934b75fe278307815e9555ab78f72f65a01e270bc3cdf67ce7b6662695`; dependency count: 214.
- [x] UI provenance: inspect Workspace, Agent Roles, and Review from the full-path exact package in one isolated window; no settings saved and no screenshot retained. This is a spot check, not a packaged workflow acceptance.
- [x] Record: AgentMemory `mem_mvhmis60_5fa5b73d16bd`; issue #9 body refreshed with 8/25 acceptance and current run.
- [x] Red (Windows): Actions run `39543488348` passed build, layout, and applicable Windows tests, then failed `public_docs` because a multiline assertion assumed LF line endings.
- [x] Green: normalize CRLF to LF in the included audit text before the multiline documentation assertions.
- [ ] Verify Windows: rerun exact-head platform validation for the regression fix; no result is claimed yet.
- [x] Acceptance: no new distinct journey; packaged native UI remains **8/25 (32%)**, with 17 workflows open.


### Restore Windows picker test harness dependencies — issue #9 / 2026-10-21

- [x] Plan: keep the Windows-only picker path's test shim aligned with its app-level `ui` dependencies.
- [x] Red: Windows Actions run `39542757398` compiled the picker fix but failed compiling `tests/windows_console.rs` because its local `ui` shim lacked `language`.
- [x] Green: re-export the existing app `ui::language` module in the test shim; no production behavior or dependency changes.
- [x] Verify local: formatting, picker/probe/share, package-evidence, docs, and source-layout tests; macOS-target `cargo check` pass.
- [ ] Verify Windows: push and rerun exact-head Windows build/tests; this host lacks the Windows target.
- [x] Cleanup: isolated UI review app exited, and only its temporary HOME/data/config/cache directories were removed; the branch is clean before this fix.

### Register and discover an OpenCode-format MCP server — issue #9 / R-90 / UC-80 / 2026-10-21

- [x] Plan: verify Settings → MCP servers independently of Jira/Confluence source import; use OpenCode JSON/JSONC and a local Rust stdio fixture.
- [x] Task: in the exact v0.1.32 package, replace the inactive example with a local server definition, save it, reopen the editor, and verify the normalized server configuration persisted.
- [x] Verify: “Discover tools” returned `get_issue`; fixture trace contained only `initialize`, `notifications/initialized`, and `tools/list`, with no tool call. No provider CLI or external service.
- [x] Cleanup: closed the single package window and removed only `/private/tmp/ac-issue9-mcp-reg-home` and `/private/tmp/ac-issue9-mcp-reg-data` after retaining compact evidence.
- [x] Acceptance: distinct MCP server registration/discovery is Workflow 8; packaged native UI acceptance is **8/25 (32%)**, with 17 workflows open. Evidence: `docs/evidence/issue9-256db1a-mcp-server-registration-ui.json`.
- [x] Red: exact-head Actions run `39541731070` failed Windows compilation because `executable` was not forwarded from `pick` into the inner `dialog` function.
- [x] Green: pass the existing picker intent through both layers; local macOS compilation and the focused regression suites pass.
- [ ] Platform: commit and push this fix, then require the new exact-head Windows build/test job to pass; Windows GUI remains unverified.

### Restore Windows picker intent after localized-title change — issue #9 / R-52 / 2026-10-21

- [x] Plan: preserve the existing executable-vs-document file filter while routing folder-picking intent through the shared Windows picker.
- [x] Red: exact-head Actions run `39541153826` found the removed `executable` parameter; run `39541731070` then found it was not passed into the inner Windows `dialog` function. Run `39542757398` confirmed production picker compilation and exposed the Windows test shim's missing `ui::language` module.
- [x] Green: restore the explicit `executable` argument at callers and pass it through the Windows dialog; folders/documents pass `false`, executable selection passes `true`.
- [x] Verify local: picker/probe/share, package-evidence, docs, and source-layout tests passed 29 tests across six suites; macOS-target `cargo check`, rustfmt, diff check, and ≤300-line checks pass.
- [ ] Verify Windows: push this second-stage fix and require the exact-head Windows build/test job to pass; this host has not compiled the Windows target.
- [x] Acceptance: this cross-platform regression fix does not add a packaged UI workflow; count remains **7/25 (28%)**.

### Correct workspace-settings folder picker prompts — issue #9 / R-52 / UC-42 / 2026-10-21

- [x] Plan: keep export/import in Settings and give each native folder picker an action-specific, localized title.
- [x] Red: add English/Korean title regression coverage for project, export, and import pickers; the test failed before `FolderPurpose` and title selection existed.
- [x] Green: pass the picker purpose through the existing async worker and native macOS/Windows picker; leave project, plan-file, and executable picker behavior intact.
- [x] Verify: `native_folder_picker_purpose`, `native_ui_probe`, and `workspace_share` pass 15 tests across three suites; `cargo fmt --all -- --check`, macOS target `cargo check`, and `git diff --check` pass.
- [x] Verify package UI: exact source package `256db1a` exported workspace settings, displayed the import preview, applied only after explicit confirmation, and restored the exported workspace name. Isolated HOME and repository; no provider CLI or external service. Evidence: `docs/evidence/issue9-256db1a-settings-share-ui.json`.
- [x] Cleanup: removed only the disposable package UI HOME/workspace/repository/export folder after preserving compact evidence; kept the exact package for review. User workspace was untouched. No screenshot was retained.
- [ ] Verify platform: Actions run `39541153826` failed Windows compilation and its macOS job is still running; rerun exact-head validation after the picker fix. Windows GUI/cross-compilation is not locally verified.
- [x] Progress checkpoint: pushed `7c668a0`; issue #9 comment `6307473584` and AgentMemory `mem_mvhkfr8u_67de9db17f0b` record the change and current limits.
- [x] Acceptance: this distinct export → preview → confirmed import journey is Workflow 7; packaged native UI acceptance is **7/25 (28%)**, with 18 workflows open.
- [ ] Complete: verify exact-head macOS/Windows Actions and continue the remaining packaged workflows.

### Revalidate recoverable workspace deletion in the exact package — issue #9 / UC-79 / 2026-10-21

- [x] Plan: verify the published behavior with a disposable HOME and project folder; confirm an incomplete workspace can be archived only after switching away and that project data is preserved.
- [x] Task/Verify: exact v0.1.32 package from source `256db1a` displayed the exact workspace name/path and preservation warning; after confirmation it removed the item from the active registry, switched to the landing workspace, and wrote a recovery record under `workspace-trash`.
- [x] Verify filesystem: the archived app-owned workspace path no longer exists, the recovery record exists, and the separate disposable project folder remains. No provider CLI, external service, or user workspace was used.
- [x] Cleanup: closed the one package window; remove only `/private/tmp/ac-issue9-uc79-home` and `/private/tmp/ac-issue9-uc79-project` after preserving evidence. No screenshot retained.
- [x] Acceptance: UC-79 is revalidated in the exact package; this repeats Workflow 2 and does not increase packaged UI acceptance beyond **7/25 (28%)**. Evidence: `docs/evidence/issue9-256db1a-uc79-archive-ui.json`.
- [ ] Platform: exact-head Actions run `39541153826` is in progress on macOS and Windows; Windows GUI remains unverified.

### Revalidate shared-folder UI flow in the exact PR package and clean verification artifacts — issue #9 / 2026-10-21

- [x] Plan: validate the exact PR merge-result app and cross-platform release packages without touching the user's saved workspace; use only a disposable repository and a deterministic Rust fixture CLI.
- [x] Task: Actions run `39533269033` built PR head `d799ad6fe4199516c6344f9f63676eb2708d9157` as merge result `d386e6a3cdd0bb3f7484b0578d61ebdb867b67d0`; both platform validation and package jobs succeeded.
- [x] UI verify: in the packaged macOS app, revalidated the previously accepted shared-folder plan-to-merge flow with three tasks, 3/3 tests, exact diff review, human approval, merge, and Completed.
- [x] Repository verify: the disposable Git repository contains only `one.txt`, `two.txt`, and `three.txt` from candidate `517e82d46cfe047589fd0164f5d42f6c1933549a`; no real provider, external endpoint, or user's workspace was used.
- [x] Evidence: `docs/evidence/issue9-d799ad6-macos-package-smoke-summary.json` preserves the exact-head four-case CLI smoke; `docs/evidence/issue9-d386e6a-pr-merge-cross-platform-packages.json` and `docs/evidence/issue9-d386e6a-native-ui-plan-to-merge.json` preserve merge-result package identities and the isolated UI outcome.
- [x] Acceptance: this repeats Workflow 1 on the exact PR merge-result package and adds no distinct user journey. Packaged native UI acceptance remains **6/25 (24%)**, with 19 workflows open; the Rust UI is not yet shipped. Screenshots were not retained.
- [x] Cleanup: removed the stale package output, downloaded CI ZIPs, extracted merge package, isolated HOME, mock binary, temporary repository, and raw smoke-output directory after preserving compact evidence. Kept the exact-head local package for review.
- [x] Verify: `public_docs` and `source_layout` pass 12/12; all three evidence JSON files parse and `git diff --check` passes. AgentMemory revalidation `mem_mvhjiwl5_c55afe21ae58`, correction `mem_mvhjq8m6_f5abbf98e595`, lesson `lsn_f7173e504bfffa86`, and corrected issue #9 comment `6307145432` preserve the acceptance accounting. Commit and push this evidence/documentation unit after final review.

### Verify PR merge-result macOS and Windows packages — issue #9 / 2026-10-21

- [x] Plan: confirm that the issue #9 PR merge result can produce and run release packages for both required desktop platforms; distinguish it from the PR branch head.
- [x] Task: CI checked out PR head `806bf3204e63fa829b13d8e4bf09168da6b0e0c1` as synthetic merge result `9e3617545e6f0c6ad8357bda7a15c541302ce8dc`, built the macOS and Windows executables, and rendered both manuals from each Rust binary.
- [x] Verify: exact package execution passed on both platforms; validation also passed native tests, source/test layout, UI launch-path, formatting, and warning-denied Clippy.
- [x] Evidence: Actions run `39524574228`; macOS artifact `12312859028` and Windows artifact `12313174827` are retained with digests in `docs/evidence/issue9-9e36175-pr-merge-cross-platform-packages.json`.
- [x] Acceptance accounting: this verifies packaged CLI behavior only; packaged native UI acceptance remains **6/25 (24%)**, with 19 workflows open and public Rust UI unshipped.
- [x] Knowledge: record PR merge-result package results in the completion audit and native UI Wiki; AgentMemory `mem_mvhg8qd7_f7ff48237803` stores the correction, and lesson `lsn_215b3738fa9a9cdd` captures how to distinguish PR heads from merge commits. Graphify refreshed to 7,743 nodes / 13,278 edges. CocoIndex Code is installed but unavailable because `mcp.server.fastmcp` is missing.
- [x] Complete: verify the documentation regression locally, then commit and push this evidence-only unit.

### Verify exact macOS package quota fallback and Windows platform validation — issue #9 / 2026-10-21

- [x] Plan: validate the quota-limit → configured company fallback → tests → exact human approval → merge path with deterministic Rust fixtures and the exact v0.1.32 macOS release binary.
- [x] Task: all three parallel tasks received `usage_limit_reached` with a recorded reset time, transferred from `paid` to `company` on attempt 2, and completed in an isolated Git repository.
- [x] Verify package: 3/3 task tests passed; the package refused merge before approval, accepted approval for the exact candidate, then reached `completed` with the three expected files and a clean repository. Evidence: `docs/evidence/issue9-b4ad2f5-quota-fallback-cli.json`.
- [x] Verify source regressions: scheduling/start journey 4 tests pass; fallback/availability/profile/scheduling batch 21 tests pass across four suites. These overlap and are not summed as distinct tests.
- [x] Verify exact-head Actions `39520326287`: macOS and Windows platform-validation jobs passed, including build, source layout, applicable tests, native UI launch-path validation, formatting, and warning-denied Clippy.
- [x] Verify docs/layout: `public_docs` and `source_layout` pass 10/10 after the plan/wiki/evidence update; JSON parsing and `git diff --check` pass.
- [x] Verify packaged macOS artifact: native release build, Rust-rendered bilingual manuals, package execution, and evidence upload passed in run `39520326287`; artifact `12312373026` is retained (21,189,925 bytes, SHA-256 `c5f8dce7a06e04ff2f59f7208b8fbaf1b4e094e6e2aa94d5937791c72ae61624`).
- [x] Verify packaged Windows artifact: native release build, Rust-rendered bilingual manuals, package execution, and evidence upload passed; artifact `12311274352` is retained (15,417,457 bytes, SHA-256 `8415762398af24ebf21d1830a4027fcff6d6b11647292235eae12b83456ab363`).
- [x] Evidence: `docs/evidence/issue9-b714887-cross-platform-packages.json` records exact-head validation, both package artifacts, artifact sizes and digests.
- [x] Acceptance accounting: this is packaged CLI runtime verification, not a native UI journey; packaged native UI acceptance remains **6/25 (24%)**, with 19 workflows open. Public Rust UI remains unshipped.
- [x] Knowledge: AgentMemory records `mem_mvhe2xb3_6ad4618a9535`, `mem_mvhe77xu_5497db2b04a4`, and `mem_mvhebe04_c4521e3ff50b`; issue #9 comments `6305133147` and `6305176508`; quota-fallback Wiki and evidence record updated. Graphify no-cluster extraction refreshed to 7,739 nodes / 13,274 edges. CocoIndex is not installed in this environment.
- [x] Complete: update the two-platform evidence, commit and push this documentation unit after all four jobs in `39520326287` passed.

### Remove temporary smoke workspaces on every exit — issue #9 / R-12 / DD-160

- [x] Plan: trace `smoke-package` callers and confirm that failed verification keeps its evidence logs but should not retain its disposable task workspace.
- [x] Red: `failed_smoke_removes_its_temporary_workspace` reproduced a leaked `ac-release-smoke-*` directory when the packaged `--help` command failed.
- [x] Task: add a small drop guard for the temporary workspace; retain the existing explicit success-path removal and evidence output.
- [x] Verify locally: all 7 `release_smoke` integration tests pass, including all four successful package scenarios and the injected failure; targeted Clippy passes with warnings denied; rustfmt and diff checks pass. Source/test files are 161/269 lines.
- [x] Verify exact package: `b4ad2f5` built a single macOS executable with 214 dependency records; manifest validation and packaged `--help` pass. Package smoke passes all four shared/worktree × Git/folder cases, with 12/12 tasks/tests, denied unapproved merges, and verified approved merges. Evidence: `docs/evidence/issue9-b4ad2f5-macos-package.json` and `docs/evidence/issue9-b4ad2f5-macos-smoke.json`.
- [x] Visual check: launched that exact package in one app window and checked the Workspace surface at 1140×772; the saved workspace values remained intact and untouched. The screenshot was not retained; this is not an accepted packaged workflow.
- [x] Acceptance accounting: this protects the package verifier's failure path and does not add a user-facing packaged workflow; native UI acceptance remains 6/25 (24%).
- [x] Knowledge: AgentMemory `mem_mvhd1zld_ff361eff92de` records this full-suite verification; earlier records `mem_mvhbyxia_23042bf061c1`, `mem_mvhc5gfc_63e0f04856f0`, and `mem_mvhbpxpq_6d226352b72c` retain the TDD, package, and visual-review evidence. Release-packaging Wiki updated; Graphify refreshed to 7,733 nodes / 12,227 edges. `cocoindex` is not installed in this environment.
- [x] Verify local full suite at documentation head `7a6256d`: `rtk cargo test --all-targets --no-fail-fast` passes **805 tests, 14 ignored, 0 failed across 272 suites**; all-target Clippy with `-D warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass. This does not add a packaged user workflow.
- [ ] Verify platform: exact-head run `39517056024` for code commit `b4ad2f5` was cancelled by a later push; successor exact-head run `39517629909` for `7a6256d` remains in progress on macOS and Windows.
- [x] Complete code/test fix: committed and pushed as `b4ad2f5`; exact macOS package and smoke evidence are retained below.

### Verify Agent Room header geometry — issue #9 / R-17 / DD-137

- [x] Plan: cover the restored context row at compact and wide supported widths in English and Korean.
- [x] Red: the initial assertion queried AccessKit for passive `ui.label` text, which is rendered but not exposed as an AccessKit node.
- [x] Task: assert the actual rendered text shapes and their visual bounds; keep product behavior unchanged.
- [x] Verify: the focused test passes all four locale/width combinations; Agent Room header, layout, and rendering suites pass 17/17; `cargo fmt --check` and `git diff --check` pass. The test remains below 300 lines.
- [x] Acceptance accounting: this geometry regression does not validate a complete packaged workflow; acceptance remains 6/25 (24%).
- [x] Knowledge: AgentMemory `mem_mvhb36q0_2892617eb052` records the passive-label geometry-testing rule; Graphify refreshed to 7,724 nodes / 12,215 edges.
- [x] Complete: commit `fcc2925` is pushed; issue #9 progress comment `6303999828` records the evidence.
- [ ] Verify platform: successor exact-head Actions run `39514382321` is queued for `fcc2925`; local focused gates are complete.

### Match Agent Room header context and title hierarchy — issue #9 / R-17 / DD-137

- [x] Plan: compare the stable v0.1.32 WebView titlebar with Rust; the legacy header includes app/version, workspace, phase, and CLI-auth context, while Rust repeated the window title and omitted that context.
- [x] Red: rendered English/Korean tests failed for the duplicate heading and missing app/version, workspace, phase, and authentication labels.
- [x] Task: remove the duplicate title and restore the existing brand/version and localized run-context row using snapshot data and existing message keys.
- [x] Verify focused: 21 tests pass across five room/layout/source-layout suites; all-target Clippy (`-D warnings`), format/diff checks, and the 300-line source/test guard pass. Graphify refreshed to 7,719 nodes / 12,210 edges.
- [x] Exact-head CI triage: run `39507118252` completed with Windows passing and one macOS failure in `review_approval_shows_start_feedback_then_opens_the_started_run`; CI timed out waiting for enabled `Open Agent Room` after returning to Workspace. The exact test passes ten consecutive local runs and the focused current-source run passes; root cause is not reproduced.
- [x] Diagnostic task: include visible AccessKit button states and labels in this UI wait's timeout so a future exact-head failure distinguishes a missed navigation click from a disabled/missing workspace action.
- [x] Audit current knowledge assets: keep the distinct measures explicit — Rust UI source coverage 25/25, packaged workflow acceptance 6/25, and externally shipped Rust replacement 0/25 while stable v0.1.32 and PR #22 remain unchanged.
- [x] Build and verify exact-source macOS package at `0935f48`: package smoke, manifest validation, packaged `--help`, ZIP integrity, 214 dependency records, and binary/archive checksums pass.
- [ ] Verify platform: push this diagnostic and current UI batch; inspect the exact-head macOS/Windows rerun before treating the navigation path as stable.
- [ ] Verify screens: inspect the exact package in both locales and themes and retain current screenshots. The existing app is open on the user's saved workspace; do not replace it without a safe user-directed close. This package build does not increase packaged workflow acceptance.

### Correct Rust UI and workflow acceptance metrics — issue #9 / 2026-10-21

- [x] Plan: compare the default Cargo feature, exact packaged binary, visible app window, issue, PR, plan, audit, wiki, and public-doc regression assertions.
- [x] Evidence: `Cargo.toml` enables `native-ui-probe` by default, and the exact package from source `016451b` opens the Rust/eframe Workspace UI. Rust source coverage is 25/25; packaged end-to-end acceptance is 6/25 (24%); GitHub v0.1.32 targets `main`, while draft PR #22 remains unmerged. Therefore public Rust-UI shipment is still pending, but calling current-branch UI conversion 0/25 is incorrect.
- [x] Task: correct issue #9 and PR #22 progress text; align the current plan, completion audit, and LLM Wiki; update the docs regression assertion.
- [x] Verify: `public_docs` passes 8/8, `source_layout` passes 2/2, `cargo fmt --check` and `git diff --check` pass, and the edited Rust test remains below 300 lines. Exact-head run `39505888033` remains active on macOS and Windows; the next push will require successor CI.
- [x] Complete evidence: AgentMemory record `mem_mvh8uab2_948ce2987c4d` saved; issue #9 and draft PR #22 now distinguish Rust source coverage, current package UI, end-to-end acceptance, and public release state.
- [x] Complete: commit and push this documentation unit as `docs(issue-9): clarify native UI progress metrics`. Full packaged workflow and visual-parity acceptance remains 6/25; 19 workflows remain; public release of this Rust-UI branch remains pending.

### Preserve custom CLI quota configuration in profile editing — issue #9 / 2026-10-21

- [x] Plan: trace profile JSON serialization/apply callers and compare the editor allowlist with the Rust `AgentProfile` quota fields.
- [x] Red: add a profile-registration regression; it fails because `quota_command` is serialized as null and custom quota rules are dropped.
- [x] Task: retain `quota_command` and `quota_rules` in profile editor JSON; expose them in the existing advanced JSON editor and normalize explicit null values consistently.
- [x] Verify focused: 15 tests pass across profile transaction, quota editor, registration, registry, refresh, and custom quota suites; `git diff --check` passes.
- [x] Verify full: native-feature all-target tests pass **803 tests, 14 ignored across 272 suites**; all-target Clippy passes with `-D warnings`; `cargo fmt --check`, `git diff --check`, and `source_layout` (2 tests, including the authored source/test 300-line guard) pass.
- [x] Verify package: source `016451bff26befd933d6cf1196cb51376fd63424` built as a macOS release package; `validate-package`, packaged `--help`, ZIP integrity, and the four-case shared/worktree × Git/folder smoke passed. Each case ran three parallel fixture tasks/tests, refused unapproved merge, and verified approved merge. In one isolated package window, Accessibility confirmed quota settings survive save and reopen; the temporary profile was removed. No screenshot was retained. Evidence: `docs/evidence/issue9-016451b-macos-package.json` and `docs/evidence/issue9-016451b-macos-smoke.json`; this does not increase workflow acceptance.
- [x] Complete local slice: update the completion audit, LLM Wiki, AgentMemory, issue #9, and PR #22; commit and push `dbf249f`.
- [ ] Verify platform: exact-head Actions run `39505888033` for branch head `57112ae` is in progress; macOS passed compilation and is checking source layout, while Windows is compiling. The package already launches the default Rust UI; full packaged workflow/parity acceptance remains 6/25 (24%).

### Stabilize Workspace action alignment and MCP settings accessibility — issue #9 / R-17 / DD-137

- [x] Plan: GitHub Platform Validation run `39488170157` on PR head `a3148d7` failed two macOS checks; Windows remains in progress. One geometry check observed the Agent Roles Save button at x=785..945 on a 1140-point viewport, and the sequential-settings journey failed to find its duplicate heading while the MCP window title and Save control were present.
- [x] Red evidence: the exact macOS CI failures are recorded in run `39488170157`; the previous absolute x-origin check was too weak to establish right-edge alignment.
- [x] Task: make the Agent Roles action row use the full available width before right alignment, and anchor the sequential MCP-dialog check to its stable window title and Save control.
- [x] Red/Green: strengthened the action test to check the right edge of the full button group, reproduced its failure locally (x1=1017), changed the Rust layout, and passed the focused test. The MCP settings journey now waits on the dialog title; its local fixture passes.
- [x] Verify full: `cargo test --all-targets --features native-ui-probe --no-fail-fast` passes **801 tests, 14 ignored across 271 suites**; all-target Clippy passes with warnings denied; `source_layout` (2 tests), Rustfmt, and diff checks pass. Both CI-specific focused regressions also pass with `test-fixtures` enabled.
- [ ] Verify a new exact-head macOS and Windows Actions run. The previous #600 failed on both OSes at these tests, so native packaging was skipped. Acceptance stays 6/25; component tests do not increment it.

### Restore plan import and manual-draft actions — issue #9 / 2026-10-21

- [x] Plan: compare the Rust Agent Roles actions with the existing workspace flow; the native screen omitted “Read plan” and “Write plan manually” although the backend already supports both operations.
- [x] Red: add an AccessKit regression for both action labels and observe the native rendering test fail before the controls were added.
- [x] Task: restore both actions, use the existing native file picker for plan import, save changed workspace settings before continuing, and route the existing draft/read-plan requests through the async worker.
- [x] Verify local: the native workspace action batch passes (20 tests); source layout passes (2 tests); all-target Clippy, formatting, diff checks, and the 300-line limit pass. The manual-draft integration clicks the rendered button with an unsaved goal, asserts that Save happens first, then verifies the reviewable three-task proposal uses the saved goal and starts no agent; a Drop guard removes its temporary repository even on assertion failure.
- [ ] Verify packaged: rebuild and inspect the exact macOS and Windows packages; this component regression does not increase packaged workflow acceptance (6/25, 24%).
- [x] Complete local slice: LLM Wiki updated; AgentMemory records `mem_mvhh7gs1_fefc3ad24c70`, `mem_mvhhdg1w_fdca8755f270`, `mem_mvhhjcso_1d2df78bebc5`, `mem_mvhht6zf_d0185c1140e9`, and test-timing lesson `lsn_0a255df929f49892` saved; Graphify refreshed to 7,776 nodes / 13,343 edges; named failed-test workspaces removed. Local validation passed. CocoIndex could not run because its Python environment lacks `mcp.server.fastmcp`.
- [x] Push first code slice: `a6b8dda` is on `codex/issue-9-native-ui-parity`; issue #9 progress comment `6306368372` records the local evidence. Exact-head Platform Validation run `39532206021` is in progress.
- [x] Push rendered-pointer test follow-up `59a1571`; issue #9 comment `6306457538` records the local evidence. The package run `39529119997` was superseded, and its macOS/Windows package jobs were canceled. Validation run `39532206021` belongs to preceding commit `a6b8dda` and is stale.
- [ ] Verify exact-head run `39532871080` for `59a1571`; issue #9 checkpoint `6306475078` records that it started. The unsaved-goal test follow-up is locally verified and will supersede this run after commit/push.

### Port current main behavior into the Rust migration branch — issue #9 / issues #79, #80, #82, #83, #84, #95, #100, #103

- [x] Plan: compare the pushed #9 branch with `origin/main` at v0.1.32; the branch is 36 main commits behind and GitHub reports PR #22 as conflicting. Preserve the Rust-only source gate while carrying forward the current application contracts.
- [x] Scope: batch case-insensitive task scopes (#79), update-download cleanup (#80), common IANA reset zones (#82), fail-closed shell forms (#83), Bearer/Basic MCP env references (#84), running-task feedback (#95), live task activity (#100), and queued reassignment/model edits for pending tasks (#103).
- [x] Red: port upstream Rust regressions and add native UI regressions where the upstream behavior is only present in WebView JavaScript; explicitly exclude `.cjs` and JS UI/build source. New running-task/pending-task control and bilingual activity assertions failed before implementation.
- [x] Task: reconcile the upstream source into this branch, retain native Rust UI, and implement current safety/lifecycle behavior, queued pending edits, running feedback, and factual activity display.
- [x] Verify focused: 36 tests pass across 12 suites for active feedback, UI controls/activity, pending reassignment/model changes, update cleanup, path case, command forms, MCP header references, quota reset zones, public documentation, and the 300-line layout guard. Rustfmt and diff checks pass.
- [x] Verify full: `cargo test --all-targets --features native-ui-probe --no-fail-fast` passes **800 tests, 14 ignored across 271 suites**; all-target native-feature Clippy passes with warnings denied; authored-file source-layout, Rustfmt, diff checks, and offline English/Korean HTML manuals pass.
- [x] Verify package smoke: the exact release-profile macOS package for `cf92f39` passes the four-case packaged smoke (shared/worktree × Git/folder, three tasks each), validates approved merge and refused unapproved merge, and has a clean ZIP CRC. Evidence: `docs/evidence/issue9-cf92f39-packaged-smoke.json`; archive SHA-256 `fd17557d0be3a27470fccbbf75c372c279ed4d93bde905eb447d54f65755fc51`.
- [x] Complete this parity batch: architecture/Wiki assets and AgentMemory are updated; issue #9 and PR #22 reflect the pushed code and package evidence. Commits `3739f65` and `cf92f39` are pushed; GitHub reports PR #22 mergeable.
- [ ] Remaining package/integration gates: exact-package UI review, Windows package/UI, exact-head Actions completion, and PR review/merge remain open. Component/full-suite/package smoke results do not increase packaged workflow acceptance.

### Restore sidebar guidance and hierarchy from the main UI reference — issue #9 / R-17 / DD-137

- [x] Plan: use existing localized copy for the `YOUR TEAM` eyebrow and the executable/authentication hint; the current Rust rail omits both while the main reference shows them above the agent cards.
- [x] Red: add English/Korean AccessKit assertions for both labels and demonstrate the current sidebar misses them.
- [x] Task: restore the two existing labels in the Rust rail without changing card actions or agent data.
- [x] Verify focused: 13 tests passed across sidebar, density, icon, localized-header, start-journey and source-layout suites; Clippy (`-D warnings`), rustfmt, diff check, and the 300-line source/test limit passed.
- [x] Verify full: the completed all-target run passes 800 tests, with 14 ignored across 271 suites; no package or screenshot evidence is implied.
- [x] Complete: updated the Wiki and AgentMemory, posted the verified progress to issue #9, and pushed commit `a078aa8`.

### Match the Workspace stage and action hierarchy to current main — issue #9 / R-17 / DD-137

- [x] Plan: inspect the exact v0.1.32 Rust package and audit screenshot provenance before judging parity. The checked-in Workspace image is an older Rust capture from `392aa20`; the Agent Settings image is from `dbd44aa`. Neither image is a current-main screenshot, so neither proves current-main visual parity.
- [x] Source audit: review main commit `3123ece` and identify the three-step navigation and lower-right primary save-action hierarchy as the comparison targets; do not cite the old checked-in images as screenshots from that commit.
- [x] Red: `issue_nine_workspace_reference_is_not_mislabeled_as_current_main_capture` failed because this plan incorrectly called the historical Rust assets current-main visual references.
- [x] Task: align the Rust Workspace screen with the established step and save-action hierarchy while preserving the screenshot provenance caveat.
- [x] Verify: opened the exact `cf92f39` v0.1.32 macOS package in one window. The Workspace screen shows the numbered full-width step row, active underline, and lower-right save action. No fields were changed or saved. This is component inspection only; no current-main screenshot comparison or reusable screenshot artifact was retained.
- [ ] Complete parity gate: capture the exact current-main UI and compare all required screens and interactions; workflow acceptance does not increase from this component check.
- [x] Red: rendered tests first failed against the compact row and left-aligned Workspace save action; their expectations cover equal-width numbered steps and right-aligned save actions.
- [x] Task: implement the grouped Workspace flow hierarchy in Rust, including equal-width numbered steps and right-aligned save actions for Workspace settings and Agent roles.
- [x] Verify focused: header/action/start-flow/accessibility/source-layout tests pass; the 300-line limit, warning-denied Clippy, rustfmt, and diff check pass. The stale text-count assertion and preceding-header click failure from the earlier interrupted run were corrected and rechecked with focused tests.
- [x] Verify full: the completed all-target run passes 800 tests, with 14 ignored across 271 suites. No workflow acceptance increase without exact packaged end-to-end evidence.
- [x] Complete code/test batch: preserved the screenshot caveat and pushed the integrated commit `3739f65`; exact-package UI review remains a separate open gate.

### Correct screenshot provenance claims — issue #9 / R-17

- [x] Plan: audit whether the checked-in captures carrying a v0.1.24 label prove parity with the published WebView UI.
- [x] Evidence: `knowledge/wiki/native-screenshot-provenance.md` traces the Workspace capture to Rust commit `392aa20` and the Agent Roles/Room captures to Rust commit `dbd44aa`; the visible version label alone does not identify the implementation.
- [x] Task: qualify current and historical claims in the completion audit and LLM Wiki; add a public-doc regression assertion that preserves the provenance caveat.
- [x] Verify: focused `public_docs` and `source_layout` tests pass; JSON evidence validates; rustfmt and `git diff --check` pass.
- [x] Complete: AgentMemory checkpoint `mem_mvh0g4bn_ac67cdb74f07` saved; GitHub #9 body and progress comment updated; documentation/evidence/test correction pushed as `d2b44ea`.

### Native MCP source import — workflow 12 / issue #9

- [x] Plan: verify the existing registered-server, tool-discovery, Jira/Confluence preview, and separate apply-approval flow against the exact packaged Rust app.
- [x] Task: in one v0.1.31 package window and disposable `HOME`, configure a local stdio fixture and a read-only coordinator, save a test workspace with `jira:AC-1` and `confluence:53553565`, discover both tools, preview both records, then approve applying them.
- [x] Verify: the tool catalog exposed `jira_get_issue` and `confluence_get_page`; preview showed both selected source records, content, URLs, and SHA-256 values; approval returned `Applied sources: 2`. Targeted offline tests passed 17/17 across three suites. Evidence: `docs/evidence/issue9-2fd8e57-mcp-import-ui.json`.
- [x] Complete: exact-package acceptance advances to **6/25 (24%)**; shipped/default Rust replacement remains **0/25**. The deterministic fixture ran locally; no vendor CLI, external MCP server, or credential was used.
- [ ] Remaining: Windows UI, Korean IME/accessibility verification, full visual parity, and the other packaged workflows.

### Restore the legacy step marker inside each stage card — issue #9 / R-17 / DD-137

- [x] Plan: compare the exact same-size checked-in reference capture and v0.1.31 Rust package across the three-step Workspace surface; record that screenshot provenance identifies the reference as an earlier Rust build, so it cannot establish parity with the published WebView UI.
- [x] Red: the native Workspace regression expected two visible `STEP 1 · WORKSPACE` labels and failed at one.
- [x] Task: reuse one localized step-label helper inside Workspace settings, Agent roles, and Review cards; keep the page header, titles, controls, and card structure intact.
- [x] Verify source: 18 tests pass across `native_ui_probe` and `native_ui_header`; the source-layout guard, all-target native Clippy (`-D warnings`), formatting, and diff checks pass. Changed Rust files remain below 300 lines.
- [x] Verify package: built the optimized package from `ca5c3f4dd8e48d5bef89c533153a4d01e92f1c02` and inspected Workspace, Agent Roles and Review in one 1140×772 app window under an isolated `HOME`; page eyebrow and active card both showed the matching step marker. Evidence: `docs/evidence/issue9-ca5c3f4-stage-card-markers.json`.
- [x] Acceptance accounting: this visual correction adds no end-to-end workflow; packaged acceptance stays **6/25 (24%)** and shipped/default Rust replacement stays **0/25**.

### Reconcile the packaged workflow acceptance inventory — issue #9 / R-17

- [x] Plan: compare the current issue, implementation checklist, completion audit, package evidence, and LLM Wiki; keep source coverage, packaged acceptance, and shipped UI conversion as separate measures.
- [x] Task: record the already-verified MCP import as packaged Workflow 6; update the current denominator to 6/25 and relabel older 5/25 checkpoints as historical.
- [x] Regression: extend `tests/public_docs.rs` to assert the current 6/25 count, exact package hash, Workflow 6 entry, and matching Wiki checkpoint.
- [x] Verify: `public_docs`, `native_ui_probe`, `native_ui_header`, and `source_layout` pass 27 tests across 4 suites; the native-feature all-target suite passes **797 tests, 14 ignored, 0 failed across 263 suites** (220.67s); all-target Clippy (`-D warnings`), rustfmt, and diff checks pass.
- [ ] Complete: refresh AgentMemory/LLM-Wiki, update issue #9, and commit/push this checkpoint.

### Enforce Rust-only UI and build sources — issue #9 / R-151 / DD-160

- [x] Plan: prohibit project-owned JavaScript/TypeScript, Python, web UI, and shell build sources under `src`, `tests`, `scripts`, and `examples`; allow Rust implementation and data/assets outside those code roots.
- [x] Red: the expanded source-layout gate found 20 ignored Python bytecode files under `tests/__pycache__`; no tracked Python or JavaScript source was present.
- [x] Task: remove the generated cache and make `source_layout` reject non-Rust code extensions in the project-owned UI/build roots; the synthetic regression detects `.tsx`, `.py`, and `.sh` fixtures.
- [x] Verify focused: source-layout passes 2/2; the file is 121 lines. Package scan found zero JavaScript/TypeScript/Python files in the exact macOS package.
- [x] Verify focused: `source_layout` passes 2/2; package scan finds zero JavaScript/TypeScript/Python files; all-target Clippy (`-D warnings`), rustfmt, and diff checks pass.
- [x] Verify complete: native-feature all-target suite passes **797 tests, 14 ignored, 0 failed across 263 suites** (220.67s); all-target Clippy (`-D warnings`), rustfmt, and diff checks pass.
- [ ] Complete: update the knowledge assets and #9 progress, then commit/push this unit.

### Match the selected Workspace step heading — issue #9 / R-17 / DD-137

- [x] Plan: compare the blank native Workspace and Agent Roles screens with the checked-in reference capture; its v0.1.24 label is historical and provenance identifies it as Rust-built, so this is not legacy WebView parity evidence.
- [x] Red: the bilingual header regression failed on English step 1 because the shared page eyebrow said `PREPARE A GOAL` instead of the selected step label.
- [x] Task: render the selected step's existing localized label in the page eyebrow and remove the repeated marker from the section cards; retain their titles and descriptions.
- [x] Verify: header and AccessKit suites pass 17 tests across 2 suites, including the out-of-range fallback; source-layout passes; rustfmt, diff checks, all-target native-feature Clippy (`-D warnings`), and optimized release build pass. The exact `2fd8e57` macOS package validates with 214 dependency records and bundled licenses; packaged `--help` and ZIP CRC pass. The serial native-feature all-target suite passes **775 tests, 14 ignored, 0 failed across 262 suites** (300.95s). The package smoke passes all four shared/worktree × Git/folder modes with 12/12 fixture task tests, parallel execution, unapproved-merge rejection, and approved-merge verification; evidence is `docs/evidence/issue9-2fd8e57-macos-smoke/`. Five changed source/test files remain below 300 lines.
- [x] Visual package check: opened the exact `2fd8e57` v0.1.31 package in one disposable profile at 1140×772 and inspected Workspace, Agent Roles, empty Review, MCP, sharing, Messenger, and cache screens in Korean dark and English light themes. Saved fourteen screenshots and SHA-256 values in `docs/evidence/issue9-2fd8e57-visual-audit/`. The visible `fixture` profile uses `/usr/bin/true`; no real workspace was loaded, and no workspace/settings save, message send, cache deletion, or agent execution occurred.
- [x] Acceptance: this is a component visual audit, not another end-to-end workflow. Packaged acceptance remains **5/25 (20%)**; shipped Rust replacement remains **0/25**. Windows rendering, Korean IME, full parity, and the remaining workflows stay open.

### Repair native-session recovery test fixture — issue #9 verification

- [x] Plan: investigate the sole failing target from the all-target validation run before accepting the migration checkpoint.
- [x] Red: `native_session::live_or_missing_prior_process_record_blocks_reuse_after_lease_release` pre-created an ownership record naming the still-running test process, so its initial lease acquisition correctly refused recovery.
- [x] Task: acquire the empty session lease first, write the live owner record while holding it, release it, then assert that re-acquisition is refused; retain the missing-record and dead-process cases.
- [x] Verify focused: `native_session` passes 3/3; source-layout, rustfmt, all-target Clippy (`-D warnings`), and diff checks pass. The edited Rust test remains under 300 lines.
- [x] Verify complete: after the fixture correction, `cargo test --offline --locked --all-targets --features test-fixtures --no-fail-fast` passes **794 tests, 14 ignored, 0 failed across 263 suites** in 258.50 seconds.
- [ ] Acceptance: this repairs test setup only; packaged workflow acceptance remains **5/25 (20%)**, with shipped Rust replacement **0/25**.

### Restore compact legacy step navigation — issue #9 / R-17 / DD-137

- [x] Plan: compare the exact v0.1.31 native Workspace at 1140×772 with the checked-in reference capture; preserve its three accessible steps and order while treating the capture as a prior Rust build, not proof of published-WebView parity.
- [x] Red: `all_step_buttons_fit_the_content_column_in_both_languages` failed because each step occupied a 48px-high third of the row.
- [x] Task: use content-sized selected buttons in a wrapping horizontal row; retain localized labels, keyboard actions, and accessible names.
- [x] Verify focused: header, compact-header, and start-journey suites pass (5 tests / 3 suites); focused Clippy (`-D warnings`), formatting, source-layout, diff, and 300-line checks pass. Source/test files are 47/177 lines.
- [x] Verify full-suite attempt: stopped after 666 passed and 14 ignored across 213 suites (exit 130) to continue the UI correction. This is incomplete and is not a full-suite pass.
- [x] Verify package: source `227854d3f7bbe01efd3953325b9acb890d77b903` builds and passes `validate-package`, packaged `--help`, and ZIP CRC. The 17,942,384-byte executable SHA-256 is `8568714474af52d04e7cf2f678dd69d27a8f95c4149c59c51b59993fa34ea97a`; archive SHA-256 is `c773cea387c4a86d19fa86ba2e145af5ef3a79ff4e538166db2a2b7c45c3cd72`. Replaced the prior process before opening one exact-package window. At 1140×772 the step row matches the compact reference and each step navigates; Review remains gated with no plan. No settings or workspace data were saved. Narrow-width wrapping is covered by the bounds test, not a native resize capture.
- [x] CI follow-up: exact-head run `39452437163` failed macOS `native_ui_settings_journey` while waiting for the MCP title. The failure exposed that the helper recomputed the language instead of waiting on the language already selected by the rendered UI; it now waits on the captured label and includes current accessibility labels in timeout diagnostics. The journey passed three consecutive local runs, then the grouped four-suite batch passed 6 tests.
- [x] Reverify pushed branch `b204eb5d5d770873db954c4c9606b4e6e7ff014c`: grouped Settings/header/start-journey tests pass (6 tests / 4 suites); source layout, rustfmt, all-target Clippy (`-D warnings`), and `git diff --check` pass.
- [ ] Remote CI distinction: run `39452437163` checked merge commit `11588965` (PR head `c5dac47` into base `fdfd6d3`), not current head `b204eb5`. Both macOS and Windows failed only `native_ui_settings_journey`; the equivalent grouped tests now pass locally at `b204eb5`. Exact-head CI has not yet been observed.
- [ ] Full-suite follow-up: this continuation's all-target run was interrupted at 67 tests / 18 suites (exit 130); the earlier 666-test interrupted attempt also remains incomplete. Do not report a full-suite pass.
- [ ] Acceptance: component change adds no end-to-end workflow; packaged acceptance remains **5/25 (20%)** and shipped Rust replacement **0/25**. Windows, Korean IME, full parity, and remaining workflows stay open.

### Align the workspace-sharing settings modal — issue #9 / R-17

- [x] Plan: audit Settings, MCP registration, workspace sharing, Messenger, and cache dialogs together; keep settings and cache actions read-only during inspection.
- [x] Red: the sharing menu journey exposed its Export action at x=30 in a 1140px viewport, while other settings dialogs use centered anchors and bounded widths.
- [x] Task: center the sharing dialog and set its initial width to 480px, matching the existing cache dialog convention.
- [x] Verify: the regression and five related Settings/MCP/Messenger/cache/theme suites pass (18 tests across 6 suites); formatting, all-target Clippy, public-doc, source-layout, and diff checks pass. Changed Rust/test files remain under 300 lines.
- [x] Acceptance: no end-to-end workflow is added by this component fix. Packaged workflow acceptance remains **5/25 (20%)**; shipped Rust replacement remains **0/25**.
- [ ] Continue: inspect the exact current package against the legacy UI with sanitized screenshots; verify Windows rendering and the remaining workflows before closing #9.

### Accept the packaged named-workspace lifecycle — issue #9 / UC-30, UC-31

- [x] Plan: use the exact v0.1.31 Rust macOS package and isolate both `HOME` and the workspace; cover create, save, role-stage navigation, switch, remove-from-recent, reopen, and archive as one lifecycle.
- [x] Task: run package source `a8be85fc2392821fab345d3e1b879a6aa9bc6714` (binary SHA-256 `74204cbc9492379a71769a6f679d19ee28012b823e280e8a6fdd03af89300057`) in one native window. The initial menu contained no saved workspaces from the user's real home.
- [x] Verify: created three workspaces; saved and reloaded name/repository/goal; inspected role rows and the unsaved-state plan gate; confirmed unsaved-state Cancel; removed one workspace from the recent list and reopened it; archived an inactive workspace with explicit confirmation. A later saved-state check found that the plan button remained enabled without a registered coordinator. No CLI provider or external service ran, and the user's workspace was not changed. Evidence: `docs/evidence/issue9-a8be85f-workspace-ui.json`.
- [x] Complete: UC-30/31 packaged macOS workflow accepted as **Workflow 2**; packaged acceptance advances to **3/25 (12%)**. Shipped Rust replacement remains **0/25**. Exact-head Windows UI and all other workflow gates remain open.
- [x] Docs Red/Green: the first acceptance-audit regression run failed on its old exact-head and Actions assertions; updated them to require the 3/25 evidence, tested code baseline, remaining count, and current Actions run. `public_docs` + `source_layout` pass (8 tests / 2 suites).
- [x] Knowledge: updated the LLM Wiki and refreshed Graphify (7,501 nodes / 12,933 edges); CocoIndex Code remains unavailable because its environment lacks `mcp.server.fastmcp`.

### Batch-fix plan gating and stabilize the related native UI journey tests — issue #9

- [x] Plan: audit the connected workspace, role, plan-review, MCP, sharing, Messenger, and cache screens together; use the exact isolated v0.1.31 package and do not count component checks as workflow acceptance.
- [x] Audit: found Generate plan could be clicked with an empty goal or an unregistered coordinator, then failed after navigation. The same pass found the Settings journey test sampled the MCP view before it settled, and the start journey could request merge while the prior UI action was still running.
- [x] Task: disable plan generation until the saved goal is non-empty and the coordinator resolves to a registered profile; show the existing localized validation reason. Wait for settled dialog visibility and UI-action completion in the two journey tests.
- [x] Red/Green: the exact package reproduced the late `coordinator not registered` failure. Added config and AccessKit regressions; focused planning tests pass (2), as do the MCP/settings/start journeys plus public-doc and source-layout checks (10).
- [x] Verify: `cargo fmt --all -- --check`, `git diff --check`, and the focused planning, settings/start, docs and source-layout tests pass (10 integration tests; 4 suites). Changed Rust/test files remain below 300 lines. The broader all-target run was stopped after five minutes at 363 passed / 14 ignored across 120 suites, with no failure reported; it is not reported as complete.
- [x] Verify package: exact pushed source `4a114ec6a76141c48ebe8138694398572cae1acd` built and passed `validate-package` and packaged `--help`; a single exact-package window confirmed the saved-goal/no-coordinator gate disables Generate plan and explains the missing coordinator. Evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`.
- [ ] Verify full native-feature suite and refresh exact-head macOS/Windows Actions result; the local all-target run did not finish, and the new Actions run is pending.
- [x] Acceptance: still **3/25** packaged workflows; this grouped control/test correction adds no end-to-end workflow. Shipped Rust replacement remains **0/25**.

### Batch audit adjacent native settings and review screens — issue #9

- [x] Plan: inspect the exact pushed package's Agent Roles, Review, MCP server registration, workspace sharing, Messenger, and cache preview screens together; perform no imports, exports, settings saves, or cache deletion.
- [x] Verify: the plan gate and missing-coordinator guidance remain visible; Approve is disabled without a reviewed plan; each settings screen exposes its expected controls, and the empty cache state disables Clear. One app process, isolated HOME, no provider/external call, no workspace edits. Evidence: `docs/evidence/issue9-4a114ec-settings-batch.json`.
- [x] Tests: grouped native UI tests pass **122 tests across 23 suites**; feature Clippy (`-D warnings`), rustfmt, and diff checks pass. This batch adds no workflow acceptance.
- [x] Acceptance: remains **3/25 packaged workflows** and **0/25 shipped Rust replacement**. Windows packaged UI and remaining workflows stay open.

### Exact packaged native UI plan-to-merge workflow — issue #9

- [x] Plan: exercise the positive path that complements the missing-coordinator gate, using one local Rust mock CLI and fresh disposable folders under isolated `HOME`.
- [x] Task/Verify: register the CLI in the app, generate and inspect the exact plan, approve its executable/scope/test command, run one task, observe 1/1 test pass, inspect the diff, approve the candidate, and confirm merge. Final run is Completed and `one.txt` exists only in the disposable project.
- [x] Fixture correction: the initial test command used `/usr/bin/test`, absent on macOS; use `/bin/test` and repeat the full flow in a fresh folder. The corrected workflow passed. Evidence: `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`.
- [x] Acceptance: count this distinct exact-package UI journey as **Workflow 4**, bringing packaged acceptance to **4/25 (16%)**. The grouped screen checks remain component evidence only; shipped Rust replacement remains **0/25**. Windows packaged UI and 21 workflows remain open.
- [x] Verify current records as a batch: public-doc/layout regression tests pass (8 tests / 2 suites), rustfmt and diff checks pass. Exact-head Actions run `39429494121` was cancelled during native build/test on both OSes; no platform result is claimed. Graphify re-extraction found 7,509 nodes / 12,943 links with no source-graph changes. CocoIndex remains unavailable because `mcp.server.fastmcp` is missing from its environment.

### Batch-verify Review and update details — issue #9

- [x] Plan: check the Workspace phase-analysis disclosure, Agent Room candidate evidence, and Updates release notes together in the existing exact-package window; do not create another app process or modify workspace/settings data.
- [x] Task / Verify: the analysis disclosure expands and exposes its returned state; phase rows expose task summary, agent, and status; candidate review exposes the evidence revision, candidate, passing test, and exact diff; Updates exposes the release-notes link and full scrollable notes. The public release metadata check ran; the link itself was not opened. Evidence: `docs/evidence/issue9-4a114ec-review-updates-batch.json`.
- [x] Tests: five grouped suites pass (23 tests); native-feature all-target Clippy reports no warnings. This is component evidence only and does not increment workflow acceptance.
- [x] Acceptance: remains **4/25 (16%)** packaged workflows and **0/25** shipped Rust replacement. No screenshot artifact was captured; Windows, keyboard/IME, and the remaining 21 workflows remain open.

### Batch-verify dark-theme readability and native layout — issue #9

- [x] Plan: inspect the exact packaged Agent Room in dark mode, then restore the isolated profile's original light theme. Check status, verification, diff, and inspector together; do not create another process.
- [x] Verify UI: at 1140×772, Completed, 1/1 test, selected Diff & tests, and the candidate diff remained readable. A later exact-package parallel run also exposed the enabled approval control in dark mode; white action text remained readable on the green control and the green Completed panel. No contrast defect reproduced. No screenshot file was retained.
- [x] Tests: `interface_theme`, `native_ui_appearance`, `native_ui_profile_contrast`, `native_ui_workspace_style`, and `native_room_layout` pass (15 tests / 5 suites); rustfmt and diff checks pass.
- [x] Acceptance: this component audit adds no workflow; packaged acceptance remains **4/25 (16%)**, shipped Rust replacement **0/25**. Windows and public screenshot parity remain open.
- [x] Remote-check status: Actions run `39431587615` for `befdeb7` was cancelled on both OSes during native build/test; the latest docs-only head `459ba84` has no reported run. Keep related changes batched before the next push to avoid superseding active checks.

### Exact-package parallel plan-to-merge and enabled-action contrast — issue #9

- [x] Plan: reuse the single v0.1.31 package process with an isolated HOME and a fresh disposable shared folder; assign a local Rust mock profile with three sessions, then check the parallel run, per-task review, approval, integration, and dark-theme active action together.
- [x] Task/Verify: the reviewed plan contained three independent development tasks with disjoint paths and no dependencies. Recorded task intervals overlap; all three completed and all three `/bin/test -f` checks exited 0. Reviewed the exact three-file diff, approved the candidate, confirmed integration, and observed Completed. The folder is non-Git, so this verifies app-level shared-folder integration, not a Git commit/branch merge.
- [x] UI: inspected the active approval action and Completed status in dark mode at 1140×772; labels remained readable. Restored light mode in the same app process. No screenshot file was retained.
- [x] Grouped Agent Room check: Activity shows per-task instructions/results, Raw logs exposes the JSON result and clean stderr, Lessons explains explicit promotion, and the read-only team briefing summarizes all three Completed tasks with no action required. The mock CLI has no native session ID/model, so provider-session metadata remains unverified.
- [x] Local regression: `cargo test --offline --locked --all-targets --features native-ui-probe --no-fail-fast` passes **770 tests**, **14 ignored**, **0 failed** across **260 suites** (216.85s); all-target feature Clippy passes with `-D warnings`.
- [x] Evidence: `docs/evidence/issue9-4a114ec-parallel-ui-plan-to-merge.json`. Two fixture-only plan attempts were rejected by the expected JSONL and required-role validation; the corrected compact JSONL Rust fixture passed.
- [x] Acceptance: this rechecks the existing parallel shared-folder workflow; packaged acceptance remains **4/25 (16%)**, shipped Rust replacement **0/25**. Windows UI/package and screenshot parity remain open.

### Continue issue #9 with grouped UI audit and full local verification — 2026-10-20

- [x] Plan: continue #9 only; audit neighboring screens as a batch, preserve the single existing app window and current user workspace, and do not count component checks as packaged workflow acceptance.
- [x] Verify handoff: read Claude's latest AgentMemory signal `sig_mvgo0qng_9ea40889a2f2`; it contains the current package hashes, grouped dialog evidence, 2/25 acceptance, remaining gates, and PR #22 draft status.
- [x] Audit: scanned native UI source for `TODO`, `FIXME`, and unimplemented macros; none found. Confirmed the existing Settings dialog batch covers all six custom dialogs. No additional source defect was established in this pass.
- [x] Verify complete: `cargo test --offline --locked --all-targets --features native-ui-probe --no-fail-fast` passes 768 tests, 14 ignored, 0 failed across 260 suites (211.51s); all-target native-feature Clippy with `-D warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass.
- [x] Docs Red/Green: refreshing the audit made its old `before fdc3528` assertion fail; the regression now requires the exact current head, full-suite totals, and pending exact-head CI run. `public_docs` and `source_layout` pass (8 tests / 2 suites).
- [x] Verify remote status: exact PR head `27e307ec362d85b45aaa98b34d4f426fb173267e`; macOS and Windows Platform Validation jobs in run `39423261706` are pending. No remote result is claimed.
- [x] Acceptance: remains **2/25 (8%)** packaged workflows; shipped Rust replacement **0/25**. No UI workflow was accepted by source tests alone.
- [ ] Continue: verify remaining packaged workflows, exact screenshot parity, Windows package/UI, accessibility/Korean IME, provider integration, idle resource use, and final license closure.

### Restore an accessible close action across Settings dialogs — issue #9 / R-17 / DD-160

- [x] Plan: finish the recorded sequential Settings/MCP accessibility audit as one path through MCP, cache, and workspace-sharing dialogs; keep all request/save behavior unchanged.
- [x] Red: a connected native-probe journey failed after the MCP editor had no accessible in-content Close control; the Save action remained visible after the attempted close.
- [x] Task: add a localized Close action beside Save; route it through the same editor-open state used by the window close affordance.
- [x] Verify focused: Settings/MCP/cache/sharing/profile/messenger/update plus public-doc and source-layout tests pass, 54 across 12 suites.
- [x] Verify complete: all-target local suite passes 767 tests (14 ignored across 260 suites); Clippy, formatting, diff, public-doc, source-layout, and 300-line checks pass.
- [x] Complete: grouped fix committed/pushed; #9/PR #22, AgentMemory, and Claude signal updated. Packaged acceptance requires exact-build OS review and remains separate.

### Add accessible dismissal to the remaining native Settings dialogs — issue #9 / R-17 / DD-160

- [x] Plan: audit all six native custom dialogs in one pass and preserve existing save, import, and close semantics.
- [x] Red: connected pointer journey found sharing and Messenger dialogs had no in-content Close; a source-reference component test reproduced the same defect.
- [x] Task: add localized Close actions to the Jira/Confluence reference dialog, workspace sharing, and Messenger; make the sharing state respect both title-bar and in-content close.
- [x] Verify: sequential settings/source-reference tests pass (10 tests across 4 suites); full native-feature suite passes 768 tests (14 ignored across 260 suites); Clippy, formatting, diff, public-doc, source-layout, and 300-line gates pass.
- [x] Complete: batch pushed; Wiki, #9/PR #22, AgentMemory, and Claude signal updated. Packaged acceptance remains separate.

### Build and exercise the current macOS package — issue #9 / R-12, R-17 / DD-160

- [x] Plan: produce an optimized binary and package from the current clean source revision without replacing the user's open app or prior archive evidence.
- [x] Task: build `aarch64-apple-darwin`; package exact source `a8be85fc2392821fab345d3e1b879a6aa9bc6714`; run package validation, packaged `--help`, archive integrity, and four mock workflow cases.
- [x] Verify: binary is 17,942,384 bytes; SHA-256 `74204cbc9492379a71769a6f679d19ee28012b823e280e8a6fdd03af89300057`; archive SHA-256 `1965a123dcd5988c24ffeee4d88bfc2b84a62696faf16e6c893c6d49e9c60547`; 214 dependency notices included. Evidence: `docs/evidence/issue9-a8be85f-macos-package.json`.
- [x] Complete: launched one exact-package window in Korean/light mode and verified accessible in-content Close plus dismissal for MCP server settings, source-reference entry, workspace sharing, Messenger, and cache. No external agent/service ran; the disposable workspace was removed and the user's workspace was not modified. Graphify refreshed (7,492 nodes / 12,924 edges).
- [x] Evidence: `docs/evidence/issue9-a8be85f-packaged-dialog-ui.json`. This component-level dialog check does not raise packaged workflow acceptance (2/25); no screenshot artifact was retained.
- [ ] Continue: exact native GUI parity and Windows package/UI verification remain required before merge or release.

### Reconcile the exact issue #9 head and CI status — 2026-10-20

- [x] Plan: refresh current PR head and Actions state after the package-evidence commit; keep package-source evidence separate from the latest documentation/test head.
- [x] Red: update the public-doc regression to require the latest run and cancellation reason; it failed because the completion audit had not recorded the new run.
- [x] Task: retain `7289562` as exact package source and distinguish cancelled run `39415057947` from the in-progress replacement run `39417351655`.
- [x] Verify: the public-doc and source-layout suites pass (8 tests across 2 suites); rustfmt and diff checks pass. The exact packaged source and full-suite evidence remain unchanged; packaged UI acceptance stays **2/25**.
- [x] Complete: commit/push this checkpoint, refresh issue #9 and draft PR #22 with the live head/run, and record/signal the result in AgentMemory. Remote CI remains asynchronous.

### Reconcile exact issue #9 progress checkpoints — 2026-10-20

- [x] Plan: audit the completion audit and native UI Wiki against the branch/PR head, packaged acceptance evidence, latest complete versus partial test results, and exact-head Actions state.
- [x] Red: add a public-doc regression for exact head/CI/acceptance provenance; restore the stale `bb0ff39` checkpoint briefly and confirm the test fails.
- [x] Task: mark `7419557` full-suite/package evidence historical, report the integrated suite as partial, record in-progress run `39412569165`, and refresh the R-14 tool status from current Graphify/CocoIndex checks.
- [x] Verify: `public_docs` and `source_layout` pass (8 tests total), rustfmt and `git diff --check` pass; Graphify refreshed without clustering to 7,422 nodes / 12,771 edges.
- [x] Additional grouped UI regression check: 56 tests across 17 native suites for Room language/stages, MCP registration/sources/keyboard, manual rendering, and release updates pass locally. This automated suite does not add packaged workflow acceptance.
- [x] Complete: issue #9 updated (comment `6289505883`); AgentMemory saved `mem_mvgkzako_8897e734e9f0`; Claude signaled `sig_mvgkzans_fad71ca9f497`; LLM Wiki and Graphify refreshed. Run `39412569165` is in progress and `39412961485` is pending. Commit/push and PR #22 head update follow this checklist commit.

### Align timeout retry guidance across the product — issue #9 / #44 / 2026-10-20

- [x] Plan: audit timeout recovery references across requirements, design decisions, QAS, use cases, code comments, bilingual manuals, generated HTML, and the LLM Wiki.
- [x] Red: the public-manual regression failed because the English and Korean manuals still described timeout as blocked-for-Resume.
- [x] Task: document automatic bounded retry with Stop cancellation; reserve Resume for stopped or terminally blocked runs; clarify DD-07 is superseded by DD-163 and update implementation-facing comments and wiki labels.
- [x] Verify: regenerate both manuals with the Rust renderer; `public_docs` and `source_layout` pass (7 tests total), along with rustfmt and `git diff --check`.
- [x] Complete: pushed `b7140c3`; updated GitHub #9 (comment `6289441518`), saved AgentMemory fact `mem_mvgktb6l_e7ef46be3bd7`, and sent Claude signal `sig_mvgku8sd_b018431c371f`. Exact-head Actions returned no workflow runs. Packaged UI acceptance remains 2/25; this guidance correction adds no accepted workflow.

### Port timeout auto-resume and Stop-aware retry backoff — issue #9 / #44 / #70 / #71 / #72

- [x] Plan: sync the issue branch with the latest `origin/main` reliability fixes and preserve Rust-specific module boundaries.
- [x] Red: the imported Stop/backoff regression first failed because the timeout counter stayed at zero; after counting timeouts it exposed an immediate-retry race caused by second-resolution rounding and a missing pending-retry guard.
- [x] Task: count timed-out attempts, retain per-attempt process evidence, honor a rounded-up retry deadline, wait interruptibly for Stop, and use deterministic slow work in the Stop/resume test. Preserve per-root Git index isolation and messenger re-notification from main.
- [x] Verify: targeted workflow, stop/resume, availability timer, read-only workflow loading, messenger, Git, and source-layout suites pass (24 passed, 1 ignored across 8 suites); all-target native-feature Clippy (`-D warnings`), rustfmt, diff checks, and the 300-line layout gate pass.
- [x] Complete: merge commit `fdc3528` is pushed to the issue-linked branch; issue #9 and PR #22 are current, and AgentMemory/Claude received the retry semantics, evidence, and verification limits. Packaged workflow acceptance remains 2/25; this reliability batch is not UI acceptance.

### Batch native UI parity regressions — issue #9 / 2026-10-20

- [x] Plan: keep scope on issue #9 and combine independent regressions found in one audit; retain the current **2/25 (8%)** packaged-workflow acceptance count.
- [x] Red: reproduced missing Workspace return action in a running Agent Room, loss of the selected Review step after return, missing stop-and-delete for a running plan, and profile editor clipping at a 640px viewport.
- [x] Task: add a guarded stop-and-delete flow that waits for the running plan and metadata queue to settle; keep Agent Room navigation available without stopping the run; preserve the selected Workspace step; make the profile editor fit and stack fields in narrow windows.
- [x] Verify focused: the 24-test batch and a follow-up 8-test core batch pass; changed navigation/profile files are within the 300-line limit.
- [x] Verify complete: final `cargo test --offline --locked --all-targets --features native-ui-probe --no-fail-fast` passes 759 tests, 14 ignored, 0 failed across 258 suites; focused UI regressions pass 25/25 across 10 suites; source-layout passes; all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass.
- [x] Complete: implementation commit `cb49fa1` is pushed; issue #9 progress and AgentMemory/Claude handoff are recorded. Keep PR #22 draft and issue #9 open because packaged Windows and full parity acceptance remain. Packaged acceptance stays **2/25**.

### Package the current issue #9 head — 2026-10-20

- [x] Plan: build the exact pushed source `7419557168ddc3ffe76c446cf292152cc9fc1932` with the Rust package tool; do not count CLI package smoke as native UI acceptance.
- [x] Task / Verify: `package-release`, `validate-package`, packaged `--help`, ZIP CRC, and archive checksum pass. The manifest records one 17,925,856-byte binary, 214 dependencies, and bundled SQLite; binary SHA-256 `6988a2dd852b23665be6f43f4d2d7e20092dbef5e20db49574cd6d1982aa6836`; archive SHA-256 `610f3987658acd5e8c319d827539ab0fe2745d45ee28d2dced754c50365becfb`.
- [x] Verify package smoke: all four shared/worktree × Git/folder cases pass, each with three parallel mock tasks and tests, unapproved-merge refusal, and approved-merge verification. Evidence: `docs/evidence/issue9-7419557-macos-package.json` and `docs/evidence/issue9-7419557-macos-smoke.json`.
- [x] Complete: preserve package evidence in LLM Wiki and AgentMemory; remove stale generated packages/logs; refresh Graphify without clustering (7,399 nodes / 12,744 edges). CocoIndex is not installed on this host. Native UI acceptance remains **2/25**; this does not verify the GUI, Windows package, or a release.
- [x] Verify packaged UI in one isolated macOS window: exact source `7419557` visually checked Workspace, Settings/MCP, profile registration, and Agent Roles; created two temporary workspaces, archived an inactive one, then started a delayed Rust mock plan and confirmed Stop planning and delete. The coordinator returned to Ready and the active workspace was archived into a fresh workspace. No provider or external service ran. Evidence: `docs/evidence/issue9-7419557-packaged-ui-audit.json`.
- [x] Acceptance accounting: this covers component interactions only, not a complete workflow; native UI acceptance stays **2/25**. The Rust package remains open for review in the same single window. Windows, authenticated providers, full workflow parity, public screenshots, accessibility/IME, resource use, and exact-head CI remain open.

### Refresh the full-product completion audit — 2026-10-20

- [x] Plan: reconcile `docs/09-completion-audit.md` with current issue #9 evidence and keep source coverage, packaged UI acceptance, and shipped replacement as separate measures.
- [x] Task: update the current checkpoint and the R-12–R-17/R-24 rows; leave GUI, Windows, authenticated CLI, accessibility/IME, resources, and final license closure visibly open.
- [x] Verify: `public_docs`, `source_layout`, rustfmt, and diff checks pass; Graphify was refreshed without clustering to 7,400 nodes / 12,745 edges.
- [x] Complete: audit and LLM-Wiki updates are pushed as `bb0ff39`; PR #22 metadata/body and issue #9 progress now identify the current head and 2/25 packaged acceptance; AgentMemory and Claude received the grouped UI audit and verification limits.

### Current packaged native UI acceptance — 2026-10-23

- [x] Workflow 1 — shared-folder parallel implementation: exact v0.1.31 macOS package; three scoped mock tasks, tests, review, approval, and merge. Evidence: `docs/evidence/issue9-35a5c09-native-workflow-01.json`.
- [x] Workflow 2 — named-workspace lifecycle: create/save/switch/reopen/archive in an exact isolated package. Evidence: `docs/evidence/issue9-a8be85f-workspace-ui.json`.
- [x] Workflow 3 — CLI profile management: exact package binary SHA `04325483569c54cf2359c002e4066e7b6899280f45d38030bf4b66ceacc267bd`, one isolated macOS app window, empty HOME. Registered `/usr/bin/true` as `ui-smoke`, edited its ID to `ui-smoke-edited`, confirmed the explicit remove prompt, and verified the empty-agent state. No task was run and no provider CLI was invoked. Evidence: `docs/evidence/issue9-2fc7bc2-agent-profile-ui.json`.
- [x] Workflow 4 — native UI plan-to-merge: registered the local Rust fixture in the packaged app, reviewed/approved the exact plan and diff, passed 1/1 test, confirmed merge, and reached Completed. Evidence: `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`.
- [x] Workflow 5 — repository-switch and parallel worktree plan-to-merge: in the exact macOS package, switched a saved goal to a clean disposable Git repository, generated three disjoint worktree tasks, passed 3/3 tests, reviewed/approved the exact diff, merged, and confirmed the final worktree clean. Evidence: `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`.
- [x] Workflow 6 — Jira/Confluence source import: exact v0.1.31 package discovered both MCP tools, previewed both records with content/URLs/hashes, then applied them only after separate operator approval. Evidence: `docs/evidence/issue9-2fd8e57-mcp-import-ui.json`.
- [x] Workflow 7 — workspace settings export/import: exact `256db1a` macOS package exported settings, previewed the import, waited for explicit “Import and replace settings” confirmation, then verified the original name was restored. Evidence: `docs/evidence/issue9-256db1a-settings-share-ui.json`.
- [x] Workflow 8 — MCP server registration/discovery: exact v0.1.32 package saved and reloaded an OpenCode-shaped local server, then discovered `get_issue` from a Rust stdio fixture. Evidence: `docs/evidence/issue9-256db1a-mcp-server-registration-ui.json`.
- [x] Workflow 9 — post-approval charter drift refusal: exact macOS package `aa8090405239` approved the exact candidate in a disposable copy, then the stored plan goal was changed without changing its matching baseline charter. The packaged merge command exited 1 with `[goal_drift]`, recorded one judgment, and left the repository HEAD unchanged. The Agent Room displayed the localized blocked judgment and disabled Merge while retaining 3/3 task/test evidence. Original approval fixture restored untouched; no provider CLI or external service ran. Evidence: `docs/evidence/issue9-aa80904-goal-drift-workflow.json`.
- [x] Revalidation — exact PR merge-result package repeated Workflow 1's shared-folder three-task plan-to-merge journey with 3/3 tests and explicit merge approval. This confirms the latest package but does not add a distinct workflow. Evidence: `docs/evidence/issue9-d386e6a-native-ui-plan-to-merge.json`.
- [x] UC-79 regression: exact packaged v0.1.31 showed the active workspace could be archived, then replaced it with a fresh default workspace. The new AccessKit test failed first because `Archive workspace Current` was exposed; the Rust view now hides Archive for the current item and retains it for inactive items.
- [x] Verify macOS package at exact source `4de454a3e605cc5f8f368eb264cec8fc601145e6`: `package-release`, `validate-package`, packaged `--help`, and ZIP CRC checks pass. Binary SHA-256 `fe776f6275c6b1bf4ce0898bf44a2e282aafca0048f77b2b74c221fb486b5504`; archive SHA-256 `e99de4c6aa1e1108c7cb9fa2618238a00669536cf4976de2129d6210ecbabe79`; all four shared/worktree × Git/folder smoke cases passed with approval-gated merges. Exact package UI inspection showed no Archive action on the active item and Archive on the inactive item; the user's workspace was inspected read-only.
- [x] Verify focused: `native_ui_workspaces` 7/7, targeted native-feature Clippy with `-D warnings`, `cargo fmt --check`, `git diff --check`; changed source/test files are 209/289 lines. Evidence: `docs/evidence/issue9-4de454a-active-workspace-archive-guard.json`.
- [x] Full local all-target suite at product code `1b1a920`: **794 passed, 14 ignored, 0 failed across 263 suites**; all-target Clippy (`-D warnings`), rustfmt, diff, public-doc, and source-layout checks pass.
- [x] Exact-source macOS package at current HEAD `0d09b8843fd66bdd55c7fce2f43e5780c1b2dc6a`: package build, manifest validation, binary `--help`, ZIP CRC, and package-content scans pass. Binary SHA-256 `1425a6384fd7b24382acc91226de007fe8646c6b579e5d9d632d4b3f427b51db`; archive SHA-256 `3c06a3d3ea4407359a0fdc48fdfb04341a3627ad892c2b7872f81701fd81f009`; 214 dependency records. Evidence: `docs/evidence/issue9-0d09b88-macos-package.json`. This package check does not add UI workflow acceptance and was not launched, preserving the existing app window.
- [x] Exact-source macOS package at current product HEAD `97a16bb4ca40070760d6052e5bd095ef6b1c4a3c`: `package-release`, manifest validation, packaged `--help`, and ZIP CRC pass. Binary SHA-256 `2f9402e0f14b9c90083a7c897211e0e7968be08039543178d12b43ac4ea94eaf`; archive SHA-256 `5380a52ad58763375a168324b928db26cf95d787f57e884ed6da2e3aa83b49e6`; 214 dependency records. Evidence: `docs/evidence/issue9-97a16bb-macos-package.json`.
- [x] Exact-package visual check: after stopping prior package processes, launched one `97a16bb` window with an isolated HOME/workspace. The share dialog is centered in light and dark themes; MCP, Messenger, and cache dialogs are centered and expose their expected controls. The cache is empty and Clear is disabled. No provider ran, settings were saved, or cache deleted. The app remains open on the isolated workspace. No screenshot file was retained; no workflow acceptance is added.
- [x] Grouped stage check in that package: profile registration exposes executable, sessions, quota-refresh interval, fallback and Cancel; all five role rows render; the empty Review stage says no plan is reviewed and keeps Approve disabled until Workspace settings and roles are saved. Registration was canceled; no profile or workspace settings were saved.
- [ ] Exact-head Platform Validation run `39450867050` for `97a16bb` is pending. Windows package/UI and exact current-package screenshot parity remain unverified.
- [x] Exact-package screenshot evidence: retained fourteen 1140×772 images from source `2fd8e57`, covering Workspace, Agent Roles, Review, MCP, sharing, Messenger, and cache in Korean dark and English light themes. Evidence manifest: `docs/evidence/issue9-2fd8e57-visual-audit.json`.
- [ ] Public screenshot release gate: current README/manual assets identify v0.1.24, and the published v0.1.31 app is not this Rust branch. The retained exact-package captures include a disposable `fixture` agent; do not replace release screenshots until the Rust UI is accepted and a clean user-facing state is captured.
- [x] Knowledge tools: Graphify updated locally without clustering (7,557 nodes / 11,988 edges) and its query located the packaged-workflow, exact-package, and completion-audit assets. CocoIndex Code was retried; it still cannot import `mcp.server.fastmcp`, so no CocoIndex indexing result is claimed.
- [x] Acceptance accounting: packaged native UI is **8/25 (32%)**; 17 distinct workflows remain, and shipped Rust replacement remains **0/25**. PR #22 remains draft; no merge or release.

### Bound the coordinator analysis popup — issue #9 / 2026-10-21

- [x] Plan: inspect every native `menu_button`/popup for unbounded content after the execution-consent and workspace-menu reflow fixes; keep the page geometry stationary.
- [x] Red: a rendered egui test with a long analysis string found the popup text clip covered the entire 900×700 viewport, so content could extend beyond the display.
- [x] Task: cap the coordinator-analysis menu's inner vertical scroll area at 70% of viewport height and let long content scroll inside the popup.
- [x] Verify: the focused test now checks the painted clip rectangle, passes after the fix, and fails before it. 22 tests across seven UI suites, all-target native-feature Clippy (`-D warnings`), rustfmt, diff checks, and the 300-line source/test layout gate pass; changed Rust/test files are 300/271 lines.
- [x] Runtime: saved and closed the prior app, then opened one current-source binary in a disposable workspace. The empty Workspace screen was visually checked; no provider or external service was used. Screenshot was viewed in-session but not retained in the repository.

### Keep current packaged acceptance assertion current — issue #9 / 2026-10-21

- [x] Red: exact-head run `39606419496` failed `public_docs` on both macOS and Windows because the current plan had advanced to 8/25 while the regression still expected 6/25; Windows UI Automation passed independently.
- [x] Task: update only the current-plan assertion; preserve 6/25 assertions for historical audit/evidence snapshots.
- [x] Verify: the targeted completion-audit test passes; the full `public_docs` suite passes (11/11), `source_layout` passes (2/2), rustfmt and `git diff --check` pass, and both affected UI/doc test files remain at or below 300 lines.
- [x] Knowledge tools: Graphify refreshed this checkpoint without clustering and its query linked the plan, wiki, and `public_docs.rs`. CocoIndex is unavailable on this host (`cocoindex` executable is missing); no indexing result is claimed.
- [ ] Complete: push the grouped issue #9 checkpoint and verify exact-head macOS and Windows Actions. Previous run `39606419496` failed both platforms only on the assertion now fixed.
- [ ] Continue: full screen-by-screen parity, Windows UI Automation behavior, public screenshot refresh, 17 remaining packaged workflows, authenticated providers, accessibility/IME, resource use, and release audit remain open.

### Exact packaged shared-folder workflow recheck — 2026-10-20

- [x] Plan: use the exact v0.1.31 macOS package from product commit `c7f9125` with an isolated `HOME`, isolated AgentConductor data, and disposable shared-folder Git repository.
- [x] Task: register two local Rust mock CLI profiles, generate a three-task plan, and manually inspect each task's assigned agent, disjoint path, arguments, and test command.
- [x] Verify: the first generated proposal was rejected because the mock planner named `mock` while role rules allowed `mock-worker`. Renaming the fixture profile updated its role references; retry generated a valid plan. The operator reviewed the exact execution summary, started the run, and observed 3/3 tasks complete and 3/3 task tests pass.
- [x] Complete: inspected the candidate diff and per-task exit codes, approved the verified candidate, reloaded evidence for the new revision, confirmed the separate merge action, and verified the run completed. Only `one.txt`, `two.txt`, and `three.txt` appeared as untracked files in the disposable shared-folder repository; all contain the expected `implemented` result. The repository had no baseline commit, so this proves shared-folder application, not a Git commit/merge. No provider CLI or external service ran.
- [x] Acceptance accounting: this rechecks the same shared-folder parallel-workflow criterion on the exact v0.1.31 package; it does not add another distinct workflow to the conservative 1/25 count.
- [ ] Remaining: exact-head macOS/Windows CI, the other 24 packaged workflows, full screen-by-screen parity, public screenshots, Windows UI/package, authenticated provider CLI, accessibility, Korean IME, and idle-resource verification.

### Continue issue #9: packaged Agent Roles width — 2026-10-20

- [x] Red: a rendered native probe measured the Agent Roles card at 657px in an 835px content area.
- [x] Green: set the card's minimum width from its available content width; the focused rendered regression passes.
- [x] Verify local: 13 focused tests across native role/workspace/layout suites, all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. Source and test files remain below 300 lines.
- [x] Package and inspect: exact product commit `c7f9125b35da7f652a64a8a7a668dcb5bae6a8c0` built and passed `validate-package`; binary SHA-256 `13ff514f00d0c0c6419e017033fcc0cf0374787ec3644960a84cec75bbb1db87`. Opened that package once in a disposable workspace and verified the card spans the content area. The role selectors remain at their legacy intrinsic width, matching v0.1.31 `.roles-table` (`select` width 130–220px). One AgentConductor process remains open.
- [x] Push: product commit `c7f9125` is pushed to the issue-linked branch for PR #22.
- [ ] Verify exact pushed head: Platform Validation run `39390746942` is pending. Keep PR #22 draft; this visual slice does not increase packaged workflow acceptance beyond 1/25.
- [ ] Continue acceptance: 24 end-to-end workflows, remaining screen-by-screen parity, screenshots, Windows UI/package, authenticated provider CLI, accessibility, Korean IME, and idle-resource checks remain open.

### Continue issue #9 from Claude handoff — 2026-10-20

- [x] Plan: Claude assigned issue #9 to Codex; continue only #9. Keep PR #22 draft and do not merge or release before packaged UI, Windows, IME, accessibility, and resource gates pass.
- [x] Task: integrate main v0.1.31 into `codex/issue-9-native-ui-parity`; merge commit `7805b97` is pushed.
- [x] Verify local: 17 tests across five focused suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass.
- [x] Update progress: issue #9 current checkpoint now names exact PR head `7805b9724665309704630d814c3c7c18024b07ce` and Platform Validation run `39384623386`.
- [ ] Verify remote: Platform Validation for `7805b97` is still running; record macOS/Windows results before claiming cross-platform package validation.
- [x] Verify package: exact source `d884d7c769b51c8478fa3919777628b328621d4f` built and validated as a 17,909,232-byte macOS executable; 214 dependency notices, archive ZIP test and checksum pass. All four shared/worktree × Git/folder smoke cases passed with three parallel tasks/tests, rejected unapproved merge, verified approved merge, and no external CLI/service. No desktop window started; archive scan found no project-owned `.js` or `.py` files. Evidence: `docs/evidence/issue9-d884d7c-macos-package.json`. This runtime smoke does not advance native UI acceptance beyond 1/25.
- [ ] Acceptance: packaged native workflow acceptance remains 1/25; 24 end-to-end workflows, visual parity/screenshots, Windows UI/package, authenticated CLI, accessibility, Korean IME, and idle-resource verification remain open.

### Audit README/manual screenshots and inspect the current native package — 2026-10-20

- [x] Inspect current public image assets: `ui-workspace-current.png`, `ui-workspace-dark-current.png`, `ui-agent-settings-current.png`, and `ui-agent-room-native.jpg` all show v0.1.24 and cannot serve as current v0.1.31 Rust screenshots.
- [x] Reuse the existing single app process: close the prior v0.1.30 package and launch exact package source `d884d7c` once. Its AX tree confirms v0.1.31, loaded saved workspace, all three configured agents, model/quota controls, and Workspace step navigation. No new window was left running beside it.
- [x] Visual review: inspected the exact v0.1.31 Workspace screen at 1140×772 in light and dark themes against `docs/design-guide.md`. The selected tab, surfaces, form boundaries, Korean labels, and three-agent rail remained visible without clipping in these views. Restored the saved light theme afterward. This does not cover other workflows, keyboard/IME, or accessibility acceptance.
- [x] Verify local opt-in native launch path on the same source: 86 tests passed across 11 suites; `agentconductor` and `agentconductor-native-ui-probe` checks pass; focused Clippy reports no warnings. No second desktop app process was started. This does not erase the exact-head macOS full-suite failure below.
- [ ] Screenshot export: CUA now returns inspectable screenshots in-session, but `screencapture` still fails with `could not create image from display`; the image also contains a private local path, goal, and quota state. No public image was replaced. Capture an isolated/anonymized exact-package workspace before refreshing README/manual assets.
- [x] Red: exact PR #22 Platform Validation run `39385260992` failed `review_approval_shows_start_feedback_then_opens_the_started_run` after 15.84s. Its first readiness predicate accepted the static Workspace tab before the asynchronous initial snapshot had rendered the configured agent roster.
- [x] Green: wait for both configured agents' edit controls before synthesizing the navigation click, so initial snapshot arrival cannot shift the roster during the interaction. Product behavior and timeout remain unchanged.
- [x] Verify locally: the focused journey passes five consecutive runs; `cargo test --all-targets --locked --features test-fixtures --no-fail-fast`, the 300-line layout gate, all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass.
- [x] Verify prior platform run: Windows validation in `39385260992` passed build, tests, layout, opt-in launch, rustfmt, and Clippy. Its macOS failure was on the prior test setup; push this correction and validate the exact new head on both platforms.

### Repair the design guide verification command — issue #9 / 2026-10-20

- [x] Red: the documented `room_task_rows` target does not exist, so the design-guide command failed before running tests.
- [x] Green: point the guide to the current theme, language, appearance, bilingual, room-layout and row-selection targets.
- [x] Verify: the corrected offline command passes 16 tests across 7 suites. Manual packaged UI review confirmed identical Workspace layout in light/dark themes and restored the prior light setting; no JavaScript-baseline comparison or screenshot file was obtained.
- [ ] Verify: exact PR head `26ca4a9` run `39366430380`; macOS validation passed and Windows build is in progress.
- [ ] Acceptance: no UI workflow count change; packaged native acceptance remains 1/25.

### Validate an exact-head macOS release package — issue #9 / 2026-10-20

- [x] Plan: package the clean issue #9 HEAD `1ff6814fa0f6bce185e3f03fa1e5ce7b894a2cdd` locally for `aarch64-apple-darwin` without opening another app window.
- [x] Verify: `cargo run --offline --locked --release -- package-release` completed; `validate-package` and packaged `--help` both exited 0. Archive SHA-256 `398218d123869d9a3ec01f9dfb84f0188c1f4c6ad71b09500bb57db3ea19a979`; binary SHA-256 `f9654d2799175d058c5951579fed429a4810b007203f7a2ce25ba0c50f6712bd`; 214 dependency licenses. Evidence: `docs/evidence/issue9-1ff6814-macos-package.json`.
- [x] Verify smoke: exact packaged binary passed shared/worktree × Git/folder cases, with three parallel tasks and tests per case, refused unapproved merges, and verified approved merges. This is package/runtime evidence only; it does not increase the native UI workflow acceptance count.
- [ ] Verify: exact PR head `26ca4a9` Platform Validation run `39366430380`; macOS passed and Windows remains in progress.
- [ ] Acceptance: native packaged UI parity remains 1/25 accepted workflows; visual parity/screenshot refresh and 24 end-to-end workflows remain open.

### Build and inspect the current isolated macOS package — 2026-10-20

- [x] Plan: build the exact local #9 source `35a5c095ef2d00871065233dc82be20d53db8abd`; use only an isolated HOME and disposable workspace with three `/usr/bin/true` sample profiles.
- [x] Verify package: `package-release`, `validate-package`, and packaged `--help` pass for `aarch64-apple-darwin`. Archive SHA-256 `76a0c0e4947e48417e91b2f708045e5055c8a6d8de07149cd97b43f50dadb546`; binary SHA-256 `f9654d2799175d058c5951579fed429a4810b007203f7a2ce25ba0c50f6712bd`; 214 dependency licenses. Evidence: `docs/evidence/issue9-35a5c09-macos-package.json`.
- [x] UI review: exact packaged v0.1.30 opened once in its own window. Workspace setup, agent roles, settings/MCP, and review states were inspected. Shared-folder setup saved with separate metadata and project directories; three sample agents and role assignments appeared. No real agent CLI, API, or external service was called.
- [x] Failure-path check: `/usr/bin/true` exits before reading stdin, so plan generation correctly surfaced `Broken pipe (os error 32)` with failed status, logs location, and retry control. This dummy-process result is not an agent workflow acceptance case.
- [x] Workflow acceptance 1/25: the exact v0.1.30 package used a Rust mock coordinator and three mock worker sessions in an isolated shared-folder Git repository. It generated three disjoint tasks, ran all three concurrently, collected passing `git diff --check` results, presented and received human review approval, then merged the candidate. Final status was Completed and the repository was clean. No real CLI, API, or external service was called. Evidence: `docs/evidence/issue9-35a5c09-native-workflow-01.json`.
- [x] Wiki sync: `knowledge/wiki/packaged-workflow.md` now records the 1/25 checkpoint and marks the earlier 2026-10-19 consent-only setup as historical.
- [ ] Screenshot refresh: no exportable screenshot asset was captured; keep README/manual screenshots unchanged until an exact-source image can be saved and reviewed.
- [ ] Acceptance: packaged native UI visual review and the remaining 24 end-to-end workflows are open; accepted workflow count is `1/25`.

### Keep the native start-journey test stable under the full suite — issue #9 / 2026-10-20

- [x] Red: exact-head Platform Validation run `39363867674` failed the final `native_ui_start_journey` wait after 15 seconds; the rendered Workspace still showed the reviewed plan. The local full suite reproduced the same failure. Four focused runs passed, pointing to suite-load timing rather than a confirmed runtime defect.
- [x] Green: extend only the final test wait for the opened Agent Room briefing from 15 to 30 seconds; application behavior is unchanged.
- [x] Verify: focused start-journey runs pass; full serialized native-feature suite passes 744 tests, 14 ignored, 0 failed across 252 suites. Formatting and `git diff --check` pass; source-layout passes.
- [x] Verify: all-target native-feature Clippy with `-D warnings`, formatting, diff check, source-layout, and the 300-line guard pass. Push this test-only correction and validate the exact new PR head on macOS and Windows.

### Reconcile latest main before completing PR #22 integration — 2026-10-20

- [x] Plan / audit: PR #22 head `e508349` is three commits behind `main` at `7c4a4ae`. A no-commit merge analysis found 11 overlapping files, including the profile/model editor, task model picker, plan schema, requirement docs, and Wiki index. No merge changes were retained.
- [x] Task: merge commit `9a0fa8b` integrates the already-merged declarative CLI profile changes while preserving the native UI module boundaries and custom CLI profile behavior.
- [x] Verify: declarative profile (11), editor (2), native profile UI (10), native task UI (9), language regression, source-layout, serial all-target native-feature suite, formatting, staged/unstaged diff checks, and all-target Clippy pass. Changed Rust source/test files remain at or below 300 lines (max 298).
- [x] Acceptance update: the packaged shared-folder planning, parallel execution, test, human review, and merge journey passed; overall workflow acceptance is 1/25. This does not close the remaining journeys or visual-parity review.

### Give the native start-journey UI wait a CI-safe bound — issue #9 / 2026-10-20

- [x] Red: Platform Validation run `39345540466` failed `native_ui_start_journey` on macOS while waiting up to five seconds for the execution-review control; the rest of the applicable macOS tests before that target passed.
- [x] Green: raise only this test helper's UI-state wait bound to 15 seconds; application runtime behavior is unchanged.
- [x] Verify: the CI-matching local all-target test command passed 749 tests, 14 ignored, 0 failed; the affected native start journey passed three consecutive focused runs; `cargo fmt --all -- --check` and `git diff --check` pass.
- [x] Verify: exact pushed head `e508349` passed Platform Validation on macOS and Windows in [run 39347740983](https://github.com/neodamul/AgentConductor/actions/runs/39347740983), including native packaging, bilingual manual rendering, and execution of the exact package binaries. Do not count this test fix as workflow acceptance; the cross-platform UI audit remains 0/25.

### Fix cross-platform formatting failure — issue #9 / 2026-10-20

- [x] Red: Platform Validation run `39339146839` failed `cargo fmt --all -- --check` on both macOS and Windows for two formatting-only differences. Builds, source layout, native tests, and opt-in UI launch passed on both platforms; native-package was skipped.
- [x] Green: apply rustfmt output to `src/native_ui_probe/render.rs` and `src/native_ui_probe/workspace/sidebar.rs` in commit `a4f88cc`. Local `cargo fmt --all -- --check`, `cargo test --offline --locked --test source_layout -- --test-threads=1` (1/1), `cargo clippy --offline --locked --all-targets --features test-fixtures -- -D warnings`, and `git diff --check` pass.
- [x] Environment: upgraded stale Homebrew LLVM 23.1.3_1 to 23.1.3_2 to match installed Z3 5.3.1; removed the temporary alias. Plain cargo/rustfmt commands pass.
- [x] Package: exact local source `badbb221e0b4e44ad21c46ee2732d7f61f7a4a29` built for `aarch64-apple-darwin`; `validate-package`, binary `--help`, ZIP integrity, and all seven retained archive checksums pass. Archive SHA-256 `368f25124239ba1f24a606fe6395eeb46810c7004bcec16c7e50f22f9030a0d8`; binary SHA-256 `a4911de4c1126cebf5be4ead2c962b3e4f6cecb6faa1ab95cd6df3dd17a421f4`; 214 dependency licenses; archive scan confirms zero JavaScript/Python files and zero HTML inline scripts. Evidence: `docs/evidence/issue9-badbb22-macos-package.json`.
- [x] Verify smoke: all four shared/worktree × Git/folder cases pass with three parallel tasks/tests each, refusal of unapproved merge, and verification of approved merge. Evidence: `docs/evidence/issue9-badbb22-macos-package-smoke.json`.
- [x] Verify: pushed five local commits through `56b2f8043679d54edfc747d33359fd83e8f8f972` after run `39339146839` terminated. Platform Validation run `39341425774` passed on macOS and Windows: build, source/test layout, native tests, opt-in UI launch, formatting, and Clippy.
- [x] Verify: exact current source passed `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1`: 729 passed, 14 ignored, 0 failed across 250 suites. This automated suite does not count as packaged desktop visual acceptance.
- [x] Package verification: run `39341425774` built, packaged, and executed the macOS release binary from merge source `4f70114fb63e0c01ad3d9531c3c87f52a2be0320`. The 446-entry ZIP checksum matches `690260dd0406e9b60c232f2d5cdbb5d8b679174fa24e33f64177df24d5564089`; its manifest binary checksum matches `c4e97af38bfab603a9ebb5028bb3fc7af42723bf705173cc0d55c755234a1b21`, lists 214 dependencies, and the archive contains no project JavaScript/Python files. Evidence: `docs/evidence/issue9-4f70114-ci-macos-package.json`.
- [x] Package verification: run `39341425774` built, packaged, and executed the Windows release binary from the same validated merge source. The 414-entry ZIP checksum matches `4050e36b58559f93661cde1180fda352f3f19be5b0673e51228d58ba4c95eb66`; its manifest binary checksum matches `fc125386684c048addc4fcc32d3a396e4a8abfa8147befba9f01359e255c1c64`, lists 201 dependencies, and the archive contains no project JavaScript/Python files. Evidence: `docs/evidence/issue9-4f70114-ci-windows-package.json`.
- [x] Package verification: both exact-source artifacts passed checksum and manifest audit; binaries were built, packaged, and executed by their platform CI jobs. No GitHub Release has been published.
- [ ] Verify / screenshot refresh: current README Workspace capture is a verified Rust image from `392aa20`/v0.1.24, not WebView; the open v0.1.30 window is package `fd177ea`, differs from latest UI source, and holds populated workspace data. Capture current screenshots only from an isolated exact-HEAD window. Do not advance UI acceptance from these older captures.
- [ ] Acceptance: packaged native UI visual review and all 25 end-to-end workflows remain open; accepted workflow count stays `0/25`.

### Previously pushed PR-head macOS package — 2026-10-20

- [x] Plan: package pushed issue #9 head `4b0a928f65c00fece86a42992602bd7e945ae51d`; Rust application sources were unchanged after the preceding package evidence.
- [x] Verify: `package-release` and `validate-package` pass for `aarch64-apple-darwin`; archive SHA-256 `8db25928b98f8ccd0c858f9fc3c32f3789d7ec201579c2d247d16e976f4217df`; binary SHA-256 `150dfe6f8473ba38c28606c1b0d2e248761022ec5b7d7225199584a3d94666cb`; 214 dependency license records. Evidence: `docs/evidence/issue9-4b0a928f-macos-package.json`.
- [x] Verify smoke: four shared/worktree × Git/folder cases passed; all six then-retained archive checksums passed.
- [x] Verify: run `39339146839` is terminal. Both platforms passed build, layout, native tests, and UI-launch checks, then failed rustfmt; native-package was skipped.

### Integrate reduced-motion status behavior from latest main into the #9 Rust UI

- [x] Plan: synchronize the issue branch with `origin/main` at `b2b3063`; preserve the #9 module splits and the source/test 300-line limit while carrying the reduced-motion behavior into the native UI.
- [x] Red: before the port, focused tests failed to compile because the shared motion-aware status color and planning signature were absent from the #9 module layout. The original #59 regression also reproduced pulse-only repaint scheduling with animations disabled.
- [x] Green: use egui's `animation_time` to keep running lights steady when motion is disabled and skip pulse-only repaints in task rows, agent cards, and snapshot lists. Apply the same color choice to the run banner, verification label, and coordinator indicator; preserve the review's independent elapsed/output refresh and Agent Room polling.
- [x] Verify: full local `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1`, all-target feature Clippy (`-D warnings`), rustfmt, staged diff checks, and the source-layout 300-line guard pass. The authenticated provider integration tests remain ignored by their explicit opt-in guards.
- [x] Package: exact current source `1e7e1bee3baa41296daa9470785cf34f24489a86` built and passed `package-release` plus `validate-package` on macOS. The archive SHA-256 is `072e21a87edf97ed93e33be93c5c1ba8212678007419e5d30bb05c61505f5824`; binary SHA-256 is `150dfe6f8473ba38c28606c1b0d2e248761022ec5b7d7225199584a3d94666cb`; 214 dependency license records are included. All four packaged shared/worktree × Git/folder smoke cases passed with three parallel tasks/tests and approval-gated merge. Evidence: `docs/evidence/issue9-1e7e1bee-macos-package.json`.
- [ ] Acceptance: exact-head Windows package and native visual workflow acceptance remain open; do not advance the 0/25 end-to-end counter from package/CLI evidence alone.

### Retry invalid overlapping phase scopes — issue #9 parity / #52
- [x] Plan: preserve scope rejection; only the exact unsafe/overlapping-scope validation error may trigger one correction attempt requesting literal disjoint repository-relative paths.
- [x] Red: the deterministic fixture returned duplicate `score:` paths; the #9 branch failed with `phase 01-game-selection: unsafe or overlapping scope: score:` before the fix.
- [x] Green: keep the safety validator, retry once with its exact error and a corrected-plan instruction, and retain both subprocess logs; a second invalid response remains a failure.
- [x] Verify: 23 planner/schema/MCP/overlap/source-layout tests pass; full serialized suite passes 725 tests, 14 ignored, 0 failed across 249 suites with debug symbols and incremental artifacts disabled. All-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass. Rust/test files are 274, 45, 73, and 14 lines.
- [x] Release: rebuilt the optimized binary from the updated source; `target/release/agentconductor --help` exits successfully.
- [x] Complete: the Rust #52 parity port and its TDD evidence are on this #9 issue branch; exact latest-main integration and cross-platform package/UI acceptance remain open under #9.

### Wait through transient SQLite writer contention — issue #9 parity / #56
- [x] Plan: preserve transaction behavior and use one bounded timeout for writable and read-only SQLite connections.
- [x] Red: `tests/state_contention.rs` held `BEGIN IMMEDIATE` for 2.5 seconds; the #9 branch failed after its current two-second timeout with `database is locked`.
- [x] Green: apply a shared ten-second timeout to both connection paths; keep existing transaction boundaries and visible timeout errors.
- [x] Verify: full serialized suite passes 726 tests, 14 ignored, 0 failed across 250 targets; all-target Clippy (`-D warnings`), rustfmt, `git diff --check`, and the source-layout 300-line guard pass. `src/state.rs` is 243 lines and `tests/state_contention.rs` is 56 lines.
- [x] Release: exact optimized binary rebuilt successfully; `--help` exits 0; SHA-256 `9313d3adfa62682d1b85930eaffe9956ca364dc352587c586204ae19282d8562`.
- [x] Complete: the Rust #56 parity port and TDD evidence are on this #9 branch; exact latest-main integration and cross-platform package/UI acceptance remain open under #9.

### Keep finalization progress visible in the native Agent Room — issue #9 / R-156
- [x] Plan: the default JavaScript behavior was merged by PR #51; this issue #9 change carries the same finalization feedback to the Rust-native run board.
- [x] Red: rendered regressions failed when approved runs with an active merge job showed no distinct Merge stage or persistent busy status.
- [x] Green: show Verify → Merge → Workspace, derive in-progress state from the existing merge job snapshot, explain the apply step in the primary banner, and disable duplicate merge actions.
- [x] Verify: 17 focused tests, the native UI feature-gate guard (2/2), and the complete serialized all-target suite pass after adding the missing gate. All-target native-feature Clippy with warnings denied, formatting, diff checks, and the changed-file 300-line check also pass.
- [x] Verify release build: `cargo build --offline --locked --release` succeeds; `target/release/agentconductor --help` exits successfully. Binary SHA-256: `0652485f9bbf9d2964e50b9b7585194889898f94acb2f21b7d235f8d31aed85a`.
- [x] Package: exact source `ca17284b68e531bfb16476981d80844b53bbaab2` produced a 17,627,872-byte macOS executable, validated 214 dependency licenses, rendered bilingual manuals, and passed all four shared/worktree × Git/folder package smoke cases. Each case ran three parallel tasks and tests, refused an unapproved merge, and verified the approved merge. Archive SHA-256: `786e83c54de8194bbda019f96aaac62d70a5583498638ad3a0b535254d5de933`; binary SHA-256 matches the optimized build. Evidence: `docs/evidence/ca17284-macos-package.json`.
- [x] Commit: `ca17284` contains the feature-gate correction and checklist/Wiki record.
- [x] Complete: pushed the exact macOS package evidence in commit ca17284 after the older Platform Validation workflow finished.
- [x] Verify locally: current HEAD passes the focused finalization, rendered-room, and source-layout suites (15 tests), rustfmt, diff check, the changed-file 300-line guard, and the optimized release CLI build/help check.
- [x] Re-verify after latest-main integration at `842744d`: `native_room_finalization` passes 2/2 and `source_layout` passes 1/1 using the existing macOS Z3 compatibility shim. The merge changed documentation only; these headless tests do not count as packaged visual acceptance.
- [x] Verify: exact current HEAD `4b0a928` macOS package validation passes; Windows package validation and native end-to-end acceptance remain open at `0/25`. See `docs/evidence/issue9-4b0a928f-macos-package.json`.
- [ ] Acceptance: visual desktop workflow acceptance remains open; the 25-workflow native UI acceptance count remains `0/25`.
- [ ] Acceptance: inspect the exact packaged Rust UI and complete the cross-platform operator workflow. Component tests do not advance the verified end-to-end count; it remains `0/25`.

### Render native role-table headers with the active theme foreground — issue #9
- [x] Plan: inspect the exact current macOS screen; the role-table header labels existed in the accessibility tree but were not readable in the light UI.
- [x] Red: add a regression requiring every header to use the light and dark theme foreground; it failed for all four headers in both themes.
- [x] Green: apply the current UI text color to the bold header labels.
- [x] Verify: `native_ui_roles` (7 tests), `native_ui_workspace_style`, `source_layout`, rustfmt, and diff checks pass locally; source and test files remain below 300 lines.
- [x] Runtime: build the exact current release binary (SHA-256 `159c1ff98ec489083e5deadeb1bfe905ce5d1af8def62e2831a21fafe0ede81f`) and verify all five Korean table headings are visible in the isolated macOS app window.
- [ ] Acceptance: confirm equivalent packaged Windows rendering; native end-to-end workflow acceptance remains `0/25`.

### Label a proposed current phase as under review — issue #9
- [x] Plan: compare v0.1.30 `renderPlanPhases()`; when a proposal has a phase, its header says `Reviewing phase n/N` rather than `Next: phase n/N`.
- [x] Red: rendered regression failed because a phase already in the proposal still appeared as the next phase.
- [x] Green: render the localized reviewing-phase heading and status for any non-completed proposed phase; completed stale proposals continue to show the next phase.
- [x] Verify: 2 `native_workspace_goal_list` tests, 9 `native_workspace_phase_progress` tests, native approval/start journey, source-layout, formatting, and warning-denied all-target native-feature Clippy pass.
- [ ] Acceptance: verify both stale-proposal and current-review states in exact packaged macOS/Windows UI; overall workflow acceptance remains 0/25.

### Show goal slugs and progress stages in the goal list — issue #9
- [x] Plan: compare v0.1.30 `renderGoals()` with native Workspace; the legacy list identifies each saved goal by slug and localized stage, then shows its description and selection action.
- [x] Red: a rendered regression failed because the native list showed only goal descriptions and omitted active/inactive slug and stage labels.
- [x] Green: show localized stage and slug for the active goal and saved alternatives; retain each alternative's description and emit the existing revision-bound select action from a separate Select goal button.
- [x] Verify: goal-list and 9 phase-progress tests, native approval/start journey, review copy, source-layout, formatting, and all-target native-feature Clippy pass locally.
- [ ] Acceptance: check compact multi-goal rendering and selection on exact macOS/Windows packages; overall workflow acceptance remains 0/25.

### Preserve mismatch warnings when no phase rows exist — issue #9 / DD-63
- [x] Plan: inspect `slug_progress::progress()` error output; unreadable plans can carry a mismatch while returning an empty phase list.
- [x] Red: a rendered regression failed because Goals & Progress returned before showing that warning.
- [x] Green: keep the progress card when mismatch warnings exist and avoid incorrectly labeling an empty phase list complete.
- [x] Verify: all 9 `native_workspace_phase_progress` tests, `source_layout`, formatting, and warning-denied all-target native-feature Clippy pass locally.
- [ ] Acceptance: verify the malformed-plan warning in exact packaged macOS and Windows apps; overall workflow acceptance remains 0/25.

### Show phase-progress mismatch warnings — issue #9 / DD-63
- [x] Plan: compare legacy `renderGoals()` with the native progress card; legacy displays every progress mismatch beneath the goal.
- [x] Red: a rendered regression failed because inconsistent phase/task state was omitted from native Goals & Progress.
- [x] Green: render each mismatch with the existing warning color and visible warning marker.
- [x] Verify: all 8 `native_workspace_phase_progress` tests, `native_ui_review`, `source_layout`, formatting, and all-target native-feature Clippy with warnings denied pass locally.
- [ ] Acceptance: confirm warning appearance/reading in exact packaged macOS and Windows apps; overall workflow acceptance remains 0/25.

### Restore the coordinator analysis disclosure — issue #9
- [x] Plan: compare v0.1.30 `renderPlanPhases()` with native Goals & Progress; the legacy phase panel includes a collapsed, expandable whole-goal coordinator analysis.
- [x] Red: a rendered regression failed because the localized analysis disclosure was absent.
- [x] Green: show non-empty saved plan analysis in the existing localized, collapsed-by-default disclosure before phase details.
- [x] Verify: all 7 `native_workspace_phase_progress` tests, `native_ui_review`, `native_ui_start_journey`, `source_layout`, and the three-task shared/worktree approval-and-merge test pass; formatting and all-target native-feature Clippy pass with warnings denied.
- [ ] Acceptance: check the disclosure and keyboard operation in exact packaged macOS and Windows apps. Overall native workflow acceptance remains 0/25.

### Restore expandable phase briefs and planned task summaries — issue #9
- [x] Plan: compare v0.1.30 `renderPlanPhases()` with the native Goals & Progress card; legacy phase rows expose briefs and task ID, summary, agent, and state.
- [x] Red: a rendered native regression failed because the upcoming phase brief and task details were absent.
- [x] Green: make each phase expandable, open the current/reviewed phase by default, show localized task status, and preserve the legacy review notice instead of duplicating tasks under review. Align the UI journey with separate accessible task headings/summaries and real scroll-to-control behavior.
- [x] Verify: all 6 `native_workspace_phase_progress` tests, `native_ui_review`, `native_ui_start_journey`, `source_layout`, and the three-parallel-task `shared_and_worktree_modes_run_parallel_then_require_exact_human_approval` test pass. Formatting, warning-denied all-target native-feature Clippy, and diff checks pass locally. Rust source/test files are 235/300, 229/300, 243/300, and 112/300 lines.
- [ ] Acceptance: confirm the exact packaged macOS and Windows rendering and keyboard/screen-reader interaction. This component slice does not advance end-to-end acceptance from 0/25.

### Restore the clickable release-notes link in the native Updates panel — issue #9 / UC-123 / DD-142
- [x] Plan: compare the Rust Updates panel with the requested link-plus-scrollable-body behavior.
- [x] Red: regressions failed because the release-notes control had no accessible link node and arbitrary remote origins could be rendered as browser links.
- [x] Green: replace the clipboard button with an egui hyperlink restricted to `https://github.com/neodamul/AgentConductor/releases/`; preserve the localized accessible label and the existing read-only bounded notes body.
- [x] Verify: `native_ui_updates` 7/7, `native_ui_update_links` 2/2, `source_layout` 1/1, rustfmt, diff checks, and focused Clippy-driver check with warnings denied pass locally. Files are 239/300, 203/300, and 102/300 lines.
- [x] Runtime: built the current guarded Rust source to `target/release/agentconductor` (SHA-256 `9582c6cf9396c2b1baf20a4c8c7a794d861a4e78fbdc6e2326904fc95ac5cc4c`) and opened one isolated macOS window. Settings → Updates showed the actual v0.1.30 release body and the trusted, accessible Release notes link; the check resolved to v0.1.30. Closed the app and removed the temporary bundle and HOME without changing a workspace or opening the external link.
- [ ] Acceptance: verify link activation, keyboard traversal, and scroll behavior in exact packaged macOS and Windows apps. This component fix does not advance overall native workflow acceptance from `0/25`.

### Published v0.1.30 artifact verification
- [x] Plan: verify the already-published stable release without replacing its tag or assets.
- [x] Verify release: [v0.1.30](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.30) points to source commit `2da56d91d35f26448e49a789b1f1ab1e717f5698`; GitHub archive digests and each archive's internal binary checksum pass for macOS and Windows.
- [x] Verify contents: both platform ZIPs contain `README.md`, English/Korean Markdown manuals, generated HTML manuals, screenshots, manifest, license inventory, notices, and SBOM.
- [x] Cleanup: removed downloaded archives and extracted copies from `/private/tmp/ac-release-v0130-verify` after checking.
- [x] Complete: the release predates commit `db68218`, so it does not contain that Windows CI fix.

### Exact-head platform validation and package status — issue #9 / `de95fc5`
- [x] Red: run [39265015072](https://github.com/neodamul/AgentConductor/actions/runs/39265015072) attempt 2 passed the Windows build, layout guard, and full tests, but its 20-minute limit cancelled the native UI validation before it ran.
- [x] Task: raised the platform-validation job timeout to 45 minutes in commit `de95fc5676d6a2907e2718cb3c93a20be088ce24`.
- [x] Verify: run [39271534953](https://github.com/neodamul/AgentConductor/actions/runs/39271534953) attempt 2 passed all macOS and Windows platform checks, including UI tests, formatting, and Clippy. Windows release build, Rust bilingual manual rendering, package execution/smoke, and artifact upload passed; artifact `12197887256` has SHA-256 `0bc32e45402eb7a87aff17a8fa5f21018f42862158e65f7256d35b1a4a50696d`.
- [x] Verify locally at exact HEAD `47a8b37`: optimized release build and `package-release` generated the macOS archive; `validate-package` confirms the source commit, 214 dependencies, and binary SHA-256. All four packaged approval-gated smoke cases passed with three parallel tasks/tests each, refused unapproved merge, and verified approved merge. README, English/Korean Markdown and HTML manuals, license inventory, notices, and SBOM are in the archive. Evidence: `docs/evidence/issue9-47a8b37-macos-package.json`. This local result closes the macOS package-build/smoke gap; it does not establish packaged UI visual acceptance.
- [ ] Acceptance: latest packaged UI screenshots and all 25 cross-platform end-to-end workflows remain unverified; issue #9 stays open at `0/25`.

### Windows custom CLI registry build fix — issue #9
- [x] Plan: use the PR #22 Windows compile failure as the regression and retain the no-follow file-open behavior for registry files.
- [x] Red: Actions run [39253052432](https://github.com/neodamul/AgentConductor/actions/runs/39253052432) failed at `src/custom_profile_registry.rs:63`; Rust reported `E0425` because `std::os::windows::fs` does not export `FILE_FLAG_OPEN_REPARSE_POINT`.
- [x] Task: use the already-enabled `windows-sys` `Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT` constant. No dependency or behavior change.
- [x] Verify: `cargo fmt --all -- --check`, `cargo test --offline --test custom_profile_registry` (5 passed), and `git diff --check` pass locally. This macOS host cannot compile the `cfg(windows)` branch.
- [x] Verify: the `windows-latest` all-target build step in run [39254806260](https://github.com/neodamul/AgentConductor/actions/runs/39254806260) succeeded with the corrected Win32 constant; local registry tests and formatting pass.
- [x] Verify: exact-head retry attempt 2 of run [39265015072](https://github.com/neodamul/AgentConductor/actions/runs/39265015072) passed Windows build, the source/test layout guard, and all applicable Windows tests; macOS passed every validation step.
- [ ] Complete: GitHub cancelled the Windows UI validation compile at 20 minutes, immediately after it started. The platform job limit is now 45 minutes; rerun all Windows UI checks, formatting, and Clippy on the exact updated HEAD before closing this item. Issue #9 remains open and native UI acceptance remains `0/25`.

### Make native UI accessibility regressions locale-independent — issue #9
- [x] Plan: preserve the user's selected interface language while asserting visible retry and settings-sharing controls.
- [x] Red: run [39254806260](https://github.com/neodamul/AgentConductor/actions/runs/39254806260) exposed two English-run failures: the review test searched only for Korean text, and the sharing-menu test searched only for `설정`.
- [x] Task: compare rendered text with the app's localized message IDs; derive the integration assertion language from the visible Settings label instead of assuming Korean.
- [x] Verify: both previously failing focused UI tests pass locally; formatting and `git diff --check` pass. They are not product UI regressions.
- [ ] Verify / Complete: push the locale-stable assertions and rerun platform validation on the exact updated HEAD. Test-only locale corrections do not advance the `0/25` packaged end-to-end count.

### Keep the mock start journey wait bounded but CI-safe — issue #9
- [x] Red: exact-HEAD macOS run [39256513641](https://github.com/neodamul/AgentConductor/actions/runs/39256513641) failed only in `native_ui_start_journey`; logs show the full mock workflow took 6.13 seconds while the final Agent Room UI wait expired after 5 seconds.
- [x] Task: preserve the five-second bounds for immediate UI transitions and give only the post-start Agent Room transition a 15-second bound; no product timeout or runtime behavior changes.
- [x] Verify locally: native start journey, native header, and source-layout suites pass (5 tests); rustfmt and `git diff --check` pass. The test helper remains within the 300-line cap after extraction.
- [ ] Verify / Complete: push this correction and rerun the cross-platform checks on its exact HEAD. This fixture stability correction does not count toward packaged UI parity (`0/25`).

### Restore full-width Workspace stage navigation — issue #9 / R-128 / DD-137
- [x] Plan: compare the legacy `v0.1.21` Workspace stepper with the Rust `workspace/header.rs`; the old UI gives each of its three stages an equal-width tab across the content column.
- [x] Red: the new localized rendered-bounds regression failed on the old native tabs at 480 px; English widths were `142/178/205px`.
- [x] Task: render the existing three Rust stage buttons at equal widths with a full-width selected underline; preserve labels, keyboard focus, selection, and current navigation events.
- [x] Verify: native-header and source-layout tests pass (4/4); rustfmt and `git diff --check` pass. English and Korean tabs are equal width at 480px and 836px without leaving the viewport.
- [ ] Verify / Complete: inspect the exact app window against the legacy hierarchy on macOS and Windows. Existing UI process was left untouched and no additional window was opened. This component change alone does not advance packaged workflow parity from `0/25`.

### Match the shared Workspace page eyebrow — issue #9 / R-128 / DD-137
- [x] Plan: compare the legacy startup header at `v0.1.21:src/ui/startup.html`; it uses the same `Prepare a goal` eyebrow on all three stages, while the stage name is shown once inside the selected card.
- [x] Red: the bilingual rendered header regression failed on English step 1 because `Prepare a goal` was missing; each step displayed a second stage-specific eyebrow.
- [x] Task: use the existing localized `startup.prepare_a_goal` copy in the native page header and keep stage names in the full-width stepper/card.
- [x] Verify locally: native-header and source-layout suites pass (4 tests); focused Clippy, rustfmt, diff check, and 300-line guard pass.
- [ ] Verify / Complete: inspect the exact Rust app screenshot and regenerate public manuals after visual review. Workflow acceptance stays `0/25` until packaged workflows pass.

### Align the native Workspace accessibility probe with the shared eyebrow — issue #9 / R-128 / DD-137
- [x] Plan: retain a single visible step label after moving the shared page eyebrow out of the stage-specific header.
- [x] Red: exact-head Platform run `39259634447` failed `native_probe_controls_have_accessible_names`; the rendered label count was `1`, while the test expected the old duplicated count `2`.
- [x] Task: update the existing AccessKit/rendered-text assertion to expect one step label and describe the no-duplicate behavior.
- [ ] Verify: push this assertion correction and rerun exact-head macOS/Windows tests. Local compilation is currently unavailable because Homebrew LLVM requires missing `libz3.5.1.dylib`.
- [ ] Complete: visually inspect the current Rust app and refresh the Workspace screenshots/manuals. This assertion correction does not advance end-to-end acceptance from `0/25`.

## Exact packaged Agent Room runtime acceptance — issue #9 / UC-79

### Main 0.1.30 integration regression — issue #9 (2026-10-19)
- [x] Plan: integrate the latest `origin/main` changes into the issue-linked Rust branch; preserve Rust-native UI paths when resolving removed WebView modules.
- [x] Task: reconcile the split Rust modules, quota command parsing, active-run feedback, and existing UI/test callers. Update timeout recovery so blocked errors remain visible, and align regression fixtures with the current explicit-resume behavior.
- [x] Verify: `cargo fmt --all -- --check`, offline all-target Clippy with `-D warnings`, and `cargo test --offline -- --test-threads=4` pass on macOS. Both offline HTML manuals were regenerated from the Markdown sources.
- [x] Cleanup: remove only this checkout's generated `target/debug` build/test artifacts after verification; preserve source, `target/release`, `dist`, and active temporary worktrees.
- [x] Complete: pushed merge commit `d769088` and posted GitHub #9 progress comment `6267098722`. Exact packaged acceptance and cross-platform 25-workflow UI parity remain open; the truthful status remains `0/25` until each workflow is individually verified.

### Exact 0.1.30 macOS package and native Workspace — issue #9
- [x] Plan: build from committed HEAD `dbad31da69cc5dde04559af1e4695686dc2ae53b`, then check the package manifest and open one isolated native window without invoking an authenticated provider.
- [x] Verify package: Rust `package-release` completed all packaged shared/worktree × Git/folder smoke cases; `validate-package` accepted the exact executable, 214 dependency license records, and source commit. Binary SHA-256 `d2b66628f69d248e5dc92b1244c035afb783ab4280057831fa20174ae0e38e85`; ZIP SHA-256 `07b8956372b0f10ddbbacbfd74e645d1ac345a1c1eeebf2e971acd57a740f9eb`.
- [x] Verify native window: opened that executable once in a temporary macOS bundle at 1140×772 with an isolated HOME. The v0.1.30 screen exposed the Workspace, Agent roles, Review plan, CLI agent and Settings controls. Saved a disposable workspace and confirmed its name in the header and its goal/folder in `startup.json`; no agent CLI or external service ran.
- [x] Cleanup: closed the app and removed its temporary bundle, HOME and workspace. Preserved the exact validated 20 MiB archive and package directory for inspection; removed the temporary Z3 soname alias after Rust tooling finished.
- [x] Complete: exact committed macOS package build, manifest validation, four package smoke scenarios, one native Workspace save, and test-window cleanup are recorded in the package Wiki and AgentMemory.
- [ ] Acceptance: this is exact-package start and Workspace-save evidence, not a legacy UI comparison or complete journey. Windows and exact released UI comparison remain open; verified native end-to-end parity stays `0/25`.

### Exact package planning-to-merge journey — issue #9 (2026-10-19)
- [x] Plan: use the v0.1.30 package binary from source `dbad31da69cc5dde04559af1e4695686dc2ae53b` (SHA-256 `d2b66628f69d248e5dc92b1244c035afb783ab4280057831fa20174ae0e38e85`), an isolated HOME, a disposable Git repository, and local Rust mock CLIs only.
- [x] Task: register a read-only planner and a mock worker; assign development, testing, and documentation roles; generate three disjoint tasks for `one.txt`, `two.txt`, and `three.txt`.
- [x] Verify plan: after the original staged temporary fixture had been cleaned, recreate the fixture. The regenerated plan digest was `9e764764d3d9a0cf57960b04c58c2fe1f61e2686e820cb5766b22d743e5e9381` on base `ed1841f7db554725409a9e4be04539be35d08f32`; this differs from the removed staged plan (`a44381e…`, base `0c17ea0`). Before starting, inspect the regenerated goal, three exact writable paths, CLI arguments, and `git diff --check` commands in the app; the user-approved execution scope remained those three disposable files.
- [x] Start: the v0.1.30 app showed `Starting agent execution…`; all three mock tasks completed with separate session IDs.
- [x] Verify: the app recorded three test exit codes of 0 and showed only the three approved added files. Independently run `git diff --check`; it exits 0. No provider CLI, credential, endpoint, or external service was used.
- [x] Human review and merge: inspect the candidate diff and all three test results in Agent Room, approve candidate `8e152dbfae9432c32d2bfb0a77d1764acd328ea9`, then confirm the separate merge. The app reached `Completed`; the disposable repository has commit `8e152db` on top of fixture baseline `ed1841f`, exactly three added files, and a clean worktree.
- [x] Cleanup: closed the test package process (verified no process referenced the fixture path) and removed only `/private/tmp/ac-issue9-approved`, including its isolated HOME, workspace, disposable Git repo, app bundle, mock binary, and temporary Z3 compatibility link.
- [ ] Acceptance: this mock macOS journey is one evidence slice, not a complete released-UI comparison. Run [39265015072](https://github.com/neodamul/AgentConductor/actions/runs/39265015072) passed every macOS step; Windows build/layout passed, but Windows tests were cancelled when superseded. The exact-current-source screenshot remains unverified: the local compiler aborts in Homebrew LLVM because its required Z3 soname is missing, and a temporary link did not fix Cargo's child process. The link is removed. Windows runtime and the documented cross-platform workflow audit remain open; overall issue #9 acceptance stays `0/25`.

- [x] TDD fixture: use disposable Rust planner/worker CLIs and a temporary Git repository; no provider credentials or API endpoints.
- [x] Build and validate the exact current macOS package from source `eeb90a9ad19d0a9266137dd887eed6cfa36ad0b9`; binary SHA-256 `e3c37bb7ad0e02535a390bd21316cd406c25659b80bebd162627c7e63e634fd5`; archive SHA-256 `0492942fb5a44c2c5b2edcf86e0435776708075d4e37f82dac2e93f439463ef7`.
- [x] Runtime: exact packaged app planned three scoped tasks, presented the exact-plan approval gate, showed `Starting agent execution…`, ran all three tasks, collected their diffs, and passed all three `git diff --check` commands. Agent Room showed `3/3` completed and `3/3` tests passed.
- [x] Human review and merge: in a second isolated run, loaded and inspected the exact diff and test evidence, confirmed candidate approval, and confirmed the separate merge action. The app reached `Completed`; the disposable repository recorded candidate commit `0e3a59d` containing all three expected files on top of test baseline `845fadd`, with a clean worktree.
- [x] Failure-path check: a planner fixture missing a required role was rejected with `task task-0 no role despite role rules`; assigning the planned roles allowed the second plan to proceed.
- [x] Cleanup: closed each sole packaged-app process and removed only the two disposable runtime fixtures, test repository, extracted package copy, and temporary library alias. Preserved the validated 20 MiB archive for review. `/private/tmp` measured 456 MiB after cleanup.
- [ ] Windows exact-package UI/runtime and full released-UI/design/platform comparison remain open. The macOS execution/approval/merge path is verified, but the 25-workflow parity audit remains `0/25` until each workflow satisfies the documented cross-platform comparison criteria.

## Active issue #9 UI acceptance status (2026-10-19)

### Local build-artifact cleanup and regression — issue #9
- [x] Cleanup: `/private/tmp` measured 456 MiB; this checkout had 14 GiB of generated `target/debug` artifacts. After verification, removed only `target/debug`, preserving source, `target/release`, and `dist`. Free disk increased from 26 GiB to 34 GiB.
- [x] Verify: `CARGO_INCREMENTAL=0 DYLD_LIBRARY_PATH=/private/tmp/ac-z3-compat cargo test --offline --locked --all-targets` passed with no failures. The temporary Homebrew Z3 soname alias was removed afterward. Subscription-authenticated and live external-service cases remain ignored by their test guards.
- [ ] Acceptance: this local regression and build-cache cleanup do not count as packaged UI parity; issue #9 remains at 0/25 verified end-to-end workflows.

### Follow-up generated-artifact cleanup — issue #9 (2026-10-19)
- [x] Cleanup: after confirming no AgentConductor app was running, removed the regenerated 6.4 GiB `target/debug` cache and the stale October 17 package output (61 MiB) from the temporary issue #9 package worktree; preserved `target/release`, all sources, and registered worktrees. Pruned one already-missing worktree record.
- [x] Verify: available disk increased from 27 GiB to 32 GiB. `/private/tmp` is now 2.2 GiB because the active issue #16/#36 worktrees started during cleanup; those active directories were preserved.
- [ ] Acceptance: cleanup does not advance UI workflow acceptance; issue #9 remains at 0/25.

### Exact current-HEAD macOS package validation — issue #9
- [x] Build: optimized Rust binary built offline from source HEAD `a8e023b9e4f26a63cc6480d2db9374bd02193cf8` in a temporary Cargo target directory.
- [x] Verify package manifest: the package binary starts and `validate-package` passes for the exact HEAD; binary SHA-256 `ab023bf255a7e0fb87c81dc0b3aac96c27403b0a41481f029be093180a55fb74`, 214 dependency license records, target `aarch64-apple-darwin`.
- [ ] Verify runtime: packaged smoke reached the run phase but the command-spawn boundary returned `Operation not permitted` in this desktop task sandbox, including with `TMPDIR` redirected to `/private/tmp`. No runtime pass or end-to-end workflow is claimed from this attempt.
- [x] Cleanup: removed the task-owned 712 MiB Cargo target, failed package fixture, compatibility alias, isolated HOME/workspace and temporary source checkout; `/private/tmp` returned to 466 MiB.
- [ ] Acceptance: current exact-package UI/runtime, Windows, and the 25-workflow parity gate remain open; issue #9 remains 0/25.

### Replace stale Workspace screenshot with exact Rust capture — issue #9 / DD-137
- [x] Plan: Wiki provenance reported the checked-in Workspace image was captured from the previous JavaScript UI; retain the shared image path so README/manual references and the Rust in-app manual stay aligned.
- [x] Red: existing docs tests validate asset paths and manual embedding, but do not validate screenshot process provenance; the recorded old capture was not Rust.
- [x] Task: open one isolated v0.1.24 Rust app process from source `dbd44aa`, inspect Workspace settings, save only an empty-state capture, and close that process.
- [x] Verify: screenshot is 1140×772, SHA-256 `ed495d6d5c47591ba351db36bc1efa7961dbe896386c643b4be5236ea42b8e1b`; regenerated both offline HTML manuals with the Rust renderer. Twelve docs/manual/native-manual/source-layout tests pass; repository-wide formatting and diff checks pass.
- [x] Complete: updated the screenshot-provenance Wiki, AgentMemory, and GitHub #9; commit/push this capture and verification. Korean/dark Workspace and Agent Room captures remain outstanding; this does not advance packaged UI acceptance beyond 0/25.

### Capture Workspace locale and dark theme from native Rust — issue #9 / DD-137
- [x] Plan: the Rust in-app manual contains separate Korean and dark Workspace assets; provenance did not establish the renderer for either image.
- [x] Red: the bundled manual test verifies screenshot loading and accessibility text, but the existing files were not captured from the exact current Rust app.
- [x] Task: in the same isolated v0.1.24 app process, switch to Korean and then dark theme, capture Workspace settings at 1140×772, and close the app without saving workspace settings.
- [x] Verify: Korean PNG SHA-256 `1712dbc142ad60a777f6ca7b06ace6363931fee47763df22b17b5813cfbab443`; dark Korean PNG SHA-256 `e74c8db7626e8d4910d0ece2b2590aa4e277d8d424ba2ee2b38a44cd270a74e3`. First run reproduced stale generated HTML after image replacement; regenerated both manuals with the Rust renderer. All 12 docs/manual/native-manual/source-layout tests now pass; repository-wide formatting and diff checks pass.
- [x] Complete: updated the Wiki, AgentMemory, and GitHub #9; localized captures are pushed to the issue-linked branch. Exact Agent Room capture and overall packaged acceptance remain open at 0/25.

### Replace Agent Room image with exact Rust capture — issue #9 / DD-137
- [x] Plan: README and both manuals referenced an Agent Room image whose renderer provenance was not established; preserve the shared JPEG path and make the example caption explicit.
- [x] Red: public-doc tests checked asset references/captions but not the process that produced screenshot pixels.
- [x] Task: launch one v0.1.24 Rust app process from source `dbd44aa` against a disposable completed-run record; capture the Agent Room and close the process. The report was seeded directly; no agent CLI or merge was executed.
- [x] Verify: AccessKit reports the goal, completed state, 1/1 tasks, Activity, Diff & tests, Raw logs, Lessons, and task inspector. JPEG is 1140×772, SHA-256 `45a11761b25c3e8465dfad67e5adff8794ba7067a6f7dc387cda0e1809380584`. Rerendered both manuals with the Rust renderer; all 12 public-doc/manual/native-manual/source-layout tests, repository-wide formatting, and diff checks pass.
- [x] Complete: updated Wiki/AgentMemory/GitHub #9 and pushed the screenshot/manual update. This demo screenshot does not count as an end-to-end workflow; packaged acceptance remains 0/25.

### Replace stale agent-settings screenshot with exact Rust UI capture — issue #9 / DD-137
- [x] Plan: audit the screenshot referenced by README and both offline manuals against the current default Rust UI and the screenshot-provenance record.
- [x] Red: the published image visibly identifies itself as v0.1.21 and shows the former WebView shell; current source is v0.1.24-era native Rust UI, and documentation tests only verify the asset reference.
- [x] Task: rebuild `HEAD` (`dbd44aa`) and open the agent-profile editor in one isolated v0.1.24 Rust app bundle; replace the shared image only with pixels from that process.
- [x] Verify: 12 public-doc/manual/native-manual/source-layout tests pass; `cargo fmt --all -- --check` and `git diff --check` pass. Regenerated both offline HTML manuals with the Rust renderer. Captured PNG is 1140×772, SHA-256 `b9d68874b7a47aa3fdfa3fb40cdaf2faf03fbf6bdd09403bde15814e495ea866`; exact release binary SHA-256 `e3c37bb7ad0e02535a390bd21316cd406c25659b80bebd162627c7e63e634fd5`.
- [x] Complete: update AgentMemory and GitHub #9, then commit/push this evidence. Graphify rebuilt (6,766 nodes / 11,722 edges), and CocoIndex structural grep confirmed the manual viewer embeds the shared asset. This asset correction does not by itself advance 0/25 packaged end-to-end acceptance.

### Retire unverified Agent Room status screenshot — issue #9 / DD-137
- [x] Plan: audit every public and in-app manual image reference against the current Rust screenshot provenance record.
- [x] Task / Red: add a public-doc regression that failed because README and both manuals still embedded the October 10 status-light capture without a current Rust process/binary record.
- [x] Task / Green: remove that unverified image from README, English/Korean manuals and the Rust in-app manual bundle; preserve the exact-current Rust Agent Room capture already referenced by the docs.
- [x] Verify: public-doc, native-manual and HTML-renderer tests pass (12 tests); source-layout passes (1 test); both offline HTML manuals were regenerated with the Rust renderer; no stale screenshot reference remains in README, manuals, app bundle or public HTML. Formatting and diff checks pass.
- [x] Complete: document the retired asset in the screenshot-provenance Wiki and remove its unverified bytes from the public asset set. The current exact-Rust Agent Room capture remains linked. This documentation correction does not advance end-to-end acceptance beyond 0/25.

### Embedded locale resources under the 300-line gate — issue #9 / R-158 / DD-168
- [x] Plan: the project goal requires every implementation file to stay within 300 lines; trace the extension through R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. Treat locale JSON under `src/` as embedded runtime resources, not exempt data.
- [x] Task / Red setup: extend the Rust source-layout guard to scan `.json` resources and retain duplicate-ID, locale parity, placeholder and lookup checks.
- [x] Verify / Red: the source-layout test failed on the two original 1,283-line locale catalogs before they were split.
- [x] Task / Green: split English and Korean catalogs into feature shards of at most 300 lines, embed through one shared Rust manifest, merge once with cross-shard duplicate rejection, and preserve all existing IDs and lookup behavior.
- [x] Verify / Green: catalog, source-reference, source-layout and language tests pass; the complete offline test suite passes (the separate doc-test rerun uses the local LLVM/Z3 compatibility path); focused Clippy is warning-free; changed-file rustfmt and diff checks pass; sorted English/Korean reconstruction exactly matches the pre-split catalogs; no authored implementation file exceeds 300 lines.
- [x] Complete: 16 feature shards, maximum 216 lines, preserve every message ID and translation, and reject duplicate IDs. Recorded in LLM Wiki and issue #9; pushed as a focused unit. This implementation-file gate is separate from UI workflow acceptance (0/25).

### Native Agent Room task-detail routing — issue #9 (2026-10-19)

- [x] Plan: verify the approved-run workflow end to end with a local Rust fixture in an isolated temporary Git repository; no subscription CLI or external service is used.
- [x] Task / Red: add a request-contract regression for Room-only packets; it fails because the shared UI worker sent `{"kind":"evidence"}` without the `room:true` discriminator.
- [x] Task / Green: mark every JSON object from the Agent Room worker with `room:true`; when a workspace worker transitions into or out of Agent Room, synchronize its routing mode. Workspace requests remain unchanged.
- [x] Verify: the local three-task plan ran concurrently with disjoint `one.txt`, `two.txt` and `three.txt` scopes; all task exit codes and `git diff --check` results are zero. Initial runtime inspection exposed that a workspace worker stays alive when opening Agent Room. A controller-backed regression now exercises that transition and task lookup.
- [x] Verify: full `cargo test --features native-ui-probe --lib --tests`, doctests, all-target Clippy with warnings denied, targeted `rustfmt --check`, and the native app build pass after the transition fix.
- [x] Runtime acceptance: rebuilt and relaunched one app window; the Workspace-to-Agent-Room transition completed 3/3 fixture tasks. Selecting each task showed its own file, session, and activity. Diff & tests showed the exact candidate and three passing checks; Raw logs showed the JSONL result and empty stderr; Lessons showed the empty reviewed-memory state. Review/reject controls are visible, merge remains disabled pending human approval, and the candidate is unmerged.
- [ ] Acceptance: this is one source-runtime correction only; verified packaged end-to-end parity and shipped replacement remain 0/25. Windows, IME/screen-reader, packaged binaries, and all 25 workflows remain open.

### Rust-only source and 300-line guard checkpoint — issue #9 (2026-10-19)

- [x] Verify: `git ls-files '*.py' '*.js' '*.mjs' '*.cjs' '*.ts' '*.tsx'` returns no project-owned source or test files on this branch; the earlier estimate of 19 Python contract modules is obsolete.
- [x] Verify: `cargo test --offline --locked --test source_layout` passes 1/1 on HEAD `a11848e`, confirming the current source/test separation and 300-line bound for the exact checkout.
- [ ] Acceptance: source-layout completion does not establish end-to-end UI parity; packaged replacement remains 0/25, with Windows, live IME/accessibility, and all workflow checks open.

### Native Agent Room goal context — issue #9 (2026-10-19)

- [x] Plan: match the released Agent Room's current-goal context above run status; retain the existing three-column task/activity/inspector layout and expose the goal accessibly.
- [x] Task / Red: add an AccessKit rendered-layout assertion requiring `CURRENT GOAL` and the full goal above `RUN STATUS`; `native_room_layout` fails because the Room header omits both.
- [x] Task / Green: render the existing localized goal label and the exact `report.plan.goal` above the status banner; add no dependency or new persisted state.
- [x] Verify: native Room layout (3), review tab (1), task selection (1), Room language (3), and source-layout (1) pass; all-target native-feature Clippy (`-D warnings`), changed-file rustfmt, offline native binary build, and `git diff --check` pass.
- [x] Runtime: replaced the existing isolated review app in-place and inspected the actual Agent Room. AccessKit and the screen show the full goal before run status; the 3/3 candidate remains unapproved and unmerged. No external CLI, user workspace, or additional app window was used.
- [x] Knowledge: refreshed the LLM Wiki and Graphify (6,720 nodes / 11,655 edges). CocoIndex Code remains unavailable because its installed environment lacks `mcp.server.fastmcp`; no new dependency was installed.
- [ ] Acceptance: this single header correction does not establish packaged workflow parity; Windows, IME/accessibility, exact public screenshots, and overall 0/25 workflow acceptance remain open.

### Native sidebar card density — issue #9

- [x] Plan: compare the agent-card detail row with the v0.1.24 sidebar at the same 248 px rail width; a long executable plus permission wrapped to three lines and expanded the card.
- [x] Task / Red: the new rendered-bound test measured the detail label at 48 px high and failed the one-line limit while retaining the full accessible name.
- [x] Task / Green: truncate only the visible label; egui exposes the full value on hover and AccessKit retains the full executable and permission label.
- [x] Verify: sidebar 5/5, density 1/1, bilingual footer 1/1, and source-layout 1/1 pass; focused Clippy (`-D warnings`) and `git diff --check` pass. Exact Rust build inspected at 1140×772 with three isolated `/usr/bin/true` profiles; no real CLI or saved user workspace was used.
- [x] Knowledge: update LLM Wiki and Graphify (6,692 nodes / 11,604 edges); CocoIndex Code structural grep locates the new `Label::new` call; AgentMemory `mem_mveiza4l_07437964dc8a`.
- [ ] Acceptance: this removes wrapped detail rows only; card layout still differs from v0.1.24 and this component check does not increase native end-to-end workflow parity or shipped replacement beyond 0/25.

### Native sidebar compact roster — issue #9 (2026-10-19)

- [x] Plan: keep command and permission visible to assistive technology and on hover while removing their dedicated card row; place the coordinator badge beside agent actions at the measured 248 px rail width.
- [x] Task / Red: rendered-bound tests failed because command details occupied a second row and the coordinator badge was below the action row.
- [x] Task / Green: move command and permission into the agent-name accessible label and hover text; remove the detail row; align the coordinator badge with the action controls and tighten only their local spacing.
- [x] Verify: sidebar 5/5, density 2/2, bilingual footer 1/1, and source-layout 1/1 pass locally. All changed Rust files are rustfmt-formatted and remain under 300 lines.
- [x] Verify: inspect the rebuilt exact binary at 1140×772; coordinator sits beside actions, cards use one fewer row, full command/permission remains in the accessible name, and no card overflows the 248 px rail. Profiles use only isolated `/usr/bin/true`; no task was run.
- [x] Knowledge: update this plan and the LLM Wiki; Graphify now indexes 6,700 nodes / 11,612 edges; CocoIndex Code finds the drag-handle label in the changed controls module; AgentMemory `mem_mvej9gh6_c3d1912a7762`.
- [x] Complete: commit `54330ad` pushed to the issue-linked Rust branch; GitHub issue #9 updated in comment `6257289724`. End-to-end parity remains 0/25.
- [ ] Acceptance: this component increment does not change native end-to-end parity or shipped replacement from 0/25; Windows, full accessibility, IME, packaged release, and all 25 workflows remain required.

### Issue #33 Windows MCP test stability — 2026-10-19

- [x] Task: port merged PR #35 to the issue #9 Rust branch as commit `135197f`, retaining its Rust fixture while adding the 30-second test deadline and the explicit `tools/list` discovery assertion.
- [x] Verify: `mcp_auto_sources` passes 4/4; source-layout, formatting, and diff checks pass locally. The checkpoint is pushed to `codex/issue-9-native-ui-parity`.
- [x] Knowledge: carry the upstream LLM Wiki evidence; issue #33 is closed on `main` after macOS and Windows Actions passed.

### Native sidebar local-first footer — issue #9

- [x] Plan: compare existing English/Korean footer messages against the Rust sidebar; both were present in catalogs but not rendered.
- [x] Task / Red: rendered-text regression using the actual 248 px left panel failed without the footer.
- [x] Task / Green: render the localized heading and approval summary below the scrolling agent list, reserving room so both remain visible.
- [x] Verify: sidebar suite 5/5, localized footer test 1/1, and source-layout test pass; formatting and diff checks pass locally. No agent run or saved state was changed.
- [x] Knowledge: update the LLM Wiki and Graphify (6,680 nodes / 11,590 edges); CocoIndex structural grep found the sidebar entry point without initializing an index; AgentMemory `mem_mvedynhg_5787cdf797cd`.
- [ ] Acceptance: compare the exact v0.1.24 screen and validate packaged macOS/Windows output, keyboard/screen-reader traversal, and constrained window heights. This is component evidence only; accepted workflow parity and shipped replacement remain 0/25.

### Issue #16 start feedback — 2026-10-19

- [x] Plan: reproduce the missing start-progress feedback after plan consent when the Workspace window is resized short.
- [x] Task / Red: the native UI journey test failed against the scroll-contained status because “Starting agent execution…” was not rendered at 700px height.
- [x] Task / Green: render pending/error feedback in a bottom panel outside the scroll area; keep the existing pending labels and async dispatch behavior.
- [x] Verify: the full native-feature offline all-target suite passed, including the new approval-to-Agent-Room journey. The focused test also passes at 700px and asserts the status text bounds fit the viewport. Source layout, formatting, and all-target Clippy pass.
- [x] Acceptance: PR #38 verifies the shipped WebView compatibility path in exact macOS/Windows packages; the existing Rust counterpart is locally verified by `native_ui_start_journey` 1/1. The remaining packaged Rust default-UI and full-workflow acceptance belongs to issue #9.

- [x] Default-path compatibility follow-up: PR #38 merged to main as `65c77b7`; GitHub issue #16 is closed after its macOS/Windows validation and exact-package smoke passed. The Rust branch already has the equivalent start feedback and exact-run transition; `native_ui_start_journey` passes 1/1 here, so no JavaScript was ported into the Rust application.
- [ ] Acceptance: the Rust source counterpart does not establish the full packaged UI workflow parity; issue #9 remains 0/25 until all required packaged workflows pass.

### Workspace settings sharing menu placement — issue #9 / R-52 / UC-42 / DD-59

- [x] Plan: apply the user's navigation requirement to existing UC-42: keep shared workspace export/import in Settings instead of adding controls below the Workspace form.
- [x] Task / Red: the Settings-menu regression failed because “Workspace settings sharing” was absent; the Workspace page still exposed both actions.
- [x] Task / Green: add the localized Settings-menu entry, open the existing sharing flow in a native window, and remove the export/import panel from the Workspace page.
- [x] Verify: the Probe integration test checks the closed Workspace has no export/import actions and the Settings path opens the localized sharing window; sharing-language 2/2, appearance 5/5, workspace/style 11/11, and source-layout 1/1 pass. All-target native-feature Clippy, rustfmt, and diff checks pass.
- [ ] Acceptance: inspect the exact packaged macOS and Windows Settings window and keyboard/screen-reader navigation. This source-level correction does not change verified parity (0/25).

### Issue-linked branch checkpoint (2026-10-18)

- [x] Plan: create the working branch from `main` through GitHub issue #9 Development so the branch is directly linked to the issue.
- [x] Task: preserve the existing Rust migration history and current UI changes on `codex/issue-9-native-ui-parity`; integration commit `91a824e` is pushed.
- [x] Verify: `cargo test --offline --locked --features native-ui-probe --all-targets`, native-feature all-target Clippy (`-D warnings`), `cargo fmt --all -- --check`, and `git diff --check` pass locally on the integrated branch.
- [x] Complete: draft PR #22 tracks the linked branch; superseded PR #18 is closed. Issue #9 remains open.
- [x] Verify: the linked-branch list becomes empty after opening the issue-linked PR; GitHub then shows the PR in the issue's Development section. For #9, verify PR #22's `codex/issue-9-native-ui-parity` head branch. This is expected per [GitHub documentation](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-a-branch-for-an-issue), not evidence that the link was lost.
- [x] Verify / exact macOS build: built HEAD `0c5b7d9` (`78f835c8b6766c3f6367ce7838951aff88c5c8e806131d68a70bfee63db31ef9`), replaced the previous owned app process, and inspected Workspace, Agent roles, and empty Review Plan at 1140×772 in one window. The saved workspace values were not changed.
- [x] Verify / settings component inspection: on that exact build, inspected the Settings menu, MCP JSONC editor, messenger form, and cache preview with screenshots and AccessKit labels. No settings were saved and no cache was cleared; the preview showed 8 removable candidates (0.0 MiB) and 4 protected/review-required items.
- [x] Verify / quota refresh runtime: on the exact packaged binary (`78f835c8b6766c3f6367ce7838951aff88c5c8e806131d68a70bfee63db31ef9`), opened and cancelled the Claude profile editor without saving; Codex refresh immediately showed progress and then 65%/79%, while Claude refresh immediately showed progress and then its CLI-observed weekly 0% and budget-wait state. No generation request was sent. This covers these two configured profiles only.
- [ ] Continue / UI audit: inspect update details, every agent profile/quota interaction, generated-plan and live Agent Room states, and platform-specific controls. These observations do not count toward 25-workflow completion.
- [x] R-20 → BG-09 → SF-11 → UC-11 → FR-02 → QAS-11 → DDR-04 → DD-13: make the native profile registry distinguish an unobserved model catalog from an available model count. Existing locale strings define “Models not queried” / “모델 미조회”; no provider behavior or catalog data changes.
- [x] TDD / Red → Green: AccessKit regression failed when the empty custom profile catalog rendered the misleading “0 models” label; the registry now uses the existing localized unqueried state, matching the main Agent list.
- [x] Verify: profile catalog status regression (English and Korean), native profile actions (10), metadata refresh (2), rustfmt, diff check, and 300-line bound pass locally. This display correction does not advance end-to-end workflow acceptance.
- [x] R-22 → BG-10 → SF-19 → UC-16 → FR-13/NFR-09 → QAS-17 → DDR-07 → DD-26: show an explicit unavailable quota state for paid agents before the first observation, consistent with the main agent list.
- [x] TDD / Red → Green: an AccessKit regression first failed because a new Claude profile showed no quota value or accessible explanation; Team now renders the localized dash/unavailable label until metadata arrives.
- [x] Verify: profile status suite covers English/Korean missing-model and missing-quota states; profile, refresh and source-layout tests pass, as do formatting, diff checks and native-feature warnings-denied checks. No quota CLI is invoked by the render test.
- [x] R-64 → BG-24 → SF-54 → UC-54 → FR-50/NFR-24 → QAS-57 → DD-70: keep each profile's quota value and refresh control adjacent in a compact one-line group.
- [x] TDD / Red → Green: the AccessKit layout test measured a 104px gap because the model refresh action sat between the quota badge and refresh; move quota controls into a nested 5px group and use the small icon button.
- [x] Verify: AccessKit bounds require the badge and refresh action to have the specified 5px gap (±1px) and the button to stay at or below 19px high; retain accessible labels, busy spinner, and custom-profile observed badges.
- [x] R-63 → BG-22 → SF-53 → UC-53 → FR-49/NFR-23 → QAS-56 → DDR-07 → DD-69: use the specified pencil, trash, Models/reload, and quota/reload actions in the Team profile registry.
- [x] TDD / Red → Green: the profile icon regression failed because Edit/Remove were text/emoji controls with no vector strokes; reuse the existing Agent-list vector painter and preserve action names/tooltips.
- [x] Verify: AccessKit action bounds contain the vector strokes for both Edit and Remove, while the Models text/reload label remains visible. Native profile and sidebar action suites pass without adding fonts or dependencies.
- [x] Verify / profile modal: the exact native Rust window exposed an enabled `Save profile` button rendered with the default gray control style, against the design guide's primary-action accent. The action now uses the active theme accent pair; rendered-color tests pass in light and dark themes, and the rebuilt native app screenshot confirms the visible blue primary action.
- [ ] Continue: packaged end-to-end acceptance remains 0/25; complete the remaining platform, accessibility, and release gates below before closing #9.
- [x] Verify / reporting: keep the 25/25 native UI source-coverage inventory separate from 0/25 verified native end-to-end parity and shipped replacement acceptance in this plan, the LLM Wiki, and GitHub issue #9.
- [x] Verify / Workflow 3 source runtime (2026-10-19): exact Rust binary built after source commit `74d3872`, SHA-256 `2e4bcc2ff02576f09c54afd0182389eb23d4a4411b632cd4f447c7ff85905cc0`, opened once in an ad-hoc signed macOS app with isolated HOME and workspace. Added a custom `/usr/bin/true` profile, edited and saved its ID, confirmed the named delete/cancel prompt, then removed it and observed the empty state. No agent task ran. This closes the source-coverage inventory item only; no shipped-release comparison or Windows/package acceptance was performed, so end-to-end parity remains 0/25.

- **Native UI source-coverage inventory: 25/25 workflows (100%) have source implementations; 0 partial, 0 not started. This inventory does not prove cross-platform end-to-end parity.** Source/control presence does not prove functional parity.
- **Verified native end-to-end parity: 0/25 (0%); shipped Rust replacement: 0/25 (0%).** Published v0.1.24 still uses WebView, and exact packaged Windows, full accessibility/IME, and complete cross-platform workflow acceptance remain open. Component tests and package smoke checks do not count as accepted end-to-end workflows. Keep all metrics separate and do not close issue #9 while any release gate remains unchecked.
- [x] Verify / issue #16 source trace: current `main` has no default `native-ui-probe` feature, so its default macOS/Windows path still uses Cocoa/WebView2 and does not show start progress while the start request is pending. This branch defaults to the Rust eframe UI; `Pending::Start` renders a spinner and “Starting agent execution…”. Existing Rust tests cover the pending label. Source analysis confirms the branch difference, not packaged runtime acceptance.
- [ ] Acceptance / issue #16: in the exact packaged macOS and Windows Rust app, click the reviewed-plan start action and verify the pending indicator remains visible from dispatch through Agent Room transition; close #16 only after #9 is integrated and this runtime check passes.
- [x] Task / Red: an actual Rust preview showed the agent-profile editor inline and clipped at the bottom of the Team page.
- [x] Task / Green: render profile editing as a centered, scrollable window; keep it open and read-only while a save is pending.
- [x] Task / Red-Green: the exact Rust Workspace screenshot exposed a white-on-white sharing heading; a rendered-text contrast test reproduced `#FFFFFF` in light mode, and the heading now uses the app text token.
- [x] Task / Red-Green: the exact Team screenshot exposed white-on-white role labels and narrow role cards; rendered-color and AccessKit bounds tests reproduced both, and role rows now span the content width with their add actions aligned to the right.
- [x] Task / Red-Green: Team's per-role add button remained enabled with no selectable agents; AccessKit reproduced this, and the button now disables unless a registered, unassigned agent is available.
- [x] Task / Red-Green: compare the Rust Workspace, Team, and Plan Review screens against the pre-migration layout in `ce5b58c`; restore the shared goal headline and three-stage tabs, the Workspace settings card, the five-column role table, Generate Plan on Team, and the empty review card. Header and role-table regressions failed before the layout changes and pass afterward.
- [x] Verify / exact macOS UI: build and open the updated Rust app; inspect all three stages at the same 1140×772 viewport without editing the saved workspace. At that time the generated-plan state was unverified; the native journey test below now verifies the generated task rows, while exact packaged-plan acceptance remains open.
- [x] Verify / regression: `cargo test --all-targets --offline --locked --features native-ui-probe` passed; focused workspace/header/role/review suites, Clippy with warnings denied, formatting, and the 300-line source-layout guard also pass.
- [x] Verify: English/Korean accessible fields, modal role, and pending-field locks pass focused TDD tests.
- [x] Task / Red-Green: native profile registration's enabled Save action lacked primary-button contrast; a rendered-shape regression failed before the theme accent pair was applied and passes in light and dark themes after the fix.
- [x] Verify: the local default all-target baseline passed 611 tests (0 failed, 14 ignored, 214 targets). After the order-independent execution comparison changed, all 24 tests across six affected persistence/consent targets passed; the eight feature-gated fixture suites plus the current 300-line layout guard passed 21 tests across nine targets. This separates baseline evidence from follow-up coverage.
- [x] Verify: build the exact current local optimized Rust prototype and inspect Workspace, Team, Settings, Review Plan, and Agent Room on macOS; exercise workspace creation/editing, team role assignment, agent registry/profile editing, executable picker, cache cleanup, MCP settings, messenger settings, manual/tools navigation, and disposable-workspace archiving in an isolated temporary home.
- [x] Verify: compare light/dark appearance and English/Korean accessible labels; inspect profile-editor controls at 450px and 740px heights. Regression tests cover accessible bounds, rendered text/contrast, role availability, modal pending state, profile Cancel/Save events, and cache-dialog close behavior.
- [x] Verify / Keyboard-only Workspace: on the exact Rust binary (SHA-256 `c4618e60f777de0f96e21c69864fbe34267686a70f771eebf435eb921cc36d7f`), Tab reaches every enabled Workspace control in order, skips disabled coordinator/import controls, and Space changes execution isolation; the temporary worktree selection was restored to Shared. This is one-screen evidence only, not the full keyboard/screen-reader gate.
- [x] Verify / Keyboard-only Team: on that exact native macOS package, Tab reached the enabled sidebar actions, agent refresh/profile/reorder controls, all three step tabs, and role add/agent/model/remove controls through the final Save control. The lower role rows required scrolling; no save or other state-changing action was activated.
- [x] Verify / Settings-menu keyboard navigation: Down moved focus through English/Korean, Light/Dark, MCP, messenger, cache, and update actions to the final menu item; preference values remained unchanged, and Escape closed the menu.
- [x] Verify / Red-Green: the profile editor's Save/Cancel controls were below the visible viewport at short height; scroll only the editable fields so validation and actions remain visible. The cache dialog's body Close button left the dialog open; preserve the close request and test the full Settings → Cache → Close interaction.
- [ ] Verify: exercise generated-plan rendering after a real coordinator run, live CLI/session/quota/model actions, MCP-backed requests, and messenger delivery; retain these as explicit manual acceptance because they can consume subscription limits or send external requests.
- [ ] Verify: complete keyboard-only/screen-reader acceptance for every screen, real Korean IME composition, and native Windows UI/accessibility/resource acceptance.
- [x] Verify / macOS exact package: current HEAD `2996dc6af777dfe2b511d207e02e83ffdabf81c6` packages for `aarch64-apple-darwin`; manifest and archive checksums validate, 214 dependency license texts and SBOM are included, and all four Git/folder × shared/worktree smoke cases pass with tests plus approval-gated merge.
- [x] Verify / screenshot provenance: packaged exact local HEAD `8e603dd26fabb23486912cdbb3a3505f036ce350` and inspected the embedded `ui-workspace-current.png` and `ui-agent-room-native.jpg`; both visibly show the legacy WebView UI. The currently open installed app is also v0.1.24/WebView. These assets and that app are not evidence of Rust UI acceptance.
- [x] Verify / exact Rust agent cards: opened the rebuilt local binary (`abdaa6aba7919c5d3244b35f0307146e939ce532b763f99e56e86df1d241dde2`) with an isolated HOME and three fixture profiles pointing only to `/usr/bin/true`; checked sidebar order controls, icons, status, model state, and action labels. No authenticated CLI was invoked or user workspace changed. This is a component audit only; total workflow acceptance remains 0/25.
- [x] Verify / exact current-HEAD native window (2026-10-19): built the Rust binary from clean HEAD `392aa20` (SHA-256 `c4eb698b307ed3900dcd4c5bc37d56f08aa91cf920697d7454818b45be6a385f`), assembled a temporary `.app` with the Rust `bundle-macos` command, and opened one isolated window using a disposable HOME/workspace. macOS identified `dev.agentconductor.desktop`; CUA's live tree and screenshot showed the current Rust Workspace. Settings → Workspace settings sharing opened the separate window with Export/Import; the Workspace form had no sharing panel. No settings were changed. Closed the app and removed the isolated bundle and data.
- [x] Cleanup / exact-current inspection: removed 133 MiB of completed Cargo test fixtures from `target/debug/tmp` and the isolated app artifacts; `/tmp` is 0 B and `/private/tmp` is 379 MiB. Preserved reusable Cargo dependency/incremental caches and `target/release`.
- [x] Cleanup / screenshot staging (2026-10-19): confirmed no AgentConductor process remained, then removed the exact disposable capture bundle, HOME, workspace, and duplicate PNGs under `/private/tmp/ac-issue9-visual-392aa20` after the verified screenshots were committed. Other active temporary worktrees were left untouched.
- [x] Verify / screenshot: replaced all three Workspace PNGs with 1140×772 CUA captures from the exact current Rust binary: English/light `27a77fa35456427261bf214ca1828df92e989a7d163fa7fbfc0ece3d7531e2c6`, Korean/light `a6084f81585d81140b2c30d5dc7ed8f7bad300577cd0db84a92117cfa14bd78e`, Korean/dark `d8936c6aad3299b5a4e8310f9fa34fbf2f261b597a2e8e75ebcef529130aaeb7`. Visual comparison confirms the removed Workspace sharing panel is absent and the Settings entry remains. Regenerated both offline HTML manuals with the Rust renderer. Exact packaged Windows and all-workflow parity remain separate gates.
- [x] Verify / embedded manuals: rebuilt optimized `agentconductor` after refreshing the screenshot assets; SHA-256 `0a5a150c45ac696012d5301ee6ae206fc51369fb3a2b57f6885c4d40cb85859d`. The binary now embeds the current Rust-rendered manual assets; Windows package and end-to-end workflow gates remain open.
- [x] Verify / current source layout: `cargo test --offline --locked --test source_layout` passes on the current branch, confirming source/test separation and the repository's 300-line per-file guard for this exact tree.
- [x] Verify / isolated macOS window inspection (2026-10-19): launched the exact `f9eaf26` release build from a temporary app bundle with an isolated HOME; inspected Workspace, Team, Register CLI agent, and MCP server registration. The agent and MCP settings windows share the same centered native frame, title bar, close affordance, and resize behavior; the MCP OpenCode JSONC sample is visible. No profile, workspace, or MCP settings were saved. Closed the review window and confirmed its process exited. This component inspection does not count toward 25-workflow acceptance.
- [x] Verify / exact macOS package workflow (2026-10-19): `package-release` for HEAD `1dbfc2be14a55de5d41abe01c03c6919a2512ec3` produced the `aarch64-apple-darwin` package with 214 dependency license records. Manifest binary SHA-256 is `02781d8408ca0fc10edf64b6c737eedbb949d5b47368ac4fbdac1e25063964a7`; ZIP SHA-256 is `0e3250708aba4474ca94aa9a2526ce881b60f0e22b1cbdadc425974c9801d736`. All four packaged smoke cases passed (shared/worktree × Git/folder, three parallel tasks and tests each, approved merge accepted and unapproved merge refused). This is local macOS package evidence, not a published release or cross-platform UI acceptance.
- [x] Verify / bug-first branch ports (2026-10-19): PR #34 fixed #32 and PR #35 fixed #33; their main-based macOS/Windows Actions and native-package jobs passed before merge. Ported #32 as `d4b7734` and #33 as `135197f` to this issue-linked branch. #32 retains the atomic `request_replacing_changed_context` helper and adds exact tool-call-ID plus stale-approval rejection coverage; #33 uses the Rust MCP fixture with a 30-second timeout and checks `tools/list` evidence. Local focused tests pass: native_worker 4/4, gates 5/5, mcp_auto_sources 4/4, and source_layout 1/1; focused offline Clippy (`-D warnings`), formatting, and diff checks pass. AgentMemory: `mem_mvee4h1v_1606fc039071`. These bug fixes do not advance packaged UI acceptance from 0/25.
- [ ] Continue / exact Rust UI: inspect the packaged Rust application in one window with a disposable workspace, preserve the saved user workspace, and capture new screenshots only after visual acceptance. Rebuild the README and both manuals from those captures.
- [x] Verify / macOS idle sample: the exact package binary SHA-256 `c4618e60f777de0f96e21c69864fbe34267686a70f771eebf435eb921cc36d7f` remained at 0.0% CPU and 62 MiB RSS across a 10-second native `top` sample (5 threads, sleeping); no second AgentConductor process was started.
- [ ] Verify / Windows exact package, installation, and license gates; no release until cross-platform and UI acceptance is complete.
- [ ] Verify / Documentation: capture screenshots only from that exact inspected Rust build and refresh README plus both offline manuals.
- [ ] Complete: close issue #9 only after every acceptance check passes.

### Immediate agent registration — issue #9 / R-48 / UC-40

- [x] Plan: trace R-48 → BG-09 → SF-41 → UC-40 → FR-33 → QAS-41 → DD-56/DDR-13; require transactional profile drafts, immediate profile-only persistence, and retained unsaved workspace fields.
- [x] Task / Red: pointer-event regressions reproduced Save mutating the live registry before success and sidebar Add failing to open the editor.
- [x] Task / Green: open registration directly; stage edits until the asynchronous profile-only save succeeds; preserve the editor on rejection and refresh the revision for retry. Remove the redundant Team registration menu.
- [x] Verify: five UI-to-Room regressions cover Add, Cancel, successful persistence/reopen, rejected save/retry, coordinator/reference reconciliation, and unchanged unsaved goal/name. Three editor boundary tests cover duplicate IDs/capacity, provider switch/cancel, and a concurrently removed agent. The existing Save regression now requires the live registry to remain unchanged.
- [x] Verify / native window: the actual macOS Rust window opens Add directly; Cancel leaves no card; saving custom profiles closes the modal and shows the saved notice; coordinator is set after first registration; an unsaved name/goal remains visible and absent from startup.json. CUA setValue plus AX readback provided the input evidence.
- [x] Task / Red-Green: native inspection found the missing None translation and invisible registration title. Explicit theme ink, registration-specific title, and English/Korean None messages pass a rendered/AccessKit regression in both themes.
- [x] Verify / baseline comparison: shipped v0.1.24 `profileForm(-1)` starts with custom `aipro`. Red reproduced a mismatched Claude default; registration now shares the custom template with the provider selector, and the five UI-to-Room regressions pass again.
- [x] Plan: QAS-124/DD-137 requires every theme surface and text to recolor with readable semantic ink; audit the actual Rust agent-card quota value against both theme surfaces.
- [x] Task / Red: rendered quota regression measured white-on-white at 1:1 contrast in the light theme.
- [x] Task / Green: apply the active theme's semantic text color while retaining the compact percentage and tooltip.
- [x] Verify: the actual sidebar widget render and registered profile labels pass light/dark contrast tests; touched/new files pass the 300-line guard.
- [x] Verify: the complete pre-fix local suite passed 643 tests / 14 ignored / 0 failed across 217 targets; after the color fix, both contrast tests pass. Clippy, formatting, source-layout and the optimized build are being rerun for the final diff.
- [x] Complete checkpoint: evidence recorded, issue #9 updated, and the isolated unit commit pushed; this increment does not establish overall UI replacement acceptance.

### Component audit — native Rust prototype vs shipped UI

- [x] Plan: audit R-102 → BG-57 → SF-89 → UC-92 → FR-87 → QAS-98 → DD-110 against the actual native agent rail; reorder controls/drag behavior are missing despite the previous prototype-complete claim.
- [x] Task / Red: reproduce missing accessible agent move controls in a native widget regression (`missing reorder action claude: Move up`).
- [x] Task / Green: restore drag and keyboard-accessible move actions through the existing ordered-profile save path.
- [x] Verify: exercise both pointer paths, English/Korean boundary/disabled states at 210px rail width, saved order after reopening, and retained coordinator/role/fallback/private-environment references (4 tests).
- [x] Task / Red-Green: reproduce `ready` becoming `null` after a pure reorder, then canonicalize the shared execution view by agent ID (R-50/DD-58). The digest survives reorder/reopen while a real argv change invalidates the proposal; all 24 tests across six affected persistence/consent suites pass.
- [x] Checkpoint: record the implementation evidence in the wiki/AgentMemory and push `12d130d`; keep issue #9 open. Native-window follow-up below records defects that the widget tests initially missed.

### Native-window follow-up — issue #9, 2026-10-18

- [x] Red/Green: actual tabs overflowed the right window edge. The bounds regression reproduced the third tab ending at 556.7px in a 480px column; reserve frame margins and gaps and wrap button text. Both locales pass at 480px/836px.
- [x] Red/Green: actual input fields looked borderless despite the earlier color-only test. Requiring an actual stroke width reproduced 0px borders; inactive input/control strokes now use the paired 1px token.
- [x] Red/Green: the edit font glyph appeared as a missing-character box. Edit/remove buttons now paint vector strokes; a regression rejects the old glyphs and requires visible icon strokes inside the accessible buttons.
- [x] Verify: the first label-based drag passed raw pointer tests but failed native-window dragging. Use egui's dedicated `dnd_drag_source`; the actual macOS window then moved gamma from third to first. The saved file and a new app process both showed gamma/alpha/beta while coordinator beta and the workspace draft stayed intact.
- [x] Verify: 20 focused UI/layout tests pass; all-target/all-feature Clippy with warnings denied, formatting, and diff checks pass. The native-window follow-up used an isolated debug build; the final optimized follow-up executable also builds (17 MiB), and `--help` exits successfully. Its SHA-256 is retained in the raw audit.
- [x] Acceptance / R-48 / UC-40: immediate registration and profile-only save are verified by the native-window Add/Cancel/save/reopen/retry checks above and commit `9112335`.
- [ ] Acceptance: complete 25-workflow comparison, current Rust documentation captures, Windows, IME/accessibility, and final exact-package validation remain open.

### Agent profile editor window parity — issue #9

- [x] Plan: match the agent profile editor's native window frame to the other settings popups, which use titled windows with a title bar and close control.
- [x] Red: the AccessKit regression found the profile editor exposed as a dialog, unlike the other settings windows.
- [x] Green: replace the modal frame with a centered, resizable titled window and remove the duplicate in-form heading; pending edits remain read-only.
- [x] Verify: all 9 focused profile-editor tests, formatting, Clippy with warnings denied, release build, and diff checks pass. The full suite remains blocked by sandbox-denied local TCP binds in unrelated memory tests.
- [x] Acceptance / exact macOS build: rebuilt binary SHA-256 `33e358537cb607a25077d7ef1e3f7b3b882885ddce7569916767c4d8af4030f9` shows the agent profile editor and MCP registration window with matching centered title bars and close controls. Cancelled profile editing and closed MCP without saving. The earlier QA launcher failure came from a stale bundle signature; ad-hoc signing a bundle containing the exact binary opened the app. This component check does not complete an end-to-end workflow or change 0/25.
- [ ] Acceptance: exact Windows app visual comparison remains pending. Issue #9 acceptance count remains 0/25 until a complete user workflow passes its end-to-end gate.
- [x] Verify / isolated macOS workspace save-and-reload smoke: exact source binary SHA-256 `33e358537cb607a25077d7ef1e3f7b3b882885ddce7569916767c4d8af4030f9` opened with a Rust launcher that pinned `HOME` to a disposable directory; saved a new workspace name, repository path, and goal through the visible UI; exited and relaunched the same binary; the visible form and `startup.json` retained all three values. No CLI agent, MCP server, or external service was invoked. Removed the temporary app bundle, launcher source, isolated home, and workspace after verification. This confirms one macOS source-build persistence path only; the exact legacy-flow comparison and Windows/package gates remain open, so verified replacement acceptance remains 0/25.

### Release-notes keyboard access — issue #9 / UC-123 / DD-142

- [x] Plan: compare the current Rust update panel with the shipped legacy update dialog; preserve literal release-body rendering and bounded scrolling while restoring keyboard-focusable body access.
- [x] Red: the rendered regression found no multiline focusable control for release notes; legacy `src/ui/startup.html` at `6ea45ab^` used `<pre id="start-update-body" tabindex="0">`.
- [x] Green: render the release body with egui's read-only multiline text control inside the existing 260-point `ScrollArea`; do not parse remote Markdown or HTML.
- [x] Verify: the focused regression confirms an AccessKit multiline read-only text value and clipped long-content geometry. All 7 update UI tests pass locally.
- [ ] Acceptance: verify real keyboard focus/scroll in the exact packaged macOS and Windows apps; this slice does not close issue #9 or claim end-to-end UI parity.

### Run status banner — issue #9 / R-99 / QAS-95 / DD-107

- [x] Plan: compare the native Agent Room with the legacy `#run-banner`; the Rust view only showed a compact status tag and omitted the run-level completion count and last error.
- [x] Red: `native_run_status_banner::failed_run_banner_shows_status_progress_and_last_error` failed before integration; rendered output had no `Tasks completed: 1/3` or integration-test error.
- [x] Green: add a native, state-colored banner before Agent Room actions; show the translated run state, completed task count, and report/failed-task error when present.
- [x] Verify: the new banner test and all nine `native_room_render` tests pass; source files remain below 300 lines.
- [ ] Acceptance: inspect the actual packaged macOS and Windows Agent Room at supported window sizes and compare banner placement/contrast to the released UI. No workflow is marked accepted by this widget-level slice.

### Failed-run feedback action — issue #9 / UC-100 / DD-118

- [x] Plan: compare the native failed-run banner with UC-100 and DD-118; the banner showed an error but had no action to direct the failed task.
- [x] Red: `native_run_status_banner::fix_with_feedback_selects_failed_task_and_drafts_its_test_output` failed because the accessible failed-run action was absent.
- [x] Green: expose the localized Fix with feedback action for a failed test/task mapped to the reviewed plan; activation selects that task and puts bounded captured output into its existing editable feedback draft.
- [x] Verify: the rendered interaction test clicks the accessible action, checks the failed task becomes selected, then confirms captured test output appears in the visible feedback editor. Rust files remain under 300 lines.
- [ ] Acceptance: inspect real packaged macOS/Windows rendering and verify send-feedback continues the failed run and repeats verification end to end. This widget test does not mark a replacement workflow accepted or change the 0/25 count.

### Review Plan task disclosure — issue #9 / UC-106 / DD-124

- [x] Plan: compare UC-106/DD-124's collapsed-card behavior with native `workspace/tasks.rs`; Rust showed every editable field expanded and omitted the one-line task summary.
- [x] Red: `native_plan_task_cards::review_task_cards_start_compact_and_expand_to_editable_fields` failed because the compact task heading was absent and editor fields were visible immediately.
- [x] Green: render each task closed by default with the shipped two-line summary (`01 · ID` and the first non-label instruction line), a localized role badge beside the ID, and the localized `Remove task` action; truncate instruction summaries at 160 characters with an ellipsis, while keeping all fields editable after expansion.
- [x] Verify: the rendered interaction test confirms the two-line structure, role badge within 480px, truncation marker, English/Korean role and remove labels, hidden full instructions initially, disclosure interaction, and editable fields. Task-edit, review-copy, and source-layout suites pass (10 tests); changed source/tests remain below 300 lines.
- [ ] Acceptance: compare the actual packaged macOS and Windows Review Plan at supported window sizes, including keyboard/screen-reader expansion and multi-task editing. Widget coverage does not change 0/25 accepted workflows.

### Review Plan snapshot-base cue — issue #9 / R-21 / UC-15

- [x] Plan: compare v0.1.30 `startup.js::tasks()` with native `workspace/review.rs`; the shipped Review Plan shows the first 12 characters of the captured repository base and reminds the operator to inspect executables, permission modes, task scope, and tests.
- [x] Red: `reviewed_plan_discloses_its_snapshot_base_and_execution_checks` rendered a ready plan and failed because the native Review Plan omitted this cue.
- [x] Green: use the existing localized message and the proposal's captured repository base; the exact execution-consent dialog remains unchanged and continues to show its full evidence.
- [x] Verify: five Review Plan unit tests, ten task-card/edit/review-copy/source-layout tests, formatting, warning-denied all-target Clippy, diff checks, and the 300-line guard pass.
- [ ] Acceptance: verify the cue and complete consent transition in exact packaged macOS and Windows apps. This rendered component correction does not advance the 0/25 end-to-end acceptance count.

### Agent Room review evidence pane — issue #9 / UC-04 / DD-06

- [x] Plan: compare the legacy Agent Room at `3796b54^:src/ui/room.html` with the native task-activity pane; legacy exposes Activity, Diff & tests, Logs, and Lessons as separate views, while native rendered evidence inline beneath Activity.
- [x] Red: `native_room_review_tab::review_tab_separates_approval_evidence_and_refreshes_stale_results` failed to compile because the native `Tab::Review` was absent.
- [x] Green: add a dedicated localized Diff & tests view; keep evidence out of Activity; render only evidence whose revision matches the current run; show an accessible stale warning and `Review latest` action that requests fresh evidence.
- [x] Verify: the rendered pointer-interaction test selects the Review tab, confirms diff/test output appears there and not in Activity, changes the run revision, then confirms the warning and `Event::Evidence` refresh action. Existing bilingual Activity/Logs/Lessons rendering remains covered.
- [ ] Acceptance: inspect the exact packaged macOS and Windows Room, keyboard/tab navigation, stale evidence after a real run update, and approval/merge gating after refresh. Widget tests do not increase the verified 0/25 workflow count.

### Task-row status attention — issue #9 / R-99 / QAS-95 / DD-107

- [x] Plan: compare the legacy session-row requirement with native `render::snapshot`; native rows had status pills but no failed-row outline or waiting-for-decision/tool pulse.
- [x] Red: `native_run_status_rows::failed_task_has_a_red_row_border_and_tool_wait_border_pulses` failed before row-frame integration because no failed-row border was rendered.
- [x] Green: draw a red frame around failed task rows and pulse approval/tool-wait frames; when egui animation time is zero, keep the waiting frame static and do not schedule animation repaints.
- [x] Verify: status-row rendering test covers red failure outline, changing pulse states, and static reduced-motion state; related Room render, banner, and source-layout suites pass.
- [ ] Acceptance: verify blink legibility and reduced-motion behavior in exact packaged macOS/Windows apps. This slice does not establish the broader row selection/session-list behavior or workflow acceptance.

### Agent Room task/session row selection — issue #9 / UC-89 / DD-107

- [x] Plan: compare legacy `src/ui/room.html`'s `#sessions` navigation with native `render::snapshot`; native task rows exposed status text but could not select a task, leaving only the inspector combo as a route to another session.
- [x] Red: `native_room_task_selection::selecting_a_session_row_loads_that_tasks_activity_and_inspector` failed because the session row was absent as an accessible button.
- [x] Green: make each native task row a selectable, accessible control without changing its prior layout allocation; selection updates the shared task control state so Activity, task details, and the inspector follow the selected task while per-task drafts continue through the existing selection-change path.
- [x] Verify: the pointer interaction test locates the second task by its accessible task/status label, clicks it, and verifies selection plus a `TaskDetails(1)` request. Compared `native_room_layout` at parent `56182ca` and current: its two width assertions fail identically at baseline (`307.8125px` review editor; approval button x1=`906px` against a 792px viewport). Selectable-row sizing initially shifted the approval column further right; the full-row interaction overlay removes that introduced shift and restores the exact baseline x1=`906px`. Packaged visual acceptance remains open.
- [ ] Acceptance: verify row hit area, selected styling, keyboard/accessibility navigation, and Activity/Inspector synchronization in the exact packaged macOS and Windows builds. Do not count this widget slice as an accepted workflow.

### Agent Room narrow-window review and approval layout — issue #9 / UC-04 / DD-06

- [x] Plan: reproduce both existing `native_room_layout` failures on the branch: the task-direction editor was 307.8125px at a 1140px viewport, and the Approve action ended at x=906 in a 792px viewport.
- [x] Red: the same 2 assertions failed against `ec761d0` before changing the layout.
- [x] Green: allocate a wider Inspector when the viewport is at least 1000px; cap decision-action button width to the visible content area and let its label wrap at narrow widths.
- [x] Verify: all three `native_room_layout` tests now pass. The 1140px test confirms the editor is at least 420px and actions remain in bounds; the 792px test confirms all three decision actions remain within the viewport. The standard 1200px column/accessibility test also passes.
- [ ] Acceptance: inspect the exact packaged macOS and Windows Room at minimum and regular window sizes; confirm scroll, keyboard, and visual wrapping behavior. Do not change the 0/25 workflow acceptance count.

### Agent Room approval next-step actions — issue #9 / R-111 / UC-101 / DD-119

- [x] Plan: the Rust status banner showed state/progress/errors, but did not guide the operator through candidate review, approval, and merge. Keep this slice to the approval sequence; completed-run workspace navigation remains separate.
- [x] Red: `native_room_next_step::approval_banner_moves_from_review_to_approve_then_merge` failed because the waiting-approval banner exposed no Review changes action.
- [x] Green: show a localized next-step panel for waiting-approval and approved states. Review opens the Diff & tests pane and loads evidence; Approve and Merge start their existing confirmation flow only when evidence matches the current run revision.
- [x] Verify: the rendered pointer test follows Review changes -> Evidence request, rejects stale evidence by returning to Review changes, then exposes Approve candidate and Merge candidate with the matching existing decisions. No alternate approval or merge request path was added.
- [ ] Acceptance: inspect the exact packaged macOS and Windows banner, keyboard focus, and decision flow. This widget slice does not complete UC-101, change the verified 0/25 workflow count, or close issue #9.

### Agent Room completed-run return route — issue #9 / R-111 / UC-101 / DD-119

- [x] Plan: a completed run had no next action. Route Workspace-origin runs back to the existing Workspace window; standalone Agent Room runs must explain that no Workspace navigation context is available.
- [x] Red: `native_room_next_step::completed_run_banner_offers_return_to_workspace` failed because the completed banner had no Return to Workspace control.
- [x] Green: add a localized completion next-step. Workspace-origin runs expose an accessible action which closes the Room view, restores Workspace stage 1, and refreshes its snapshot; standalone Rooms show localized open-Workspace guidance without a broken control.
- [x] Verify: rendered interaction tests cover the visible/clickable return action and the standalone guidance state. Focused UI tests, Clippy, formatting, source-layout, and diff checks are recorded with this change.
- [ ] Acceptance: verify the return transition and localized banner in exact packaged macOS and Windows apps. This source-probe slice does not complete UC-101 or increase issue #9's accepted workflow count.

### Active goal phase progress and next-phase entry — issue #9 / R-112 / UC-102 / DD-121

- [x] Plan: compare the legacy Workspace goal card and Review Plan phase list with the Rust Workspace. The legacy screen shows each phase's stage/task counts and offers Plan next phase when the active goal has unfinished phases; the Rust Workspace currently ignores `snapshot.slugs`.
- [x] Red: the new rendered regression initially failed to compile because the native Workspace had no phase-progress component or action contract.
- [x] Green: render a compact localized active-goal phase summary in Workspace and route the revision-bound next-phase request through the existing async worker into Review Plan. A completed plan and a phase already open for review do not expose a duplicate action. Keep phase selection, non-active goal switching, and full task/spec disclosure out of this slice.
- [x] Verify: 22 tests passed across the phase-progress, Workspace probe, Review Plan layout/style, and source-layout suites. The request test checks exact `next_phase` kind/revision. Clippy, format, <=300-line layout, and diff checks pass.
- [ ] Acceptance: compare this phase card and transition with the exact packaged legacy/Rust apps on macOS and Windows. This source-probe slice does not increase issue #9's accepted workflow count.

This audit records what was inspected in the unpublished native source build (the current branch defaults to eframe). “Inspected” and passing widget tests do not prove behavior parity with the released WebView application.

| Screen / component | Verified in this slice | Still required for replacement parity |
| --- | --- | --- |
| Workspace | Name, goal, coordinator, project-folder field/picker, worktree choice, reference controls, save, and archive confirmation were inspected; create/save/archive used disposable data. | Compare each validation state and import/export flow with the shipped app; execute Jira/Confluence preview and apply against configured test MCP servers; test shared/worktree paths end to end. |
| Team and agent registry | Role rows, add-role choices, agent cards/status, add/edit/remove affordances, provider/executable/arguments/input/session/quota/fallback fields, pending-state lock, and Save/Cancel visibility were inspected. UI-event tests cover role availability and profile Cancel/Save events. | Compare all add/remove/rename/reference side effects with the shipped app; save and reopen real profiles in isolated config; exercise real model/quota CLI requests without consuming production accounts. |
| Review Plan | Pre-generation state, step title/description, and generation gate were inspected. | Generate a plan with a disposable fixture coordinator; inspect task scopes/dependencies, editing, reassignment, approval, retry, and completion behavior against the shipped flow. |
| Agent Room | Summary, activity, inspector, task/session/model controls, status text, minimum-width action bounds, and feedback input were inspected/tested. | Compare complete live orchestration, approval, stop/resume/reassign, tool decisions, logs, and results with the shipped app. |
| Settings and dialogs | Theme/language controls, Settings menu, Cache dialog open/close, MCP server editor/sample, Jira/Confluence reference editor, and messenger settings were inspected; cache-close has an interaction regression test. | Compare save/reload semantics; verify cache deletion against disposable fixtures in the UI; test update checking, MCP requests, and messenger delivery with safe test endpoints. |
| Manuals and tools guide | Both in-app panels opened and their scrolling/content were inspected. | Agent settings, all three Workspace variants, and Agent Room use exact v0.1.24 Rust captures. The Agent Room image shows seeded demo state, not actual provider execution; compare every screenshot/caption with the shipped app. |
| Global input/accessibility/platform | Light/dark and English/Korean labels, AccessKit bounds, modal state, and selected keyboard traversal were tested. | Full keyboard/screen-reader pass for every screen, real Korean IME composition, native Windows UI/accessibility/resource review, and exact macOS/Windows package acceptance. |

## Active issue #9 status (2026-10-17)

- [x] Plan: reconcile issue #9 with the current canonical branch and its Rust-only application/build-verification scope.
- [x] Task / Red: run the 300-line source-layout test and record the remaining oversized Rust modules and fixture files.
- [x] Task / Green / Refactor: remove legacy browser UI assets and Python/Node-owned tests; keep localization and interaction coverage in Rust native UI tests.
- [x] Task / Green: port subprocess test fixtures to Rust and gate the standalone test fixture binary behind `test-fixtures`.
- [x] Task / Refactor: split oversized Rust modules while preserving their public call paths; every scanned Rust source and test file is at most 300 lines.
- [x] Task / Red-Green: retain Agent Room summary, activity, and inspector as ordered native columns; render inspector selection before requesting activity details and keep task actions ahead of background detail requests.
- [x] Task / Green: make workspace stages vertically scrollable, keep the Settings menu readable, and retain goal text contrast in the Rust theme.
- [x] Verify: native macOS Agent Room fixture visibly renders summary/activity/inspector in three columns; AccessKit exposes all three regions and the status controls.
- [x] Verify: 50 focused native UI tests passed across 7 suites; `cargo fmt --all -- --check`, `git diff --check`, and native-feature all-target Clippy passed.
- [x] Verify: `cargo test --offline --all-targets --features test-fixtures` — 596 passed, 14 ignored, 0 failed across 203 suites.
- [x] Verify: `cargo check --offline --all-targets --features test-fixtures`, `cargo fmt --all -- --check`, and `git diff --check` passed.
- [x] Verify: `cargo clippy --offline --all-targets --features test-fixtures -- -D warnings` passed after the extracted helper borrow fixes.
- [x] Verify: `cargo build --offline --release --locked --bin agentconductor` passed; packaged binary `--help` smoke exited successfully without opening the desktop UI.
- [x] Verify: exact-source macOS package at `98d5357`; ZIP SHA-256 `9b5f9a76a177887865e6ffca585c64f0e341a87889811ff22fa7ed8169846190`; all four packaged smoke scenarios and archive integrity passed locally.
- [x] Verify: exact upstream license/source provenance and package metadata for all 214 active host dependencies.
- [x] Task / Red-Green: exact-source run `38995553011` exposed a release-license test fixture that copied `sources.json` but omitted the matching pinned upstream file and assumed every packaged text was named `LICENSE-MIT`. Reproduced the failure locally; the fixture now copies matching exact files and compares packaged bytes to the reviewed source.
- [x] Verify: focused license provenance, package, and smoke suites — 12 passed across 4 suites; focused Clippy, formatting, diff checks, and source-layout guard passed.
- [x] Task / Red-Green: extend the package regression to reject unresolved SPDX copyright placeholders; pin source-version license texts for all 28 native dependency records that previously packaged placeholders. TDD red reproduced `block2 0.5.1`; the focused package suites pass after repair. `dispatch` 0.2.0 has no upstream LICENSE file, so its MIT notice is reconstructed from Cargo metadata and the initial source commit; company legal review remains required.
- [ ] Verify / Documentation: replace the README and both manuals' unverified Agent Room screenshots with captures made from the exact native Rust binary. Asset names and filename-only tests are not provenance evidence.
- [x] Verify: manual-render tests confirm bundled PNG/JPEG images are rendered with accessible alt text. They do not establish which UI produced those images.
- [x] Verify: `cargo test --offline --locked --all-targets --features test-fixtures` — 499 passed, 14 ignored, 0 failed across 160 suites; full-target Clippy with `-D warnings` and rustfmt passed locally.
- [x] Verify: exact-source macOS package for `e70f79c` passed `package-release` and `validate-package`; all four Git/folder × shared/worktree smoke cases passed, including tests and approval-gated merge. Manifest binds source `e70f79cf795d53840055e8d503ca370a2141d6fc`, 214 dependencies, binary SHA-256 `63f0c11e6f906b6eabd48a2394be6e0c57616c3b5fcb65e529289d579154c50c`; ZIP SHA-256 `4d27a4e618baaf357866ecd02705a063f829ce5585ea1a167bc9b419a2299c2c`.
- [x] Knowledge: Graphify refreshed to 6,102 nodes / 9,779 edges / 465 communities; AgentMemory `mem_mvbu1vo7_a8727dff1aed`; Claude signal `sig_mvbu1vrd_e04bd7b4fe9e`.
- [ ] Verify: replace the unverified Workspace/Agent Room screenshots with captures from the exact current Rust build; review them against `docs/design-guide.md`, then regenerate both HTML manuals. Complete live Korean IME and Windows native UI/accessibility checks.
- [x] Verify: controlled idle sample of the native macOS Agent Room on a completed local fixture over three five-second `top` samples: 0.0%, 1.2%, 1.6% CPU and 58 MB MEM; no child process. The idle view retains its five-second refresh cadence.
- [x] Complete / source-migration increment: pushed `3796b54`; updated this plan, the issue, the migration wiki, AgentMemory, and Claude handoff.
- [ ] Complete issue #9: validate the exact current source package on macOS/Windows; capture/review the current build; complete Korean IME, Windows native UI/accessibility and resource checks; finish license attribution review; close only when every acceptance gate passes.

Current source audit for the screenshot correction: `native-ui-probe` is the default Rust UI feature; no tracked JavaScript, TypeScript or Python application files are present, `src/ui/room.js` is absent, and every tracked Rust source/test file is at most 300 lines. The historical oversized-file ledger later in this document predates the current branch state. The checked-in screenshots still lack evidence that they came from this Rust build.

- [x] Task / Red: public-doc tests treated `*-native.jpg` filenames as proof of Rust provenance; reproduce this by rejecting the README/manual “Native Rust Agent Room” captions.
- [x] Task / Green: remove unsupported Rust-native captions; identify these JavaScript captures as ineligible for Rust visual acceptance.
- [x] Verify: 22 focused tests across native UI, public docs, native manual, and source layout pass; release build, all-target Clippy, rustfmt, and diff checks pass. The duplicate Workspace stage eyebrow was reproduced as two rendered text shapes, then removed; its focused regression passes. Empty form fields and the empty-agent rail now show localized guidance.
- [x] Verify: CUA visually inspected the exact rebuilt Rust release binary (SHA-256 `d12f7befcbf3b27500c0ef8e0d64e2f11f50dcf0fead215f3760a9b85581103a`) in English/Korean and light/dark themes. The placeholder and no-agent states are visible, and the Workspace title appears once.
- [ ] Verify: obtain and review saved screenshots from the exact Rust application build before marking visual migration complete.

Issue #9 remains open. Run `38995553011` exposed the fixture regression and was canceled after its macOS failure. Run `38996644132` completed all macOS/Windows validation and package jobs successfully for `afa218261`; it predates the license fix `2061777` and later screenshot/manual commits, so it is not exact-HEAD Windows evidence. Local exact-source macOS package and all four packaged smoke cases now pass on `e70f79c`. No extra remote run was dispatched under the local-first verification direction. Exact-current Windows package, fresh screenshot review, Korean IME, Windows accessibility/resource, and final license-review gates remain pending. Screenshot/manual work is pushed at `141686a`/`e70f79c`; memory `mem_mvbtz5ui_12517c527b6f`/`mem_mvbu1vo7_a8727dff1aed`; Claude signals `sig_mvbtz5z3_43422d7ad56d`/`sig_mvbu1vrd_e04bd7b4fe9e`.

The dated implementation checkpoints below preserve their status at the time recorded; this active block supersedes older statements that still describe the production native UI as opt-in or the 300-line guard as failing.
## Reconciled current baseline (2026-10-11)

- [x] Plan: review Claude source `318f930`, added/amended R-36–59 and trace through DD-65; preserve original constraints and latest task/release policy.
- [x] Task: fetch and fast-forward the clean canonical checkout from `36d3420`; prior Codex corrections are already in history, not replayed.
- [x] Verify: 16 focused local tests (plan size, roles, slugs, settings sharing, 500-line layout) passed.
- [x] Verify: preview.6 has actual macOS/Windows ZIP assets and checksums; latest source CI 38112177633 jobs were not started because of GitHub account billing/spending limits.
- [ ] Complete: R-55–59 remaining increments and native acceptance; refer to the slug checklist below.

Detailed review: [requirements reconciliation](../knowledge/wiki/requirements-reconciliation-2026-10-11.md).
The historical checkpoints below retain their original scope and do not describe current implementation status.


Historical DD-41 checkpoint: modular source/test separation, 2026-10-10. Full macOS
regression: 213 passed, 0 failed, 9 ignored; 57 suites. Files are bounded at 500 lines.
Canonical development checkout is `/Users/neodamul/Development/AgentConductor`.
Platform CI `38038926444` on `0ab97d8` is terminal: macOS passed; Windows tests and
formatting passed, but Clippy rejected two `&PathBuf` fixture parameters. These are
corrected to `&Path`; native Windows verification of this checkpoint remains pending.
Previous checkpoint details below retain their original source scope.

Current status: full product under development. Cross-agent reconciliation on 2026-10-10
confirmed remote `2211ba3`, 19 commits after the preceding Codex checkpoint `3eb669c`.
CI `38037224504` on `bbabefe` completed: macOS passed; Windows built successfully,
including passing Job Object containment and WebView2 smoke tests, but failed
`gates::simultaneous_callbacks_cannot_consume_one_human_approval_twice` during directory
cleanup with Windows error 32. The reopened SQLite store must close before cleanup.
The following historical checkpoint remains evidence for its original source only.

Latest earlier verified slice: Claude Code CLI
integration (subscribed native sessions, guarded three-worker workflow and a read-only
Claude planning coordinator), based on continuation checkpoint `cff4646`.
Reviewed on 2026-10-10. The latest full regression passed 201 tests, with 8 ignored and
0 failures across 51 suites. The ignored live Claude tests (`native_claude`,
`native_claude_workflow`) passed separately against a claude.ai-subscribed CLI.
The existing-agentmemory-daemon test passed separately at the preceding transport checkpoint;
it was not repeated in this slice.
Clippy with warnings denied, formatting and whitespace checks passed at that implementation checkpoint.
Evidence: `.agentconductor/verification/cooperative-continuation/`,
`.agentconductor/verification/cooperative-recovery/` and
`.agentconductor/verification/cooperative-exit/`, with preceding worker/transport
checkpoints retained separately.

The native macOS interface has been exercised with controlled CLI fixtures, including
human review, team briefing and status lights. This does not establish authenticated
all-provider acceptance or native Windows support. The latest observed CI run,
`38027108692` on `c4f6291`, completed with macOS success and a Windows build failure
(E0425: Unix-only `process::run_controlled_async`, plus four platform warnings).

### Current checkpoint and remaining acceptance

### Cross-agent continuity and Windows cleanup (2026-10-10)

- [x] Plan: reconcile Claude handoff with committed source, agentmemory and actual CI.
- [x] Task: preserve the preceding Codex working changes in a named Git stash; fast-forward to `2211ba3` without replaying the already integrated continuation code.
- [x] Task / Red: Windows CI failure at `tests/gates.rs:172`; simultaneous approval assertions passed before cleanup failed.
- [x] Task / Green: explicitly drop the reopened SQLite store before cleanup.
- [x] Verify: focused macOS gate regression (4 passed, 0 failed, 0 ignored).
- [x] Verify: native Windows CI for the cleanup correction (windows-latest CI 38056911056 and Release run 38069707257; `gates` passes on Windows).
- [x] Complete: preserve source/CI evidence and shared agentmemory handoff (`mem_mv25cgtz_2a8a07bee3cc`).
- [x] Verify: Rust formatting and whitespace checks.

Cross-agent handoff procedure: [development continuity](12-development-continuity.md).
The Windows CI result above supersedes the older Windows build-failure observations below.

| Area | Verified checkpoint | Remaining work |
|---|---|---|
| Rust execution and review | Parallel subprocess fixtures, shared workspace default, optional worktrees, durable SQLite state, diff/test evidence and exact candidate approval | Authenticated multi-provider end-to-end acceptance and platform containment |
| Sessions and account limits | Session controls, restriction/reset scheduling, reviewed per-agent fallback contracts, actual Codex quota observation and claude.ai-subscribed Claude CLI generation, exact-session resume and concurrent-session isolation (`tests/native_claude.rs`), and a three-worker guarded Claude validate/run/approve/merge workflow (`tests/native_claude_workflow.rs`); missing-file Read/Edit is a per-call deny, not a worker abort | Native fallback registration acceptance, Claude headless quota capture and complete provider model/session coverage |
| Native macOS interaction | Setup-to-merge fixture, read-only three-session briefing, green/yellow/red indicators and running pulse | Live provider interaction, expired-approval presentation, OS reduced-motion and sustained resource checks |
| Cooperative agentmemory | Typed ledger/transport, real daemon round-trip, custom bootstrap, checkpoint polling, quiescent exit and completed pause/handoff crash recovery with explicit resume, and pending/accepted rework continuation after interruption | Automatic memory collection, worker authentication, company CLI acceptance, continuation across reassignment and automatic reviewed-target handoff |
| Knowledge assets | English source-linked wiki, development observations and local memory fallback | Live worker lesson/wiki ingestion, source revalidation and native knowledge interaction |
| Windows and releases | Committed development handoff and actual Windows CI failure evidence | Portable runtime, native Windows UI, verified packaged binaries, license closure and private macOS/Windows Release |
| Overall supervision | Deterministic Rust coordination and factual team briefing | Fine-grained idle/unresponsive policy and optional designated AI coordinator lifecycle |

### Claude CLI integration and native app acceptance (2026-10-10)

- [x] Verify: `claude` CLI subscribed login; live stdin/stream-json session, exact `--resume` and concurrent-session isolation (`tests/native_claude.rs`).
- [x] Task / Red/Green: guarded workers aborted when Claude read a not-yet-created file; missing-path Read/Edit is now a per-call deny after ancestor inspection.
- [x] Verify: live three-worker guarded Claude validate/run/diff/approve/merge workflow (`tests/native_claude_workflow.rs`).
- [x] Task / Red/Green: Claude can be the planning coordinator through the pinned guarded template restricted to `Read,Grep,Glob` with MCP/slash commands disabled; altered or unverified profiles stay refused.
- [x] Verify (native app, accessibility automation): workspace setup, Claude profile capability edit, Claude-coordinated plan, plan edits (reassignment), exact plan consent, Agent Room auto-open and per-request tool approval.
- [x] Plan: R-30 → BG-13 → SF-25 → UC-22 → FR-17/NFR-13 → QAS-23 → DDR-01/09 → DD-32 (live tool decisions); functional/non-functional analysis (`04b`) and design drivers (`05b`) added for the full trace.
- [x] Task / Red/Green: guarded workers hold a human-gated callback open (`AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS`, default 600 s); `tool-approve`/`tool-reject` decide through the ledger while the run is active; approval runs the exact input once in attempt 1, rejection denies only that call (`tests/live_tool_decision.rs`).
- [x] Task / Red/Green: native acceptance found the Agent Room did not list ledger-only live requests (Red: empty `tool_requests`); the snapshot now includes pending requests of running tasks for the current attempt.
- [x] Task / Red/Green: identical commands in later tool calls (or resumed attempts) collided with consumed/approved ledger entries and waited until timeout (Red reproduced with a read-only repeated command); live requests now carry the CLI tool-call identifier.
- [x] Task / Red/Green: Room live approvals were disabled and refused while the Room-resumed coordinator occupied the job slot; live approve/reject now decide synchronously with the Room executable (`tests/live_tool_decision.rs`, 5 tests).
- [x] Verify: native macOS app with three Claude Code workers — Claude-coordinated plan, plan edits, exact consent, live approvals from the Agent Room consumed in the same attempt, coordinator loss recovered (absence verified) and resumed, candidate `917074f` reviewed (3 tests exit 0), approved and merged; repository clean.
- [x] Plan: R-31 → BG-13 → SF-26 → UC-23 → FR-16/NFR-02 → QAS-24 → DDR-02 → DD-33.
- [x] Task / Red/Green: planning runs in a disposable tracked-files snapshot (`Repository::export_tree`, no symlinks/submodules/untracked/ignored files) outside the repository; the snapshot is removed and HEAD, status and the ignored-file listing must be unchanged. Unverifiable coordinators are accepted (`coordinator::planning_args`); verified templates keep their restrictions. A hostile mock planner that deletes and writes files leaves the target untouched (`tests/planner.rs`).
- [x] Verify: modal dialogs are exposed to accessibility — with the discard dialog open, `AXFocusedUIElement` is the "Keep editing" button inside the labelled dialog group within the web area; the earlier "page disappears" observation was a traversal-tool artifact over inert background content. Approved tool requests no longer offer "Review & allow" (6fc77b6). Duplicate values reported for dynamic task cards remain a tooling observation, not a confirmed defect.
- [x] Plan: R-32 → BG-13 → SF-27 → UC-24 → FR-18/NFR-10 → QAS-25 → DDR-07 → DD-34.
- [x] Task / Red/Green: Korean goals round-trip into the planning prompt and plan; polling and the Cmd+Enter shortcut never act during input-method composition (`tests/korean_input.rs`).
- [x] Verify: Korean (IME) text entry in the native macOS app with the system 390 Sebeolsik input method — virtual key codes typed 300 ms apart across 1-second polls composed exactly `안녕 하세요` in the goal field, and Save stored the identical UTF-8 (`ec9588eb859520ed9598ec84b8ec9a94`). Synthetic `keystroke` text bypasses the input method and is not a valid IME test.
- [x] Verify: complete native-app run through review, approval and merge with Claude workers (see live decision verification above).

### Workspace back navigation (DD-39)

- [x] Plan: R-33 → BG-09 → SF-28 → UC-25 → FR-19/NFR-10 → QAS-26 → DDR-07 → DD-39.
- [x] Task / Red/Green: titlebar "Back to run" and "Discard changes" with confirmation; view-only discard (`tests/workspace_navigation.rs`).
- [x] Task / Red/Green: persistent `last_run` link so saving the workspace never orphans the current run (`tests/startup.rs`); sticky, legible titlebar actions.
- [x] Verify: native app — Korean goal edit, Discard changes → confirmation (focus on Keep editing) → saved goal restored, `startup.json` byte-identical; Back to run covered by the startup flow test. Regression 208 passed.
- [x] Complete: wiki asset, log and agentmemory.

### Windows default desktop launch (R-23 / DD-38)

- [x] Plan: R-23/BG-10/SF-20/UC-17/NFR-10/NFR-11/QAS-18/DDR-03/DDR-08/DD-38; no-argument native startup must work without HOME.
- [x] Task / Red: selection regression failed with the former HOME-only error (1 passed, 1 failed).
- [x] Task / Green: pure nonempty HOME selection, Windows-only USERPROFILE fallback and no-argument CLI wiring.
- [x] Verify: macOS selection tests (2), CLI/startup/layout regression (8) and all-target Clippy; precedence and empty/missing values covered.
- [x] Verify: native Windows startup without HOME and with empty HOME; sequential window cleanup enforced (`windows_shell`, windows-latest CI 38056911056 and Release run 38069707257).
- [x] Complete: source-linked wiki and shared agentmemory checkpoint prepared; native Windows acceptance remains open.
- [x] Complete: implementation committed as 0308003 and pushed with 6ccd4bf.
- [x] Verify: native Windows results superseded by green windows-latest CI 38056911056 and Release run 38069707257.

### Native package prerequisites (R-24 / DD-27)

- [x] Plan: R-24/BG-10/SF-20/UC-17/QAS-18/DDR-03/DDR-08/DD-27; README support notes and manifest must disclose the native platform runtime.
- [x] Task / Red: both package target cases failed because their desktop runtime was absent from external_requirements.
- [x] Task / Green: Windows WebView2/macOS WebKit requirements in manifest and corrected public README support notes.
- [x] Verify: seven Python package/smoke contracts pass; README wording matches the native shell branches and initialization error. Native Windows execution remains separately gated.
- [x] Complete: wiki (`native-release-packaging`), agentmemory and commit/push.

### Missing upstream license texts (R-24 / DD-27)

- [x] Plan: R-24/BG-10/SF-20/UC-17/QAS-18/DDR-03/DDR-08/DD-27; native Windows CI 38040929721 failed on missing webview2-com license text after release compilation succeeded.
- [x] Task / Red: omitted WebView2 crate text regression failed before package output with the same CI error.
- [x] Task / Green: retained official MIT copies bound to exact package/version/SPDX, VCS commit and SHA-256; no packaging-time network or generic substitution.
- [x] Verify: seven Python packaging/smoke contracts and source-layout enforcement passed; actual offline closure resolves 42 macOS and 108 Windows dependencies.
- [x] Task / Red: CI 38041844312 Windows Rust tests passed, but packaging contract refused the checkout-converted license hash.
- [x] Task / Red/Green: actual Git autocrlf checkout preserves retained upstream license bytes (`.gitattributes` `-text`; contract fails without it, passes with it).
- [x] Verify: native Windows package execution after the byte-preserving checkout fix (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)).
- [x] Complete: source-linked upstream-license wiki, shared memory and 6ccd4bf committed/pushed; CI 38041844312 is in progress.

### Packaged native workflow acceptance (R-24 / DD-27)

- [x] Plan: existing R-24/BG-10/SF-20/UC-17/QAS-14/QAS-18/DDR-03/DDR-08/DD-27; exact packaged executable and native platform execution evidence.
- [x] Task / Red: copied-binary integrity/refusal tests failed on the absent packaged workflow runner.
- [x] Task / Green: three scoped CLI workers, shared/worktree parallel runs, diff/tests, unapproved merge denial and exact approved merge through the package binary.
- [x] Task / Red/Green: actual ZIP failed on pre-1980 registry timestamps; date clamp preserves contents/execute bits and overwrite refusal. Five Python contracts passed.
- [x] Verify: actual macOS ZIP entry CRCs and archived binary checksum; 2,978,551 bytes.
- [x] Verify: exact da746b9 macOS package completed both parallel fixture modes with diff/tests, unapproved merge refusal and approved candidate merge.
- [x] Verify: native Windows packaged execution and CI archives (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)).
- [x] Complete: source-linked wiki and agentmemory (`mem_mv26gi9j_1f9a63545a04`); unit commit/push follows local checks.
- [x] Verify: retained native CI archives; published as v0.1.0-preview.1 … preview.3 with verified `SHA256SUMS`.

### Native release package contract (R-24 / DD-27)

- [x] Plan: R-24 → BG-10 → SF-20 → UC-17 → NFR-03/NFR-11 → QAS-14/QAS-18 → DDR-03/DDR-08 → DD-27.
- [x] Task / Red: missing isolated assembly API; existing script selects no Windows .exe and carries stale mockup metadata.
- [x] Task / Green: native executable name, bounded public asset selection, target-locked reviewed license closure, notices, SBOM and exact binary checksum; no wiki mutation.
- [x] Verify: two Python contract tests cover both platform names, exact binary/evidence, overwrite refusal and pre-output license rejection; native CI 38040271132 passed both platforms on da746b9.
- [x] Verify: actual macOS release package from da746b9; 5,115,648 bytes, 42 dependency notices, exact checksum and packaged --help exit 0.
- [x] Verify: Windows package and exact packaged workflow/review/merge smoke on both platforms (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)); reviewed artifacts published.
- [x] Complete: source-linked wiki and agentmemory; unit commit/push follows successful contract checks.

### Source layout limit (DD-41)

- [x] Plan: R-35 → BG-05 → SF-12 → C-04/NFR-07 → DDR-04/DDR-06 → DD-41.
- [x] Task / Red: compiled CI layout check failed on eight oversized files and three inline test bodies.
- [x] Task / Green: responsibility modules and private tests under tests/unit/; maximum source/test file is 485 lines.
- [x] Verify: full macOS regression 213 passed, 0 failed, 9 ignored across 57 suites before checkout migration; focused layout and formatting passed again in the new checkout.
- [x] Verify: CI 38039813985 on 47eb2d7 completed successfully on native macOS and Windows (build, layout, all applicable tests, formatting, Clippy).
- [x] Complete: source-layout wiki, local evidence and agentmemory (`mem_mv25wmpa_f63ae0e2a551`); Windows completion remains open.

### Application icons (DD-40)

- [x] Plan: R-34 → BG-14 → SF-29 → UC-26 → FR-20/NFR-10/NFR-12 → QAS-27 → DDR-10 → DD-40.
- [x] Task / Red/Green: dependency-free PNG data-URL encoder with size bounds (`tests/icons.rs`).
- [x] Task / Red/Green: snapshot icons for built-in providers only; custom stays letters.
- [x] Task / Red/Green: macOS NSWorkspace icon resolution (live test passed); Windows executable icon extraction implemented, unverified.
- [x] Task / Red/Green: workspace and Agent Room render icons with accessible labels.
- [x] Verify: native macOS app shows Claude, ChatGPT (Codex) and OpenCode icons in workspace and Agent Room.
- [x] Complete: runtime-icons wiki asset, index/log and agentmemory; five macOS icon tests passed on 2026-10-10.
- [x] Verify: Windows per-user install-path discovery, Win32 icon extraction and PNG encoding (`tests/windows_icons.rs`, CI 38056051105).
- [x] Task / Red/Green (Windows CI 38056911056): Start Menu shortcut discovery through `SHGetFileInfoW` for installers that do not use the fixed per-user paths; Store packages fall back to letters.

### Windows native port (DD-35 … DD-38)

- [x] Plan: R-23/R-24 → BG-10 → SF-20 → UC-17 → NFR-03/NFR-11 → QAS-18 → DDR-03/DDR-08 → DD-35–DD-38.
- [x] Task / Red/Green: cross-platform kernel leases (DD-35); lease tests run on macOS and Windows runners (faae6f9).
- [x] Task / Red/Green: Windows Job Object process supervisor, ownership and quiescence (DD-36); `windows_containment` passed on windows-latest (CI 38036749822).
- [x] Task / Red/Green: Windows file identity and session randomness (DD-37, f79d3a5).
- [x] Task / Red/Green: Windows WebView2 desktop shell (DD-38); `windows_shell` smoke passed on windows-latest (CI 38036749822).
- [x] Verify: Windows CI green for build, tests, fmt and clippy; packaged Windows binary smoke test (windows-latest CI 38056911056 and Release run 38069707257).
- [x] Task / Red/Green: tag-triggered `release.yml` reuses platform validation and package smoke, then only the publish job (`contents: write`) creates a pre-release with both archives and combined `SHA256SUMS` (`tests/test_release_workflow.py`).
- [x] Verify: CI 38051732098 on b1e404e passed all four jobs (both platforms' validation and packaged workflow smoke).
- [x] Complete: `v0.1.0-preview.1` published by release run 38052572665 (macOS arm64 and Windows x86_64 archives + SHA256SUMS); downloaded macOS archive and inner binary checksums verified.
- [x] Task / Red/Green: the Windows archive checksum line was CRLF, so `shasum -c` could not find it; generated package text is now written as LF bytes (contract checks no CRLF).
- [x] Complete: `v0.1.0-preview.2` (release run 38054751920, source d84b876) published; downloaded archives verify with `shasum -c SHA256SUMS`, and each package's own `SHA256SUMS` verifies `agentconductor` / `agentconductor.exe`.

### Next implementation slice — cooperative worker integration

- [x] Plan: DD-29 defines addressed checkpoint directives and local execution authority.
- [x] Inspect: identify missing worker capability/bootstrap wiring and live custom-session address binding.
- [x] Task / Red: missing capability/bootstrap fails compiled-worker tests.
- [x] Task / Green: explicit custom capability, persisted owned session, addressed worker context and checkpoint commands; retain Rust approval authority.
- [x] Verify: three compiled-worker tests cover before/during-work polling in both workspace modes, monotonic reports, addressed rework, conflicting session rejection and mandatory merge approval.
- [x] Task / Red/Green: completed pause previously created a merge candidate; reconcile clean quiescent exits to stopped/blocked instead of completed.
- [x] Verify: live worker after completed acknowledgment refuses reassignment; partial-work stop, unchanged due scheduling, explicit attempt-2 resume and stale-address rejection tested.
- [x] Verify: pending/accepted directives at clean exit block completion; relinquished attempts reject further directives.
- [x] Verify: killed coordinator after completed pause/handoff acknowledgment; live child refuses recovery/reassignment, quiescence permits stopped recovery and exact-session attempt-2 resume.
- [x] Task / Red/Green: explicit resume preserves unexpired pending/accepted rework as provenance-bound context; supersession and receipt commit in one transaction; pause/handoff is superseded without replay.
- [x] Verify: killed coordinator after accepted rework; resumed attempt 2 receives the original instruction, inspects the partial file and continues; full regression 199 passed.
- [x] Task / Red/Green: automatic reviewed-target handoff transitions with no competing writer (DD-43). Red: completed handoff left the run `stopped`; Green: `tests/cooperative_handoff.rs` — the company worker stays the owner while alive, then the reviewed fallback continues `partial` work in a new session to `waiting_approval`; the no-fallback case stays stopped (`cooperative_worker`). Full regression 224 passed / 10 ignored / 0 failed. Shipped in v0.1.0-preview.4 (CI 38071626681, Release run 38072717471 on macOS and Windows; `SHA256SUMS` verified).
- [ ] Verify: company LLM CLI acceptance, continuation across reassignment and automatic handoff reassignment.
- [x] Complete: update traced design/wiki and retain verified bootstrap evidence; full cooperative integration remains open.

No whole-product completion or release readiness is claimed. Historical checkpoints below
retain their original evidence; later verified slices refine their remaining scope.

Each task: **plan** (linked R/BG/SF/UC/QAS/DD and acceptance criteria) → **task**
(Red → Green → Refactor) → **verify** (counterexamples, integration, review) → **complete**
(evidence, report, lessons). QAS defines quality goals; tests verify them.

| Task | Dependency | DD | Tests to fail first | Completion evidence |
|---|---|---|---|---|
| T-01 Profile/plan schema | Design documents | 01/03 | Wrong task count, cycles, overlapping scopes, argv injection | Schema/boundary tests |
| T-02 SQLite state/audit | T-01 | 02 | Illegal transitions, duplicate events, stale lease recovery | Restart tests |
| T-03 Subprocess supervisor | T-01 | 04/07 | stdin EOF, stderr flood, split UTF-8, stop grandchildren | Mock process contract |
| T-04 Three parsers/resume | T-03 | 03/04/07 | Fixtures to normalized events, errors, exact sessions | Version-pinned fixtures; separate live smoke |
| T-05 Workspace/scheduler | T-02/03 | 05 | Both workspace modes, three parallel executions, fixed base, dependency blocks, scope violations | Temporary Git integration |
| T-06 Diff/test/approval/merge | T-04/05 | 05/06 | Untracked files, failing tests, unapproved/changed candidate rejection | Protected-branch before/after SHA |
| T-07 Memory/wiki | T-02 | 08/09 | Outage outbox, duplicate delivery, stale sources | Offline fallback/source checks |
| T-08 CLI E2E/release | T-01–07 | 10 | Register → three tasks → verify → approve → merge; rejection/revision | Measured QAS, license/SBOM |

Verification: rtk cargo test; rtk cargo clippy --all-targets -- -D warnings;
rtk cargo fmt --check; lockfile-based license/text checks and SBOM generation through
the packaging script. `cargo deny` is an optional additional check and is not installed
on the current host.
Unit tests use mocks/captured fixtures, not paid model calls. Record live CLI smoke separately
using existing authentication. Completion requires passing tests, tested integration candidates,
and human approval matching exact SHAs.

Trace example: R-07 → BG-03 → SF-05 → UC-04 → QAS-02 → DD-06 → T-06 →
approval_changed_head_rejected. Add actual test names and evidence during implementation.


## Open acceptance that needs external resources (2026-10-11)

The checklist below distinguishes verified implementation from external acceptance.
Each external item names the resource it needs; newly active work is tracked separately.

| Need | Items |
|---|---|
| A real company/non-steerable CLI | Live checkpoint polling and handoff acceptance with a real company CLI, company fallback chains and consent interaction |
| Codex quota (reset 2026-10-16) or a second Codex account | Codex 0.162.1 live re-run, multi-provider briefing query |
| A Windows machine with a real subscribed CLI login | Windows account-capacity acceptance; fixture CI does not prove subscription access |
| Operator OS settings changes | Live reduced-motion preference |
| Product decisions requested from the user | Desktop planning without an LLM coordinator; redistribution beyond the configured fallback |

## Progress checklist

Checked items have current evidence. A foundation is not completion of the entire task.
Update this checklist after verification, not merely after writing code.

### T-01 — Profile and plan validation

- [x] Plan: requirements and acceptance criteria linked to DD-01/03.
- [x] Task: three-task schema, explicit workspace mode, DAG and scope validation.
- [x] Verify: invalid plans, path escapes, overlapping scopes, and custom profiles tested.
- [x] Complete: foundation committed; TDD evidence recorded.

### T-02 — Persistent state and audit

- [x] Plan: lifecycle, audit, and recovery requirements defined.
- [x] Task: SQLite transitions and event persistence implemented.
- [x] Verify: illegal transitions, duplicate events, and database reopen tested.
- [x] Task: workflow document, audit, and memory-outbox transactions integrated.
- [x] Verify: interrupted coordinator recovery and stale process ownership (`crash_recovery`: killed coordinator, live children, incomplete ownership evidence).
- [x] Complete: recovery scope verified and documented (manual §2.3, wiki `session-recovery`).

### T-03 — Subprocess supervisor

- [x] Plan: process contract and timeout policy defined.
- [x] Task: direct argv, cwd, stdin EOF, concurrent stdout/stderr spool, Unix process groups.
- [x] Verify: six cases cover large I/O, nonzero exit, spawn failure, timeout descendants,
  unread stdin, and forced-kill escalation.
- [x] Task/verify: explicit user stop and restart recovery (`stop_resume`, `crash_recovery`).
- [x] Complete: lifecycle controls verified; Windows Job Object containment (`windows_containment`) passes on windows-latest CI 38056911056 and Release run 38069707257.

### T-04 — CLI protocols and resume

- [x] Plan: native headless and resume commands researched.
- [x] Task: Claude/Codex/OpenCode/custom JSONL normalization and native argv builders.
- [x] Verify: synthetic contracts, malformed output, exact session selection, resume flag differences.
- [x] Task: environment/input-mode/permission profiles connected to workflow execution.
- [ ] Verify: installed-version fixtures and live CLI smoke. Claude 2.1.287 passed (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)); Codex passed on 0.162.0, and the 0.162.1 re-run is blocked by the account usage limit until 2026-10-16 (classified correctly, `availability`); OpenCode 2.0.25 passed live (`native_opencode`: task-file turn and exact session resume, free hosted model, 2026-10-11).
- [x] Complete: supported CLI/version contracts documented with evidence (user manual Part 3: verified versions, live evidence and what is not yet verified per CLI).

### T-05 — Workspace and scheduler

- [x] Plan: shared default, optional worktrees, disjoint ownership, dependency semantics.
- [x] Task: Git candidate snapshots with a temporary index and optional worktree creation.
- [x] Verify: three Git tests cover new files, unchanged HEAD/index, scope violations, exact base.
- [x] Task: three-agent parallel scheduler and dependency propagation.
- [x] Verify: shared/worktree E2E, overlapping mock attempt intervals, failed-dependency blocking.
- [x] Complete: shared and worktree modes verified (`workflow`, packaged smoke on both platforms) and documented (manual §1.4).

### T-06 — Evidence, approval, and merge

- [x] Plan: bind approval to candidate, base, diff, and test evidence.
- [x] Task: integration candidate, coordinator tests, review artifacts, approve/reject/merge commands.
- [x] Verify: failing tests, modified files/evidence, changed target branch (`target_branch_advanced_after_approval_blocks_merge`), missing approval (`workflow`, packaged smoke).
- [x] Complete: reviewed candidate merged in mock E2E test with exact before/after SHAs.

### T-07 — Memory and knowledge

- [x] Plan: SQLite authority, memory outbox, local fallback, wiki rules defined.
- [x] Task: English source/wiki/schema assets and development progress in agentmemory.
- [x] Task/verify: outbox delivery, offline continuity, local FTS recall, HTTP mock delivery.
- [x] Task/verify: runtime lessons and source revalidation (`knowledge`).
- [x] Verify: offline continuity, delivery retries/deduplication, changed sources (`memory`, `memory_transport`, `knowledge`).
- [x] Complete: runtime memory and knowledge workflow verified.

### T-08 — End-to-end verification and binary

- [x] Plan: MVP and release evidence defined.
- [x] Verify: initial native validate/normalize binary built and smoke-tested; 35 baseline tests passed.
- [x] Task: host dependency license declaration and source-text inventory (42 packages).
- [x] Verify: registered-profile planner tests and mock CLI run → parallel tasks → evidence → approval → merge E2E.
- [x] Verify: 42 dependency license texts, notices, CycloneDX inventory, and startup resource measurement; live agent resources remain unmeasured.
- [x] Task: macOS ARM64 binary, checksum, usage examples, and release manifest.
- [ ] Complete: full user-requested product acceptance audit and final binary. Prior mock CLI release is a verified checkpoint only.

## Distribution requirements

Distribute one native executable with rusqlite bundled SQLite, without required Python/Node
sidecars. Apply release optimizations. Track OS/architecture artifacts, checksums, dependency
and license manifests as release evidence.

## Commit and push cadence

Keep commits small and reviewable. Push accumulated commits at meaningful checkpoints
or when remote review is needed; pushing after every change is not required.
This updates the earlier request to commit and push each small unit.

## T-09 — Availability, multi-session roles, and models

- [x] Plan: R-18–20 → BG-07–09 → SF-09–11 → UC-09–11 → QAS-09–11 → DD-11–13.
- [x] Task: Red tests for restriction parsing, persisted scheduling, reassignment, and model approval invalidation.
- [x] Verify: three restriction parser tests pass (explicit offsets, relative delays, ambiguous messages, multiple windows).
- [x] Task: persisted due-work execution and bounded automatic retries.
- [x] Task: multiple sessions, per-profile concurrency, account groups, and audited reassignment.
- [x] Task: native catalog query/import and session model selection/change.
- [x] Verify: five library recovery tests, binary session-control E2E, two catalog tests; native Claude/Codex metadata smoke.
- [x] Complete: CLI controls documented, 59 tests and native metadata evidence retained, agentmemory updated, release binary produced.

## T-10 — Native Workspace profile controls and localization (GitHub issue #9)

- [x] Plan: keep the existing Rust dispatcher and profile API; localize the Team/profile workflow in English and Korean.
- [x] Task: localize role, profile, model/quota refresh, and removal-confirmation controls with accessible names.
- [x] Verify: 20 focused tests pass across profile localization, profile actions, role assignment, pending status, and i18n contracts; native-feature all-target Clippy passes with warnings denied. The broader run reported 533 passed and 14 ignored across 158 suites before the RTK runner was interrupted after its summary (exit 130).
- [x] Task: split touched Rust modules so every changed source and test file stays within 300 lines.
- [x] Verify: repository-wide source-layout gate; 34 unrelated existing files still exceed 300 lines.
- [x] Complete: pushed `863f2ae`, saved AgentMemory fact `mem_mvb0p8wv_fdd4b446642a`, signaled Claude, and updated GitHub issue #9 progress. The issue remains open.

## T-11 — Bilingual native workspace settings sharing (GitHub issue #9)

- [x] Plan: extend existing settings-sharing use case UC-42 and backend contract R-52/DD-59 under the Rust UI migration decision DD-160 without changing request/approval semantics.
- [x] Red: new AccessKit tests failed on untranslated preview, dirty-state, permission, and result labels.
- [x] Green: localize the preview, confirmation, permission choice, and async completion results; retain the compact English prompt and existing accessible action names.
- [x] Task: keep import/export result data language-neutral so changing the selected language also updates already-visible results.
- [x] Verify: 13 tests pass across sharing-language, workspace, and i18n suites; 13 native probe checks and native-feature all-target Clippy pass. Formatting and diff checks pass. Graphify: 6,301 nodes / 10,546 edges / 478 communities.
- [x] Verify: changed Rust files are below 300 lines; repository-wide layout still reports 34 pre-existing oversized files.
- [x] Complete: refreshed Graphify and wiki; saved AgentMemory, signaled Claude, posted issue #9 progress, and committed/pushed this slice.

## Current verification evidence

- `tests/availability.rs`: explicit/relative reset, multiple windows, offset parsing, ambiguous messages.
- `tests/recovery.rs`: restart scheduling, role transfer, model approval invalidation, concurrency cap, retry ceiling.
- `tests/models.rs`: metadata-only CLI protocol mocks; malformed catalogs rejected.
- `tests/cli_workflow.rs`: exact plan consent and merge approval; CLI schedule/reassign/model/resume commands.
- `tests/workflow.rs`: shared/worktree integration, retained failures and changed evidence.
- `tests/planner.rs`, `tests/git.rs`, `tests/memory.rs`: three-task planning, temporary-index snapshots, outbox and offline recall.

The full product backlog is larger than the deliverable CLI MVP. Unchecked scope must not
be described as verified or shipped. Native metadata smoke is not live paid execution.

## T-03 follow-up — Explicit stop and resume

- [x] Plan: preserve process containment (DD-04/07), ownership, artifacts, and human review gates.
- [x] Task: Red tests for process-tree stop, stop-before-launch, and workflow stop/resume.
- [x] Task: durable stop request, interrupted attempts, and explicit resume commands.
- [x] Verify: two supervisor stop tests and two workflow stop/resume tests, including interruption during integration verification.
- [x] Complete: documented CLI controls, binary-command verification, and passing regression suite.

A human stop must terminate the owned process group, retain raw I/O and partial files,
prevent automatic restart, and revoke any candidate approval. Resume must verify the target
Git baseline and permission configuration, append a new attempt, and repeat integration
verification. Unknown/crashed ownership is a separate recovery operation.

Full completion is governed by [the requirement-by-requirement audit](09-completion-audit.md).

## T-02 follow-up — Crash ownership and leases

- [x] Plan: recover only after kernel ownership is idle and all recorded processes/groups are terminal; preserve unknown ownership for intervention.
- [x] Task: Red tests for persistent exclusive leases and symlink rejection.
- [x] Task: kernel-backed coordinator lease and durable process ownership records.
- [x] Task: explicit crash recovery and cross-run repository ownership claims.
- [x] Task: cross-run native session leases and paused-workspace checkpoint validation (DD-14).
- [x] Verify: live-owner rejection, SIGKILL lease release, absent/ambiguous process ownership refusal, and resumed evidence preservation.
- [x] Verify: killed coordinator's live children also prevent a different run; dirty-repository rejection does not create an unknown ownership claim.
- [x] Verify: native session reuse refuses live/unknown previous ownership; workflow does not spawn while leased elsewhere; shared/worktree edits block resume.
- [x] Complete: compiled CLI recovery/control verification and documented limits. Native provider generation remains a separate full-product acceptance item.

A PID or leftover lock file alone is not proof of live ownership. Keep the lock inode stable;
use an OS advisory exclusive lease rather than deleting a possibly active owner's file.
Record owned process/group identity before starting I/O collection. Recovery cannot assume
that an absent process record means the subprocess did not start.

## T-05 follow-up — Review feedback and dependency rework

- [x] Plan: R-04–07/09 → BG-02/03 → SF-02–06 → UC-03/04 → QAS-02/06/10 → DD-15.
- [x] Task: Red tests for human feedback, transitive invalidation, preserved prior attempts and refreshed dependency contents.
- [x] Task: bounded review feedback, audited rework, updated dependency context and safe worktree refresh.
- [x] Verify: shared/worktree dependency chains, failure feedback, conflicts, stop/resume and exact fresh approval (82-test regression suite).
- [x] Complete: compiled CLI controls, evidence, documentation and agentmemory update.

Changed predecessor results must be visible before a dependent retries. Preserve old
worktrees and evidence rather than resetting them in place. A refresh conflict blocks
that task with retained evidence; it must not launch a worker against ambiguous contents.

Structured coordinator review is verified below with mock subprocesses. Native provider
review contracts and enforced pre-execution permissions remain separate acceptance items.

## T-06 follow-up — Structured coordinator review

- [x] Plan: R-04/06/07/09 → BG-02/03 → SF-02/04/05/06 → UC-03/04 → QAS-02/03/05/06 → DD-16.
- [x] Task: Red tests for typed final JSON, unknown task/evidence rejection, bounded context and coordinator read-only preservation.
- [x] Task: registered subprocess review, strict proposal validation, bounded revision rounds, audited scoped feedback and retained ownership.
- [x] Verify: failure → reviewer → revised tasks → new tests → exact human approval; malformed proposals, altered evidence, stale feedback, stopped reviews and write violations.
- [x] Complete: compiled `review`/`apply-review` controls, documentation, agentmemory and audit evidence for the mock CLI contract.

Verification: `rtk cargo test` passed 96 tests (one intentionally ignored subprocess helper),
27 suites in 37.42 seconds; Clippy and formatting checks passed. Graphify extracted
234 nodes/737 edges without an LLM; CocoIndex located saved-proposal application in
CLI and workflow modules. No new runtime dependency was added. Native execution,
pre-execution permission gates, runtime knowledge and functional UI remain open.

## T-03/06 follow-up — Pre-execution approval transport

- [x] Plan: DD-17, native callback failure semantics and retained process containment.
- [x] Task: Red tests for permission request/reply, stdin lifetime, malformed output and failed callback containment.
- [x] Task: bounded bidirectional JSONL transport integrated with the existing supervisor.
- [x] Verify: mock denial creates no side effects; exact human ledger approval permits one execution; protocol failure kills/reaps owned processes and preserves raw evidence.
- [x] Complete: transport/controller/ledger component evidence and documentation; this does not complete native permission enforcement.
- [x] Task/verify: SQLite exact-request/context approvals, maximum 900-second expiry, single consumption under concurrent callbacks, central audit/outbox.
- [x] Verify: installed Claude 2.1.286 accepts SDK PreToolUse registration in metadata-only initialization; no generation prompt sent.
- [x] Task/verify: native controller preserves a permission decision owed by active work after a result frame (Red → Green).
- [x] Task/verify: durable typed human decisions, provider capability integration and actual native denial/approval (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)).

Passing transport mocks does not prove complete R-08 enforcement. CLI profile labels,
ordinary shell hooks and post-execution integrity checks cannot substitute for native
pre-execution negative tests.

Current component regression: 107 tests passed, two opt-in/helper tests ignored,
30 suites in 37.64 seconds; native metadata test also passed separately
with `rtk cargo test --test native_control -- --ignored`. The native test is opt-in;
ordinary test discovery skips it and the existing subprocess helper. Clippy and formatting
checks pass. Graphify: 284 nodes/869 edges; CocoIndex locates the atomic consumption path.

Native metadata evidence is retained locally under
`.agentconductor/verification/claude-control-metadata-3042/process/`; stdout SHA-256
`935b6d736d8562ab968ac4a6fa440f34f46f0a00f3d3753e09e488a84857f5f1`.
Initialization succeeded, no user frames were sent, and ownership is terminal/quiescent.
`claude auth status` currently reports no login; actual generation/denial tests await
existing-subscription login. API keys are not a fallback.

## T-03/06 follow-up — Bounded worker policy and callback lifecycle

- [x] Plan: DD-18, file ownership and explicit approval boundaries.
- [x] Task: Red → Green for duplicate tool-use identities before approval consumption and requests after input closure.
- [x] Task: Red → Green for bounded tool inputs, owned edits, sensitive paths and filesystem alias rejection.
- [x] Verify: policy connected to bidirectional callback component, with pre-execution denial and retained process evidence.
- [x] Complete: component verification, local knowledge asset and agentmemory decision record.
- [x] Task/verify: production worker context binding, human CLI controls and live native denial/approval (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)).

Regression: 116 tests passed, two opt-in/helper tests ignored, 32 suites in 38.76
seconds. Clippy with warnings denied, formatting and whitespace checks passed.
Graphify extracted 308 nodes/941 edges without an LLM; CocoIndex found the classifier
and its evaluator bridge. No runtime dependency was added. Ponytail remains guidance;
no callable Ponytail tool or installed CLI was available in this environment.

Policy tests include replacement of the workspace directory at the same path,
directory/file symlinks, hardlinks, FIFO, ambiguous paths, unsupported input fields,
bounded input, secret filenames and task ownership. The controlled subprocess test
verifies no cross-task write, no pending-command side effect, one approval consumption
and denial after consumption. It uses a mock native protocol and fixed test context;
it does not verify production workspace fingerprints or native CLI enforcement.

## T-03/06 follow-up — Worker-bound native decisions

- [x] Plan: DD-19, bounded asynchronous verification and immutable request binding.
- [x] Task: Red → Green for asynchronous control cancellation and verifier containment.
- [x] Task: authoritative context verifier derives session/workspace/plan/Git binding from stored state; ignored sensitive targets use metadata without reading contents.
- [x] Task: opt-in Claude native callback worker integration with preassigned owned sessions.
- [x] Task: operator list/refresh/approve/reject controls and distinct waiting state.
- [x] Verify: compiled mock workflow controls in shared/worktree modes, stale/expired/denied decisions, native-session retention and separate merge approval.
- [x] Complete: worker/operator implementation checkpoint evidence, wiki and agentmemory; full-product completion remains open.
- [x] Verify: actual authenticated Claude tool denial/approval and containment: scope escape terminates the owned process group before writing; human rejection denies only that call (2026-10-11 live Claude 2.1.287 suites (`native_claude`, `native_control`, `native_claude_workflow`, `native_claude_denial`)).

Verifier/transport checkpoint: 121 tests passed, two ignored, 34 suites in 43.06
seconds. Clippy, formatting and whitespace checks passed. Compiled verifier binding
tests cover both workspace modes, changed working files, unknown tasks and foreign
write scopes. Owned async tests cover stop, callback cancellation without a cooperative
stop listener, pending callback timeout and normal-exit final-frame draining.
Actual Claude metadata-only initialization passed separately in 0.90 seconds; no
generation prompt was sent. This does not prove native tool enforcement.
Graphify extracted 321 nodes/987 edges without an LLM; CocoIndex located the verifier.
No runtime dependency was added. Worker/operator mock-contract implementation is now
present; live provider enforcement remains open. The follow-up adds durable decision
epochs, exact-ID receipt lookup, exclusive approved retries, session identity checks,
stopped-state preservation and explicit rejection handling. Red → Green covers old
receipts beyond the listing limit and tampered payloads. Compiled CLI tests cover both
workspace modes, pending/consumed permissions, expiry/refresh, rejection/revision,
explicit stopped resume, changed paused workspaces and separate final merge approval.

Worker/operator checkpoint: 127 tests passed, two ignored, 34 suites in 49.61 seconds;
Clippy, formatting and whitespace checks passed. Actual Claude metadata-only control
initialization passed separately in 0.90 seconds. Graphify extracted 341 nodes/1,083
edges without an LLM; CocoIndex located exact-ID receipt reads. No runtime dependency
was added. Native tools, provider containment, runtime knowledge and functional UI
remain open in the full-product audit.

## T-03/05/07 follow-up — Live intermediate reports

- [x] Plan: DD-20, owned spool observation, independent attempt evidence and bounded local summaries.
- [x] Task: Red → Green tests observe normalized evidence before worker completion, reject malformed streams and verify final draining.
- [x] Task: bounded observer and SQLite attempt event ledger connected to both worker transports.
- [x] Verify: regression, cancellation, final records, retained raw evidence and bounded queries.
- [x] Complete: evidence, wiki and agentmemory checkpoint; native runtime proof remains separate.

Live-progress checkpoint: 131 tests passed, two ignored, 35 suites in 50.14 seconds.
Clippy, formatting and whitespace checks pass. Graphify extracted 352 nodes/1,110
edges without an LLM; CocoIndex confirmed both worker transports use the observer.
No runtime dependency was added. A deferred SQLite read-to-write upgrade conflict
was reproduced in native-worker mock rework and fixed with an immediate transaction.

## T-07 follow-up — Runtime reviewed lessons

- [x] Plan: DD-21, bounded typed promotion, repository source hashes and task-scoped handoff.
- [x] Task: Red → Green for exact promotion consent, stale source exclusion and scope/alias refusal.
- [x] Task: authoritative local assets, derived wiki/index/log, operator controls and worker context.
- [x] Verify: compiled digest/accept controls, offline next-attempt task-scoped context, changed sources, mirror alias refusal and recovery.
- [x] Complete: checkpoint evidence and wiki/agentmemory; full-product acceptance remains open.

Runtime lesson checkpoint: 134 passed, two ignored, 36 suites in 51.90 seconds.
Clippy, formatting and whitespace checks pass. Graphify: 377 nodes/1,199 edges, no LLM;
CocoIndex located worker context injection. No runtime dependency added. Staleness
uses source hashes; bounded metadata checks do not freeze filesystem paths. Native
lesson service/confidence-aware retrieval, cross-run context and UI remain open.

## T-04/09 follow-up — Native Codex subscription/session contracts

- [x] Plan: existing ChatGPT authentication, explicit session IDs, no API-key fallback and retained process evidence.
- [x] Verify: installed Codex 0.162.0 stdin/JSONL initial turn and exact session resume preserving previous context.
- [x] Verify: two actual concurrent sessions retain distinct IDs and recall their own context on resume.
- [x] Task: Red → Green for nonterminal Codex error items classified as visible warnings; terminal outcomes remain separate.
- [x] Complete: regression evidence, wiki and agentmemory checkpoint; full-product goal remains active.
- [ ] Verify: native scoped implementation, dangerous-tool containment/approval and all-provider model/session behavior.

Native Codex checkpoint: 135 passed, four ignored, 37 suites in 52.91 seconds.
The two generation tests were also run explicitly: exact context resume passed in
7.74 seconds; concurrent independent contexts passed in 7.13 seconds. Installed
version: 0.162.0. Clippy/format/whitespace checks pass. Graphify: 377 nodes/1,199
edges, no LLM; CocoIndex located the nonterminal item normalization. No runtime
dependency added. Actual risky tool/native implementation tests and functional UI
remain open; these text-turn proofs do not close R-08 or the full product.

## T-08/09 follow-up — Functional intervention room

- [x] Plan: DD-22, approved visual direction, embedded native surface and exact-state control broker.
- [x] Task: Red → Green for newer-stop preservation, stale mutation refusal, local capability enforcement, cookie collisions and artifact/env exposure.
- [x] Task: embedded Agent Room with real task/evidence/approval/session controls.
- [x] Verify: backend integration and real rendered fixture feedback/reassignment/model/resume/diff/tests/approval/merge.
- [x] Verify: native Cocoa/WebKit review rendering and Cmd-W close with process exit 0 and endpoint shutdown, using a local metadata wrapper.
- [x] Task: Red → Green for same-run native-window lease, child CLI duplicate refusal and reopen after release.
- [ ] Verify: unbundled identity and remaining availability/tool-gate interactions.
- [x] Complete: actual browser UI screenshot, public usage, wiki and agentmemory checkpoint.
- [x] Verify: UI runtime resources and final distribution/license evidence. Released v0.1.0-preview.5 macOS binary (5,313,920 bytes) running `ui-new`: main process RSS 103.2–103.3 MiB and 0.7–1.0% CPU over six idle samples (2026-10-11); WebKit WebContent/GPU helpers are shared system XPC services and are not attributed. Every release archive carries license notices, CycloneDX SBOM, binary checksum and `SHA256SUMS` (release package contracts).
- [ ] Verify: goal/profile setup and all native provider/model/permission acceptance.

Intervention room checkpoint: 142 tests passed, four ignored, 39 suites in 55.78
seconds. Clippy passes. Graphify extracted 482 nodes/1,508 edges without an LLM;
CocoIndex located the local and native control implementations. No Cargo dependency
was added. The first whole-suite attempt exposed a retained PID-based temporary-run
collision; UUID fixture paths preserve failed evidence and prevent reuse. Real browser
verification uses a named local mock fixture, not authenticated provider execution.
Native Cocoa/WebKit review rendering and Cmd-W close were verified using a local app
metadata wrapper around the same binary. The first close shortcut failed; standard
application/edit/window menus fixed it, and the updated process exited 0 with its endpoint
stopped. Bare-executable computer-use identity, quota/tool-gate UI and resource acceptance
remain open. User correction requires closing prior owned verification windows/processes
and checking their exit before replacement; stored lesson lsn_5169ce1ac0c2a86d.

### Direct native transport (DD-23)

- [x] Plan: replace loopback HTTP with an embedded native message bridge.
- [x] Task / Red: missing Room compilation failure; additional fields exposed unit-variant deserialization weakness.
- [x] Task / Green: Rust Room controller, bounded typed requests, WebKit reply handler and embedded document.
- [x] Verify: full suite, targeted final-source tests, Clippy/rustfmt; actual native feedback/resume/review/approve/merge, zero TCP listeners and exit-0 cleanup.
- [x] Complete: README and actual native screenshot refreshed; direct-native wiki evidence and agentmemory records saved; checkpoint prepared.

### Unix termination evidence (DD-24)

- [x] Plan: retain numeric exit status and termination signal as separate outcomes.
- [x] Task / Red: missing signal fields failed compilation; coordinator incorrectly reported synthetic exit -1.
- [x] Task / Green: separate optional signal in process/ownership/test/coordinator evidence; completed coordinator flag prevents signal failure appearing running.
- [x] Verify: 33 targeted tests; complete suite 144 passed / 4 ignored / 0 failed; Clippy/rustfmt. Fresh older-fixture run succeeded, so historical failure remains unexplained.
- [x] Complete: source-hashed process-outcomes wiki, completion audit and agentmemory updated; checkpoint prepared.

### Native startup workspace (DD-25)

- [x] Plan: trace R-21 through business goal, feature, use case, QAS and DD-25.
- [x] Task / Red: registry persistence/redaction/stale revisions; modified profile consent bypass, late repository baseline change and oversized settings temporary-file leak reproduced.
- [x] Task / Green: local startup controller, owned catalog/planning jobs and exact consent; repository baseline rechecked under execution lease; settings bounds checked before temporary-file creation.
- [x] Task / UI: profile cards, goal setup, editable plan review and same-window run opening; macOS JSON smart quote substitution disabled and verified.
- [x] Verify: full regression 148 passed / 4 ignored / 0 failed; Clippy/rustfmt. Actual native empty startup → custom profile registration → planning → task edits → exact plan consent → same-window run → diff/tests → candidate approval → merge → home; exit 0 and kernel absence confirmed. Final text-input correction separately verified in the rebuilt native app.
- [x] Complete: public documentation and actual screenshot, source-hashed wiki and agentmemory checkpoint (`mem_mv1ukwwx_de6b3718e639`). Graphify: 548 nodes / 1,752 edges; CocoIndex located reviewed execution. Provider-authenticated planning, dialog accessibility and OS containment remain part of the full-product audit.

### Remaining subscription quotas (DD-26)

- [x] Plan: R-22 / BG-10 / SF-19 / UC-16 / QAS-17 traced; official Codex stdio and Claude status-line metadata reviewed.
- [x] Task / Red: independent quota windows, 27% used -> 73% remaining, unknown/malformed/future/stale evidence and no context/cost inference.
- [x] Task / Green: normalized local observations, bounded Codex existing-auth metadata query and native account summary refresh.
- [x] Verify: mocked metadata lifecycle and actual Codex quota query; actual native account selection/refresh and source/reset display.
- [ ] Verify: Claude headless capture, malformed-query/timeout cleanup and measured resource bounds.
- [x] Complete: public usage docs, wiki/memory evidence and quota checkpoint; Claude/resource acceptance remains open.

### Windows continuity and GitHub Releases (DD-27)

- [x] Plan: R-23/R-24 / BG-10 / SF-20 / UC-17 / QAS-18 traced; current Unix-only runtime gaps identified.
- [x] Task / Red: actual Windows CI build failure captured; latest observed run 38027108692.
- [x] Task / Red: platform-specific cancellation, lease, recovery and path-boundary tests (`windows_containment`, `leases`, `windows_shell`).
- [x] Task / Green: portable entropy, native Windows process containment/leases/filesystem checks and desktop shell (DD-35–DD-38; windows-latest CI 38056911056 and Release run 38069707257).
- [x] Verify: macOS + Windows runners, exact packaged-binary smoke tests, licenses, checksums and support notes (Release run 38069707257 `native-package` on macOS and Windows (exact packaged binary; shared and worktree three-worker runs, diff/tests, unapproved-merge refusal, approved merge)).
- [x] Complete: committed Windows development handoff.
- [x] Complete: private GitHub Release with verified macOS and Windows assets (v0.1.0-preview.3).

### Subscription-independent supervision and per-agent fallback (DD-28)

- [x] Plan: R-25/R-26 through BG-11 / SF-21 / UC-18 / QAS-19; non-steerable company CLI and optional AI coordinator boundaries recorded.
- [x] Task / Red: restricted paid worker transfers to configured company CLI; no policy means waiting; cycle/account/model/permission rejection; ordinary errors and stops never fail over.
- [x] Task / Green: per-profile fallback, durable account restrictions, drained attempt handoff and fresh target session.
- [x] Task / UI: fallback profile/model registration, company preset action and literal policy shown in execution consent.
- [ ] Verify: actual native fallback profile/model registration and consent interaction.
- [x] Verify: local fallback contracts cover retained partial work, unavailable targets, ordinary-error/stop rejection and lost process ownership.
- [ ] Verify: native shared/worktree fallback chains, restart/reset, exact approval and consent interaction.
- [x] Complete: source-hashed wiki, memory and verified component checkpoint; full fallback acceptance remains open.
- [ ] Verify: fine-grained idle detection and optional designated AI coordinator session lifecycle.

Windows CI evidence: historical run 38022076235 on d9590d7 and latest observed run 38027108692 on c4f6291 completed with macOS success and Windows build failure. The identified Windows gap includes E0425: process::run_controlled_async is Unix-only. Native Windows execution/release remains unverified.

### Cooperative agentmemory steering (DD-29)

- [x] Plan: R-27 / BG-11/BG-12 / SF-24 / UC-21 / QAS-22; company agents publish intermediate status and pull addressed instructions before work/at checkpoints alongside native steering.
- [x] Task / Red: addressed directive validation, duplicate/expiry handling and acknowledgment transitions; tests exposed expiry rollback and missing CLI commands.
- [x] Task / Red: completed pause incorrectly produced a merge candidate; live process after handoff acknowledgment must retain ownership.
- [x] Task / Green: owned process/group absence plus exact directive reconciliation; pause/handoff exits stop partial work, unresolved directives block completion and explicit resume uses a new attempt.
- [x] Verify: five worker tests, including live acknowledgment versus process presence, same-attempt relinquishment, unchanged due scheduling and explicit resume/stale-address refusal.
- [x] Verify: sixth worker test covers actual killed-coordinator pause/handoff recovery; targeted worker/crash/stop suite passes 11 tests.
- [x] Task / Green: transactional typed local ledger, checkpoint CLI polling/acknowledgment and bounded intermediate reports consumed by team briefing.
- [x] Task / Red/Green: bounded loopback REST publish/exact-ID read/import, shared outbox transport, receipt validation, EOF handling and capped pending batches.
- [x] Task / Green: explicit custom profile capability, owned session before launch and worker prompt/environment bootstrap.
- [x] Verify: compiled non-steerable workers voluntarily poll before/during work in shared/worktree modes; conflicting returned sessions fail and merge still requires approval.
- [x] Task / Green: memory discovery/import (DD-44: `memory-discover`, scoped to each task's current address via the agentmemory 0.9.29 `GET /agentmemory/memories?agentId=…` contract; fixture test plus real local daemon on 127.0.0.1:3113) and handoff-driven scheduler transitions (DD-43). Full regression 225 passed / 10 ignored / 0 failed. Shipped in v0.1.0-preview.5 (CI 38074563237 after a Windows cold-start timeout fix in the containment test; Release run 38075480798; `SHA256SUMS` verified).
- [x] Verify: eleven regular cooperative tests and five transport tests; local ledger/compiled CLI, restart/idempotency, contention, expiry, plan supersession, progress/briefing and namespace separation.
- [x] Verify: real configured local agentmemory daemon; typed directive/ack/progress round-trip, duplicate import, briefing source, confirmed outbox delivery and unchanged workflow authority on a controlled fixture.
- [ ] Verify: live company CLI checkpoint polling, individual worker authentication and human permission boundaries.
- [x] Complete: local checkpoint protocol documentation, source-hashed wiki and architecture memory evidence. Full cooperative steering acceptance remains open.

### On-demand team briefing (DD-30)

- [x] Plan: R-28 / BG-12 / SF-22 / UC-19 / QAS-20; factual bundle separate from optional LLM prose.
- [x] Task / Red: missing briefing/read-only API failed compilation; all-session coverage, stale/conflicting reports, missing-run refusal and state invariance tested.
- [x] Task / Green: read-only SQLite snapshot, event observation timestamps, separately observed process presence, bounded source reports and shared CLI/native request.
- [x] Verify: actual native three-session briefing, restriction/error/stop evidence, CLI/native parity, foreign/future report handling and concurrent snapshot consistency.
- [x] Verify: exact-ID agentmemory progress import appears in briefing with its real source on the configured-daemon fixture.
- [ ] Verify: active multi-provider session query, tool approval expiration and automatic live worker report collection.
- [x] Complete: public usage, source-hashed wiki and checkpoint memory evidence; automatic cooperative worker integration remains open.

### Agent status lights (DD-31)

- [x] Plan: R-29 / BG-12 / SF-23 / UC-20 / QAS-21 mapped.
- [x] Task: green/yellow/red indicator, running pulse, textual labels and stable list rendering.
- [x] Verify: actual simultaneous waiting/failed/running states and two-frame native pulse; static mapping/reduced-motion checks; UI stop and owned-process cleanup.
- [x] Verify: a live macOS preference query controls the motion policy; a rendered-egui regression confirms a quota observation snapshot refresh advances the running pulse from the shared frame clock instead of restarting its phase. Windows runtime acceptance remains open below.
- [x] Complete: public usage and source-hashed checkpoint evidence; remaining acceptance stays open.

### Apply the native operating-system reduced-motion preference — issue #9 / R-29 / DD-31

- [x] Plan: keep the existing egui animation opt-out, read macOS `NSWorkspace.accessibilityDisplayShouldReduceMotion` and Windows `SPI_GETCLIENTAREAANIMATION` during existing UI frames, and add no polling timer or thread.
- [x] Red: add regressions for disabling motion, restoring the original animation duration, preserving the current setting when unsupported, and retaining the last setting after a transient query failure.
- [x] Task: apply the native preference to egui's global animation duration; query again on subsequent frames so a changed OS setting takes effect during existing activity.
- [x] Verify: 5 focused motion regressions passed, including the live macOS preference query; full native-feature suite passed (934 passed, 14 ignored, 310 suites); warning-denied Clippy, rustfmt, source-layout/300-line guard (2 passed), and diff checks passed.
- [x] Knowledge: LLM Wiki and AgentMemory checkpoint updated; Graphify refreshed to 8,914 nodes / 14,079 edges. CocoIndex is not installed in this environment (`command -v cocoindex` returned no executable), so no CocoIndex result is claimed.
- [ ] Verify Windows: compile this target and run the Windows package/UI acceptance; this macOS host has no Windows Rust standard library installed.
- [ ] Complete: update the LLM Wiki, AgentMemory and issue #9 after verification; leave pulse continuity during quota observation refresh as its own DD-31 acceptance check.

### Preserve running-status pulse phase through quota refresh — issue #9 / DD-31

- [x] Plan: render the status dot from egui's monotonic frame clock so refreshed quota observations cannot reset its animation phase.
- [x] Task / TDD: add a shared-context render regression that changes the quota observation between frames and inspects the actual rendered status-dot colors.
- [x] Verify: pulse regression 4/4, full native-feature suite 935 passed / 14 ignored / 0 failed across 310 suites, warning-denied Clippy, rustfmt, source-layout 2/2, and diff checks pass.
- [x] Knowledge: record the invariant in DD-31 and the LLM Wiki; final AgentMemory checkpoint `mem_mvk60k44_e07cad2a7507`; Graphify refreshed to 8,916 nodes / 14,081 edges / 675 communities.
- [x] Complete: AgentMemory and LLM Wiki recorded; commits `1d2aba5` and this checklist update pushed; issue #9 and PR #22 received progress comments. Packaged workflow acceptance is unchanged.

Quota/fallback/status checkpoint: final full regression 162 passed / 4 ignored / 0 failed across 43 suites in 96.16 seconds; Clippy, formatting, whitespace and JavaScript syntax checks passed. Local evidence: `.agentconductor/verification/quota-fallback/`. Actual Codex quota refresh used existing CLI authentication and no generation; private account observations remain local. Native status fixture reached stopped through UI and all five owned PIDs were kernel-absent after exit-0 cleanup. Graphify: 1,129 nodes / 2,601 edges without an LLM; CocoIndex structural grep located apply_fallbacks. No runtime dependency was added. Ponytail capabilities are not exposed in this session; no execution claimed.

Team briefing checkpoint: full regression 169 passed / 4 ignored / 0 failed, 44 suites, 99.32 seconds; Clippy/format/JS syntax passed. Native Brief team returned three real retained fixture sessions; revision and SQLite SHA-256 unchanged; owned app PID 49618 exited 0 and kernel absence confirmed. Graphify 1,160 nodes / 2,664 edges without LLM; CocoIndex located read_only_snapshot. No Cargo dependency added. CI run 38023847399 completed: macOS all checks passed, Windows build failed. Live agentmemory collection/directive acknowledgment and native Windows remain unfinished.

Cooperative checkpoint verification: final full regression 177 passed / 4 ignored / 0 failed, 46 suites, 107.05 seconds. Eight new local contract tests; Clippy all-targets with warnings denied, formatting and diff checks passed. Graphify rebuilt 1,216 nodes / 2,847 edges without LLM; CocoIndex located report_progress. No Cargo dependency added and no verification UI was opened. Agentmemory architecture decision: mem_mv1ws2a1_1df7797727b8. Live memory transport/capability/bootstrap/scheduler and native Windows remain open.

Memory transport checkpoint: 188 passed / 5 ignored / 0 failed, 47 suites, 105.23 seconds. Separately invoked existing-daemon fixture passed; actual company CLI polling remains open. Clippy/format/diff passed; Graphify 1,280 nodes / 3,021 edges, CocoIndex located remember_note. No dependency added, no daemon/UI launched. Architecture memory mem_mv1xilna_6df13588935e. CI 38025340939 terminal: macOS success, Windows build failure. Full goal remains active.

Cooperative worker bootstrap checkpoint: 191 passed / 5 ignored / 0 failed across 48 suites (66.29 seconds summed suite time, not wall time). Clippy all-targets with warnings denied, format and diff checks passed. Three new tests exercise real CLI subprocesses and authoritative local checkpoint commands; no company LLM or new daemon/UI was launched. Graphify: 1,304 nodes / 3,070 edges without LLM; CocoIndex located worker_address. No Cargo dependency added. Automatic memory collection/authentication/scheduler handoff and native Windows remain open.

Cooperative exit checkpoint: 193 passed / 5 ignored / 0 failed across 48 suites, 67.02 seconds summed suite time (not wall time). Clippy all-targets with warnings denied, formatting and whitespace passed. Graphify: 1,319 nodes / 3,099 edges without LLM; CocoIndex located exit_directive. Lesson lsn_d9c15a15246342f9 distinguishes directive completion, task completion and process exit. No dependency/daemon/UI added. Local evidence: `.agentconductor/verification/cooperative-exit/`. CI 38027108692 on the preceding bootstrap commit completed: macOS success, Windows build failure. Automatic reviewed-target handoff and full-product acceptance remain open.

Cooperative recovery checkpoint: targeted cooperative_worker/crash_recovery/stop_resume suites passed 11 tests, no failures. A real killed coordinator and live acknowledged worker validate the existing recovery implementation without a runtime change. Clippy with warnings denied, formatting and whitespace passed; the test harness explicitly reaps its coordinator on both branches. Graphify 1,325 nodes / 3,108 edges without LLM; CocoIndex located recover. No UI/daemon/dependency added. Evidence: `.agentconductor/verification/cooperative-recovery/`. The preceding full regression remains 193/5/0; no new full-regression total is claimed. Pending/accepted instruction continuation, automatic handoff and Windows/releases remain open.

### Product README and user manual (DD-42)

- [x] Plan: R-36 / BG-15 / SF-30 / UC-27 / FR-21 / NFR-14 / QAS-28 / DDR-11 mapped; first-contact questions (orchestration engine vs optional LLM coordinator, role assignment, usage-limit redistribution, prerequisites, agentmemory optional) added to R-36/QAS-28.
- [x] Task / Red/Green: `tests/public_docs.rs` (README under 120 lines, no shell blocks, screenshots, required sections; manual documents every `--help` command; manual links only shipped assets).
- [x] Verify: `public_docs` 3/3 passed; release package contracts 13/13 passed.
- [x] Complete: v0.1.0-preview.3 (Release run 38069707257: validate + native-package on macOS and Windows, publish) ships README, `docs/user-manual.md` and screenshots; both archives verified with `shasum -a 256 -c SHA256SUMS`. Full regression 221 passed / 9 ignored / 0 failed; Clippy denied warnings clean; no source file over 500 lines.

### Manual plan authoring (R-37 / DD-45)

- [x] Plan: R-37 → BG-16 → SF-31 → UC-28 → FR-22/NFR-15 → QAS-29 → DDR-12 → DD-45 (user decision 2026-10-11).
- [x] Task / Red: `tests/manual_plan.rs` failed with `unknown variant draft`.
- [x] Task / Green: startup `draft` request writes an empty three-task proposal without a job; **Write plan manually** button; README and manual updated.
- [x] Verify: draft launches no process; unfilled/overlapping drafts refused; filled draft approved by exact digest runs to `waiting_approval` with three tests; dirty tree and moved base refused. Full regression 227 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Workspace settings and switching (R-38/R-39, DD-46/DD-47)

- [x] Plan: R-38/R-39 → BG-17 → SF-32/SF-33 → UC-29/UC-30 → FR-23/FR-24/NFR-16 → QAS-30/QAS-31 → DDR-13 → DD-46/DD-47 (user request 2026-10-11).
- [x] Task / Red: `tests/workspace_settings.rs` failed (`unknown variant pick_folder`, missing settings layout).
- [x] Task / Green: goal first, **Workspace settings** group (coordinator, repository + **Browse…**, worktrees), OS folder dialog without new dependencies, title-bar workspace menu, recent registry, in-place switch with lease handover; README/manual updated (also corrected **+ Add**).
- [x] Verify: picker selection/cancel, save without repository, plan refused without repository, switch/recent/lease release, refusal while held elsewhere and for relative paths. Full regression 229 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check (macOS dialog, Windows CI) and release.

### Named workspace registry (R-40 / DD-48)

- [x] Plan: R-40 → BG-17 → SF-34 → UC-31 → FR-25/NFR-16 → QAS-32 → DDR-13 → DD-48 (user request 2026-10-11).
- [x] Task / Red: `tests/workspace_registry.rs` failed (`unknown field name`, no name field).
- [x] Task / Green: `name` in workspace config with validation, name field above the goal, `create_workspace` (slugged unique folder under `~/.agentconductor/workspaces/`), `forget_workspace` (registry only), menu by name with Remove / New workspace… / Open existing folder…; README and manual updated.
- [x] Verify: name save/limits, create twice gives distinct folders, listing by name, current cannot be removed, removal keeps files. Full regression 231 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Release change summaries (R-41 / DD-49)

- [x] Plan: R-41 → BG-10 → SF-35 → UC-32 → FR-26 → QAS-33 → DDR-08 → DD-49 (user request 2026-10-11).
- [x] Task / Red: `tests/test_release_notes.py` failed (no script, generic `--notes`).
- [x] Task / Green: `CHANGELOG.md` with sections for preview.1–6, `scripts/release_notes.py`, publish job checks out the tag and uses `--notes-file`.
- [x] Verify: exact section extraction, missing/empty refusal, every published tag covered, workflow contract; release contracts 15/15.
- [x] Complete: preview.6 published with its CHANGELOG notes (CI 38100249588 and verify jobs of Release run 38101164095 green; the publish job was not started by GitHub because of an account billing/spending limit, so the verified CI artifacts were checksum-checked and published locally with the same notes); earlier releases' notes replaced with their sections.

Release policy (user, 2026-10-11): releases are cut only when the user asks; each feature updates the next `CHANGELOG.md` section instead.

- [x] Amendment (user, 2026-10-11): **Open existing workspace…** lists saved workspaces in `~/.agentconductor` (`saved_workspaces` / `open_workspace`; folders without `startup.json` or outside the data directory refused); Red → Green in `tests/workspace_registry.rs`.

### Explained clean-baseline gate (R-42 / DD-50)

- [x] Plan: R-42 → BG-13 → SF-36 → UC-33 → FR-27 → QAS-34 → DDR-01 → DD-50 (user request 2026-10-11; clean baseline rule kept).
- [x] Task / Red: `tests/baseline_commit.rs` failed (`unknown variant baseline_status`, no dialog).
- [x] Task / Green: `baseline_status` and revision-checked `commit_baseline` requests; dialog with reason, changed files, **Commit and continue** / **Stop** before Generate plan and Write plan manually; manual and CHANGELOG updated.
- [x] Verify: modified and untracked files listed; no commit on stop or stale revision; one commit with all changes and goal-derived message; clean repository reports nothing to commit; planning then starts from the new HEAD. Full regression 233 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Ignore OS files and continue (R-44 / DD-52)

- [x] Plan: R-44 → BG-13 → SF-36 → UC-36 → FR-29 → QAS-37 → DDR-01 → DD-52 (user request 2026-10-11).
- [x] Task / Red: `os_metadata_files_are_ignored_locally_without_a_commit` failed (no `os_files`, no dialog guidance).
- [x] Task / Green: `os_files` in baseline status, revision-checked `ignore_os_files` writing missing patterns to the local exclude file, dialog guidance and **Ignore and continue** loop; manual and CHANGELOG updated.
- [x] Verify: nested `.DS_Store` and `._*` detected; patterns added once; no commit; non-OS file remains for decision; stale revision refused. Full regression 234 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check and release.

### Visible planning progress (R-45 / DD-53)

- [x] Plan: R-45 → BG-12 → SF-38 → UC-37 → FR-30 → QAS-38 → DDR-07 → DD-53 (user request 2026-10-11, after a plan looked missing while the coordinator was still working for ~45 s).
- [x] Task / Red: `tests/planning_progress.rs` failed (no `started_ms`/`log`, no progress element).
- [x] Task / Green: `started_ms` in the stored job, `started_ms` and `log` in live and recovered job snapshots, review-area progress with elapsed time, ready notice with scroll, failure reason with log folder; manual and CHANGELOG updated. Full regression 237 passed / 11 ignored / 0 failed.
- [ ] Complete: native window check.

### Human-readable task instructions (R-46 / DD-54)

- [x] Plan: R-46 → BG-13 → SF-39 → UC-38 → FR-31 → QAS-39 → DDR-07 → DD-54 (user request 2026-10-11).
- [x] Task / Red: `tests/plan_readability.rs` failed (no section format in the planning prompt; editor 3 rows).
- [x] Task / Green: planning prompt requires labelled sections with bullets in the goal's language; instruction editor 6 rows minimum, auto-grows to 30; manual and CHANGELOG updated. Full regression 239 passed / 11 ignored / 0 failed.
- [ ] Complete: native check with a real coordinator plan.

### Agent role rules and numbered steps (R-47 / DD-55)

- [x] Plan: R-47 → BG-19 → SF-40 → UC-39 → FR-32 → QAS-40 → DDR-12 → DD-55 (user request 2026-10-11; numbered steps amendment).
- [x] Task / Red: `tests/role_rules.rs` failed (no `[roles.json]` argument, no `role` field, no table or steps).
- [x] Task / Green: `roles` module (validate, prompt section, plan check), optional `role` on tasks (omitted when absent), `plan … [roles.json]`, workspace `roles` saved and passed on Generate plan, role-aware manual drafts, step 1–3 headings, role table with Suggest defaults, role badges; README, manual and CHANGELOG updated. Full regression 242 passed / 11 ignored / 0 failed.
- [x] Amendment: role entries `{agent, model?}` (plain strings still accepted), model validated against the observed catalog, engine applies pinned models to tasks, model selectors in the role table; Red → Green `role_entries_can_pin_an_observed_model`.
- [x] Amendment: each step is its own card (Step 1 Workspace settings, Step 2 Agent roles with Save/Write/Generate, Step 3 Review the plan).
- [ ] Complete: native check with a real coordinator.

### Immediate agent registry persistence (R-48 / DD-56)

- [x] Plan: R-48 → BG-09 → SF-41 → UC-40 → FR-33 → QAS-41 → DDR-13 → DD-56 (user report 2026-10-11: added agents lost when the window was reopened).
- [x] Task / Red → Green: `tests/profile_persistence.rs`; `save_profiles` request (new `setup/profiles.rs`), coordinator/role repair, UI Use profile and Remove save at once.
- [x] Verify: agent list survives reopen, goal untouched, stale/invalid refused, coordinator and roles repaired.

### Model lists, plan retention and compact agents (R-49/R-50/R-51, DD-57/DD-58)

- [x] Plan: R-49 (amended: once on add, Models refreshes, quiet), R-50, R-51 → SF-41/SF-42 → UC-40/UC-41 → FR-34–FR-36 → QAS-42/QAS-43 → DDR-13 → DD-57/DD-58 (user reports 2026-10-11: no models in selectors; focus jumped; plan vanished after Models).
- [x] Task / Red → Green: `tests/model_catalogs.rs`, `tests/profile_persistence.rs` — catalog carry-over on save, catalog changes keep the proposal, consent compares execution views, save keeps the plan unless plan-relevant settings change, discovery only for newly added agents and silent, in-place model option refresh skipping the focused selector, compact cards.

### Shareable workspace settings (R-52 / DD-59)

- [x] Plan: R-52 → BG-20 → SF-43 → UC-42 → FR-37/NFR-18 → QAS-44 → DDR-15 → DD-59 (user request 2026-10-11).
- [x] Task / Red: `tests/workspace_share.rs` failed (`unknown variant export_settings`, no menu items).
- [x] Task / Green: `setup/share.rs` (export, preview, import), validation helpers moved to `setup/profiles.rs` (setup.rs under 500 lines), Workspace menu Export/Import with preview dialog and verification checkbox; manual, README, CHANGELOG.
- [x] Verify: no secrets/catalogs/goals/absolute repo path, `repository: "."`, LF and deterministic output; import reproduces settings, keeps local secrets/catalogs and goal, downgrades local claims unless confirmed; unknown format and stale revision refused.
- [ ] Complete: two-machine check with a real repository.

### Variable plan size and plan documents (R-53/R-54, DD-60)

- [x] Plan: R-53 (supersedes R-04's fixed three), R-54 → SF-44 → UC-43 → FR-38/FR-39 → QAS-45 → DD-60 (user decision 2026-10-11).
- [x] Task / Red: `tests/plan_size.rs` failed (three-task validation, fixed prompt, no `--from`, no `read_plan`, no add/remove).
- [x] Task / Green: 1–12 task validation, smallest-split prompt, `plan … [--from plan.md]` with `source-plan.md`, `setup/planning.rs` shared by Generate plan and Read plan (goal from heading), `pick_file` (macOS/Windows), Read plan button, Add/Remove task; old `requires_exactly_three_tasks` updated to the new rule; README, manual, CHANGELOG.
- [ ] Complete: native check with a real plan document and coordinator.

### Slugs, phases, progress, archives and plan rules (R-55–R-59, DD-61–DD-65)

- [x] Plan: R-55–R-59 → BG-21 → SF-45–SF-49 → UC-44–UC-48 → FR-40–FR-44/NFR-19 → QAS-46–QAS-50 → DDR-16 → DD-61–DD-65 (user request 2026-10-11).
- [x] Increment 1 — slug model, tree creation and local exclusion (DD-61): `src/slugs.rs`, `setup/slug.rs`, `create_slug`, slugs preserved by save; `tests/slugs.rs`.
- [x] Increment 2 — progress derivation and tree verifier (DD-63): `src/slug_progress.rs` (stages from file locations; mismatches for duplicates, undeclared files/phases, missing tasks, out-of-order phases); `tests/slug_progress.rs`.
- [ ] Increment 3 — plan rules file and workspace override (DD-65). Core done: `src/plan_rules.rs` (default rules, `~/.agentconductor/plan-rules.md` created once, override precedence), `PlanInputs`, `plan … --rules`; `tests/plan_rules.rs`. Pending: workspace `plan_rules` field, startup passing effective rules, editor (waits for Codex's uncommitted setup/UI work).
- [ ] GitHub #8 — recursively load repository planning rules (enhancement).
  - [x] Plan: extend R-59 / SF-49 / UC-48 / FR-44 / QAS-50 / DD-65 with `<repository>/.agentconductor/plan-rules/**/*.md`; preserve global file and exclusive inline workspace override; keep the default file limit aligned with the current 300-line requirement.
  - [x] Task / Red: add tests for nested/multiple files, lexical ordering, precedence, non-Markdown files, and symlink/outside-root exclusion; targeted test fails until the loader exists.
  - [x] Task / Green: collect local Markdown rules and attach them to the plan coordinator input; existing CLI `--rules <file>` behavior remains unchanged. Scope `.gitignore` exceptions to this subtree so teams can commit shared rules.
  - [x] Verify: plan-rules (5 tests), goal-to-slug planning flow, source-layout, formatting and all-target Clippy pass locally. Full all-target execution hits host sandbox `Operation not permitted` in unrelated cooperative-memory and folder-execution tests; those are not counted as passing.
  - [x] Complete: commit/push the issue branch and open a linked PR for review. Merge and application to the Rust branch remain with the parent task after review.
- [ ] Increment 4 — phased planning, one run per phase, task file movement (DD-62). Core done: `src/phases.rs` (write analysis/plan/phase/task files once, 1–6 phases × 1–12 tasks, safe ids, run plan only for the current phase with earlier-phase dependencies dropped, single-step atomic moves); `tests/phases.rs`. Coordinator phased planning done: `plan … --phases <slug-dir>` asks for analysis + 1–6 phases, validates each phase like a plan with role rules, writes the slug tree and returns phase one as the reviewable plan (`tests/phased_planning.rs`). Run hooks done: `run … --phase <slug-dir> <phase-id>` links the run (`slug-phase.json`); a tested candidate moves the phase to verification and `workflow::merge` (CLI and window) moves it to completed, reporting but not rolling back a failed tree update (`tests/phase_link.rs`). Pending: UI (startup-run passing the link).
- [x] Increment 5 — slug archives in export/import (DD-64): `src/archive.rs` (deterministic store-only ZIP, CRC-32, UTF-8 names, unsafe-entry refusal, staged extraction; validated with system `unzip -t`), `share.rs` export lists `{slug, goal, created_ms, archive, progress}` and import stages/verifies every archive before replacing (backup `.bak`); `tests/archive.rs`, `tests/slug_archives.rs`.
- [ ] Increment 6 — window: slug list, progress, plan rules editor.
  - [x] 6a backend: Generate/Read plan create or reuse the goal's slug and request phases (`--phases`) with the effective rules (`--rules`); a coordinator returning only tasks becomes phase `01-plan`; settle records the proposal's slug phase; Start passes `--phase` to `startup-run`; `next_phase` builds the next phase from the tree without a coordinator; `save_plan_rules` (workspace override / global file, written only when saved); snapshot lists slugs with progress and the current global rules; `Request` moved to `setup/request.rs`. `tests/slug_flow.rs` covers goal → slug → phase 1 → verification → merge → completed → next phase → rules; full regression 294 passed / 12 ignored.
  - [x] 6b page: Slug field (suggested from the goal), Goals & progress card (stages, per-phase counts, mismatch warnings, Plan next phase), Workspace ▾ Plan rules… dialog (global/workspace); `tests/slug_page.rs`; manual §1.4c. Full regression 295 passed / 12 ignored.
  - [ ] Native window check.

### Idle footprint (R-65 / QAS-58 / DD-71)

- [x] Plan: R-65 → NFR-05 → QAS-58 → DDR-06 → DD-71 (user directive 2026-10-11: the product exists to save resources).
- [x] Task / Red: `tests/idle_footprint.rs` failed (1 s `setInterval` in both pages, icons in every snapshot).
- [x] Task / Green: adaptive self-rescheduling refresh (1 s active, 5 s idle, paused while hidden) in workspace and Agent Room; snapshots carry icon data URLs only when `icons: true` (first load / agent kinds change).
- [x] Verify: release build `ui-new`, 30 s idle after 8 s warm-up — before: RSS 105–106 MiB, CPU avg ≈1.8 % with 3–4.5 % spikes; after: CPU avg ≈0.12 % (one 1.4 % sample), RSS 105 MiB right after opening settling to ≈88 MiB. CPU target met; RSS exceeds 100 MiB briefly at window start (WebKit helpers not attributed). Full regression 286 passed / 12 ignored / 0 failed.
- [x] Whole tree (release `ui-new`, 34 s idle): main ≈107 MiB + WebKit GPU/WebContent/Networking helpers ≈85 MiB = ≈192 MiB; average CPU ≈0.49 % across the tree. Quota/discovery children are short-lived and not resident.
- [ ] Follow-up: Agent Room during a run; bring the main process under 100 MiB at window start.

### Empty default agent list (R-66 / QAS-59 / DD-72)

- [x] Plan: R-66 → BG-09 → SF-41 → QAS-59 → DDR-13 → DD-72 (user request 2026-10-11; tests add their agents first).
- [x] Task / Red → Green: `tests/empty_default_agents.rs`; new workspaces have no agents and no coordinator, templates remain in + Add, empty registry saves, planning refused until an agent exists; quota and registry tests now register template agents at start. Full regression 287 passed / 12 ignored / 0 failed.


## Roadmap — messenger integration (R-43, planned)

Not started. Each phase follows Plan → Task → Verify → Complete with TDD.

- [ ] Plan: confirm DD-51 with the user — target company messenger and its API, relay vs. polling, who may approve, default redaction.
- [ ] Phase 1 — notifications: channel port over the audit outbox; Slack (Socket Mode) and generic webhook connectors; per-workspace channel settings and credential-store secrets; QAS-35.
- [ ] Phase 2 — decisions: Allow/Deny for pending tool requests with signature, linked approver, exact binding, expiry and replay checks; QAS-36.
- [ ] Phase 3 — company messenger decisions through the confirmed mechanism; briefing on request.
- [ ] Verify each phase with fixture servers in tests plus a live Slack workspace and the company messenger provided by the user.

### Claude quota collection — DD-66

- [x] Plan: trace R-60 → BG-22 → SF-50 → UC-49 → QAS-51 → DD-66.
- [x] Task / Red: stream conversion, unknown/malformed metadata, live persistence and metadata-only refresh regressions.
- [x] Task / Green: owned-stream collection and native board integration.
- [x] Verify: focused tests (21), source layout, complete local regression (268 passed / 11 ignored), formatting/lint. Windows validation is blocked by GitHub runner billing.
- [x] Complete: manual, changelog, source-linked wiki, agentmemory and commit/push.
- Optimized local macOS build passed. No new release published; actual Enterprise quota emission and Windows runner validation remain unverified.

### Inline agent quota — DD-67

- [x] Plan: R-61 → BG-22 → SF-51 → UC-50 → FR-46/NFR-21 → QAS-52 → DD-67.
- [x] Task: TDD badge behavior and retained-workspace snapshot.
- [x] Verify: 18 focused Rust checks, 1 Python/Node display check and JavaScript syntax; optimized build.
- [x] Complete: docs/wiki/memory, commit/push and replace the previous runtime.

### MCP Jira/Confluence goal imports — DD-68 (active)

- [x] Plan: R-62 → BG-23 → SF-52 → UC-51/52 → FR-47/48, NFR-22 → QAS-53–55 → DD-68.
- [x] Task 1 / TDD: OpenCode-compatible configuration, source bindings, redaction and settings UI.
- [x] Task 2 / TDD: explicit keywords/chips and confirmed shorthand resolution.
- [x] Task 3 / TDD: bounded read-only MCP retrieval and preview/apply with provenance.
- [x] Task 4 / TDD: imported snapshots → goal/slug → phased planning and local wiki provenance index; summaries remain reviewed knowledge-phase output.
- [x] Verify / local: local/remote connector fixtures, missing records, partial results, import safety, native preview/apply and macOS regression.
- [ ] Verify / company: authorized live company server and authentication.
- [ ] Complete: English operator docs, source-linked wiki, agentmemory and reviewed delivery.

### Workspace quota collection regression

- [x] Reproduce: active workspace has no last run; display fixtures did not test collection.
- [x] Red: fresh-workspace Codex quota request fails as unknown request.
- [x] Green: independent cache and CLI collection; actual Codex/Claude checks.
- [x] Verify: collection without runs, provider errors, cleanup, refresh scheduling and actual account values; native visual review remains separate.
- [x] Complete: docs/wiki/memory and replaced the prior owned runtime; fresh real account observations confirmed.

### Agent-specific refresh and action icons — DD-69

- [x] Plan: R-63 → BG-22 → SF-53 → UC-53 → FR-49/NFR-23 → QAS-56 → DD-69.
- [x] Task: per-agent two-minute scheduler, accessible refresh icons and Edit/Models/Remove icons.
- [x] Verify: periodic boundary checks, workspace/run cache binding, native CLI collection and cleanup, source layout and Clippy; optimized build checked before runtime replacement.
- [x] Complete: documentation/wiki/memory and runtime replacement; one native app process confirmed.


### Compact quota window presentation — DD-70

- [x] Plan: R-64 → BG-24 → SF-54 → UC-54 → FR-50/NFR-24 → QAS-57 → DD-70.
- [x] Task / Red: single-minimum display and absent plan_type fail new multi-window/Enterprise tests.
- [x] Task / Green: compact quota-controls group, provider duration labels and backward-compatible plan metadata.
- [x] Verify: 282 full Rust checks / 12 ignored before final UI refinement; final focused 14 / 1 ignored, JS display check, live account refresh, async delayed-provider and duplicate-request check, lint/build.
- [x] Complete: wiki/memory and replaced runtime with one-line numeric quota controls.

- [x] DD-70 interaction refinement: enabled button/icon pointer cursors and 5px quota-refresh spacing.


### Stable release numbering and operator screenshots

- [x] Plan: current version 0.1.0, subsequent version 0.1.1; GitHub publications are stable releases, not pre-releases.
- [x] Task / TDD: v0.1.0 notes regression fails before section migration; stable publish workflow removes --prerelease.
- [x] Verify: release-note tests, updated English operator manual and actual native screenshots.
- [x] Complete: current changes delivered with preview history preserved.

### Agent preferences and native AC identity — DD-73

- [x] Plan: R-67 → BG-25 → SF-55 → UC-55 → FR-51/NFR-25 → QAS-60 → DD-73.
- [x] Task/TDD Red: missing cadence field and outdated two-minute scheduler fail new checks.
- [x] Task/Green: settings, executable picker, consistent inputs, silent quota and native icon.
- [x] Verify: 293 Rust tests passed / 12 ignored / 0 failed, 22 Python checks, formatting, Clippy, JS syntax, optimized macOS build, actual native UI/screenshots and packaged shared/worktree smoke.
- [x] Complete: English manual/wiki, shared memory, reviewed commits and delivery.
- [x] Platform release follow-up: repository made public; v0.1.1 published with CI-verified macOS and Windows packages (Windows UI verification on a real machine remains open).

### Provider-driven profile and quota reset time — DD-74 / DD-75

- [x] Plan: R-68/R-69 → BG-26 → SF-56/57 → UC-56/57 → FR-52/53/NFR-26 → QAS-61/62 → DD-74/75.
- [x] Task/TDD Red: `tests/provider_switch.rs` (4) and `tests/test_quota_reset.py` failed first.
- [x] Task/Green: `renames` in `save_profiles`, provider onchange with `uniqueId`, dialog order, `quotaBadge` reset suffix in both pages.
- [x] Verify: 299 Rust passed / 0 failed, Python suites, clippy `-D warnings`, fmt, JS syntax.
- [x] Complete: manual, CHANGELOG (v0.1.1), wiki, agentmemory, commit/push.

### Interface language — DD-76

- [x] Plan: R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76.
- [x] Task/TDD Red: `tests/interface_language.rs` (4) and `tests/test_utf8_sources.py` (2) failed first.
- [x] Task/Green: `language.rs`, `messages.rs`, `i18n.js`, English/Korean JSON catalogs and explicit ID bindings; Settings dialog; textarea placeholders translated, text kept.
- [x] Verify: 303 Rust passed / 0 failed, 26 Python, clippy, fmt; native macOS window in Korean (`docs/assets/workspace-korean.jpg`), saved choice reloaded.
- [x] Complete: manual §1.2c, CHANGELOG, wiki, agentmemory, commit/push.

- [x] 2026-10-13 amendment: app-owned web/native messages use stable IDs and named parameters; locale data is exempt from the code line limit. Catalog coverage and live language-switch checks are in `tests/messages.rs`, `tests/i18n_runtime.cjs` and `tests/test_i18n_catalogs.py`.

### Fallback model list and icon-only Settings — DD-77

- [x] Plan: R-71 → SF-59 → UC-59 → FR-55 → QAS-64 → DD-77; R-70 amendment.
- [x] Task/TDD Red: `tests/fallback_models.rs` (3) failed first.
- [x] Task/Green: `fallbackModels`, poll refresh, help text, ⚙ button, Korean entries.
- [x] Verify: full suites, clippy, fmt.
- [x] Complete: manual, CHANGELOG, commit/push.

### Messenger notifications phase 1 — DD-79

- [x] Plan: R-43 activation → BG-18 → SF-37 → UC-34/UC-61 → FR-28/NFR-17 → QAS-35/QAS-66 → DD-79.
- [x] Task/TDD Red: `tests/messenger.rs` (3 + 1 live) and `tests/messenger_settings.rs` failed first (module absent).
- [x] Task/Green: `src/messenger.rs`, `src/messenger/send.rs`, `workflow::save` and `Startup::settle` hooks, run link before `startup-run`, `messenger`/`save_messenger`/`test_messenger`, Messenger dialog, Korean strings, flush before CLI exit.
- [x] Verify: 310 Rust passed / 0 failed, Python suites, clippy, fmt; live delivery through the system curl to a local HTTPS receiver (Korean text, quotes, JSON content type; refused port reported).
- [x] Complete: manual §1.2d, CHANGELOG v0.1.2, wiki, agentmemory, commit/push.
#### DD-68 implementation acceptance — 2026-10-11

- [x] Plan: activate existing R-62 / BG-23 / SF-52 / UC-51–52 / FR-47–48 / NFR-22 / QAS-53–55 / DD-68.
- [x] Task / Red: missing module/UI, held-open SSE timeout and duplicate local session regressions reproduced.
- [x] Task / Green: workspace settings, on-demand discovery/import, shorthand confirmation, selected source evidence and private wiki index.
- [x] Verify / macOS: local/remote fixtures, child cleanup, stale/tampered evidence, partial failures, native preview/apply, 320 Rust tests and 32 Python tests.
- [x] Verify / Windows: CI 38134378978 passed native tests, all Python contracts, fmt/Clippy and exact packaged shared/worktree execution on source 5cefa6c.
- [x] Complete: English manual/wiki, agentmemory, reviewed commits and verified stable v0.1.2 packages; macOS runtime restarted with the CI binary.
- [ ] Company acceptance: authorized company MCP server and authentication configuration supplied by the operator.


DD-68 UI amendment (2026-10-11): configure MCP connections through the top-bar Settings menu; keep reference insertion and preview beside the goal. The Settings gear uses a 26 px glyph in a 32 × 32 px pointer target.


### Windows quota and icon correction — DD-78

- [x] Plan: R-72 → BG-28 → SF-60 → UC-60 → FR-56/NFR-28 → QAS-65 → DD-78.
- [x] Task / Red: missing fallback badge, launcher precedence and Windows ConPTY timeout reproduced (CI 38131529935); user confirmed error 193.
- [x] Task / Green: extension-aware launcher discovery/query recovery, bounded ConPTY with explicit standard handles, original provider badges.
- [x] Verify / macOS: 320 Rust tests, 32 Python tests, fmt/Clippy and release package smoke (shared/worktree, approval and merge).
- [x] Verify / Windows: CI 38134378978 passed 135 native tests (0 failed, 1 ignored), 32 Python tests, saved npm Codex profile, spaced native/batch Claude quota, trust refusal, icon fixtures and exact packaged execution.
- [ ] Subscription acceptance: user Windows account refresh after installing the corrected build.
- [x] Complete: manual/wiki, agentmemory, reviewed commits and native stable v0.1.2 delivery.


### Custom CLI models without discovery - DD-80

- [x] Plan: R-73 -> BG-29 -> SF-61 -> UC-62 -> FR-57/NFR-29 -> QAS-67 -> DD-80.
- [x] Task / Red: missing manual parser and capability guidance reproduced.
- [x] Task / Green: bounded ID editor, retained IDs, observed JSON compatibility, explicit operator provenance.
- [x] Verify: actual JavaScript parser/UI contracts, durable catalog without a discovery job, translations and native Korean dialog; 320 Rust tests and 32 Python tests passed locally.
- [x] Complete: English manual/wiki, agentmemory, committed changes and updated native runtime; stable release delivery is tracked with the platform checks above.

### Stable v0.1.2 delivery — 2026-10-11

- [x] Plan: preserve v0.1.1 and publish the next stable version without a prerelease flag.
- [x] Task: package exact source 5cefa6ce8a29ee7cdf4f19e302a440be2959d1e6 for macOS Apple silicon and Windows x86_64.
- [x] Verify: [CI 38134378978](https://github.com/neodamul/AgentConductor/actions/runs/38134378978), four jobs passed; macOS 320 Rust tests / Windows 135 Rust tests, 0 failures; 32 Python tests on each platform; fmt and Clippy passed.
- [x] Verify: each packaged binary executed three parallel tasks and three tests in shared and worktree modes; unapproved merge refused and approved merge verified.
- [x] Verify: downloaded archive and binary SHA-256, ZIP integrity, source manifest, dependency notices and SBOM; 42 macOS / 108 Windows dependencies.
- [x] Complete: [stable v0.1.2](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.2), both archives and SHA256SUMS uploaded; current macOS app replaced with the verified CI binary while preserving the workspace and reviewed plan.

External acceptance remains separate: real company MCP authentication and the operator Windows subscription account have not been supplied to CI. The existing macOS accessibility issue after sequential Settings/MCP dialogs remains open; native MCP preview/apply was visually verified, while the custom model dialog exposes its controls in the accessibility tree.

### Messenger decisions phase 2 — DD-81

- [x] Plan: R-43 phase 2 → BG-18 → SF-37 → UC-35/UC-63 → FR-58/NFR-30 → QAS-36/QAS-68 → DD-81.
- [x] Task/TDD Red: `tests/messenger_inbound.rs` (3 + 1 live) failed first (module and dialog fields absent); `messenger_settings` extended for the new secrets.
- [x] Task/Green: `src/messenger/inbound.rs` (codes, parse, RFC 2104 HMAC, Slack/relay readers, `poll_once`, `spawn`), `send::fetch`, named secrets and `status`, `src/ui/setup/messenger.rs`, startup-run hook, dialog fields, Korean strings.
- [x] Verify: 323 Rust passed / 0 failed, Python suites, clippy, fmt; live round trip through the system curl against a local HTTPS stand-in for Slack (announce → history with Bearer token → deny applied → confirmation).
- [x] Complete: manual, CHANGELOG v0.1.3, wiki, agentmemory, commit/push.

### Automatic slug naming — DD-82

- [x] Plan: R-74 → BG-12 → SF-46 → UC-64 → FR-59 → QAS-69 → DD-82.
- [x] Task/TDD Red: `tests/slugs.rs` (romanization, date fallback, truncation) and `tests/slug_page.rs` (no input) failed first.
- [x] Task/Green: Revised Romanization tables in `slugs::suggest_at`, UTC civil date fallback, on-disk collision check in `create_slug`, slug field and script removed.
- [x] Verify: 323 Rust passed / 0 failed, Python suites, clippy, fmt.
- [x] Complete: manual §1.4c, CHANGELOG v0.1.3, wiki, agentmemory, commit/push.
### Settings button spacing amendment — DD-68 / 2026-10-11

- [x] Plan: retain the visible settings gear and menu access while reducing excess title-bar button whitespace.
- [x] Task / Red: native screenshot showed the 44 × 44 px button disproportionately larger than adjacent title-bar actions.
- [x] Task / Green: reduce the button to 32 × 32 px; preserve its 26 px glyph, centered alignment, pointer cursor and focus indication.
- [x] Verify: 4 existing MCP UI contracts passed, optimized build succeeded and native Settings menu opened/closed correctly; screenshot confirms reduced button whitespace.
- [x] Complete: previous owned app stopped, updated runtime reopened with the existing workspace/plan preserved; correction recorded in shared memory.

### Enterprise allocation correction — DD-83

- [x] Plan: R-75 → BG-30 → SF-62 → UC-65 → FR-60/NFR-31 → QAS-70 → DD-83.
- [x] Task / Red: installed Codex protocol budget-only Enterprise response drops individualLimit/credits; regression fails.
- [x] Task / Green: explicit budgets, scope, remaining/allocated and conservative unknown totals.
- [x] Verify / macOS: 323 Rust tests passed, 0 failed, 13 ignored; 32 Python tests passed; fmt/Clippy and optimized build passed. Metadata-only subprocess fixture returns Enterprise allocation 640/1000; native ordinary-account refresh remains functional.
- [ ] Windows Enterprise acceptance: actual Codex/Claude errors and CLI metadata supplied by the operator.
- [ ] Complete: confirmed Codex budget correction is implemented and running locally; both actual Windows Enterprise accounts, Claude Enterprise output and platform CI remain pending.


### Native window behavior correction — DD-84 / v0.1.4

- [x] Plan: R-76 → BG-31 → SF-63 → UC-66 → FR-61/NFR-32 → QAS-71 → DD-84.
- [x] Task / Red: picker is owned by an external script; Windows GUI subsystem and hidden background flags absent.
- [x] Task / Green: main-thread NSOpenPanel, Windows GUI subsystem, console-free background commands and retained CLI parent console.
- [x] Verify / macOS: 328 Rust tests passed, 0 failed, 14 ignored; 32 Python tests; fmt/Clippy and release build passed. Native Browse opens the foreground panel, Cancel preserves fields, selecting the existing repository returns the correct path; saved workspace preserved.
- [x] Verify / Windows: CI 38189018437 passed 143 native tests (0 failed, 1 ignored), including PE GUI subsystem, actual console-free child/pipe fixture, picker owner script and native metadata checks; 32 Python tests, fmt/Clippy passed.
- [x] Verify / packaging: CI 38189018437, all four jobs passed; exact macOS/Windows archives executed shared/worktree parallel workflows, tests, unapproved merge refusal and approved merge; archive/binary SHA-256, source manifest, license notices and SBOM verified.
- [x] Complete: [stable v0.1.4](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.4) published from 0b006b2ac2f1ee4c47a16e37621ca30301b0773f; both native archives and SHA256SUMS uploaded with server digests verified. Previous owned runtime stopped; exact CI macOS binary reopened at /private/tmp/AgentConductor-v0.1.4.app; existing workspace and reviewed plan preserved.
- [ ] Operator acceptance: interactive Windows desktop Browse z-order on the operator machine (CI validates the native owner bootstrap and console behavior).


### New workspace save correction — DD-85

- [x] Plan: R-77 → BG-32 → SF-64 → UC-67 → FR-62/NFR-33 → QAS-72 → DD-85.
- [x] Task / Red: saving a plain directory reproduces git rev-parse not-a-repository; unborn planning fails HEAD.
- [x] Task / Green: validate directory at save; validate baseline before planning artifacts; retain independent quota collection.
- [x] Verify / macOS: 331 Rust tests passed, 0 failed, 14 ignored; 34 Python tests passed, including actual UI cooldown scheduling. Fresh non-Git save/reopen, invalid directory preservation, unborn baseline rejection without artifacts, and quota subprocess without Git/run pass.
- [x] Complete: stable v0.1.5 published; reviewed source 3ce28b0, exact CI macOS runtime reopened, existing workspace/plan preserved; shared memory recorded.
- [x] Product flow: initialization prototype superseded by ordinary folders under DD-88.
- [x] Product correction: ordinary-folder execution is active under DD-88; private history uses Git internally.
- [ ] Reported quota acceptance: operator OS and exact error pending; local current paid-account refresh works.

- [x] Task / Red (quota): paused provider fixture reproduces a shared planning slot and blocked Save.
- [x] Task / Green (quota): separate metadata broker and quota_job ledger; refresh scheduler waits for its slot without skipping the next provider for five minutes.
- [x] Verify (quota): Save succeeds while provider is paused; metadata has no planning job; duplicate refresh is refused; independent fresh-folder subprocess and actual UI scheduling/cooldown tests pass.


### Nonblocking team configuration — DD-86 / v0.1.5

- [x] Plan: R-78 → BG-33 → SF-65 → UC-68 → FR-63/NFR-34 → QAS-73 → DD-86.
- [x] Task / Red: paused model query occupies planning slot; metadata scheduling changes the revision and blocks registration.
- [x] Task / Green: bounded one-worker FIFO, isolated model snapshots, matching-profile catalog application, stable edit revision and immediate guarded registration dialog.
- [x] Verify / targeted: catalog pause/queue/registration/stale-result fixtures and 37 actual JavaScript/Python tests passed; native Models query permits opening/canceling registration and preserves the reviewed plan. Full platform/package verification remains below.
- [x] Complete: exact CI native restart, English manual/README, commits pushed, stable v0.1.5 macOS/Windows binaries and server-verified checksums; shared memory recorded.
- [x] Coordination: agentmemory signal to Claude requesting remote-work registration; Codex retains ownership of metadata/UI/release. Claude confirmed this request meant the connected claude rc session; the registration item is closed.


### Superseded Git initialization prototype — DD-87

- [x] Plan: R-79 → BG-34 → SF-66 → UC-69 → FR-64/NFR-35 → QAS-74 → DD-87.
- [x] Task / Red: initialization request absent; plain-project planning only fails a prerequisite.
- [x] Task / Green: dedicated consent, empty tree/commit, compare-and-set HEAD, untouched index and local audit; separate file-commit consent remains.
- [ ] Verify: false approval, replay, staged/untracked preservation, actual JavaScript continuation and macOS/Windows full regression.
- [x] Closed as superseded: user selected ordinary-folder operation under DD-88. Mandatory initialization and its unreleased UI/API were removed; this prototype is not shipped.


### Ordinary project folders — DD-88 / v0.1.5

- [x] Plan: R-80 -> BG-35 -> SF-67 -> UC-70 -> FR-65/NFR-36 -> QAS-75 -> DD-88.
- [x] Task / Red: folder workflow module absent; existing planning requires Git root/HEAD.
- [x] Task / Green: private bounded folder history, immutable origin fingerprints, plan/run binding, original-change refusal and approved file application.
- [x] Verify / local: real planning subprocess, native startup request, three-worker shared/worktree execution, unapproved refusal, original-edit preservation, exclusions/aliases and modification/deletion backups pass. Full macOS regression: 339 passed, 0 failed, 14 ignored; subsequent five folder tests include the new backup case. 37 Python tests, fmt and Clippy pass.
- [x] Verify / platforms: CI 38199934220 passed macOS 342 tests (14 ignored), Windows 157 (1 ignored), 0 failures; Python 37, fmt/Clippy and four-case Git/folder × shared/worktree smoke passed for each exact package.
- [x] Complete: native interactions, English README/manual and wiki updated, source pushed; stable macOS/Windows v0.1.5 released from 3ce28b08a832c5d1aead4523fb9c04314119ca24; exact CI macOS app running.
- [x] Scope correction: Git initialization is optional. DD-87 is not the mandatory fresh-folder flow; DD-88 supersedes that prerequisite.

DD-86 live verification follow-up (passed): actual native Models refresh invalidated the quota cache because catalog observations were hashed as account identity. Add a failing retained-capacity test; exclude catalog metadata from the quota fingerprint while executable/auth settings remain binding.

DD-88 Windows CI correction: CI 38196650126 passed macOS but Windows folder workflow failed when Git retention ref paths exceeded MAX_PATH. Enable core.longpaths per Git invocation, preserving global user configuration; rerun the same real workflow regression before packaging.

### Claude pre-release review — DD-86 / DD-88

- [x] Plan: QAS-76 resilience amendment to R-78/R-80, preserving NFR-05 resource constraints.
- [x] Task / Red: invalid catalog poisons Snapshot; unstarted replaced copies remain; completed working files remain.
- [x] Task / Green: durable failed metadata observations, continued Save/retry, superseded/orphan pruning, completed working-file compaction with retained Git objects, candidate-stage cleanup and rollback diagnostics.
- [x] Verify / local: restart/retry, queue/registration continuity, evidence/knowledge access after compaction and source layout pass; full Rust regression 342 passed, 0 failed, 14 ignored, plus 37 Python tests and fmt/Clippy.
- [x] Verify / platforms: reviewed-source CI 38199934220 all four jobs passed; 342 macOS and 157 Windows tests, 0 failures; four-case exact package smoke passed on both.
- [x] Complete: reviewed source 3ce28b0, manual/README shipped in archives, stable v0.1.5 published with verified server digests and native restart.

Prior source d9e4a38 passed all four jobs in CI 38197252244, including Windows long-path regression and four-case package smoke. Review hardening requires a new final-source CI before release.

### Stable v0.1.5 delivery evidence

Source: 3ce28b08a832c5d1aead4523fb9c04314119ca24. CI: 38199934220, all four jobs passed. Release: https://github.com/neodamul/AgentConductor/releases/tag/v0.1.5 (draft=false, prerelease=false). macOS archive SHA-256: 9abc807a519704a0a2834a848007c6b021c89004506891f0814b023e0d394d9e. Windows archive SHA-256: beab75cb6c894c52c51a7b3d94509ecbc79bfd075b7af39c46f3ac72f5933003. SBOM includes 42 macOS / 108 Windows dependency records with license texts/notices; no new dependency introduced. Packaged README/manual/license contents match source, allowing Windows CRLF checkout. Exact CI app is running at /private/tmp/AgentConductor-v0.1.5.app; existing health workspace and reviewed plan retained. Native quota shows actual authenticated Claude/Codex results; actual Windows Enterprise operator acceptance remains tracked separately.


### Agent tools guide — DD-89

- [x] Plan: R-81 -> BG-36 -> SF-68 -> UC-71 -> FR-66/NFR-37 -> QAS-77 -> DD-89.
- [x] Task / Red: tests/test_agent_tools_guide.py failed with no guide dialog or button.
- [x] Task / Green: Settings button, static dialog, Korean strings and manual section 1.2e.
- [x] Verify / local: 340 Rust passed, 0 failed, 14 ignored; 41 Python tests, fmt and Clippy pass; English and Korean dialog rendered in headless Chrome with one command per line.
- [ ] Verify / platforms: native app dialog check and macOS/Windows CI after merge.
- [ ] Complete: merge after the v0.1.5 release, changelog entry, shared memory.


### Windows picker path — DD-90

- [x] Plan: R-82 -> BG-37 -> SF-69 -> UC-72 -> FR-67/NFR-38 -> QAS-78 -> DD-90.
- [x] Task / Red: tests/picker_output.rs failed (no parsing function); Windows Korean round trip added to tests/windows_console.rs.
- [x] Task / Green: UTF-8 without BOM in the owner script; BOM-safe picked().
- [ ] Verify: Windows CI round trip and user confirmation on the reporting Windows machine.


### Planning start and activity — DD-91

- [x] Plan: R-83 -> BG-38 -> SF-70 -> UC-73 -> FR-68/NFR-39 -> QAS-79 -> DD-91.
- [x] Task / Red: tests/test_planning_feedback.py and the job.output assertion in tests/planning_progress.rs failed.
- [x] Task / Green: 300 s planning-entry limit, snapshot output activity, waiting/active/stale text, reduced-motion blink.
- [ ] Verify: full regression, macOS/Windows CI and user confirmation on Windows.


### English slugs — DD-92

- [x] Plan: R-84 -> BG-39 -> SF-71 -> UC-74 -> FR-69/NFR-40 -> QAS-80 -> DD-92.
- [x] Task / Red: Codex's tests (tests/slugs.rs, tests/startup_folder_plan.rs with AC_PLAN_SLUG mock) failed on romanization.
- [x] Task / Green: date placeholder for non-English goals, prompt slug, slug.txt, settle rename.
- [ ] Verify: full regression, CI, real coordinator naming on the user's goal.


### Enterprise percentage — DD-93

- [x] Plan: R-85 -> BG-40 -> SF-72 -> UC-75 -> FR-70/NFR-41 -> QAS-81 -> DD-93.
- [x] Task / Red: Codex's tests/test_quota_badge.py expectations for 64%/100% failed.
- [x] Task / Green: percentage amount in both quotaBadge copies.
- [ ] Verify: real Enterprise account; Claude Enterprise usage on Windows still needs the actual /usage output or error to diagnose.


### Windows Claude capacity — DD-94 / v0.1.7

- [x] Plan: R-86 -> BG-41 -> SF-73 -> UC-76 -> FR-71/NFR-42 -> QAS-82 -> DD-94.
- [x] Task / Red: claude_blocker missing; Windows verbatim-folder and first-run fixture cases added.
- [x] Task / Green: git_path working folder, shared claude_blocker on macOS and Windows.
- [ ] Verify: Windows CI, then the user's Windows refresh after completing Claude CLI first-run setup.

- [x] DD-91 amendment: snapshot poll waits up to 300 s while an action is in flight (Red: tests/test_planning_feedback.py::test_snapshot_waits_for_a_running_action).


### Windows workspace corrections / v0.1.8 — DD-95–DD-98
- [x] Plan: R-87–R-90 -> BG-42–BG-45 -> SF-74–SF-77 -> UC-77–UC-80 -> FR-72–FR-75 -> QAS-83–QAS-86 -> DD-95–DD-98.
- [x] Task / Red: C:\ root became C:, delete_workspace was unknown, asynchronous creation helper and corrected MCP labels were absent.
- [x] Task / Green: display-normalized paths, reliable creation, safe archived deletion, generic MCP registration.
- [x] Verify / local: 346 passed, 0 failed, 14 ignored (includes one source-layout check); final focused registry/project/picker/layout tests pass; Python 47, fmt and Clippy pass.
- [x] Verify / native macOS: actual Enter creation, original-workspace return, exact-name/path deletion confirmation, recovery evidence and MCP registration screen verified; existing health goal/reviewed plan preserved.
- [x] Verify / delivery: final source c94bea8; CI 38215024341 passes all four jobs. macOS 348/0/14 and Windows 169/0/1 passed/failed/ignored, plus source-layout checks; Python 48 per platform; four exact-package workflow cases per platform pass.
- [x] Complete: English README/manual/wiki shipped; stable v0.1.8 published with macOS/Windows archives and server-verified SHA256SUMS. Exact CI macOS binary reopened the original workspace; reviewed plan retained.

DD-96 native verification correction: actual Create was refused by historical terminal-success startup records whose PIDs were reused by an unrelated system process (29654, mediaanalysisd). Do not kill that process. Successful reaped startup records no longer gate switching; running/failed/unknown records retain quiescence checks. Add a regression with the current live PID as historical terminal evidence. Keep the creation dialog and title visible until the native command succeeds; report failures inside it and explicitly wire confirmation actions.

### Profile-save correction — DD-99 / v0.1.8 amendment
- [x] Plan: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
- [x] Task / Red: awaited save helper and visible in-dialog error contract absent.
- [x] Task / Green: await persistence, close on success, retain errors and guard duplicate/busy submissions.
- [x] Verify / local: Python 48 passed; persisted-profile, concurrent metadata registration, source layout and Clippy pass. Native Claude -> Codex fallback saves and closes; cyclic Codex -> Claude policy stays open with inline error. Original health workspace unchanged.
- [x] Verify / platforms: final-source CI 38215024341 and both native packages pass; all delivered assets pin c94bea8.
- [x] Complete: profile save correction ships in stable v0.1.8; superseded e31b010/e71e889 packages were not published.


### Merged Windows quota correction — PR #1 / v0.1.8
- [x] Plan: clarify R-86, UC-76/UC-16 and DD-94/DD-26 for existing quota behavior.
- [x] Task: merge 466169b; preserve project preflight, independent model windows and precise Codex authentication diagnostics.
- [x] Verify / focused: Claude usage and source layout pass; extend actual badge-function fixtures for named windows and prerequisite tooltips.
- [x] Verify / delivery: CI 38215024341 passes validation and packaging on both platforms; actual macOS Claude /usage test passes with zero generation and reaped ownership; native fresh Claude 100%/91% and Codex 48%/49% observed.
- [x] Complete: updated manual/notes included in stable v0.1.8; prior owned QA app stopped, exact CI native app restarted, original health workspace and plan digest 40f0fb859aa7fce1d998fbf24f357928b4789e711b2b77c27012c3d75490cfee preserved. Shared memory handoff recorded.

Delivery evidence: [v0.1.8](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.8), [CI 38215024341](https://github.com/neodamul/AgentConductor/actions/runs/38215024341), source c94bea8b4fb4b2aa37031c3e07089073bd47eaa9. macOS archive SHA256 bad200ddb1ddaf8e73c9ee146c8b34dc6a538644982daa20411d2290c80167c0; Windows archive SHA256 75fc33222f10961e47291a8eaf7118b95c1e509dbbe2701cd351e57508f928db. MIT root license, pinned dependency license texts/notices and SBOM included (42 macOS / 108 Windows components); no new runtime dependency. Live Windows GUI/provider authentication acceptance remains separate from CI and the operator-supplied Windows usage fixture.


### Session communication knowledge asset — DD-08 / DD-29 clarification
- [x] Plan: distinguish CLI interfaces, communication, authoritative state and reviewed knowledge; retain optional agentmemory.
- [x] Task: source observation, maintained wiki/index/log, design clarification and public README explanation.
- [x] Verify: public-docs contract tests 3 passed; UTF-8, local source hashes, README/wiki links and diff checks pass.
- [x] Complete: source-linked knowledge asset and public README documented; shared agentmemory record saved; documentation delivery is separate from pending runtime implementation. v0.1.9 remains owned by the other Claude machine.

The published v0.1.10 predates the selectable adapter; its post-release implementation and acceptance are tracked below.


### Selectable signal transport with Rust fallback — DD-100
- [x] Plan: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100; requirements and acceptance criteria recorded.
- [x] Task / Red: tests/signal_transport.rs (default/persisted preference, Local never contacts a healthy server, healthy publish once and idempotent repeat, refused server falls back with cause and the same pollable directive, silent server settles locally within 10 s, briefing route) and tests/test_signal_settings_ui.py failed: no route module or Settings controls. Stale recipient, expiry, restart and duplicate/late acknowledgment were already covered by tests/cooperative.rs and tests/cooperative/memory.rs (exact-ID, idempotent acknowledge); they apply unchanged because the directive identity never changes on fallback.
- [x] Task / Green: cooperative::route (preference file per workspace, record-once signal_route per directive, one external operation per workspace via kernel lease and durable pre-dispatch attempt claim, existing 8 s memory client), checkpoint-issue routing, briefing signal_route, Settings transport/port/last route with Korean strings.
- [x] Verify / local regression: 368 Rust tests passed, 14 external/native-account tests ignored; 51 Python contracts passed; formatting and warning-free Clippy passed. Restart uncertainty and competing-owner regressions pass; original cooperative identity/approval tests remain green.
- [x] Verify / native Settings: isolated macOS workspace selected agentmemory, saved, closed and reopened the dialog; exact preference persisted. QA app stopped and the published v0.1.10 app restored.
- [x] Verify / platforms: [CI 38245260274](https://github.com/neodamul/AgentConductor/actions/runs/38245260274), source 3b0c1f690ca939afe56576de887aae42e16987b5, passed macOS/Windows tests, formatting, Clippy and exact-binary package smoke. Windows logs confirm all eight signal transport tests passed, including restart uncertainty and competing-owner cases.
- [ ] Verify / operator: actual company cooperative CLI recipient, Windows desktop Settings and remote-route absence acceptance.
- [ ] Complete: operator manual/UI documentation, runtime/wiki evidence and reviewed release.

The adapter is implemented on main after v0.1.10. It is not included in that published binary; remaining real-recipient/native acceptance stays unchecked. Review added restart-uncertainty and competing-owner tests; a transport receipt is not an execution acknowledgment.


### Compact app version — DD-101
- [x] Plan: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101.
- [x] Task / Red: generated document had no build-version label (zero matching labels).
- [x] Task / Green: both titlebars use a compile-time version placeholder and compact shared styling.
- [x] Verify / local: titlebar version, Room UI and source layout pass.
- [x] Complete: native runtime verification, shared memory and stable v0.1.10 delivery.

### Disposable cache cleanup — DD-102
- [x] Plan: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102.
- [x] Task / Red/Green: bounded lifecycle pruning and confirmed Settings cache preview/cleanup.
- [x] Verify: repeated refreshes, active/referenced/unknown protection, aliases, changed previews and platform contracts; native macOS cache preview observed, permanent removal verified with disposable fixtures.
- [x] Complete: manual/wiki, memory and stable v0.1.10 delivery.

### Abandon blocked planning workspace — DD-103
- [x] Plan: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103.
- [x] Task / Red/Green: stop-marker planning cancellation and same-window current-workspace deletion.
- [x] Verify: blocked/failed planning, owned quiescence, metadata settlement, refusal retention and project protection; native macOS stop-and-delete completed.
- [x] Complete: manual/wiki, native macOS UI, memory and stable v0.1.10 delivery.

Verification checkpoint (2026-10-12, integrated v0.1.10 source): 360 Rust tests passed, 14 external/native-account tests ignored; 48 Python tests passed; formatting and warning-free Clippy passed. Native macOS observed the version label and read-only cache preview. An independent waiting CLI was stopped through the current-workspace Delete action, its snapshot removed and app data archived, and a fresh workspace opened in the same window; project sources were preserved. At that checkpoint platform CI/release was pending; completed delivery evidence follows. Windows desktop interaction remains operator acceptance.

Delivery: [stable v0.1.10](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.10), source e445e2e221449e88f34a57346ba07025ecf0cabe; [Release CI 38233701284](https://github.com/neodamul/AgentConductor/actions/runs/38233701284) passed both platform validation/package smoke and publication. Downloaded packages passed CRC, binary/archive SHA256, exact source/manual/README, bundled icons, MIT root license, dependency notices/texts and SBOM checks (42 macOS / 108 Windows components). macOS ZIP SHA256 92c35f88b56797cc1138e6c3ff7b0602f8bcded68e99701f880b9c6aaa3b4c02; Windows ZIP SHA256 2ed98ee89a23a9d66bad4209e3d720a8b255c29dfbca74fdf07b85d9b8c1952f. User-authorized removal of 13 Releases strictly below v0.1.8 completed after this audit; all Git tags/history retained. Native Windows desktop interaction remains operator acceptance.

### Signal transport review correction — post-v0.1.10
- [x] Plan: review Claude 196e55c against restart/idempotency and bounded-operation requirements.
- [x] Task / Red: simulated dispatch-before-receipt crash republished the directive.
- [x] Task / Green: durable pre-dispatch claim plus kernel-owned workspace lease; no age-based lock stealing.
- [x] Verify: local full regression, isolated native macOS Settings persistence and macOS/Windows CI/package smoke completed.
- [x] Complete: implementation pushed as 3b0c1f6; source-linked wiki and progress evidence updated; shared memory/Claude handoff. Published v0.1.10 tag remains e445e2e; this adapter and correction are source for the subsequent release. Real company recipient acceptance remains separately pending.

### Delivery closure — 2026-10-13 (user decision)

- [x] Complete: v0.1.3 is the delivered binary (macOS aarch64 and Windows x86_64, CI-verified, SHA256SUMS).
- [ ] Follow-up (operator environment): open the Windows package on a real Windows PC.
- [ ] Follow-up (operator account): send a test message through a real Slack or company webhook and answer one tool request from the messenger.

### Agents work inside the workspace sandbox; the coordinator decides beyond it — DD-104
- [x] Plan: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Usage query accepts the Claude folder-trust prompt — DD-105
- [x] Plan: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Planning start and workspace naming feedback — DD-106
- [x] Plan: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Run and task status at a glance — DD-107
- [x] Plan: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Goal folders directly in the project folder — DD-108
- [x] Plan: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Agent CLI list is a master list — DD-109
- [x] Plan: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Reorder agent cards — DD-110
- [x] Plan: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Readable file paths everywhere — DD-111
- [x] Plan: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Activity tab reads like the CLI — DD-112
- [x] Plan: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Coordinator final step in the task list — DD-113
- [x] Plan: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Decide a live tool request from its window — DD-114
- [x] Plan: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Coordinator decides routine requests and records each decision — DD-115
- [x] Plan: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Task list without account usage — DD-116
- [x] Plan: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### The coordinator is selectable and shows its work; one name — DD-117
- [x] Plan: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Failure cause and the next action on a failed run — DD-118
- [x] Plan: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### The next step is always shown — DD-119
- [x] Plan: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Plan review and run completion are organised by phase — DD-121
- [x] Plan: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### A blocking verification finding fails integration verification — DD-120
- [x] Plan: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### The run's phase beside the workspace name — DD-122
- [x] Plan: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Readable Agent Room text and agent icons sized to their names — DD-123
- [x] Plan: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Task cards in Review the plan fold to one line — DD-124
- [x] Plan: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Review the whole phased plan up front — DD-125
- [x] Plan: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125.
- [x] Task / Red/Green: implementation and regression tests.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

### Operator refinements — 2026-10-13 (R-112, R-116)
- [x] Goals & progress: each phase carries a tag (Completed, Next/In verification, Planned) and the next phase is bold.
- [x] Review the plan heading wraps instead of breaking Korean text into single characters; phase numbers read n/m.
- [x] Fold arrows for tasks and phases are 22 px round controls instead of a small triangle.
- [x] Opening a workspace routes to the work (DD-126, operator request 2026-10-13): an unfinished run opens the Agent Room; a plan ready for review, or planning in progress, scrolls to Review the plan. The startup snapshot carries the last run state.

### A run survives closing or losing the desktop app — DD-127
- [x] Plan: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Interrupted runs recover and resume automatically — DD-128
- [x] Plan: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Interruptions and silent workers are visible — DD-129
- [x] Plan: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.
- [ ] Activity: consecutive thinking_tokens events collapse into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows.
- [ ] Running task with no output for 5+ minutes shows a No output for n minutes warning; a task whose process is gone shows Not responding — Recover.

### Reassign tasks that have not started while the run is working — DD-130
- [x] Plan: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130.
- [ ] Task / Red/Green: implementation and regression tests.
- [ ] Verify: Windows local tests and native run.
- [ ] Complete: release.

### Agents run with the highest permission by default — DD-131
- [x] Plan: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131.
- [x] Task / Red/Green: full_access templates, derive_permission, validate_permissions and validate_profiles, Codex full-access resume, read-only coordinator mapping.
- [ ] Verify: Windows local tests and a native run whose verifier opens the UI with Playwright.
- [ ] Complete: release.

### The coordinator resolves verification failures — DD-132
- [x] Plan: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132.
- [x] Task / Red/Green: workflow::repair wired after run/resume commands; review_as with the recorded coordinator; tests/repair.rs (Unix fixture).
- [ ] Verify: Windows local tests and the stuck ladder run resumed through Resume.
- [ ] Complete: release.

### Selected-folder execution without project Git metadata — DD-133
- [x] Plan: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133.
- [x] Task / Red/Green: external Git binding, recorded baseline, direct cwd, Accept/restore Reject and owned-file warning.
- [x] Verify: focused Windows regressions for external history, baseline restore and edit warnings; existing Git/private mode; installed Codex initial/resume CLI parsing; formatting, Clippy and 62 Python contracts.
- [ ] Native follow-up: Claude rebuild/relaunch and smoke-check direct outputs before removing six legacy snapshots. Ladder phases 1–4 are already completed under the previous flow (Claude signal, 2026-10-13).
- [ ] Complete: release.

### Deleting workspaces needs no replacement name — DD-133
- [x] Plan: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133.
- [x] Task / Red/Green: delete_workspace opens the most recently used other workspace; dialog name field removed; tests/workspace_registry.rs deletes until none is left.
- [x] Verify: Windows local tests.
- [ ] Complete: release.

## Bilingual release documentation — DD-134 / v0.1.16

- [x] Plan: pull the latest release source, inspect changed behavior and preserve concurrent Claude work.
- [x] Task / Red: reproduce language-dependent error assertions and missing current screenshot references.
- [x] Task: update both manuals and public README for setup tabs, direct folder execution, recovery, repair and permissions.
- [x] Task / Green: capture genuine English/Korean workspace, profile and completed-run screens; validate linked assets.
- [x] Verify: Rust 393 passed / 0 failed / 14 ignored; Python 63 passed; formatting, Clippy, native checks and package documentation coverage passed.
- [x] Complete: record verified native evidence, prepare the bilingual documentation archive and hand off for the user's v0.1.16 packaging. Commit/push identity is recorded in agentmemory.

### The app updates itself from GitHub releases — DD-135
- [x] Plan: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135.
- [x] Task / Red/Green: src/update.rs, src/ui/update.rs, title-bar icon and dialog, menu item, CLI update; tests/update.rs.
- [x] Verify: Windows self-replacement of a running 0.1.15 build with the published v0.1.16 archive; relaunch removes the leftover.
- [ ] Complete: release; macOS bundle replacement verified on a Mac.


### DD-136 Quota resume coordinator (workflow / TDD)

Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

- [x] Plan: trace requirements, durable coordinator design and cancellation boundaries.
- [x] Red: reset parsing; deadline/fallback boundaries; persistence, reassignment, stop and duplicate watcher tests.
- [x] Task: implement only after recording failing tests.
- [x] Verify: focused Rust39, Python65, formatting, all-targets Clippy; independent native coordinator due execution, lease restart, same-account reassignment and Stop cancellation.
- [x] Complete: own-file commit, main integration/push and AgentMemory handoff.

## v0.1.17 manual and README reconciliation — DD-134 / DD-135 / DD-136

- [x] Plan: compare latest code from b6890f5 with both manuals and the public README; use the requested 0.1.17 documentation version.
- [x] Task: document update controls/checksums/idle gates/private-release limits, persistent availability reservations, five-hour fallback rules, full-diff review and Save-to-Team navigation.
- [x] Task: label existing native screenshots as 0.1.16 reference captures; do not fabricate a 0.1.17 screenshot or claim it is published.
- [x] Verify: all 3 public documentation checks and 19 release/packaging checks passed; README remains 119 lines.
- [x] Complete: verified documentation prepared for commit/push; handoff and resulting commit are recorded in agentmemory.

## Latest-source native screenshot refresh — DD-134

- [x] Plan: preserve the operator's reviewed plan and inspect the current source/package version before replacement.
- [x] Task: build the optimized source with two jobs, terminate the previous owned GUI and recapture four actual native screens.
- [x] Task: execute a fresh exact-binary three-worker fixture, verify overlapping tasks, passing tests and refused unapproved merge before recording the completed run.
- [x] Verify: 3 documentation and 19 package checks passed; screenshot fingerprints recorded; reviewed operator plan unchanged; one optimized GUI restored on its original workspace.
- [x] Complete: refreshed assets and verified runtime prepared for commit/push; resulting commit and handoff recorded in agentmemory.

### Selectable themes and session status tags — DD-137

- [x] Plan: R-128 → BG-83 → SF-115 → UC-118 → FR-113/NFR-49 → QAS-124 → DD-137.
- [x] Task/TDD Red: `tests/interface_theme.rs` (4) failed first; `room_task_rows` and `interface_language` updated to the amended decision.
- [x] Task/Green: 144 color literals replaced by 24 semantic tokens (light E / dark F), named-color and alpha-hex leftovers removed, `ok-bg` split from `accent-soft`, theme preference + `set_theme` + Settings select, session status on its own line, `docs/design-guide.md`.
- [x] Verify: full Rust and Python suites, clippy, fmt; native captures of workspace and Agent Room in light, dark and Korean; token pairs ≥ WCAG AA 4.5:1.
- [x] Complete: CHANGELOG (Unreleased), wiki, agentmemory, commit/push; manual text handed to Codex.

### Plans know whether the project uses Git, and verification commands that cannot work are replaced — DD-138
- [x] Plan: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138.
- [x] Task / Red/Green: planner Git line, external Git environment for integration tests, Revision.test_command; tests in tests/folder_workflow.rs.
- [x] Verify: Windows folder_workflow (git diff --check in a plain folder passes; coordinator replaces a failing command).
- [ ] Complete: release.

## Latest GitHub design and build reconciliation — DD-137 / DD-138

- [x] Plan: pull 0211f38, inspect the new design guide and distinguish published v0.1.17 from later source features.
- [x] Task / Red: native dark-theme QA and computed CSS contrast expose an unreadable notification (light contrast 1.21:1).
- [x] Task / Red/Green: disabled primary contrast failed at 1.91:1; existing soft/fill at full opacity now passes in both themes.
- [x] Task / Green: use existing bar/bar-ink and err-bg/err pairs; contrast tests pass for both themes without new tokens or dependencies.
- [x] Task: update README and English/Korean manuals for themes, task-first rows and Git-aware test-command repair.
- [x] Verify: inspect both screens in both themes and Korean; refresh native captures; run design/docs/package checks and confirm the original plan/preferences remain.
- [x] Complete: two verified change units committed/pushed; one optimized app restored on the operator workspace; memory handoff recorded.

## Compact account cards and step explanations — DD-139

- [x] Plan: R-130 → BG-85 → SF-117 → UC-120 → FR-115 → QAS-126 → DD-139.
- [x] Task / Red: exhausted quota test expected percentages only and failed on the appended reset time.
- [x] Task / Green: tooltip-only reset details, compact badge/refresh, reduced card spacing, consistent step introductions and available-width helper text.
- [x] Verify: quota/tabs/theme/docs regression; native light/dark visual checks and updated captures.
- [x] Complete: one updated app, commit/push and agentmemory/Claude handoff.

## Focused Settings and tool guide menu — DD-140

- [x] Plan: R-131 → BG-86 → SF-118 → UC-121 → FR-116 → QAS-127 → DD-140.
- [x] Task / Red: conditional port-visibility runtime test failed before the handler existed.
- [x] Task / Green: readable label/control spacing, conditional port field with retained value, guide moved to workspace menu.
- [x] Verify: Settings/guide regression, both-theme native interaction and document checks.
- [x] Complete: restart one updated app, commit/push and handoff.

### Custom CLIs that approve their own tools can run — DD-141
- [x] Plan: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141.
- [x] Task / Red/Green: operator_auto_approve mode, agent form checkbox, tests/self_approving_cli.rs.
- [x] Verify: Windows tests.
- [ ] Complete: release.

## Stable release v0.1.18

- [x] Plan: include DD-137–DD-141 and validated UI changes; next stable version after v0.1.17.
- [x] Task: repair stale planning-progress class selector using the stable review-panel ID; update package version, changelog and bilingual manuals.
- [ ] Verify: full native/platform regressions, package smoke and macOS/Windows CI.
- [ ] Complete: publish both verified binaries and SHA256SUMS; record release/handoff.

## Scrollable release notes — DD-142 (next release)

- [x] Plan: R-133 → BG-88 → SF-120 → UC-123 → FR-118 → QAS-129 → DD-142; preserve v0.1.18 tag.
- [x] Task / Red: actual-body rendering test failed before the panel existed.
- [x] Task / Green: reuse fetched body, literal text binding and focusable bounded-height scrolling.
- [x] Verify: updater status and UI runtime tests, source layout, fmt/clippy.
- [x] CI follow-up: next-source native review test allows 15 seconds for child approval/merge under runner load, preserving all approval assertions and printing the snapshot on timeout.
- [x] Complete: commit/push for the next version; published/tagged v0.1.18 is unchanged.

### DD-143 MCP workflow / TDD

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.

- [x] Plan: latest main/design guide, requirement trace and read/summary boundaries.
- [x] Red: native oversized response and absent auto-routing API; runtime missing coordinator/server scope cases.
- [x] Implement: server-scoped UI, schema/ID-valid coordinator reads, metadata helpers and full Markdown summaries.
- [x] Verify: native/runtime regressions, Windows portable theme4/language2 and layout checks; actual screenshot capture was unavailable (Computer Use native pipe missing), so no rendered-screen claim.
- [ ] Complete: own-file main integration/push, release verification and Claude signal.


### DD-144 Planning feedback workflow

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.

- [x] Plan / Red: preserve logs and prove the previous failure remains before retry.
- [x] Implement: synchronous reset, job-scoped progress, actual CLI icon, paid-fallback card control removal.
- [x] Verify: runtime job transitions, quota scheduling, translations and semantic-theme checks.
- [ ] Complete: own-file commit/main/push/release and Claude handoff.


### DD-145 Approval and API limit workflow

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.

- [x] Plan / Red: untyped API JSON becomes Unknown; unchanged plan remains dirty.
- [x] Implement: explicit restriction cache and normalized review snapshot comparison.
- [x] Verify: real offline CLI-to-durable availability/five-hour fallback, normalized approval guards, native regressions, formatting, Clippy and Python67.
- [ ] Complete: main/push/release and AgentMemory handoff.


### DD-146 / DD-147 Workflow completion

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

- [x] Plan: scoped transient notices and offline shared menu.
- [x] Red: recoverable events rejected, coordinator rejected first reconnect; manual menu absent; quiet review claimed approval lock.
- [x] Implement: narrowly matched warnings, localized safe manual and HTTP guide; read-only review skips availability lease.
- [ ] Verify / Complete: integrated platform tests, main push, stable package and Claude handoff.

### Workspace and Agent Room feedback refinements — DD-148
- [x] Plan: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148.
- [x] Task / Red/Green: tests/goal_sources_ui.rs, tests/started_phase_label.rs, tests/room_quota_selection.rs, tests/test_quota_badge.py.
- [x] Verify: Windows Rust/Python/Node UI tests; native screenshot of step 1; tag alignment checked in a browser with the real stylesheets.
- [ ] Complete: release.

### Runs keep going and stay visible — DD-149
- [x] Plan: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149.
- [x] Task / Red/Green: tests/transient_capacity.rs, folder_workflow stray test, room_task_rows, busy_button_runtime.cjs + test_busy_buttons.py, room_routing.rs, startup.rs.
- [x] Verify: Windows Rust/Python/Node tests.
- [ ] Complete: release.


### DD-150 / DD-151 Workflow

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog

- [x] Plan: signal settings govern instructions; replace the per-click Windows helper.
- [x] Red: actual planner/worker prompts lack selected backend, picker starts PowerShell.
- [x] Implement: host-owned routing contract, optional-tool wording and direct COM picker.
- [x] Verify: planner/worker preference regressions, local/remote-failure/idempotence transport checks, Unicode/native console tests, language/theme/docs/layout checks, Python70, fmt and all-target Clippy.
- [x] Complete: own commit/main/push, verified v0.1.21 release and AgentMemory/Claude handoff.


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 300-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

## v0.1.22 documentation and offline HTML — DD-153

- [x] Plan: R-144 → BG-99 → SF-131 → UC-134 → FR-129 → QAS-140 → DD-153; review tagged0.1.21 and new UI workflow.
- [x] Task / Red: HTML rendering regression fails before converter exists.
- [x] Task / Green: semantic generated English/Korean HTML, safe local images/links, lazy native loading and updated operator text.
- [x] Verify: native English/Korean tables, in-dialog locale links and Light/Dark checks; latest screenshots; local Rust448/Python74, fmt/Clippy; both platform CI validations and four packaged workflow modes passed.
- [x] Complete: source19d08999df4b committed/pushed; stable v0.1.22 published; wiki asset and agentmemory/Claude handoff recorded.

Release note: v0.1.21 was published while this documentation update was being verified. Preserve its tag and publish this patch as v0.1.22. Windows release validation previously failed at NO_HEADING; constrain automation to the launched process and retain rendered-control assertions.

Release evidence: [successful Action38549103869](https://github.com/neodamul/AgentConductor/actions/runs/38549103869) and [stable v0.1.22](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.22). Downloaded both published archives; verified archive SHA256SUMS and every internal file hash, source commit and English/Korean semantic HTML contents (platform newlines normalized for text comparison). Formerly failing Windows WebView2 UIAutomation saved the Unicode goal successfully. Existing operator workspace was reopened without running its prepared plan; original Korean/Light preference restored. Physical Windows manual interaction and automatic updater testing remain outside this documentation/release verification; updater testing was previously assigned to Claude.


### v0.1.21 verification and next-version request

Release: https://github.com/neodamul/AgentConductor/releases/tag/v0.1.21; frozen source a0bdb7b3424aff908608c77b579bb4931b0e4340. Native Windows and macOS Rust/Python71/formatting/Clippy validation and exact release-binary packaging passed: https://github.com/neodamul/AgentConductor/actions/runs/38545630595. The tag workflow first timed out finding the Windows UI Automation heading. Its rerun passed that UI test but separate PowerShell containment fixtures timed out. The identical-source main validation and both exact native package smoke jobs fully passed; their unchanged, source/binary/archive-hash-verified artifacts were published to the existing tag. No tests were skipped and no tag or source was rewritten. Both published ZIPs were downloaded and checked against SHA256SUMS and GitHub asset digests.

DD-152: adjacent workspace export/import, localized labels and the unchanged native request routing passed runtime Python and native room-routing/layout/language checks; main integration and v0.1.21 publication completed.

Next version (requested by the user): preserve direct planning-rule editing and allow a selected folder to supply rules by recursively reading its Markdown (.md) files. Keep this separate from the frozen v0.1.21 release. Implement with requirement-to-design trace, design-guide styling, localized message IDs and TDD; show the applied source files and deterministic ordering.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

- [x] Plan / Red: saved-folder cooldown does not reset; planning invocation has no schema contract.
- [x] Implement: path-aware Claude scheduling and native CLI structured response flags.
- [x] Verify: 16 targeted Rust regressions including the schema-bound phased planner subprocess, Python75 runtime/contracts, source bounds, formatting and all-target Clippy. No authenticated onsite Enterprise account was available; its response is not claimed as verified.
- [x] Complete: own-file commit/main integration/push; local app restart and AgentMemory/Claude handoff recorded in the task completion memory.

CLI references: https://code.claude.com/docs/en/cli-reference ; https://developers.openai.com/codex/cli/reference

## Latest-source manuals and reusable release skill — v0.1.23

- [x] Plan: source b329745; document R-145/DD-154 saved-project Claude quota scheduling and R-146/DD-155 schema-bound planning. Preserve pending planning-folder requirements as unimplemented.
- [x] Task: refresh public README and both Markdown/HTML manuals; add discoverable agentconductor-release skill and portable repository copy using existing packaging scripts.
- [x] Verify: Rust452/Python75, docs/renderer/layout, fmt/Clippy and skill validation passed; native EN/KO new text checked; both platform Release tests and four packaged workflow modes passed.
- [x] Complete: stable v0.1.23 source9f2da2a published; downloaded archive and all internal hashes plus four manual files checked against the frozen tag; skill registered and portable copy committed.

Release evidence: [Action38553599059](https://github.com/neodamul/AgentConductor/actions/runs/38553599059), [stable v0.1.23](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.23). Automatic updater and physical Windows Enterprise metadata were not tested in this documentation release. Native layout was unchanged, so genuine existing screenshots were retained.

## GitHub #4 — DD-156

- [x] Plan: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156; completed room vs active execution.
- [x] Red: completed Room creation failed with return-to-workspace message; running report remained attached.
- [x] Green: check actual run/task activity and Controller::busy; detach only after successful transition.
- [x] Verify: completed-room/active-run regression, workspace registry/navigation, room routing, language/source bounds, docs and all-target Clippy passed locally; platform CI pending.
- [x] Complete: issue4 commit0523939 pushed; macOS/Windows validation passed in Action38582625952; GitHub #4 closed with evidence.

## GitHub #5 — DD-157

- [x] Plan: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157; reproduce prompt readiness independently of authenticated provider availability.
- [x] Red: native PTY timed out on ❯; actual spent/allocated-only format failed parsing.
- [x] Green: shared prompt/trust readiness, exact allocation subtraction, stage-specific timeouts and no invented date-only timestamp.
- [x] Verify: native PTY trust-choice > prompt → Enterprise allocation fixture, three prompt variants, ownership cleanup, existing usage/budget/time tests, Rust26/Python75 and Clippy passed locally; Windows CI pending.
- [ ] Complete: issue-specific commit/push; onsite Enterprise evidence before claiming the original report resolved.

- [ ] Complete: issue-specific commit/push and GitHub issue result.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

- [x] Plan / Red: actual blocked planner cannot create another workspace; stale workspace/closed-room replies replace the current view.
- [x] Implement: parked active sessions, target-first ownership, serialized navigation, deletion protection and stale-response guards.
- [x] Verify: actual planner progress/completion through workspace creation and repeated switching; same job restoration, running-room reconnection, target lease refusal, active deletion protection and idle lease cleanup; 19 unique targeted Windows Rust tests, Python77 including late-response regressions, source bounds, formatting and all-target Clippy. No macOS/native screenshot acceptance is claimed.
- [x] Complete: own-file main commit/push; local app restart and AgentMemory/Claude handoff recorded in task completion memory.

## v0.1.24 verified native release

- [x] Plan: freeze latest main f9ad6d9, including R-147/DD-156, R-148/DD-157 and R-149/DD-158; preserve existing published tags and pending planning-rule folder design.
- [x] Task: synchronize Cargo version, README and English/Korean Markdown plus generated offline HTML with implemented active workspace navigation and Enterprise quota collection. Existing screenshots show the unchanged layout; no new UI layout is introduced by this release.
- [x] Verify: full locked Rust/Python/formatting/Clippy checks and both native exact-package smoke jobs; download published archives and verify raw archive/internal hashes, tagged source and bilingual manuals.
- [x] Complete: stable v0.1.24 Release Action/publication, separate completion record, main synchronization and AgentMemory/Claude handoff.

Authenticated onsite Enterprise behavior, physical macOS interaction and automatic updater installation are not claimed by fixture or CI validation.

Release evidence: [all five successful Release jobs](https://github.com/neodamul/AgentConductor/actions/runs/38592812353), [stable v0.1.24](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.24), and [downloaded artifact verification](evidence/v0.1.24-release.json). Frozen source aa939ba6fc111bac28331b389fb51199df0e7aab includes the incoming71145c5 deterministic MCP SSE fixture. Both published archives match archive/internal raw SHA256SUMS and GitHub asset digests; README, Cargo.lock, bilingual Markdown/HTML and seven screenshots per platform match the frozen tag. Text comparison alone normalizes platform line endings. Native package receipts verify shared/worktree × Git/folder, three tasks and tests per case, refusal before approval and verified approved integration. No test skip, gate bypass or tag rewrite.

Local Python77, formatting and all-target Clippy passed. Extra local all-target native validation initially exhausted disk space; Codex incremental build cache alone was removed after realpath containment checks. Its retry hit an existing mutual_fallback60s deadline under concurrent subprocess load; failure logs are retained, and the exact-source CI Windows/macOS full suites passed. The full retry finished with284 passed,1 timed-out and1 existing ignored test; the unchanged timed-out assertion then passed alone in33.20s, and the final-source MCP SSE test passed in0.42s. Do not retroactively describe the first full retry as all green. Claude received the verified release handoff; recursive planning-rule folder support remains queued for a later version.
## GitHub #6 — DD-159

- [x] Plan: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159; one folder owner, multiple independent goal contexts and preserved execution history.
- [x] Red: real phased fixture run/approval/merge completed; new_goal request was missing.
- [x] Green: new/select goal, optional goal-run links, imported-link clearing, nonterminal/busy protection, project ownership lease and visible native controls.
- [ ] Verify: real completed-run/goal regression, UI runtime, sharing/import compatibility, language/theme/layout and native screenshots.
- [ ] Complete: issue-specific commit/push, platform CI and GitHub resolution; patch release through the registered skill.

## Rust-only application and build migration — DD-160

- [x] Plan: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160; keep installed UI/runtime, release packaging, manual generation and owned verification in Rust; preserve external CLI integration.
- [x] Task 1: inventory the runtime and release path; select the Rust binary as the offline manual-render command without adding a server or dependency.
- [x] Red / Green: Rust tests first failed without a renderer; implemented safe Markdown tables, links, code blocks and offline PNG assets in `src/manual.rs`.
- [x] Verify Task 1: generated English/Korean HTML matches committed files exactly; sanitizer tests, source-layout check and all-target Clippy pass.
- [x] Task 1 cleanup: removed the superseded Python manual renderer and its three Python-only tests; Rust tests retain generation, escaping, local-image and escaped-table-pipe coverage.
- [x] Verify Task 1 cleanup: focused Rust renderer tests, source layout, formatting and all-target Clippy pass; the three legacy renderer contracts passed before removal.
- [x] Task 3 / release notes: Rust unit tests cover exact extraction, missing/empty sections, UTF-8 and the release workflow; removed `scripts/release_notes.py` and its Python test.
- [x] Verify release notes: CLI output matches the former script byte-for-byte for v0.1.24; focused tests, formatting and Clippy pass.
- [x] Task 3: moved macOS Finder bundle assembly to Rust, preserving the executable/icon layout, plist metadata, executable mode and overwrite protection; removed `scripts/macos_bundle.py`.
- [x] Task 3: moved ZIP archive creation to Rust; retain relative package paths, DEFLATE compression, executable metadata, DOS date bounds, symlink rejection and partial-file cleanup; removed `scripts/release_archive.py`.
- [x] Verify Task 3 locally: macOS bundle and archive tests, full Rust suite, remaining Python contracts, formatting, Clippy and license-text review for both native target dependency sets pass.
- [x] Task 3: moved packaged-binary manifest, SHA-256 and checksum-file validation to Rust before execution; removed the Python validator and its duplicate tests.
- [x] Verify package preflight locally: tests cover valid packages, corrupt hashes, unsafe names, malformed commits, checksum mismatches, symlinks and the hidden CLI response.
- [x] Task 3: moved packaged-workflow acceptance into the Rust CLI; verify shared/worktree execution on Git repositories and plain folders, parallel scoped tasks, tests, diffs and the approval-gated merge; removed the Python smoke runner.
- [x] Verify Rust smoke migration locally: red-before-command and green integration test; all Rust targets (478 passed, 14 ignored), release-mode smoke suite (6 passed), Python contracts (66 passed), formatting and Clippy pass.
- [x] Fix Windows ZIP timestamp fixture: CI on 90df88e showed epoch setup was Unix-only while the 1980 clamp assertion ran on every OS; set and verify epoch before archive creation on all platforms.
- [x] Verify ZIP archive fixture locally: all 3 archive integration tests pass on macOS.
- [x] Verify ZIP timestamp fixture on exact-source Windows CI: `Execute applicable native tests` passed on commit a4a5fe0 (run 38730449653); the full Windows job later failed Clippy on the Unix-only permissions binding, with package jobs skipped.
- [ ] Verify this acceptance path in exact-source macOS and Windows package CI.
- [ ] Verify Task 3: validate generated plist and Rust archive command in exact-source macOS and Windows package CI.
### Task 3.1 — Native package assembly in Rust

- [x] Plan: preserve exact target binary and source commit, dependency closure, SPDX allowlist, reviewed upstream license provenance/digests, bundled SQLite notice, referenced README assets, macOS bundle, SBOM, license notices and checksums; refuse overwrite and validate before output.
- [x] Red: Rust package contract tests first failed with the unimplemented assembler; coverage now includes both targets, upstream revision/version/digest, SQLite notice, unsupported licenses, missing files and staging cleanup.
- [x] Green: implemented the Rust package/license modules and `package-release` CLI; the native-package CI step now invokes the Rust binary without adding dependencies.
- [x] Verify core contracts: 6 Rust package tests; full Rust suite 484 passed / 14 ignored across 163 suites; all-target Clippy and formatting pass locally; authored Rust source/test files remain below 500 lines.
- [x] Complete source migration: remove the three Python release packagers and duplicate Python package test; update the registered release skill, platform workflow, and LLM wiki.
- [x] Fix Windows Clippy regression: cfg(unix) permission metadata now uses a shadow binding; macOS Clippy passes locally.
- [x] Verify the actual native package-release command on macOS commit 46c0e4e: exact commit and binary hashes, 58 dependency notices, ZIP checksum/CRC, and all four smoke modes (Git/folder × shared/worktree), including tests, diff review, approval refusal and approved merge.
- [x] Verify exact-source macOS/Windows validation and package artifacts in GitHub CI: run 38733217698 passed all four jobs (native validation and exact-package smoke on macOS and Windows) for commit 46c0e4e; downloaded ZIPs verify with `ARCHIVE-SHA256SUMS` (macOS `f562ef5e6ae7be8aee62284300f65c7ef1c26bfe39dcd4db73cc8db63cda0d20`, Windows `e3a6b964c3de6d9b10a15526ca069de7c08f384cbec737e8d77c26beef68de3b`).
- [x] Complete: both native runners built the clean-commit binary, rendered bilingual manuals, ran package-release and approval-gated workflow smoke, and retained package/evidence artifacts.

- [ ] Task 2: measure idle resources; port the JavaScript-driven desktop UI to Rust while preserving keyboard/accessibility, bilingual behavior and cross-platform screens.
- [x] Task 2.1 / TDD: move Jira issue-key and labeled Confluence page-reference suggestion parsing and explicit-reference span extraction from JavaScript to a bounded Rust parser exposed through the existing native request channel; add integration and UI runtime regression coverage.
- [x] Verify Task 2.1: final UTF-16 span-based UI passes the complete offline Rust all-target suite, 59 Python contract tests, MCP UI runtime, source-layout limit, formatting and all-target Clippy; 14 environment-dependent Rust tests remain ignored.
- [ ] Task 2 remaining: migrate the rest of the interactive UI behavior and remove project-owned JavaScript runtime logic while preserving one-window, low-resource native desktop behavior.
- [x] Task 2.2 / TDD: replaced duplicate JavaScript quota badge formatters with Rust-localized snapshot data for startup agent cards; retained compact Enterprise percentages, window sorting, reset details, and stale/unavailable output.
- [x] Verify Task 2.2: all-target Rust regression (499 passed, 14 ignored), all 44 Python contracts, focused Clippy, formatting and diff checks pass; the slow repository-wide source-layout test was filtered after blocking, and changed Rust files were checked directly for the 500-line/test-separation rules.
- [x] Task 2.3 / Plan: move per-agent quota refresh interval decisions into Rust snapshots, retaining the current 1–60 minute setting (5-minute fallback), async metadata queue, and immediate retry when Claude's project binding changes.
- [x] Task 2.3 / Red: Rust boundary tests first failed because the native refresh policy was absent; startup snapshot tests and source-contract tests cover both consumers.
- [x] Task 2.3 / Green: added one Rust interval policy and per-profile snapshot map for startup and Agent Room; removed the duplicate JavaScript due predicates.
- [x] Task 2.3 / Verify: Rust all-target regression (502 passed, 14 ignored, 1 slow source-layout test filtered), all 44 Python contracts, quota scheduling/project-change contracts, formatting, all-target Clippy and diff checks pass; cooldown advances only after an accepted startup refresh, and rejected refreshes remain retryable.
- [x] Task 2.4 / Plan: move quota window labels from Agent Room JavaScript into the existing Rust quota display layer and pass localized labels through the native snapshot; preserve provider order and bilingual text.
- [x] Task 2.4 / Red/Green: Rust tests first failed because the room label map was absent; added it to snapshots and removed the duplicate/unused JavaScript formatters.
- [x] Task 2.4 / Verify locally: Rust all-target regression 505 passed, 14 ignored, 1 slow source-layout scan filtered; all 43 Python contracts, focused native-room/quota tests, formatting, Clippy, diff and changed-file line-limit checks pass.
- [x] Task 2.4 / Platform CI diagnosis: exact-source Windows CI passed on `b5a5cd0`; macOS CI exposed a parallel test-fixture collision in `tests/macos_bundle.rs` (timestamp-only temporary directory names).
- [x] Task 2.4 / TDD correction: replace timestamp-only fixture naming with collision-safe process/counter IDs and add a distinct-directory regression; focused macOS bundle tests pass locally (3 passed).
- [x] Task 2.4 / Verify platform CI: run 38756510671 passed macOS and Windows validation; Windows native package smoke passed. Its queued macOS package job was superseded by the next source push.
- [ ] Task 2.4 / Native interaction/accessibility: gather platform evidence; non-quota restriction/availability date formatting remains JavaScript until its Rust UI port.
- [x] Task 2.5 / Plan: move Enterprise budget-row labels and percentage/amount selection into Rust quota snapshots; preserve stale, unlimited, bilingual, and reset data behavior.
- [x] Task 2.5 / Red/Green: added Rust tests before the snapshot API existed; implemented shared remaining/allocated percentage logic, per-agent row data, and removed the duplicate JavaScript calculations.
- [x] Task 2.5 / Verify locally: quota display tests (7), native room snapshot test (1), formatting, all-target Clippy, diff check, and changed Rust file limits pass.
- [x] Task 2.5 / Verify platform: the corrected Rust snapshot contract and budget UI passed exact-source macOS/Windows validation and both native-package smoke jobs in run 38763436037. Native display/accessibility evidence remains open under Task 2.4.
- [x] Task 2.6 / Plan: move Agent Room quota window and Enterprise budget reset-time formatting to Rust snapshots; preserve local timezone and English/Korean reset labels, and keep missing resets hidden.
- [x] Task 2.6 / Red/Green: add reset-label/localization/null and UI snapshot contract coverage first; the new test failed before `window_reset_labels` existed, then passed after Rust snapshot formatting replaced JavaScript `Date` formatting for quota rows.
- [x] Task 2.6 / Verify locally: quota display and native-room suites pass (9 tests); all-target Clippy, formatting and diff checks pass; changed code/test files remain below 500 lines.
- [x] Task 2.6 / Repair stale cross-platform assertion: replace the obsolete JavaScript-share test contract with the Rust snapshot contract; the quota display, room quota selection, and native-room suites pass (11 tests).
- [x] Task 2.6 / Verify platform: exact-source macOS/Windows validation and both package/execute smoke jobs passed in run 38763436037 for `0faefc1`; Rust-generated English/Korean manuals and exact release binaries were exercised.
- [ ] Task 2.6 / Native display/accessibility: inspect both platform UIs, reset labels, screen-reader names, keyboard navigation and Korean IME behavior.
- [x] Task 2.7 / Plan: add DD-161 native Rust UI prototype gate; evaluate minimal eframe/egui (MIT OR Apache-2.0, AccessKit) against existing Wry resource budget, keyboard, bilingual, accessibility, and platform requirements before migrating production screens. No dependency added before prototype evidence.
- [x] Task 2.7 / Prototype TDD: build an isolated eframe 0.37.0 native window with English/Korean text, a goal input, and agent-state controls; an AccessKit-tree test first exposed the missing goal-field name, then passed after adding an explicit accessible name.
- [x] Task 2.7 / macOS build: offline release build and accessibility test pass; removing bundled default fonts avoids their OFL/Ubuntu font license expression, and the system-font build is 7.7 MiB with 290 normal dependency-tree entries.
- [x] Task 2.7 / macOS idle resources: after 65 seconds, `footprint` reported 47 MB physical footprint and `ps` reported 0.0% CPU; launch peak was 156 MB. Follow NFR-05 and the existing WebKit baseline by comparing physical footprint, not RSS (shared framework pages inflate RSS).
- [x] Task 2.7 / macOS interaction: packaged native window exposed the named goal text field and checkbox in AccessKit; Tab moved focus to the checkbox, Space toggled it, and Korean text rendered in the goal field.
- [x] Task 2.7 / License TDD: package-provenance test first failed on unreviewed BSL-1.0; pin exact upstream BSL-1.0 and ISC texts by source commit and SHA-256, allow these permissive SPDX licenses, and verify both notices are assembled into the Windows package. License tests pass; this does not add eframe to production.
- [x] Task 2.7 / Repository probe TDD: promote the isolated AccessKit and `Room::dispatch` tests into an opt-in `native-ui-probe` feature and separate binary. The first build failed on the absent probe module, then exposed the eframe 0.37 `App::ui` API and test texture cleanup contract; after fixing those, both accessibility and direct-dispatch tests pass. Default app/package targets do not enable eframe.
- [x] Task 2.7 / macOS candidate CI: exact-source run 38777444017 for 64446d4 passed native tests, optional probe tests/build/Clippy, package and embedded-UI checks, formatting, and lint.
- [x] Task 2.7 / Windows candidate CI: exact-source run 38777444017 passed the native build, source layout, full native tests, optional probe tests/build/Clippy, package contracts, formatting and lint. No release dependency is enabled.
- [x] Task 2.7 / TDD fix for optional binary selection: exact package step failed on macOS because Cargo could not choose between the production CLI and the feature-gated probe (`cargo run --release --locked -- manual render`). Setting `default-run = "agentconductor"` made the same offline release command render both manuals successfully.
- [x] Task 2.7 / Verify fixed package workflow: exact-source run 38780554955 on 7a1b12f passed macOS and Windows validation, optional native UI tests/build/Clippy, Rust manual rendering, optimized release build, and execution of each platform's exact packaged binary. This verifies the default package excludes the optional eframe probe.
- [x] Task 2.7 / Current-main confirmation: run 38784456587 on 1639d1c also passed macOS and Windows validation and packaged-binary smoke checks; the candidate's source and release packaging are still limited to the Rust CLI with WebView-based UI.
- [x] Task 2.8 / Goal-save exact-source verification: run 38788589621 on `ed273bf` passed macOS and Windows native tests, optional eframe probe tests/build/Clippy, source-layout and packaging contracts, formatting, all-target Clippy, Rust manual generation, and both release-package execution smoke checks.
- [ ] Task 2.7 / Remaining gates: physical Korean IME composition is untested and the skeletal prototype does not yet meet the product's visual/responsive layout; candidate-package license/resource comparison remains unverified. The workflow's current default-app package jobs do not contain eframe.
- [ ] Task 2.7 / Decision: accept eframe only if every DD-161 gate passes; otherwise evaluate another native Rust UI implementation without keeping JavaScript/WebView as the final UI.
- [x] Task 2.8 / Integration architecture: inspected the production entry points and verified `Room::dispatch` is the Rust request boundary for workspace and Agent Room operations. A native shell can reuse `Room`, `Startup`, `Controller`, approval checks and leases directly, without adding a local server or retaining WebView IPC.
- [x] Task 2.8 / Dispatch TDD proof (isolated prototype only): the direct-dispatch snapshot test failed before `RequestWorker` existed, then passed with a bounded request channel and background `Room::dispatch` worker. This proves the narrow request/result path; it is not production integration or screen parity.
- [x] Task 2.8 / Snapshot-render TDD: added a regression test that saves a goal through `Room::dispatch`, requests the Rust snapshot, and verifies the native renderer paints that saved goal; removed the static Claude/Codex preview cards and renders registered profiles from snapshot data. Opt-in probe suite: 3 passed; Clippy with warnings denied and formatting pass. This remains a prototype renderer, not production-screen parity.
- [x] Task 2.8 / Goal-save TDD: the AccessKit contract failed before the accessible Save goal action existed; added a revision-bound request that preserves the rest of the config, sends save through the bounded background worker, then fetches a fresh snapshot only after `{ "saved": true }`. The probe keeps controls disabled while saving and shows errors. The async round-trip test verifies the persisted goal; probe suite 4 passed, Clippy and formatting pass. Production integration remains pending.
- [x] Task 2.8 / Workspace identity TDD: extended the opt-in native probe's form to edit workspace name and goal together; accessible field/action contracts and the `Room::dispatch` save-refresh round trip verify both values persist. Focused probe suite 4 passed. This is still an isolated prototype slice and does not replace the shipped WebView UI.
- [x] Task 2.8 / Repository path TDD: added the repository path field to the opt-in native workspace form and verified persistence through the async Rust dispatch path. The test asserts the canonicalized macOS path; focused probe suite 4 passed and focused Clippy is clean. This remains an opt-in prototype slice.
- [x] Task 2.8 / Native folder-picker TDD: added an accessible Browse control that opens the existing platform picker on the background worker; selection fills the repository field and cancellation preserves its prior value. Worker response and cancellation-state tests pass; focused suite 6 passed, targeted Clippy is clean, and both code/test files remain under 500 lines. The screen is still an opt-in probe, not the shipped UI.
- [x] Task 2.8 / Workspace isolation TDD: the new save contract first failed to compile because the selected isolation mode was not part of the request; added Shared folder and Git worktree radio controls, snapshot restoration and mode persistence through the existing Rust save handler. The accessible native probe suite passes all 7 tests and focused Clippy is clean; this remains an opt-in probe, not shipped UI parity.
- [x] Task 2.8 / Workspace isolation exact-source verification: run 38804235226 on `8683862` passed native validation and exact release-package execution on macOS and Windows, including the updated native probe tests. The implementation is still probe-only; production UI migration remains open.
- [x] Task 2.8 / Exact-source verification: run 38799761260 on `ec1339c` passed macOS and Windows native validation, optional probe tests/build/Clippy, repository contracts, formatting, all-target Clippy, Rust manual rendering, and both exact release-package smoke checks. The native workspace controls remain in an opt-in probe; production UI parity and migration are open.
- [ ] Task 2.8 / TDD: build the bounded native request/result handoff and state rendering; keep child-process/filesystem waits off the UI frame thread, and preserve Workspace plus Agent Room as separate windows/viewports. Add interaction and workflow parity tests before replacing either existing surface.
- [x] Task 2.9 / Plan: expose registered profile templates and coordinator selection in the native Workspace team step; preserve Rust-owned profile validation, private environment carry-over and the 16-profile limit. Verify unique IDs and profile/coordinator persistence through the existing save dispatcher before treating the native screen as editable.
- [x] Task 2.9 / Red/Green: added template ID-collision/capacity and coordinator persistence regressions first; compile failed while the profile helper and request fields were missing. Added template-based agent registration, a unique suffixed ID, the 16-profile cap, coordinator radio controls, and save/restore via the existing Rust dispatcher.
- [x] Task 2.9 / Verify: native probe suite 9 passed, including a real Room::dispatch save/snapshot round trip for Claude/Codex profiles and Codex coordinator; focused feature Clippy, rustfmt and diff checks pass. src/native_ui_probe.rs (439 lines) and tests/native_ui_probe.rs (302 lines) remain under 500 lines. Production UI migration is still open.
- [x] Task 2.9 / Cross-platform verify: exact-source run 38808098099 passed after a failed-job-only retry. macOS and Windows validation, native UI probe, formatting/Clippy, bilingual manual rendering, and both exact release-package smoke jobs passed; the first Windows UI Automation `NO_HEADING` did not reproduce.
- [x] Task 2.10 / Resource: remove the full workspace snapshot clone from every native UI frame by borrowing disjoint team state fields; preserve existing Team behavior and pending-state controls.
- [x] Task 2.10 / Verify: the existing native probe regressions (9 passed), feature Clippy and workspace `cargo fmt --check` pass; `src/native_ui_probe.rs` is 439 lines and its test remains 302 lines. Exact-source run 38812385807 passed macOS and Windows validation plus bilingual manual and exact release-package smoke jobs.
- [x] Task 2.11 / Plan → Red → Green: add a Rust-native Agent Room snapshot renderer for the run goal, run state, and each task's assigned agent/status; the regression first failed because only the workspace goal was rendered. Active tasks pulse green, approval/blocked states are amber, failures red, completed green, and pending/unknown gray. The view requests a 500 ms repaint only while work is running.
- [x] Task 2.11 / Verify locally: 11 native UI regressions pass, focused feature Clippy and rustfmt pass; `src/native_ui_probe.rs`, its Rust renderer module and the integration test are 434, 69 and 54 lines. The 500 ms blink test exposed that a continuous sine pulse could repeat the same color at repaint boundaries; use alternating half-second phases instead. This rendering slice remains in the opt-in prototype; actual production-window integration, cross-platform UI acceptance and exact-source CI remain open.
- [x] User workflow directive: keep normal push/PR CI triggers; verify each implementation unit locally and continue without waiting for hosted CI, which may run independently. Release still runs the reusable macOS/Windows validation.
- [x] Task 2.12 / Plan: render the actual `Controller::snapshot` envelope in the native Agent Room probe while retaining the startup/proposal snapshot and existing prototype fixtures; preserve the existing Rust snapshot schema.
- [x] Task 2.12 / Red: a regression using the real `report.state`, `report.plan` and `report.tasks` shape failed because the renderer only read top-level fields.
- [x] Task 2.12 / Green: read the nested report when present, with fallbacks for startup `run_state`, workspace proposal plan and existing top-level probe snapshots.
- [x] Task 2.12 / Verify locally: 12 native UI tests, all-target compile, source-layout (500-line/test separation), feature Clippy, rustfmt and diff checks pass. Graphify extraction of the touched renderer/test produced 10 nodes and 13 edges. CocoIndex is unavailable as a CLI or enabled tool here; no dependency was added.
- [x] Task 2.12 / Complete: the bounded renderer slice is complete, but production Agent Room connection, screen/workflow parity and native platform acceptance remain open.
- [x] Task 2.13 / Plan: add a probe-only launch path that opens a real run through `Room::open`, shows the real controller snapshot in an Agent Room surface, and keeps the existing workspace probe path intact. Do not launch WebView or construct a second Room/controller; retain standard Room recovery behavior.
- [x] Task 2.13 / Red: the real-run/accessibility regression did not compile because the probe had no Agent Room connection constructor.
- [x] Task 2.13 / Green: add `--room <run-dir>` to the existing opt-in probe binary; construct through `Room::open`, render only Room controls, and refresh snapshots asynchronously on the existing bounded worker. Active jobs refresh every second and idle runs every five seconds, matching the shipped Room cadence.
- [x] Task 2.13 / Verify locally: 15 native UI tests, native probe binary check, default all-target compile, source-layout (500-line/test separation), feature Clippy, rustfmt and diff checks pass. Touched source/test files remain under 500 lines.
- [x] Task 2.13 / Complete: the actual run controller is now reachable in the opt-in native probe without starting WebView or constructing another Room/controller. `Room::open` retains its standard recovery/auto-resume policy. Production UI replacement, full workflow/accessibility parity, native macOS/Windows UI acceptance and resource measurement remain open.
- [x] Task 2.14 / Plan: expose a confirmed, reasoned stop action in the native Agent Room probe using the existing `Controller::action`/`workflow::stop` boundary; preserve busy-state disabling and audit semantics.
- [x] Task 2.14 / Red: accessible stop/confirmation and revision-bound stop-request regressions failed because the native Room had no stop controls or request builder.
- [x] Task 2.14 / Green: show Stop only for stoppable states; require confirmation and a nonempty reason; send the revision-bound request through the existing worker; preserve the accepted message and refresh the controller snapshot.
- [x] Task 2.14 / Verify locally: the real SQLite-backed fixture transitions from `waiting_availability` to `stopped`; stop controls and reason field are accessible, terminal runs hide/clear a stale confirmation, and empty or >16 KiB reasons are rejected. All 17 native UI tests, default all-target compile, source-layout, focused feature Clippy, rustfmt and diff checks pass. All modified Rust source/test files are ≤500 lines (`native_ui_probe.rs` is exactly 500).
- [x] Task 2.14 / Complete: the stop interaction is implemented in the probe. Existing coordinator stop processing remains authoritative; an accepted request is not described as an instantaneous process kill. Production UI replacement, other Room actions, cross-platform acceptance and resource measurement remain open.
- [x] Task 2.15 / Plan: add a read-only candidate evidence view using `Controller::evidence`; show the exact validated diff, test outcomes and evidence revision, load asynchronously, and invalidate evidence when the run revision changes.
- [x] Task 2.15 / Red → Green: add an accessible, on-demand evidence action to the Room probe; collect existing `Controller::evidence` asynchronously, show the candidate, test outcomes/output and validated diff, and discard results from a stale run revision. A renderer/contract regression was introduced before the evidence renderer and reply path existed.
- [x] Task 2.15 / Verify locally: full default all-target suite passed (333 passed, 14 ignored); 21 focused Room/native UI/diff evidence tests passed; feature Clippy with warnings denied, formatting and source-layout pass. Touched Rust source/test files are below 500 lines. Remote push/PR CI is not awaited. Production UI migration remains incomplete and the probe is still opt-in.
- [x] Task 2.15 / Complete: the read-only evidence view is available in the native Room probe; production UI parity, app-window integration and macOS/Windows acceptance remain open.
- [x] Task 2.16 / Plan: advance the opt-in Rust Workspace probe toward the specified 1. Workspace settings → 2. Agent roles → 3. Review the plan flow (R-45/R-47 and R-151; SF-39/SF-40 and UC-17; DD-160/DD-161). Keep the shipped WebView path unchanged until workflow parity is verified.
- [x] Task 2.16 / Red: the revision-bound Prepare regression initially failed because the request builder was absent; step navigation also exposed an outdated AccessKit button contract.
- [x] Task 2.16 / Green: connect step navigation, the existing Rust plan-generation/progress/proposal-review path through `Room::dispatch`, team/coordinator saving, role-agent assignment and observed-model selection. Model choices come only from each registered profile's observed catalog; saved role rules pass through existing Rust validation.
- [x] Task 2.16 / Verify → Complete: 26 focused native UI tests, source-layout (500-line bound), feature Clippy with warnings denied, rustfmt, and `git diff --check` pass locally. Largest touched Rust file is `tests/native_ui_probe.rs` at 492 lines. Wiki and AgentMemory record the verified slice and its opt-in boundary. Production WebView replacement, cross-platform UI acceptance and resource measurement remain open.
- [x] Task 2.17 / Plan → Red → Green → Verify → Complete: expose registered agent choices and observed model catalogs in the Team step; save them in the existing RoleRules shape through the Rust workspace dispatcher. A save-contract test first failed while the role-aware path was missing; AccessKit verifies the role/model selector names. The generated planner already validates and applies these same rules. Included in Task 2.16's 26-test, Clippy, source-layout and formatting verification.
- [x] Task 2.18 / Plan (R-78, QAS-56/QAS-73, DD-86): bring per-profile quota badges and asynchronous model/quota refresh controls into the native Team step. Reuse the existing Rust metadata queue and authenticated CLI subprocesses; poll only while metadata work is active and keep refresh/actions independent per profile.
- [x] Task 2.18 / Red: the profile UI/request regressions first failed to compile because the native Team screen had no profile metadata controls.
- [x] Task 2.18 / Green: expose per-profile observed model counts, quota badges/tooltips, and accessible model/quota refresh actions; bind requests to the current snapshot revision and dispatch them through the bounded Rust worker. Refresh snapshots while metadata is active at the existing 1s cadence and use 5s while idle. Merge only matching observed catalogs so unsaved/new profile drafts survive.
- [x] Task 2.18 / Verify → Complete: 30 focused native UI tests, source-layout (500-line bound), feature Clippy with warnings denied, rustfmt, and diff checks pass locally. Largest touched Rust file is `tests/native_ui_probe.rs` at 492 lines. Wiki and AgentMemory updated. This is still opt-in probe behavior; custom-agent catalog editing, full profile editor, shipped UI replacement, and native platform/resource acceptance remain open.
- [x] Task 2.19 / Plan (R-67 → BG-25 → SF-55 → UC-55 → FR-51/NFR-25 → QAS-60 → DD-73): add native per-agent runtime preference editing for executable, JSON argv, input mode, session cap, and 1–60 minute quota cadence; reuse the existing native OS picker and save only the profile list so unrelated workspace drafts stay unsaved.
- [x] Task 2.19 / Red → Green → Verify → Complete: accessible editor/save/picker and persistence regressions were added first; cadence-only saves retain a ready proposal, executable changes clear stale observed catalogs and require review, and path picking does not launch a CLI. Focused native UI suites (33), quota preferences (2), source-layout, feature Clippy, formatting, diff checks and the 500-line bound pass locally. Wiki and AgentMemory updated. Provider switching and ID renames remain a separate R-68 slice.
- [x] Added the root `goal.md` with the requested delivery, architecture, TDD, documentation, tooling, memory, GitHub issue and release workflow.
- [x] Task 2.20 / Plan (R-68 → BG-26 → SF-56 → UC-56 → FR-52/NFR-26 → QAS-61 → DD-74): port complete provider-template replacement to the opt-in native Team editor; generate a collision-free provider ID, clear stale catalog data, and submit `renames` so coordinator, role, and fallback references follow the profile. Keep executable discovery filesystem-only and asynchronous/off the UI thread.
- [x] Task 2.20 / Red → Green → Verify → Complete: tests first failed for missing provider replacement and rename request support. The native selector now replaces the whole profile template, resolves executable paths on the bounded background worker without launching a CLI, and sends the backend reference rename map. Local coordinator/role/fallback drafts follow the new ID and persisted profiles are reloaded after save. Verification passed 36 native UI tests, 5 provider-switch tests, source-layout, feature Clippy, formatting, diff checks, and the ≤500-line bound (largest touched file: `profile_tools.rs`, 388 lines). Wiki and AgentMemory updated. Custom CLI protocol authoring stays in issue #12.
- [x] Task 2.21 / Plan (R-152 → BG-107 → SF-139 → UC-142 → FR-137/NFR-49 → QAS-148 → DD-162): issue #12 is classified as `enhancement`; define a machine-local, versioned JSON registry with declarative CLI invocation, JSONL/model/quota rules, explicit bounds, and no script/API-key execution. Preserve verified Rust process-control adapters for capabilities that cannot be safely declared as data.
- [x] Task 2.21 / Task → TDD: add bounded JSON Pointer quota rules for custom `usage` JSONL events and normalize them into the shared quota window model. Red caught missing parsing and accepted 101% usage; Green rejects malformed/duplicate rules and out-of-range values. Rules inspect owned stdout only and add no command, API key, or network query.
- [x] Task 2.21 / Local Verify: the offline all-target suite passes (690 passed, 14 ignored, 241 suites); all-target Clippy with `-D warnings`, rustfmt, and diff checks pass. Graphify extracts the parser and its tests; every touched Rust source/test file is at or below 300 lines.
- [ ] Task 2.21 / Remaining Verify → Complete: verify Windows registry behavior and add the separate bounded CLI quota-query invocation and profile-editor controls; unsupported declarations stay explicit failures. These are distinct from the verified stdout-observation mapping.
- [x] Task 2.21 / Registry-template subunit: load `~/.agentconductor/cli-profiles.json` once when a workspace controller opens and append custom CLI profiles to the native provider selector by profile ID. Invalid files leave built-ins usable and expose a localized warning; registry environment values are refused so credentials remain in the CLI's own login/configuration. TDD covers load-on-start, template visibility, malformed-file diagnostics, and distinct custom-provider labels.
- [ ] Task 2.22 / Plan — production native UI conversion (R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160/DD-161): move the eframe/egui implementation from an opt-in probe into the actual Workspace and Agent Room launch paths in reviewable slices. Before changing the default launcher, inventory each shipped screen/action against the probe, define parity tests for approvals/settings/navigation/accessibility, and preserve the current WebView as fallback until the native path passes those contracts.
- [ ] Task 2.22 / Task → TDD: integrate one production screen at a time through `Room::dispatch` and the bounded background worker; keep process/filesystem waits off the UI frame thread. Test the real `ui-new`/`ui` launch paths, screen actions, keyboard/accessibility labels, bilingual strings, and no duplicate Room/controller creation.
- [ ] Task 2.22 / Verify → Complete: measure idle CPU and physical footprint against the shipped baseline; verify Korean IME, macOS and Windows behavior, app resources/licenses, and workflow parity. Remove WebView/owned UI JavaScript only after both windows and settings/workspace management are covered. Update AgentMemory/wiki, validate locally, and commit/push each complete slice; do not release a partial cutover.
- [x] Task 2.22 / Initial launch integration: the actual `ui-new` and `ui` commands now select Workspace and Agent Room surfaces explicitly. With the opt-in native feature they open eframe through one shared launcher and reuse the command's existing Rust `Room`; default builds still use the shipped native WebView shell.
- [x] Task 2.22 / Initial verification: 37 native UI tests across five suites, native-feature all-target Clippy (`-D warnings`), source-layout (500-line bound), default and feature all-target compilation, formatting, and diff checks pass locally. This validates launch-path wiring only; production default remains WebView and full UI parity is still open.
- [x] Task 2.22 / Native palette TDD: the new egui palette regression first failed against stock dark visuals; `appearance::apply` now maps the guide's E/F surface, text, fill, selection, warning/error, and active-control values while retaining the saved theme choice.
- [x] Task 2.22 / Native palette verify: 11 tests across appearance, interface theme/language, and source-layout suites; native-feature all-target Clippy (`-D warnings`), rustfmt, diff checks, the 500-line bound, and offline probe build pass.
- [x] Task 2.22 / Native palette evidence: inspected the existing v0.1.36 bundle at `dist/agentconductor-aarch64-apple-darwin-f1602e7d3202` in Workspace Korean/dark; its default feature and `ui::native` route invoke the Rust eframe surface. The workspace was saved before inspection and no second window was opened.
- [x] Task 2.22 / Visual checkpoint: the visible dark Workspace surfaces and accent controls match the paired dark tokens in `docs/design-guide.md`. This is a partial observation, not a full visual acceptance result.
- [x] Task 2.22 / Theme matrix: Workspace and Agent Room have been inspected in Korean in light and dark themes in the exact v0.1.36 package; no second app process was started, and the user's workspace was restored.
- [x] Task 2.22 / Manual assets: refreshed bilingual screenshots from the current Rust UI. The English and Korean manuals reference retained 1140×960 captures, and the offline HTML manuals/native viewer use the same assets. Evidence: `docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.
- [x] Task 2.22 / Screenshot-status correction → TDD/Verify: the new regression first failed on the stale legacy-image claim, then passed after the plan and Wiki were corrected. Four focused suites pass 15 tests; source-layout, rustfmt, and diff checks pass.
- [ ] Task 2.22 / Remaining acceptance: exact-package element-by-element comparison against the design guide remains open. These source-built screenshots document current Rust UI but do not establish full visual parity or add workflow credit; packaged workflow acceptance remains 15/25 and shipped/default Rust UI remains 0/25.
- [x] Task 2.23 / Plan → TDD: port Agent Room's human candidate decisions into the native surface. Approve/merge require on-demand evidence for the exact current revision, each action is explicitly confirmed, rejection requires a bounded reason, and requests carry the current candidate/revision into the existing Rust controller.
- [x] Task 2.23 / Red → Green: exact candidate/revision action payload tests and the stale-evidence confirmation regression were added first; the launch suite first failed to compile without the decision request, then the UI test caught an approval/merge confirmation that survived a revision change. The native controller now clears stale confirmations; backend revision, diff and candidate checks remain authoritative.
- [x] Task 2.23 / Verify → Complete: 41 tests across five native UI suites, native-feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 448 lines, `native_room_render.rs` 392, `native_ui_probe.rs` tests 497. Scoped Graphify: 197 nodes / 383 edges. Wiki and AgentMemory updated. UI still runs only in the opt-in feature; shipped screen parity remains open.
- [x] Task 2.24 / Plan → TDD: port the Agent Room's Resume and Recover controls. Match current run-state gating; Recover requires explicit confirmation and leaves live/unknown process refusal to the existing Rust recovery guard. Bind both actions to the current revision and keep work on the existing background worker.
- [x] Task 2.24 / Red → Green: resume/recover request tests first failed to compile because the shared request builder was missing; the UI contract then exposed private confirmation state. The final controls preserve shipped state gating, revision-bound requests, explicit recovery confirmation, and clear the dialog when the run leaves `running`.
- [x] Task 2.24 / Verify → Complete: 43 tests across five native UI suites, native-feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 455 lines, `room_actions.rs` 128, `native_room_render.rs` 464. Scoped Graphify: 214 nodes / 408 edges. Update wiki/AgentMemory and commit/push this slice; feature remains opt-in while broader parity is incomplete.
- [x] Task 2.25 / Plan → TDD: port the Agent Room's human-gated tool request panel. Display the exact tool/input/task/status from the Rust snapshot, allow one-time approve, reject, or refresh only for a pending request, confirm each decision, and bind the request to the live revision and exact request ID.
- [x] Task 2.25 / Red → Green: request tests first failed to compile without the exact-receipt request builder; the AccessKit panel test confirmed visible decision controls and caught a test assertion that confused text nodes with accessible button names. The corrected renderer displays the full bounded request input and clears confirmation on revision/status changes. Live decisions remain available only for active runs; idle approvals auto-resume after successful action completion.
- [x] Task 2.25 / Verify → Complete: 45 tests across five native UI suites, native-feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 461 lines, `room_actions.rs` 334, `native_ui_launch.rs` 235, `native_ui_probe.rs` tests 497. Scoped Graphify: 220 nodes / 432 edges. Wiki and AgentMemory updated. The native path remains opt-in pending remaining workflow parity and platform acceptance.
- [x] Task 2.26 / Plan → TDD: port the Agent Room's read-only team briefing to native UI. Fetch through the existing Rust Room dispatcher on the background worker; show observed session/process state, work, last activity, availability restrictions, attention, and approval summary. No agent control or side effect is added.
- [x] Task 2.26 / Red → Green: the native rendering regression first failed because `State` had no briefing payload. The dispatcher integration test first used a startup controller, which correctly rejects Agent Room requests; changing it to open a completed run exercised the real `briefing` endpoint on the worker. The panel renders captured sessions and approval summaries with local timestamps and remains read-only.
- [x] Task 2.26 / Verify → Complete: 47 tests across five native suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `room.rs` 474 lines, `room_actions.rs` 334, `room_briefing.rs` 145, `native_ui_launch.rs` 311. Scoped Graphify: 232 nodes / 446 edges. CocoIndex is unavailable. Wiki/AgentMemory updated. Native UI remains opt-in pending remaining parity and platform acceptance.
- [x] Task 2.27 / Plan (UC-02, UC-04, UC-141): add explicit native review consent and start for the exact ready proposal. Render task assignments, owned paths, model, tests, repository base and digest before confirmation; bind the start request to the current startup revision and proposal digest. Preserve all existing backend validation and human approval recording.
- [x] Task 2.27 / Red → Green: request tests first failed because the revision-bound start request was missing. Added explicit consent showing goal, repository/base, digest, workspace mode, task agent/model/executable/arguments/permission/fallback, writable paths, and test command. Confirmation clears on revision/digest/phase change, and start routes through the bounded Rust worker. A native-worker integration fixture generated a real reviewed plan and verified the exact approved plan starts through `Room::dispatch`.
- [x] Task 2.27 / Verify → Complete: 50 tests across five native suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed files: `native_ui_probe.rs` 456 lines, `workspace.rs` 404, `execution.rs` 123, `native_ui_launch.rs` 454. Scoped Graphify: 247 nodes / 471 edges. Wiki/AgentMemory updated; native prototype parity at that checkpoint was 12/25 (48%), 1 partial, 12 not started. Release remains WebView and 0/25 converted.
- [x] Task 2.28 / Plan (UC-28, UC-38, UC-141): add native task-plan editing for assignment, session model, instructions, owned paths, dependencies and test command. Support bounded add/remove, validate the edited typed plan, save against the current revision/digest, and require the updated plan to be reviewed and approved again.
- [x] Task 2.28 / Red → Green: added request validation first (missing export failed to compile), then rendered editable task fields. Invalid JSON argv remains editable until corrected. The dispatcher integration test creates a real ready proposal, refuses a stale digest, saves the edited plan, and verifies a new digest; the AccessKit test verifies accessible field names.
- [x] Task 2.28 / Verify → Complete: 54 tests across six native UI suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest changed file is `src/ui/setup/dispatch.rs` at 496 lines. Updated the fixed denominator: prototype parity 13/25 (52%), one partial and 11 not started; release conversion remains 0/25. Wiki/AgentMemory updated; native UI remains opt-in pending production parity and platform acceptance.
- [x] Task 2.29 / Plan (GitHub #16, UC-02, UC-141): make the native Review surface display an explicit start-in-progress status immediately after approval, then route to Agent Room only after the report for the exact approved run exists. The shipped WebView consent path also needs progress before its start request resolves; the Rust transition remains the target and #9 will remove the WebView path.
- [x] Task 2.29 / Red → Green: pending-label and exact-run readiness tests first failed to compile without the shared Rust helpers. The transition retains the returned run path, polls asynchronously, refuses a prior run's report, opens the existing Room dispatcher, and loads its snapshot before replacing Review with Agent Room. Failed or unopenable runs stop the transition and show the error.
- [x] Task 2.29 / Verify → Complete: 57 tests across seven native UI suites, 11 compatibility tests across three suites, feature all-target Clippy (`-D warnings`), source-layout, formatting and diff checks pass locally. Largest touched Rust file is `workspace.rs` at 479 lines. Prototype parity stays 13/25 (52%), one partial and 11 not started; release conversion remains 0/25. Issue #16 awaits merge of the default-path follow-up below; #9 remains responsible for making the Rust UI the default. Wiki/AgentMemory updated and commit/push completed.
- [x] Task 2.29 / Default-path follow-up (GitHub #16): the shipped WebView consent handler now renders a localized start-pending status directly below the approve button before awaiting `action(start)` and keeps it busy while the accepted run opens; declined or failed requests clear it. The regression in `tests/start_feedback_runtime.cjs` reproduced the absent feedback before the fix. This compatibility path remains only until #9 ships the Rust UI by default.
- [x] Task 2.29 / Verify: the start-feedback runtime regression, planning-progress runtime test, five planning-feedback checks, Rust message/UI/pending tests, formatting and diff checks pass locally. macOS/Windows packaged behavior is not yet verified.
- [x] Task 2.30 / Plan (GitHub #15, UC-28): when an edited task plan differs from the reviewed digest, make the save action visually prominent and state that saving requires reviewing the new plan again. Preserve the existing disabled state when no edits exist or a save is pending.
- [x] Task 2.30 / Red → Green: the accessibility-tree regression first failed because the dirty state had no explicit next-step label. The native editor now exposes “Plan changes need review” and an accent-filled “Save changed plan and review again” action only for dirty drafts.
- [x] Task 2.30 / Verify → Complete: 58 tests across seven native UI suites, feature all-target Clippy (`-D warnings`), source-layout, rustfmt, diff and 500-line checks pass locally; largest touched files are 237 and 220 lines. Graphify updated the isolated UI graph to 562 nodes / 1,075 edges. CocoIndex incrementally indexed 519 files into 5,493 chunks with zero errors. Workflow parity remains 13/25 (52%) prototype and 0/25 release conversion. GitHub #15 remains open until the shipped UI includes the fix. Wiki/AgentMemory updated; commit/push completed.
- [x] Task 2.31 / Plan (GitHub #11, UC-06, UC-16): make the active coordinator visible in native plan review with a pulsing status light, reusing the Room running-state color and repaint cadence. Keep terminal and non-running states static.
- [x] Task 2.31 / Red → Green: a regression first failed because the plan screen had no activity-color source. The shared planning module now returns a pulse color only for a running plan, and Review renders it as an accessible coordinator indicator while its existing 500 ms repaint keeps the pulse moving.
- [x] Task 2.31 / Verify → Complete: 59 tests across eight native UI suites, feature all-target Clippy (`-D warnings`), source-layout, rustfmt, diff and 500-line checks pass locally; touched files are 472, 26, 497 and 18 lines. Graphify updated the isolated UI graph to 596 nodes / 1,152 edges. CocoIndex indexed 521 files into 5,497 chunks with zero errors. Workflow parity stays 13/25 (52%) prototype and 0/25 release conversion. GitHub #11 remains open pending shipped UI verification. Wiki/AgentMemory updated; commit/push completed.
- [x] Task 2.32 / Plan (GitHub #14, UC-23): surface each newly pending Claude Code tool request immediately in the native Agent Room, with exact tool/task/input evidence and an explicit decision. Preserve the existing revision-bound confirmation, allow/deny validation, and inline request list; defer only the request the operator dismisses.
- [x] Task 2.32 / Red → Green: the new native UI regression first failed because pending receipts appeared only in the inline list. A per-request modal now displays exact request evidence and accessible allow/deny/defer controls. Allow and deny still require the existing second confirmation; deferring one request keeps it in the list and advances the next pending request.
- [x] Task 2.32 / Verify → Complete: 61 tests across nine native UI suites and five live Claude tool-decision integration tests pass; feature all-target Clippy (`-D warnings`), source-layout, rustfmt, diff and 500-line checks pass locally. Largest touched files are 481 lines (`room.rs`), 438 (`room_actions.rs`), 475 (`native_ui_launch.rs`) and 123 (`native_ui_tool_dialog.rs`). Graphify updated the isolated UI graph to 653 nodes / 1,272 edges. CocoIndex indexed 522 files / 5,512 chunks with zero errors. Workflow parity remains 13/25 prototype and 0/25 release conversion. GitHub #14 remains open until shipped UI verification; wiki/AgentMemory updated and commit/push completed.
- [x] Task 2.33 / Plan (CI regression): ensure every integration test that imports the optional `native-ui-probe` module is Cargo-gated by that feature, so the production-default `cargo build --all-targets --locked` remains valid.
- [x] Task 2.33 / Red → Green: the new Rust manifest contract first failed for the four missing Cargo test targets; all nine native UI integration targets are now explicitly gated by `native-ui-probe`. Platform validation runs all nine feature suites together using a literal YAML block, and the release-workflow contract checks the block style and target coverage.
- [x] Task 2.33 / Local Verify: default `cargo build --all-targets --locked`, default all-target Clippy, feature-enabled Clippy, source-layout, targeted rustfmt, diff checks and the nine UI suites plus release/manifest contracts (69 passed) are green. An initial full default all-target test run exposed one stale workflow assertion; it was corrected and the affected contract suites pass.
- [ ] Task 2.33 / Cross-platform Verify: Actions run 38838080638 passed macOS, including the default all-target build and test suite, but Windows failed the Rust release-workflow contract because the checkout's CRLF line endings did not match its LF-only assertion. Both workflow contract paths now normalize CRLF to LF, with a synthetic CRLF regression; verify the exact pushed source on Windows before closing this CI task.
- [x] Task 2.34 / Plan (UC-141, FR-136, QAS-147, DD-161; UI workflow #14): expose bounded cache preview and explicit digest-bound clearing from the native Workspace screen. Reuse existing Rust cache policy and `Room::dispatch` on the background request worker; keep protected work/evidence outside the clearable inventory. Do not block the UI thread, add a service, or enable destructive action before preview.
- [x] Task 2.34 / Red: add a feature-gated native cache contract covering the preview/clear request payloads, accessible summary/path rendering, and empty-cache state; it first failed because the native cache surface did not exist.
- [x] Task 2.34 / Red regression: the existing exact Workspace accessibility test then failed because it did not include the new cache control; update that shared button contract so the addition remains visible to assistive technology.
- [x] Task 2.34 / Green: connect the Workspace cache control to async preview, display reviewed disposable paths/size and protected or uncertain count, then clear only the exact preview digest through the existing Rust handler.
- [x] Task 2.34 / Verify → Complete: the background Room worker previews and clears a real disposable cache fixture while preserving `startup.json`; 6 cache-specific tests and all 12 native feature/workflow suites (76 passed) pass. The default all-target regression passed 518 tests with 14 ignored; the subsequent CRLF-adjusted `release_notes` suite passes 8 tests. Default all-target build, both default/feature Clippy (`-D warnings`), source-layout, formatting, diff and 500-line checks pass. CocoIndex: 525 files / 5,536 chunks; scoped Graphify: 698 nodes / 1,328 edges. Prototype parity is now 14/25 (56%), with 1 partial (4%) and 10 not started (40%); shipped/default UI remains 0/25 pending full parity and platform acceptance.
- [x] Task 2.35 / Plan (UC-141, FR-136, QAS-147, DD-161; UI workflow #3): finish the native registered-agent lifecycle by adding a trash-icon removal action with explicit confirmation. Save through the existing Rust `save_profiles` boundary, remove coordinator/role/fallback references using its established semantics, cancel editing of a removed profile, and disable removal during active metadata/profile-edit operations.
- [x] Task 2.35 / Red: extend native profile accessibility and lifecycle regressions for the remove control, explicit confirmation, reference cleanup and profile-registry request. The UI accessibility contract was observed failing before the remove action was implemented.
- [x] Task 2.35 / Green: confirmed removal updates the local profile draft, clears stale coordinator/role/fallback references, closes editing for the removed profile, and emits `SaveProfiles` through the existing asynchronous Rust dispatcher path.
- [x] Task 2.35 / Verify → Complete: all 12 native profile tests and 5 provider-switch persistence/reference tests pass; default `cargo check --all-targets`, feature Clippy (`-D warnings`), changed-file rustfmt, diff checks, and the explicit 500-line count pass. The broad feature test run was stopped after 120 passed / 2 ignored to avoid an unnecessary full-suite wait; focused profile and persistence tests are the completion evidence. CocoIndex indexed 525 files / 5,549 chunks; scoped Graphify is 714 nodes / 1,458 edges. Prototype parity is now 15/25 (60%), with 0 partial and 10 not started (40%); shipped/default UI remains 0/25.
- [x] Task 2.36 / Plan (R-39/R-40/R-88/R-89 → BG-17/BG-43/BG-44 → SF-33/SF-34/SF-75/SF-76 → UC-30/31/78/79 → FR-24/25/73/74, NFR-16 → QAS-31/32/84/85 → DDR-13 → DD-47/48/96/97; UI workflow #9): add one native workspace menu backed by the existing asynchronous `Room::dispatch` operations. Cover list/reopen/switch, named creation, remove-from-list, and confirmed archival deletion using exact name/path; surface backend refusals and leave the active workspace intact on failed actions.
- [x] Task 2.36 / Red: feature-gated tests first failed because the native workspace navigation module was absent. Contracts cover list/create/open/switch/forget/delete, discard confirmation, exact archive confirmation, accessible menu actions, and workspace reload.
- [x] Task 2.36 / Green: implemented native workspace navigation in `workspace/navigation.rs` (439 lines), reusing the existing request worker and Rust backend validation. Create/open/switch refresh the workspace snapshot asynchronously; forgetting only removes the registry entry; archival requires the exact workspace name and states project data is preserved.
- [x] Task 2.36 / Verify → Complete: 5 navigation, 4 workspace registry, and 8 release-workflow tests pass; default all-target check, native-feature all-target Clippy (`-D warnings`), source-layout (including the 500-line bound), formatting, and diff checks pass. Prototype parity is 16/25 (64%), 0 partial, 9 not started; shipped/default UI remains 0/25 pending full parity and platform acceptance. Issue #9 remains open.
- [x] Task 2.36 / CI follow-up: added the missing `Open workspace menu` accessible button label to the macOS probe expectation; all 14 `native_ui_probe` tests and feature-gated Clippy pass on `codex/issue-9-accessible-workspace-menu`. Exact-source macOS/Windows validation and native-package jobs passed for `7bfad6e` in [run 38844468171](https://github.com/neodamul/AgentConductor/actions/runs/38844468171).
- [x] Task 2.36 / Package-gate regression: exact-source run 38991687927 passed macOS/Windows validation but both package jobs exposed missing `egui-wgpu 0.37.0` license text. Added Cargo-source VCS pin `7c0b404`, official release-license revision `841a328`, SHA-256 provenance, and a native-package regression; focused provenance and packaging tests pass locally.
- [x] Task 2.36 / Local package follow-up: the canonical run exposed missing provenance for `objc2-metal` 0.2.2/0.3.2, `objc2-quartz-core` 0.3.2, and `spirv 0.4.0+sdk-1.4.341.0`; exact upstream documents/VCS/digests now replace the duplicate generic record. The 214-dependency audit also found Cargo's `Apache-2.0/MIT` spelling; packaging now accepts it and normalizes SBOM output to SPDX `OR`. 3 provenance tests, 2 package tests, and the regression pass.
- [x] Task 2.36 / Smoke-log regression: package assembly succeeded after the 214-dependency audit, then repeated local smoke found a `command-0` collision because evidence reused a fixed directory. A TDD regression failed on the second same-path run; each smoke now gets a unique evidence subdirectory and both runs pass.
- [x] Task 2.36 / Verify local package: exact-source `cargo run --release --locked -- package-release` passed on `6eb27d5781b5`; macOS archive SHA-256 is `e3b081a417da6938be903aac26ccecde57a988af455f4af9bec0c550368fb8c9`. Manifest records the exact source commit and 214 dependencies; all four packaged smoke cases passed.
- [ ] Task 2.36 / Platform verify: rebuild the final documented source, then rerun exact-source macOS/Windows validation and packaging; continue issue #9 acceptance gates.

### Native workspace settings import/export (R-52 / DD-59; UI workflow 10)
- [x] Plan: R-52 → BG-20 → SF-43 → UC-42 → FR-37/NFR-18 → QAS-44 → DDR-15 → DD-59; complete the existing share workflow in the opt-in native Workspace UI without changing the secure Rust backend.
- [x] Red: added native request and accessibility contracts; the first run failed because the native sharing module did not exist.
- [x] Green: reuse the asynchronous `RequestWorker` and existing `export_settings`, `preview_settings`, and `import_settings` requests. Disable import/export while the workspace has unsaved edits; show the imported settings summary and backend warnings; default permission claims to unverified and require explicit import confirmation; refresh after import.
- [x] Verify: 7 native workspace navigation/sharing tests, 2 existing real workspace-share tests, focused native-feature Clippy, and rustfmt pass. The 500-line bound holds (largest changed Rust file: 490 lines).
- [x] Verify: exact-source CI run 38852463148 passed all four jobs for `a69132b`: macOS validation (6m42s), Windows validation (14m49s), macOS package build/render/execute (6m2s), and Windows package build/render/execute (10m5s).
- Prototype parity: 17/25 (68%); 8 not started; 0 partial. Shipped/default UI conversion remains 0/25, so issue #9 stays open.
- [x] CI regression triage: run 38851108577 passed macOS build, source-layout, and applicable tests, then failed the native UI contract because `native_probe_controls_have_accessible_names` did not expect the new Import/Export buttons. The same assertion failed locally; updated it to include both accessible labels.
- [x] Follow-up local verification: all 76 native UI tests across 11 suites, both native binary checks, and the exact feature-gated Clippy command passed before the contract-only correction. The corrected `native_ui_probe` suite (14 tests) and focused Clippy pass; every changed Rust file is at most 499 lines.
- [x] Re-run exact-source macOS/Windows CI for commit containing the accessible-label test correction; all validation and exact native-package jobs passed in run 38852463148.
- [x] Plan: R-128 → BG-83 → SF-115 → UC-118 → FR-113/NFR-49 → QAS-124 → DD-137; this slice ports only theme selection into the opt-in native UI.
- [x] Red: add `tests/native_ui_appearance.rs` first; it failed because no native appearance module or selector existed.
- [x] Green: add the shared navigation menu, persist the existing light/dark preference, apply the saved theme from cached state, and use semantic panel fill/stroke for workspace cards.
- [x] Verify: 2 appearance and 7 workspace tests, feature all-target Clippy (`-D warnings`), rustfmt, diff checks, and a rebuilt macOS preview. Accessibility exposes Appearance/Light/Dark; the dark preference survives app restart and workspace cards render in dark mode.
- [ ] Remaining workflow 11: native language switching and status-tag parity; keep prototype parity at 17/25 complete, 1 partial, 7 not started. Shipped/default UI remains 0/25 because this feature is opt-in.
- [x] Plan: R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76; this slice adds the shared native selector and localizes only shared navigation labels.
- [x] Red: add a restart-persistence/accessibility contract; it failed because native settings exposed no language selector or localized step tabs.
- [x] Green: persist English/Korean in the existing machine-wide preferences; localize the shared Settings menu, theme choices, workspace-menu accessible label, and three workflow tabs from the existing catalogs.
- [x] Verify: 1 language persistence/accessibility, 2 appearance, 7 workspace, 13 native probe, 5 i18n-catalog, and 2 existing interface-language tests pass; native-feature all-target Clippy, changed-file rustfmt, changed-file diff checks, and changed-source line counts pass. Manual macOS preview confirms Korean menu/tabs and saved selection.
- [ ] Remaining workflow 11: translate the remaining Agent Room controls/content. Native status tags now have their own verification slice; prototype parity remains 17/25 complete, 1 partial, 7 not started; shipped/default UI remains 0/25.
- [x] Native Workspace form localization (R-70 / DD-76 / UC-58): translated workspace name, project folder, goal, isolation choices, save action, and settings-sharing labels/actions in visible text and accessible names; user-entered values remain untouched and bilingual catalogs are reused.
- [x] Red: the expanded native language contract failed because execution-isolation and sharing heading labels were absent from the accessible tree.
- [x] Green: localized the Workspace form and settings-sharing controls using existing catalogs and explicit accessible names; updated the native probe contract to follow the active language.
- [x] Verify: 1 language, 7 workspace, 2 appearance, 13 native-probe, 5 catalog, and 2 interface-language tests pass; native-feature all-target Clippy (`-D warnings`) passes. Rebuilt and inspected the single macOS preview window with persisted Korean selection and user-entered values intact.
- [ ] Native task/run status tags (R-70 / DD-137 / UC-58): localize states, expose task identity and status to assistive technology, use theme-specific WCAG AA color pairs, and pulse the running indicator without polling or disk reads.
- [x] Red: the run-render accessibility test failed because task state was only colored text with no task/status accessible name.
- [x] Green: render compact semantic status tags for run and task states, reuse the bilingual catalog, preserve the active pulse with scheduled repaints, and classify blocked as an error.
- [x] Verify: 2 native status, 12 room render, 12 launch, 2 tool-dialog, 13 native-probe, and 5 i18n tests pass; feature all-target Clippy, changed-file rustfmt/diff and 500-line checks pass. Rebuilt preview confirms Korean task labels and subdued amber status tags in light and dark themes.
- [x] Plan (R-70 / DD-76 / UC-58): localize the primary Agent Room controls and stop-confirmation flow in both visible text and accessibility names; keep stop gating and review dispatch unchanged.
- [x] Red: add a Korean Agent Room accessibility contract first; it failed because the refresh, briefing, stop, and review controls exposed English labels.
- [x] Green: reuse the existing bilingual catalog for those controls and the stop dialog, retaining the existing confirmation validation and bounded stop-reason input.
- [x] Verify: 35 focused Agent Room, status, launch, tool-dialog, and i18n tests pass; feature all-target Clippy (`-D warnings`), rustfmt, and 500-line checks pass (`room.rs` 486 lines; touched probe test 494 lines). A full feature all-target run exposed the old probe contract's English-only refresh label under saved Korean settings; the contract now reads the selected-language catalog and its regression passes. Rebuilt preview inspection is in progress.
- [x] Plan (R-70 / DD-76 / UC-58): localize candidate approve/reject/merge actions and confirmation prompts, including the rejection-reason field, while preserving exact-candidate evidence checks, explicit confirmation, and bounded reason validation.
- [x] Red: the new Korean candidate-decision accessibility contract first failed because candidate buttons still exposed English names.
- [x] Green: reuse existing approve/reject/merge catalog entries, add bilingual confirmation and rejection-reason entries, and expose each confirmation prompt through the accessibility tree.
- [x] Verify: 33 focused candidate, Agent Room language/render, launch, tool-dialog, and i18n tests pass; focused native-library plus candidate-test Clippy (`-D warnings`) and direct formatting checks pass. `room.rs` is 492 lines and the new test is 86 lines. The full feature all-target run was cut short after its earlier locale-only failure was corrected; source-layout was not run to completion.
- [x] Plan (R-70 / DD-76 / UC-58): localize pending tool-request review, decision prompts, actions, task/status context, and accessible names in the native Agent Room; preserve revision-bound request validation and approval gating.
- [x] Red: extend the tool-dialog AccessKit regression in Korean; it failed because the request actions still exposed English accessible names.
- [x] Green: reuse catalog actions where available and add bilingual labels for tool confirmations, context, fallback text, and accessible action names. The underlying approve/reject/refresh eligibility and exact request ID remain unchanged.
- [x] Verify: 9 focused tool-dialog, Agent Room language, candidate, and i18n tests pass; one unrelated broad source-reference test was excluded after it stalled reading the full source tree. Rustfmt and diff checks pass; changed Rust files are within 500 lines (`room.rs` 493, `room_actions.rs` 482, tool-dialog test 186). The rebuilt opt-in preview reopened as one macOS window on the preserved Korean Workspace data. Scoped Graphify extraction: 258 nodes / 325 edges. CocoIndex indexed 534 files / 5,662 chunks (7 reprocessed, 0 errors).
- [ ] Verify remaining for this slice: native-feature Clippy and full source-layout test; Clippy was interrupted after prolonged full-crate source reads. This opt-in build still does not change the shipped WebView UI.
- [x] Plan (R-70 / DD-76 / UC-58): localize native run-resume and recovery controls plus the recovery safety explanation, preserving the Rust run-state gates and refusal to recover live/unknown processes.
- [x] Red: extend the Korean Agent Room AccessKit contract; it failed because Resume and Recover controls retained English names.
- [x] Green: reuse the existing interrupted-run and recovery-safety catalog entries, add bilingual Resume/Cancel/Confirm labels, and pass the selected language through the existing Room control renderer.
- [x] Verify tests: the Korean Agent Room Accessibility test passes for resume/recover labels, cancellation/confirmation, and the visible recovery safety text; the bilingual catalog key/placeholder contract also passes. Rustfmt is applied.
- [x] Verify remaining: the Korean Room language test and bilingual catalog key/placeholder test pass; Rustfmt and changed-file diff checks pass; Rust files remain below 500 lines (`room.rs` 493, `room_actions.rs` 489, Room language test 82). The native probe builds with debug info disabled and was reopened as one macOS window; existing Korean Workspace values remain intact. Scoped Graphify: 256 nodes / 323 edges. CocoIndex: 534 files / 5,667 chunks (7 reprocessed, 0 errors).
- [x] Follow-up verification: native-feature all-target Clippy (`-D warnings`) and the full `source_layout` integration test pass on the current checkout. These checks do not change the fact that the opt-in native build has not replaced the shipped WebView UI.
- [x] Plan (R-70 / DD-76 / UC-58): pass the selected language into the read-only Agent Room team briefing and localize its visible labels, known statuses, fallback values, timestamps, and approval summary. Preserve the existing read-only data and dispatcher behavior.
- [x] Red: add a Korean rendering regression for briefing title, completed and availability-wait states, session metadata, process state, and the empty approval summary; it failed while the briefing remained English-only.
- [x] Green: reuse the bilingual message catalog, status-label helper, and template interpolator; no new dependency, polling, or mutable action was added.
- [x] Verify: both Agent Room language tests and the bilingual catalog ID/placeholder contract pass; native probe builds, Rustfmt and diff checks pass, and changed Rust files remain under 500 lines (`room.rs` 493, `room_briefing.rs` 179, language test 135). The refreshed macOS preview has one window and preserves the existing Korean Workspace values. Scoped Graphify: 54 nodes / 111 edges; CocoIndex: 534 files / 5,674 chunks, 7 reprocessed, 0 errors. Workflow 11 remains partial at 17/25 prototype workflows; shipped/default conversion remains 0/25.
- [x] After commit: update AgentMemory and GitHub issue #9 with the team-briefing localization and its verification.
- [x] Plan (R-70 / DD-76 / UC-58): localize the Agent Room heading and read-only review-evidence summary while preserving raw test output, diff bytes, current evidence checks, and the existing English `render_evidence` API.
- [x] Red: Korean Room rendering first failed on the English heading; a new evidence contract also failed to compile because the shared renderer had no language-aware entry point.
- [x] Green: route the Room through `evidence_in_language`, keep the existing public English wrapper, and reuse bilingual catalog interpolation for revision, candidate, test exit/timeout, output stream, and truncation labels.
- [x] Verify: 3 native Room-language tests, 12 Room-render tests, and the catalog ID/placeholder contract pass; native-feature all-target Clippy (`-D warnings`), `source_layout`, rustfmt, diff checks, and the native probe build pass. All changed Rust files are within 500 lines (`native_ui_probe.rs` 497, `room.rs` 493, `render.rs` 270; affected tests 180 and 452). The updated preview has one window and retains the current Korean Workspace values. Scoped Graphify: 129 nodes / 283 edges; CocoIndex: 534 files / 5,686 chunks, 9 reprocessed, 0 errors.

### Native MCP server registration — DD-98 partial workflow 12
- [x] Plan: use existing R-90 → BG-45 → SF-77 → UC-80 → FR-75 → QAS-86 → DD-98; registration remains distinct from Jira/Confluence source import.
- [x] Red: add tests for loading saved OpenCode-shaped definitions, empty settings, revision-bound JSONC save requests and the 64 KiB dispatcher limit; they first fail because the native MCP editor does not exist.
- [x] Green: add the registration entry to the shared Settings menu and a native JSON/JSONC editor. Save uses the existing background Rust worker and revision-bound `mcp_save`; saving only persists definitions and never launches the server. Refresh the snapshot without replacing other workspace drafts.
- [x] Verify: focused MCP/settings/language/native-probe/backend/source-layout tests pass (24 total); native-feature all-target Clippy passes. An initial source-layout check caught `workspace.rs` at 501 lines; move `execution_ready` into its execution module and keep both `native_ui_probe.rs` and `workspace.rs` at 500 lines.
- [x] Verify: 24 tests across six MCP/settings/language/native-probe/backend/source-layout suites; native-feature all-target Clippy (`-D warnings`), `cargo fmt --check`, `git diff --check`, and the ≤500-line source-layout guard pass. Built and reopened the existing single macOS preview process; Settings → MCP servers renders the OpenCode JSON/JSONC editor with the saved empty maps. Graphify scoped 14 files: 220 nodes / 467 edges. CocoIndex command is installed but cannot start because `mcp.server.fastmcp` is missing.
- [x] Complete: pushed implementation commit `927575c`; AgentMemory record `mem_mvaojp4g_504e4c27874c` and [issue #9 progress update](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6210944269) are saved.
- [x] Workflow 12 remainder: exact-package tool discovery, Jira/Confluence preview, and separately approved apply are verified in `docs/evidence/issue9-2fd8e57-mcp-import-ui.json`.
- Prototype parity remains 24/25 complete (96%); exact-package accepted workflows are 6/25 (24%); shipped/default UI conversion remains 0/25.

### Native messenger settings and delivery — workflow 13
- [x] Plan: reuse R-43 → BG-18 → SF-37 → UC-61 → FR-28/NFR-17 → QAS-35/QAS-66 → DD-79; expose the existing Rust save/test boundary from native Settings without exporting secret values.
- [x] Red: the bilingual native form/accessibility and revision-bound request tests first failed because the native messenger editor and Settings route were absent.
- [x] Green: add localized Settings access, asynchronous status/save/test handling, masked Slack/company webhook fields, explicit test delivery, saved-secret preservation, and pending-state interaction lockout through the existing Rust dispatcher.
- [x] Verify: 28 tests across seven targeted suites; native-feature all-target Clippy (`-D warnings`), `source_layout`, bilingual message catalog, formatting, diff checks, and the ≤500-line limit pass locally.
- [x] Complete: native prototype parity is 18/25 (72%), with 2 partial and 5 not started; shipped/default conversion remains 0/25. The existing single preview process was not replaced or duplicated.
- [ ] Acceptance: compare the rebuilt native screen against `docs/design-guide.md` and the current manual screenshots, verify keyboard/Korean IME and accessibility on macOS and Windows, and perform an authorized live webhook test.

### Native Workspace roster and goal header — workflows 11/16 partial
- [x] Plan: keep the #9 migration opt-in and preserve the shipped Workspace behavior and the 25-workflow denominator; follow the existing light/dark design tokens and screen-reader labels.
- [x] Red: add bilingual AccessKit coverage for the goal headline and all three stage actions; it first fails because the native header module is absent.
- [x] Green: add a persistent agent roster with status, session/model counts, quota details and refresh, coordinator marking, and edit/register actions. Restore the goal-first heading, step eyebrow, clearer card hierarchy, and legible secondary text; keep polling limited to active planning.
- [x] Verify: 43 tests across nine native/UI/catalog/source-layout suites; feature all-target Clippy with warnings denied; native probe build; formatting, diff and 500-line source checks pass locally.
- [x] Preview: replace the existing single macOS preview process with the rebuilt binary; visually confirm the bilingual goal heading, selected stage, agent roster, and retained workspace draft.
- [ ] Acceptance: review both themes against the documented design and current manual screenshots, then verify Korean IME, keyboard/screen-reader operation, and macOS/Windows behavior. Prototype parity remains 18/25 (72%), with 2 partial and 5 not started; shipped/default conversion remains 0/25.

### Native task feedback and reassignment — workflow 18
- [x] Plan: reuse task feedback R-110 → BG-65 → SF-97 → UC-100 → FR-95 → QAS-106 → DD-118 and pending-task assignment R-121 → BG-76 → SF-108 → UC-111 → FR-106 → QAS-117 → DD-130; keep the native UI on the existing Rust Room boundary.
- [x] Red: add revision-bound task feedback/reassignment, bilingual accessibility, and real Room-dispatch persistence tests before adding controls; initial compile failed because the task controls module was absent.
- [x] Green: add localized task/profile selectors, per-task feedback, bounded validation, audited Room requests, and resume after accepted feedback; keep assignment disabled while work is active.
- [x] Verify: 50 tests across nine suites, native-feature all-target Clippy (`-D warnings`), `cargo fmt --check`, offline native binary build, `source_layout`, diff check, and all changed Rust files under 500 lines.
- [x] Complete prototype slice: native workflow parity is 19/25 (76%), 2 partial, 4 not started; shipped/default conversion remains 0/25. Live-window interaction, Korean IME/keyboard and Windows acceptance remain open.

### Native task session and model selection — workflow 19
- [x] Plan: reuse R-20 → BG-09 → SF-11 → UC-11 → FR-02 → QAS-11 → DD-13; selecting a task selects its owned session, while the provider session ID remains read-only and the chosen model must come from that agent's observed catalog.
- [x] Red: test bilingual session/model controls, revision-bound model actions, catalog rejection, and persistence through Room dispatch; first run failed because `Action::SetModel` and the controls did not exist.
- [x] Green: show the selected task's provider session ID with an accessible name and allow an idle task's assigned agent model to be changed through the existing Rust action.
- [x] Verify: 51 tests across nine native suites, 8 native-session/recovery tests, native-feature all-target Clippy (`-D warnings`), rustfmt, offline native build, source layout, diff checks, and all changed Rust files under 500 lines. Scoped Graphify AST extraction: 360 nodes / 726 edges across 24 Rust files. CocoIndex CLI is unavailable in this environment.
- [x] Complete prototype slice: native parity is 20/25 (80%), 2 partial, 3 not started; shipped/default conversion remains 0/25. Live GUI, Korean IME/keyboard, and Windows acceptance remain open.

### Native schedule after availability restrictions — workflow 20
- [x] Plan: reuse R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136; accept explicit RFC 3339 timestamps with timezone, keep scheduling limited to a waiting task, and send through Room's existing Rust scheduler.
- [x] Red: add bilingual schedule-control, revision-bound request, timestamp/status validation, and persisted manual-schedule tests; the first run exposed the missing action and the fixture was changed to produce a real quota-waiting run.
- [x] Green: expose the restriction reason, timezone-explicit timestamp field and schedule action only for waiting tasks; route persistence through the existing Rust scheduler without starting work before the due time.
- [x] Verify: 61 tests across 11 native UI/session/recovery suites, feature all-target Clippy (`-D warnings`), rustfmt, offline native build, source layout, diff check, and all changed Rust files under 500 lines. Scoped Graphify extraction: 20 nodes / 31 edges across 2 Room-control files; CocoIndex CLI is unavailable in this environment.
- [x] Complete prototype slice: native parity is 21/25 (84%), 2 partial, 2 not started; shipped/default conversion remains 0/25. Live GUI, Korean IME/keyboard, and Windows acceptance remain open.

### Native task activity, logs, and lessons — workflow 21
- [x] Plan: reuse R-104 → BG-59 → SF-91 → UC-94 → FR-89 → QAS-100 → DD-112 and the existing bounded `Room::Task` detail response; keep provider event summaries separate from JSONL raw logs and show only reviewed lesson assets.
- [x] Red: add localized Activity/Raw logs/Lessons rendering, Claude/Codex/OpenCode event normalization, unknown-event and no-raw-JSON checks, plus real Room-dispatch coverage; first run failed because the native module did not exist.
- [x] Green: load selected-task details through the existing async request worker; refresh with the Room snapshot cadence and render bounded output.
- [x] Verify: 66 tests across 12 native UI/session/recovery suites, feature all-target Clippy (`-D warnings`), rustfmt, offline native build, source layout, diff check, and all changed Rust files under 500 lines. Scoped Graphify extraction: 57 nodes / 125 edges across 4 Room-control files; CocoIndex CLI is unavailable in this environment.
- [x] Complete prototype slice: native parity is 22/25 (88%), 2 partial, 1 not started at this checkpoint; shipped/default conversion remains 0/25. Live GUI, Korean IME/keyboard, and Windows acceptance remain open.

### Native agent tools guide — workflow 15 slice
- [x] Plan: reuse `R-81 → BG-36 → SF-68 → UC-71 → FR-66/NFR-37 → QAS-77 → DD-89`; keep the guide in the Workspace menu, read-only, bilingual, and independent of Settings.
- [x] Red: extend native UI tests to expose the guide entry and every command to assistive technology; the new test first failed because the native guide did not exist.
- [x] Green: render an inline, scrollable native guide with selectable commands and explicit copy buttons; copy only writes to the clipboard and no tool process/network request is made.
- [x] Verify: 11 guide/workspace/language tests, native-feature all-target Clippy (`-D warnings`), `source_layout`, offline native build, rustfmt, diff checks, and the ≤500-line guard pass. Scoped Graphify extraction: 30 nodes / 61 edges. CocoIndex remains unavailable because its CLI environment lacks `mcp.server.fastmcp`. No additional app window was opened.
- [x] Complete this slice: workflow 15 moves from not started to partial. Native parity remains 22/25 complete (88%), 3 partial, 0 not started; shipped/default conversion remains 0/25. Manual reading and release updates remain to be ported before workflow 15 is complete.

### Native offline user manual — workflow 15 slice
- [x] Plan: reuse `R-144 → BG-99 → SF-131 → UC-134 → FR-129 → QAS-140 → DD-153`; parse the existing Markdown once per open/language change and render headings, lists, tables, and code with native egui controls.
- [x] Red: add Rust tests for block parsing and menu exposure; the initial run failed because the native block parser and manual view were missing.
- [x] Green: share the block parser with HTML generation, connect User manual from the Workspace menu, and render the current manual with native egui controls and code-copy actions.
- [x] Verify: 17 tests across five suites, committed HTML generation parity for both languages, native-feature all-target Clippy (`-D warnings`), `source_layout`, offline native build, rustfmt, diff checks, and the ≤500-line guard pass. Scoped Graphify extraction: 47 nodes / 101 edges. CocoIndex remains unavailable because its CLI environment lacks `mcp.server.fastmcp`.
- [x] Screenshot TDD / Red: extend the native AccessKit test over every screenshot referenced by the English and Korean manuals; the new image-role assertion failed while the native viewer exposed only screenshot text.
- [x] Screenshot Green: render the six whitelisted bundled assets as bounded egui images with accessible alt text and captions; decode on demand through an optional native-only Rust image dependency, cache textures in egui context data, and release the cache when the manual closes.
- [x] Screenshot Verify: all referenced English/Korean screenshots render with accessible image labels; native manual tests (3) and HTML renderer tests (4) pass offline. The changed Rust/test files remain below 300 lines; full repository layout remains red on 52 pre-existing oversized files.
- [x] Full local verification: all-target native regression reached the known source-layout failure (52 files remain oversized); focused native manual/HTML tests, all-target Clippy, default/probe binary builds, rustfmt and diff checks pass.
- [x] Tooling refresh: Graphify rebuilt the structural graph without LLM calls (6,086 nodes / 11,051 edges); CocoIndex refreshed 7 changed files with zero errors (566 files / 5,962 chunks).
- [x] Exact-package UI check: in the existing v0.1.36 window, opened the offline manual and returned to Workspace, then opened the update panel. Accessibility exposes installed v0.1.36, latest v0.1.37, the release link, release-note content below it, and a scrollbar; observed notes include #162, #165, #149, and #156. No update was installed and no workspace data changed. Evidence: `docs/evidence/issue9-f438820-manual-update-ui-ax-20261023.json`.
- [ ] Complete: workflow 15 stays partial until screenshots and release updates are also available in the native interface.

### Native release update panel — workflow 15 slice
- [x] Plan: reuse the existing `update_status`, `check_update`, and `apply_update` Rust dispatcher and its checksum-verified installer; keep network work on the existing request worker and preserve the all-workspaces idle gate before install.
- [x] Red: add native status, release-note, check/apply action, and accessibility contracts before rendering the update panel; the focused test first failed because the panel/module was missing.
- [x] Green: open the localized updates panel from Settings, show scrollable release notes, and poll only while the panel's check/install is active.
- [x] Verify: 19 focused tests across six suites, native-feature all-target Clippy (`-D warnings`), `source_layout` (442 files; 197.21 seconds), changed-file rustfmt, offline native probe binary build, diff check, and ≤500-line checks pass. Cargo required a temporary `CARGO_HOME` because the default global cache stalled before spawning rustc. Scoped Graphify: 72 nodes / 134 edges. CocoIndex remains unavailable because `mcp.server.fastmcp` is missing.
- [x] Exact-package UI check: opened the update panel in the existing v0.1.36 window. The current/latest versions, release-note link, release-note text underneath it, and scrollbar were exposed; the control became enabled after the check. The update/restart action was not clicked. Evidence: `docs/evidence/issue9-f438820-manual-update-ui-ax-20261023.json`.
- [ ] Complete: workflow 15 remains partial until native screenshots and automatic update notification are preserved.

### Automatic native release notification — workflow 15 / DD-135
- [x] Plan: reuse `R-126 → BG-81 → SF-113 → UC-116 → FR-111 → QAS-122 → DD-135`; check immediately and every 600 seconds, keep checks asynchronous, poll an active update while its panel is closed, and show a localized accessible update button beside Settings.
- [x] Red: add deterministic ten-minute scheduling, pending-request deferral, closed-panel status polling, and AccessKit notification contracts; they fail before the native scheduler/button exists.
- [x] Green: use one egui repaint deadline, the existing Rust update dispatcher, and the existing SettingsAction route; avoid duplicate checks and idle polling. Defer scheduling until the initial snapshot arrives so response ordering stays valid.
- [x] Verify: 16 focused update/appearance/workspace tests, all-target native Clippy, default and native-probe builds, rustfmt, changed-file 300-line limits, and diff checks pass. The complete suite remains blocked by the separately tracked repository-wide 300-line migration.
- [x] Complete: native prototype workflow 15 now retains screenshot and automatic update notification behavior; shipped/default UI migration and cross-platform acceptance remain separate open work.

### Safe OpenCode MCP registration examples — GitHub #17 / DD-163
- [x] Plan: trace R-153 → BG-108 → SF-140 → UC-143 → FR-138/NFR-50 → QAS-149 → DD-102/DD-163; show examples only when no MCP server is registered, preserve source bindings, and keep sample servers inert JSONC comments.
- [x] Red: add a test that the empty native editor exposes local/remote OpenCode fields while parsing to empty settings; observe it fail before seeding the form.
- [x] Green: show examples only when the MCP server map is empty; preserve any existing source bindings, suppress the sample note for registered servers, keep examples commented, use `{env:VARIABLE}` and `.invalid`, and retain save-without-start behavior.
- [x] Verify: 16 tests across MCP, i18n, message-catalog, and source-layout suites; native-feature all-target Clippy (`-D warnings`); `cargo fmt --check`; `git diff --check`; offline native probe build; and 163/151-line Rust files. The existing single macOS preview window stays open and visibly shows examples for its empty server map. Graphify scoped extraction: 149 nodes / 305 edges. CocoIndex remains unavailable because `mcp.server.fastmcp` is missing.
- [x] Complete: commit `b8929bf` is pushed on the issue-linked branch `codex/issue-17-opencode-mcp-examples`; issue #17 is closed after acceptance, issue #9 progress is updated, and AgentMemory record `mem_mvap9360_680561d463cb` captures the handoff. The native UI remains opt-in and shipped UI parity is unchanged.
- [x] Task 3.2 / TDD first slice: port all five agent-tools-guide checks to `tests/agent_tools_guide.rs`; the new Rust check first failed while the Python unittest file existed, then passed after its removal.
- [x] Verify first slice: 5 Rust guide tests, all 55 remaining Python contracts, source-layout, formatting, diff check and all-target Clippy pass.
- [x] Task 3.2 second slice: port the MIT license/manifest/README consistency contract into `tests/project_license.rs`; its guard first failed while the Python module existed, then passed after removal.
- [x] Task 3.2 third slice: port stable-release triggers, validation dependency, permissions, archive/checksum publication and pinned-action checks into `tests/release_notes.rs`; remove the redundant Python release-workflow contract.
- [x] Task 3.2 fourth slice: port PNG/ICO/ICNS/RGBA container and dimensions, embedded-titlebar artwork identity, and Cocoa/Windows shell icon contracts to `tests/app_brand.rs`; remove the Python unittest file.
- [x] Task 3.2 fifth slice: port UTF-8 text-I/O and embedded UI charset contracts to `tests/utf8_sources.rs`; remove the Python unittest module.
- [x] Task 3.2 sixth slice / TDD: replace the Python/Node update-note display contract with a Rust integration test; prove the text-binding/scroll contract and absence of the prior Python module.
- [x] Verify sixth slice: Rust all-target regression 503 passed, 14 ignored, 1 slow source-layout test filtered; all 43 remaining Python contracts pass; formatting, all-target Clippy, diff and changed-file line-limit checks pass.
- [x] Task 3.2 seventh slice / TDD: move i18n catalog integrity, placeholder parity, explicit source references, static-markup checks and legacy-helper checks from Python into Rust; retain the live language-switch/user-content runtime regression in the existing Rust test target until the production WebView UI is replaced. The Rust ownership guard failed first while the Python module existed.
- [x] Verify seventh slice: 5 Rust localization-contract tests and 3 existing message/runtime tests pass; all-target Rust regression passes 515 tests with 14 ignored and the one slow source-layout test filtered; all 38 remaining Python tests pass; the source-layout test passes separately; formatting, all-target Clippy, diff check and the 307-line test-file limit pass.
- [x] Cross-platform TDD / Red: exact-source run 38817585058 passed macOS but Windows `mcp_remote::streamable_http_reads_sse_without_waiting_for_server_disconnect` timed out at the fixture's 3-second per-request deadline; the focused macOS test passes.
- [ ] Cross-platform TDD / Green: relax only the slow-runner fixture deadline while retaining its open-SSE/no-EOF assertion, then rerun exact-source Windows and macOS CI.
- [x] Task 3.2 remaining: no project-owned Python/JavaScript/TypeScript source or test files are tracked on the current issue branch (verified 2026-10-19); the old 19-module migration estimate is superseded.
- [ ] Task 3: finish exact-source cross-platform verification of Rust package assembly, license manifest, archive smoke checks and remaining release/manual tooling; then assess remaining Python verification paths.
- [ ] Verify: all workflows and cross-platform packages pass with no project-owned JS/Python runtime or build dependency; inspect archive contents and idle resource use.
- [ ] Complete: remove superseded authored JS/Python paths and CI steps, commit/push independently verified units, update documentation and release through the registered skill.

### 300-line source and test bound — R-158 / GitHub #9
- [x] Plan: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. Add the limit to `goal.md`, default planner rules, requirements, and architecture documents.
- [x] Red: change `tests/plan_rules.rs` to require 300 lines; confirmed it fails against the old default.
- [x] Red: lower `tests/source_layout.rs` to 300 lines; confirmed it fails and names the 57 oversized files.
- [x] Task / Green slices: split review/recovery CLI dispatch and native task-activity views/transcript formatting by responsibility; retain public dispatch/render paths.
- [x] Verify slices: 14 review/stop/recovery CLI tests and 5 native task-activity UI tests pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; all new files are below 300 lines.
- [x] Task / Green subunit: split src/native_ui_probe.rs into app, probe, and worker modules while keeping open_window and RequestWorker re-exports at their original paths.
- [x] Verify subunit: 88 tests across 16 native UI suites; native-feature all-target Clippy, rustfmt, and diff checks pass; the four files are 63/88/263/78 lines. The global layout guard now reports 53 remaining files.
- [x] Task / Green subunit: route settings/profile/MCP/messenger replies through poll/settings.rs; separate workspace refresh and error handling into poll/workspace_ops.rs and poll/errors.rs while preserving pending-state transitions.
- [x] Verify poller subunit: the same 88 tests across 16 native UI suites, native-feature all-target Clippy, rustfmt, and diff checks pass; poll.rs and its three child modules are 279/144/45/19 lines. The global layout guard now reports 52 remaining files.
- [x] Task / Green subunit: move the independent legacy i18n assertion into `tests/i18n_legacy_compatibility.rs`, reducing `tests/i18n_contracts.rs` from 307 to 294 lines.
- [x] Verify i18n test split: all 5 tests across both suites pass; rustfmt and diff checks pass; both files are below 300 lines. The global layout guard no longer lists this file and reports 51 remaining violations.
- [x] Task / Green subunit: split metadata-refresh and catalog-preservation checks into `tests/native_ui_profile_refresh.rs`, reducing `tests/native_ui_profiles.rs` from 326 to 293 lines.
- [x] Verify profile test split: all 12 tests across both suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; the files are 293 and 32 lines. The global layout guard reports 50 remaining violations.
- [x] Task / Green subunit: move missing-workspace-file denial coverage into `tests/guard_policy_missing_file.rs`, reducing `tests/guard_policy.rs` from 315 to 287 lines.
- [x] Verify guard-policy split: all 7 tests across both suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; the files are 287 and 65 lines. The global layout guard reports 49 remaining violations.
- [x] Task / Green subunit: split native task scheduling and waiting-state controls from `tests/native_ui_task_controls.rs` into `tests/native_ui_task_scheduling.rs`; preserve the task controls and timezone-aware scheduling assertions.
- [x] Verify task-control split: 6 tests across both suites, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 217 and 183 lines. The global layout guard reports 48 remaining violations.
- [x] Task / Green subunit: split Unix process approval and protocol tests into `tests/process_control.rs` and share the process fixture in `tests/support/process.rs`; lifecycle tests remain in `tests/process.rs`.
- [x] Verify process-test split: all 12 tests across both suites pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 159, 146, and 40 lines. The global layout guard reports 47 remaining violations.
- [x] Task / Green subunit: separate reviewed-license, license-notice, and upstream-byte checks into `tests/release_package_licenses.rs`; share only the package fixture with assembly/failure tests.
- [x] Verify release-package test split: all 7 tests across both suites pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 104, 211, and 58 lines. The global layout guard reports 46 remaining violations.
- [x] Task / Green subunit: move cooperative-worker process harness functions into `tests/support/cooperative_worker.rs`; keep all behavioral cases and assertions in `tests/cooperative_worker.rs`.
- [x] Verify cooperative-worker split: all 7 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 74 and 297 lines. The global layout guard reports 45 remaining violations.
- [x] Task / Green subunit: move the company CLI packet integration case into `tests/cooperative_cli.rs` and share setup/issue fixtures across both suites.
- [x] Verify cooperative CLI split: 12 tests pass and 1 remains ignored across both suites; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. The global layout guard reports 44 remaining violations.
- [x] Task / Green subunit: move the explicitly ignored local-agentmemory daemon round-trip case to `tests/cooperative/daemon_round_trip.rs` while retaining the existing shared fixtures and target.
- [x] Verify cooperative memory split: 11 tests pass and 1 remains ignored; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. The global layout guard reports 43 remaining violations.
- [x] Task / Green subunit: move native room stop/confirmation and candidate-decision contracts into a nested module under the existing integration target.
- [x] Verify native room split: all 12 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 293 and 163 lines. The global layout guard reports 42 remaining violations.
- [x] Task / Green subunit: move native launch/tool-decision request contracts into nested `tests/native_ui_launch/actions.rs` under the existing integration target.
- [x] Verify native launch split: all 12 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 295 and 185 lines. The global layout guard reports 41 remaining violations.
- [x] Task / Green subunit: isolate native Workspace persistence, profile/team, and Room probe cases in a nested module under `tests/native_ui_probe.rs`.
- [x] Verify native probe split: all 13 tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 239 and 258 lines. The global layout guard reports 40 remaining violations.
- [x] Task / Green subunit: move Git-reference, report/SQLite, notification, and run-knowledge persistence into `src/workflow/persistence.rs`; preserve the private workflow-level `save` path for child modules.
- [x] Verify workflow persistence split: 14 tests across workflow, recovery, checkpoint, and CLI suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 297 and 65 lines. The global layout guard reports 39 remaining violations.
- [x] Task / Green subunit: move frozen-evidence review orchestration to `src/workflow/review/orchestration.rs`; preserve `workflow::review`, coordinator-only review, and `review_as` visibility paths.
- [x] Verify review split: all 9 review tests pass, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 182 and 222 lines. The global layout guard reports 38 remaining violations.
- [x] Task / Green subunit: move subprocess/Git/workspace identity verification into `src/workflow/permissions/context.rs`; preserve `workflow::verify_tool_context` for CLI, scheduler, and native-guard callers.
- [x] Verify permission-context split: 8 tests across tool-context, live-decision, and guarded-control suites pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass; files are 252 and 88 lines. The global layout guard reports 36 remaining violations.
- [x] Complete subunits: commits `3c2640b` (review command dispatch), `599307b` (native task-activity views), and `1f4b155` (native UI launcher modules) are pushed to `codex/issue-9-accessible-workspace-menu`.
- [x] Knowledge tools: Graphify refreshed after the tool-controls split: 6,349 nodes / 10,545 edges / 484 communities. CocoIndex was invoked but its installed cocoindex-code environment fails to import mcp.server.fastmcp; no fresh CocoIndex index is claimed.
- [x] Red: the 300-line source-layout gate reported 34 violations before this slice, including `src/native_ui_probe/workspace/messenger.rs` at 339 lines.
- [x] Task / Green subunit: move `State::show` and its rendering helpers into `workspace/messenger/view.rs`, preserving `messenger::State::show`, request builders, event behavior, and dispatcher entry points.
- [x] Verify subunit: both `native_ui_messenger` tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 135 and 213 lines; the global source-layout gate now reports 33 remaining violations.
- [x] Complete subunit: pushed the messenger-view split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line gate reported 33 violations before this slice, including `src/native_ui_probe/room/task_controls.rs` at 417 lines.
- [x] Task / Green subunit: split task-control rendering and request validation into `view.rs` and `request.rs`, preserving the public `task_controls::show` and `task_controls::request` paths.
- [x] Verify subunit: 6 task-control/scheduling tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 41, 281, and 104 lines; the source-layout gate now reports 31 remaining violations.
- [x] Complete subunit: pushed the task-controls split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line guard reported 31 violations before this slice, including `src/ui/setup/share.rs` at 326 lines.
- [x] Task / Green subunit: split export, preview, and import/archive staging into focused child modules; preserve the existing `Startup` dispatch methods and `crate::ui::setup` visibility.
- [x] Verify subunit: 6 workspace-share/archive/messenger/MCP tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 41, 70, 37, and 192 lines; the source-layout gate now reports 30 remaining violations.
- [x] Complete subunit: pushed the settings-sharing split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line guard reported 30 violations before this slice, including `src/native_ui_probe/room.rs` at 497 lines.
- [x] Task / Green subunit: move the Agent Room view and decision controls into `room/view.rs`; preserve `room::show`, state, event, and renderer call paths.
- [x] Verify subunit: 19 render/language/tool-dialog/candidate tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 237 and 268 lines; the source-layout gate now reports 29 remaining violations.
- [x] Complete subunit: pushed the Agent Room view split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Red: the 300-line gate reported 29 violations before this slice, including `src/native_ui_probe/room_actions.rs` at 491 lines.
- [x] Task / Green subunit: move tool-request rendering and confirmation controls to `room_actions/tool_view.rs`; preserve `room_actions::tool_controls`, request validation, and action dispatch paths.
- [x] Verify subunit: 27 native launch/tool-dialog/Room-render tests pass before and after extraction; native-feature all-target Clippy, rustfmt, and diff checks pass. The files are 211 and 286 lines; the source-layout gate now reports 28 remaining violations.
- [x] Complete subunit: pushed the tool-controls split, updated GitHub #9, saved AgentMemory, and signaled Claude.
- [x] Verify: current tracked `src/` and `tests/` inventory contains no source file above 300 lines; `cargo test --offline --locked --test source_layout` passes. `src/ui/room.js` and other tracked JavaScript/Python application sources are absent.
- [ ] Verify: exact-source platform packages and runtime/UI acceptance; screenshot provenance, Korean IME, Windows accessibility/resource, and final license review remain open.
- [ ] Complete: issue #9 remains open until every current acceptance gate passes; do not start other issue triage first.

### Release package writer split — issue #9 / 300-line source bound
- [x] Plan: preserve `release_package::assemble` and package outputs while separating archive writing from package validation and license review.
- [x] Red: `source_layout` failed with `src/release_package.rs` at 350 lines; the seven release-package and license tests passed before extraction.
- [x] Green: move the private `PackagePlan` and `write_package` implementation into `src/release_package/archive.rs`; keep the public API and helper error paths unchanged.
- [x] Verify: seven release-package/license tests, native all-target Clippy (`-D warnings`), the locked application build, rustfmt, and diff checks pass. The strict source-layout test now reports 28 other files; the extracted files are 219 and 140 lines.
- [x] Complete subunit: pushed `6d763fa`, recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219267552), saved AgentMemory `mem_mvb5aiww_40844f57405f`, and signaled Claude (`sig_mvb5b1pu_c9527e792667`). The source-layout guard reports 28 remaining violations; issue #9 remains open.

### Release smoke package validation split — issue #9 / 300-line source bound
- [x] Plan: preserve `release_smoke::validate_package` and `release_smoke::run_smoke` while separating package validation, subprocess orchestration, and workflow scenarios.
- [x] Red: the 300-line source guard identified `src/release_smoke.rs` at 453 lines; the six exact-package smoke tests passed before the move.
- [x] Green: split the existing functions into `validate.rs`, `run.rs`, and `workflow.rs`, retaining the public re-exports and process timeout/error behavior.
- [x] Verify: all 6 release-smoke tests pass before and after; native all-target Clippy (`-D warnings`), locked application build, rustfmt, and diff checks pass. The strict source-layout test now reports 27 other files; new Rust files are 6, 74, 146, and 242 lines.
- [x] Complete subunit: pushed `ecf2414`, recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219375990), saved AgentMemory `mem_mvb5ie1k_0362c7154e2b`, and signaled Claude (`sig_mvb5ig35_7102ed248986`). The source-layout guard reports 27 remaining violations; issue #9 remains open.

### Native MCP tool discovery — workflow 12 / DD-98
- [x] Plan: extend the existing OpenCode-compatible MCP settings workflow; keep discovery explicit, bound it to the saved workspace revision and server, and reuse the background Rust dispatcher.
- [x] Red: add request serialization and AccessKit contracts first; the focused test failed because native tool discovery had no revision-bound request.
- [x] Green: expose saved-server selection and an accessible **Discover tools** action. Start discovery only on click, show the returned tool catalog, and poll the existing dispatcher once per second only while that MCP job is active.
- [x] Verify: 28 tests across the native MCP, native probe, and language suites; 4 backend MCP workspace/source-binding tests; native-feature all-target Clippy (`-D warnings`); formatting and diff checks pass. The full native-feature all-target regression passes every test except the strict `source_layout` guard, which reports 36 files above 300 lines. Every changed Rust source/test file is below 300 lines.
- [x] Complete: native Settings registration/discovery and Jira/Confluence preview, source review, and explicit apply are implemented in the opt-in prototype. Prototype parity is 24/25; shipped/default conversion remains 0/25.

### Native Jira/Confluence source review — workflow 12 / DD-68, DD-143
- [x] Plan: keep source references explicit as `jira:KEY-123` and `confluence:PAGE_ID`; reuse the existing MCP preview/apply commands and asynchronous Rust dispatcher; preserve the exact-preview, read-only human review gate.
- [x] Red: revision-bound preview/apply tests were added first and failed because the native request builders were missing; they now define the exact saved-revision and displayed-selection contracts.
- [x] Green: add source insertion, preview status, bounded record review, optional page selection, and explicit apply to the native Workspace; keep each source/test file at or below 300 lines.
- [x] Verify: 32 tests across six native/MCP suites, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. Full native-feature all-target regression passes all targets except the strict `source_layout` guard, which reports 36 files above 300 lines. Every changed Rust source/test file is at or below 300 lines.
- [x] Complete: pushed `3100a96` on `codex/issue-9-accessible-workspace-menu` and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6215970001). The LLM Wiki records workflow 12 as complete in the opt-in prototype; shipped/default UI conversion remains 0/25.

### Native task editor localization — workflow 11 / DD-160 (active)
- [x] Plan: pass the saved interface language into the native task editor and resolve visible labels and accessible names through the existing English/Korean message catalogs; leave task IDs, goals, prompts, paths, and agent output as user data.
- [x] Red: add a Korean AccessKit regression for the task editor's field labels, assignment/model controls, and review/save actions; it failed because `task_controls_in_language` did not exist.
- [x] Green: localize task editor headings, labels, controls, and review/save actions while preserving user data, edit validation, request digest, and the existing English `task_controls` entry point.
- [x] Verify: 11 task/editor and native language/catalog tests pass; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. Changed Rust source/test files are 279, 295, 99, and 273 lines, all within the 300-line limit.
- [x] Complete: pushed `88db903` on `codex/issue-9-accessible-workspace-menu` and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6216084493); the LLM Wiki records the slice. Workflow 11 and the 25-workflow denominator remain unchanged until all native controls are bilingual.

### Native role editor localization — workflow 11 / DD-160 (active)
- [x] Plan: pass the selected language into role assignment controls and translate role names, menu actions, model defaults, and accessible names while keeping agent/model IDs unchanged.
- [x] Red: add a Korean AccessKit regression for role headings, add/remove actions, model defaults, and accessible names; it failed because `show_role_rules_in_language` did not exist.
- [x] Green: localize role names, add/remove actions, default-model text, and accessible names while preserving the existing English `show_role_rules` entry point and registered-agent/model validation.
- [x] Verify: 5 role and message-catalog tests pass; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. Changed Rust files are 296, 86, 132, and 84 lines, within the 300-line limit.
- [x] Complete: pushed `d82e17d` on `codex/issue-9-accessible-workspace-menu` and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6216166951); the LLM Wiki records the slice. Workflow 11 remains partial until all native Workspace and Agent Room controls are bilingual.

### Native plan-review localization — workflow 11 / DD-160 (active)
- [x] Plan: route Review headings, progress, elapsed/output details, failures, logs, actions, and save guidance through the existing English/Korean catalogs; keep coordinator IDs, log paths, and agent-provided errors as data.
- [x] Red: add English/Korean catalog and interpolation regressions first; the focused test failed because the new Review messages were absent.
- [x] Green: localize the plan-review surface while preserving plan generation, task editing, execution consent, and existing error content.
- [x] Verify: 22 focused native/message tests pass; the native-feature all-target regression passes 202 tests across 58 suites with 4 ignored when excluding the source-layout guard; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. The strict `source_layout` test separately fails on 36 pre-existing files above 300 lines. Changed Rust/test files are 118 and 56 lines.
- [x] Complete: refresh the LLM Wiki and Graphify, commit/push only this slice, update GitHub #9 and AgentMemory, and signal Claude. Workflow 11 remains partial; prototype parity stays 24/25 and shipped/default conversion stays 0/25.

### Native workspace navigation localization and split — workflow 11 / DD-160 (active)
- [x] Plan: localize Workspace menu and confirmation copy in English/Korean, give the new-workspace field an accessible name, and split the oversized navigation module without changing request/event contracts.
- [x] Red: add Korean AccessKit tests for menu actions, dynamic workspace names, archive safety copy, dirty-workspace loss warnings, and create/open warning interpolation; the test first failed because `show_in_language` was absent, then exposed untranslated controls.
- [x] Green: keep the existing `show` entry point, add a context-based localized renderer, split view/confirmation/helpers into focused modules, and keep each changed source/test file at or below 300 lines.
- [x] Verify: 14 navigation/i18n tests pass; native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. The strict source-layout guard now reports 35 other files above 300 lines. Changed Rust/test files are 297, 209, 124, 13, 9, 111, and 300 lines.
- [x] Complete: refreshed the LLM Wiki and Graphify (6,264 nodes / 10,512 edges / 472 communities), committed/pushed this slice, updated GitHub #9 and AgentMemory, and signaled Claude. Workflow 11 remains partial; prototype parity stays 24/25 and shipped/default conversion stays 0/25.

### Native execution-consent localization — workflow 11 / DD-160 (active)
- [x] Plan: reuse the existing bilingual workflow requirement; localize only the exact-plan consent panel and accessible action names while preserving the selected revision/digest, command values, and public English `controls` entry point.
- [x] Red: the new bilingual AccessKit/render regression first failed because Workspace exposed no language-aware execution controls.
- [x] Green: pass the saved language from Plan Review, render all static consent labels through the existing English/Korean catalogs, and retain the original English wrapper and revision-bound start event.
- [x] Verify: 15 tests across execution consent, existing launch compatibility, and catalog contracts; native-feature all-target Clippy (`-D warnings`), formatting, diff checks, and the default all-target compile pass. Changed Rust source/test files are below 300 lines.
- [x] Tooling: Graphify refreshed the canonical tree (6,363 nodes / 10,574 edges / 492 communities). CocoIndex was retried; its launcher still fails to import `mcp.server.fastmcp`, so no fresh CocoIndex index is claimed.
- [x] Complete: committed and pushed as `a242d2a` on the issue #9 branch; posted [progress update](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218320478), refreshed the LLM Wiki/Graphify, and saved AgentMemory records `mem_mvb35m0t_ac1b8524bc43` and `mem_mvb3hgge_406665e57871`. The repository-wide 300-line guard still reports 29 other files; prototype parity stays 24/25 and shipped/default conversion stays 0/25.
- [x] Workflow 11 subunit: localize the native cache-cleanup panel in English and Korean; the broader visible Workspace/Agent Room bilingual audit remains open.

### Default-build native test feature gates — verification bug
- [x] Red: `cargo check --all-targets` failed in the default build because auto-discovered native UI tests referenced optional `eframe`/`native_ui_probe` without requiring `native-ui-probe`.
- [x] Green: add a regression that scans native UI integration test crates and requires either a crate-level feature gate or an explicit Cargo target requirement; gate the existing native-only test crates and list every explicit native test target.
- [x] Verify: feature-gate regression (2 tests), default `cargo check --all-targets`, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks pass. The 300-line repository gate remains red on 29 existing files.
- [x] Complete: included in `a242d2a`, pushed on the issue #9 branch, and tracked in the same [issue progress update](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218320478). AgentMemory bug record: `mem_mvb3hgge_406665e57871`.

### Native cache-cleanup localization — workflow 11 / DD-160 (active)
- [x] Plan: reuse the existing English/Korean catalogs for the native cache panel's heading, state copy, fallback path, action labels, and accessible name; keep paths and counts as data.
- [x] Red: add a rendered-copy regression for Korean and English preview, removed, loading, and empty states; it first failed because the language-aware renderer was missing.
- [x] Green: keep the existing English `show` entry point, add `show_in_language`, and pass the selected Workspace language through one cache-panel method.
- [x] Verify: 8 tests across native cache and language suites pass; default `cargo check --all-targets`, native-feature all-target Clippy (`-D warnings`), formatting, and the native probe build pass. Changed source/test files are 202 and 233 lines, within the 300-line limit.
- [x] Tooling: Graphify refreshed the canonical repository to 6,373 nodes / 10,593 edges / 487 communities. CocoIndex was retried but its CLI still cannot import `mcp.server.fastmcp`; no fresh CocoIndex index is claimed.
- [x] Complete subunit: refreshed the LLM Wiki and Graphify; saved AgentMemory `mem_mvb3rvo0_9541202c30e2`; sent Claude signal `sig_mvb3sfe1_a8dff65a4949`; and recorded [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218462009). Workflow 11 still requires a complete bilingual visible-copy audit; native prototype parity remains 24/25 and shipped/default conversion remains 0/25. Commit/push this verified slice separately.

### Native default launch and activity summaries — issue #9 / workflow 11
- [x] Plan: route the source-build `ui-new` and `ui` commands to the existing Rust eframe surface by default, and localize synthesized Agent Room activity summaries in English and Korean.
- [x] Red: the bilingual render regression first found the probe-only Workspace header and English-only generated session/activity summaries; the default-launch contract also failed before eframe became a default feature.
- [x] Green: remove the probe label, use existing message catalogs for generated session, task-list, usage, file-change, exit-code, and clipped-line summaries, and enable the existing native UI feature in default builds.
- [x] Verify: 14 focused native localization tests, the default build, all all-target suites before the layout guard, every all-target suite after it, native all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. The full locked all-target run stops at `source_layout`: 29 files still exceed 300 lines; every changed Rust/test file is below the limit.
- [x] Complete subunit: refreshed the LLM Wiki and Graphify (6,386 nodes / 10,610 edges / 488 communities), saved AgentMemory `mem_mvb55br8_bd0c3051a682`, pushed commit `1c3413e`, posted [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219183508), and signaled Claude (`sig_mvb55edm_6600327d067f`). The 29-file layout guard, full UI parity, and macOS/Windows package acceptance remain open.


### Quota display split — issue #9 / 300-line source limit

- [x] Plan: preserve the existing `quota` public API while moving rendering and time-label helpers into responsibility-based modules.
- [x] Red: the strict layout guard included `src/quota/display.rs` at 404 lines.
- [x] Green: split into `src/quota/display/mod.rs` (115), `labels.rs` (141), and `badge.rs` (156); existing exports and callers are unchanged.
- [x] Verify: quota display regression passes 8/8 before and after; locked application build, native-feature all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass.
- [ ] Verify: repository-wide 300-line guard remains red on 26 other files. Continue issue #9 until the guard is green and native/release acceptance is complete.
- [x] Knowledge: Graphify refreshed (6,413 nodes / 10,623 edges / 488 communities); AgentMemory `mem_mvb5qjfg_74253af00fa0`; Claude signal `sig_mvb5qlrj_00a813ca9aa4`. CocoIndex CLI is unavailable on this host.
- [x] Progress: posted the verified delta and remaining work to [GitHub issue #9](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219503212).


### Bilingual native UI completion — issue #9 / workflow 11 / DD-160

- [x] Plan: finish the English/Korean copy and accessibility audit for the native Workspace and Agent Room while keeping user/provider values unchanged.
- [x] Red: `native_ui_sidebar` failed on the hardcoded English avatar accessible name; `native_ui_status` failed on the hardcoded visible `Run` status prefix.
- [x] Green: route both labels through the existing English/Korean catalogs and interpolate the profile ID as data.
- [x] Verify: 33 tests across 14 UI/i18n suites pass; default application build and native-feature all-target Clippy (`-D warnings`) pass; rustfmt and diff checks pass. Changed source/test files remain below 300 lines.
- [x] Audit: targeted native UI scan found no remaining hardcoded English status/avatar labels; only the invariant `AgentConductor` product heading remains. Workflow 11 is complete in the source build, bringing native prototype parity to 25/25 (100%).
- [x] Knowledge: Graphify refreshed (6,416 nodes / 10,626 edges / 478 communities); AgentMemory `mem_mvb62w4q_5fedb050b69f`; Claude signal `sig_mvb62yin_0028abb68615`.
- [x] Progress: updated GitHub issue #9 to native source-build parity 25/25 while keeping released v0.1.24 conversion at 0/25; [posted the verification](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219714532).
- [ ] Complete issue #9: the published v0.1.24 remains WebView and release conversion is 0/25 until exact macOS/Windows packages and visual, IME/accessibility, resource, and license acceptance pass. Keep the issue open.

### Native launcher replaces the WebView shell — issue #9
- [x] Plan: keep the Rust eframe launcher as the default application UI and remove the unused Cocoa/WebView launcher and direct `wry`/`tao` dependencies.
- [x] Red: the branding regression referenced the deleted WebView2 implementation; replace it with a native eframe icon contract, which first failed because the native window had no icon.
- [x] Green: set the embedded AC RGBA icon on the native viewport, remove WebView-specific launch errors and dependency requirements, and update public install instructions.
- [x] Verify: focused release, i18n, branding, and Windows suites pass (9 tests); full all-target tests pass except the repository-wide `source_layout` guard, which still reports 25 files above 300 lines. Native-feature all-target Clippy (`-D warnings`), default all-target compilation, rustfmt, and diff checks pass. The optimized release build succeeds.
- [ ] Complete: exact macOS/Windows archive acceptance, visual/IME/accessibility/resource checks, and the remaining source-layout violations are still open.

### Remove the legacy browser document from the application — issue #9
- [x] Plan: remove the unused Rust HTML/JavaScript document assembly from `Room`; keep the existing Rust dispatcher and eframe surfaces as the only application UI.
- [x] Red: add a Rust regression that rejects production source references to browser UI assets; replace the old titlebar test with eframe rendering checks, which first failed because both native surfaces lacked the version label.
- [x] Green: remove `Room::document` and setup document assembly, retain the manual in the native viewer, add small version labels to Workspace and Agent Room, and remove obsolete DOM-only tests.
- [x] Verify: 39 tests across 10 focused native/UI suites pass; all-target compile, native-feature all-target Clippy (`-D warnings`), release build, rustfmt, and diff checks pass. Release binary scan found no browser-runtime or UI JavaScript markers.
- [x] Complete: tracked authored application/build/test source has no browser-runtime files; generated offline manual HTML remains a documentation artifact. The 300-line source/test guard passes. The negative asset-absence regression remains. Packaged workflow acceptance remains 15/25 and shipped/default Rust UI remains 0/25; see `docs/evidence/issue9-rust-source-inventory-20261024.json`.

### Retire the oversized Agent Room script — issue #9 / DD-41
- [x] Plan: delete `src/ui/room.js`; retain quota/state decisions in Rust and use native Agent Room rendering.
- [x] Red: `native_ui_assets` first failed because the 367-line script remained; `source_layout` also reported it above the 300-line limit.
- [x] Green: remove the script and its static-only task-row, quota, routing, icon, idle-poll, and IME assertions; keep Rust quota, room-dispatch, snapshot, and native UI coverage.
- [x] Verify: 21 tests pass and 1 environment-dependent icon test is ignored across 8 suites; all-target check, native-feature Clippy (`-D warnings`), optimized release build, rustfmt, and diff checks pass. The release binary contains no legacy JS asset names or linked WebView engine.
- [ ] Complete: repository-wide `source_layout` now reports 24 remaining files above 300 lines. Other unused startup/MCP/manual browser assets and their static tests remain; real-device Korean IME and exact platform package acceptance remain open.

### Enforce the 300-line authored-file limit — R-158 / DD-168

- [x] Plan: trace the limit through requirements, business goal, feature, use case, quality attributes, design drivers, and design decision; update the global plan rules.
- [x] Red: record the initial layout-check failure on 2026-10-16, which identified 56 authored files above the new limit.
- [x] Green: retain the strict Rust source-layout check and use 300 lines as the repository-wide maximum.
- [x] Verify: `cargo test --offline --locked --test plan_rules --test source_layout` passes (3 tests); `git diff --check` passes.
- [x] Complete this requirement: the current source-layout check reports no authored files above 300 lines and no inline production tests.
- [ ] Complete issue #9: full regression, exact macOS/Windows package validation, and native-window visual/accessibility/IME/resource acceptance remain outstanding.

### Native custom CLI model catalog editor — issue #9 / workflow 5

- [x] Plan: compare the legacy v0.1.24 custom-agent Models action with the Rust Workspace; implement the already-specified R-73 → UC-62 → DD-80 manual catalog flow without adding CLI discovery or an API endpoint.
- [x] Red: focused catalog tests failed to compile because the Rust profile editor had no manual catalog parser, editor state, or save event.
- [x] Green: add a parser with a 256 KiB total-input limit for newline/observed-JSON input, operator provenance, an accessible multiline editor, sidebar action routing, and revision-bound save/refresh handling in the existing local Rust controller.
- [x] Verify locally: `native_ui_catalog` (5 tests), `native_ui_sidebar` (5), `model_catalogs` (5), `native_ui_profiles` (10), `native_ui_profile_language` (1), `source_layout` (1), and `plan_rules` (2) pass offline with locked dependencies; all-target native-probe Clippy with `-D warnings`, rustfmt, and diff checks pass. The oversized-JSON regression fails without the byte guard and passes with it. The rendered AccessKit tree confirms the multiline field and Save button; clicking Save emits the expected catalog event and revision-bound request. All edited Rust source/test files are at or below 300 lines.
- [ ] Acceptance: this remains opt-in native-probe behavior and does not prove shipped UI parity. Physical native-window visual review, Windows package validation, and end-to-end persistence against the exact packaged app remain open; issue #9 stays open.

### Windows native renderer on hosted runners — issue #9

- [x] Red: exact-source Windows UI acceptance failed because the hosted runner exposes OpenGL 1.1, below eframe Glow's OpenGL 2.0 requirement.
- [x] Green: select eframe WGPU only on Windows and retain Glow on other targets; move the renderer regression into `tests/` to preserve test/source separation.
- [x] Verify locally: renderer and source-layout tests, complete all-target run except the renderer test's initial source-layout failure, native-feature Clippy, formatting, diff checks, and Windows-target dependency graph pass. The source-layout failure was corrected and retested independently.
- [ ] Verify remotely: rerun exact-source Windows build, native window/UI Automation, and package smoke on this commit; local Windows target compilation is unavailable because the Windows Rust standard library is not installed.

### Native Workspace review corrections — issue #9

- [x] Plan: compare the live Rust Workspace against the shipped flow and design guide; keep user acceptance separate from control-migration counts.
- [x] Red: rendered Workspace tests exposed single-line goal input, insufficient light/dark input boundaries, a missing Workspace-step coordinator, misplaced cache cleanup, and missing agent-card controls/details.
- [x] Green: use a multiline goal field and paired input-border tokens; move coordinator selection into Workspace settings; place cache cleanup under Settings and open it in its own window; expose model refresh, edit, remove, executable, permission, and fallback details in the agent rail; style the three steps as framed tabs with the selected step title.
- [x] Verify: 28 focused tests across Workspace, sidebar, cache, appearance, header, and probe suites pass; all-target Clippy (`-D warnings`), formatting, and diff checks pass locally. Every changed Rust/source-test file remains below 300 lines; the repository-wide layout test was not run for this slice.
- [x] Verify: local all-target regression, `-D warnings` Clippy, formatting, diff checks, and a one-window macOS review of the exact local optimized Rust prototype pass. The review covered Workspace, Team, agent profile controls, Settings/cache close, MCP, messenger settings, Review Plan's pre-generation gate, Agent Room, manuals, tools guide, menu, and workspace archive confirmation; live subscription, external MCP, real delivery, and generated-plan execution were deliberately not triggered.
- [ ] Verify: Windows package/UI Automation, real Korean IME, full keyboard/screen-reader traversal, refreshed Rust screenshots, and end-to-end live CLI flows remain pending.
- [ ] Complete issue #9: the historical 25-row source inventory does not prove parity; verified replacement acceptance remains 0/25 until compared against the existing app end to end. Do not merge or release before that review and platform acceptance.

### Native MCP editor keyboard traversal — issue #9 / workflow 11
- [x] Plan: keep the MCP JSONC editor monospaced while allowing Tab to move focus to Save; preserve the explicit save action and do not change settings unless the user activates it.
- [x] Red: add a rendered keyboard regression; it reproduced Tab being inserted into the JSONC example because `TextEdit::code_editor()` locks focus.
- [x] Green: set the multiline editor's font to Monospace without enabling the focus lock.
- [x] Verify: the regression confirms Tab does not mutate editor contents and Enter after Tab emits the existing Save event; 16 native MCP/source/dialog tests, targeted native-feature Clippy (`-D warnings`), formatting, and diff checks pass.
- [x] Knowledge: saved the root cause and regression contract to AgentMemory (`mem_mvdi7hab_376aafb21da0`).
- [ ] Complete issue #9: exact macOS/Windows package review, current native UI screenshots, IME/screen-reader coverage, repository-wide workflow parity, and remaining end-to-end acceptance stay open; this focused fix does not change the issue completion status.

### Workspace goal selection from phase summary — issue #9
- [x] Plan: compare the native phase summary with the existing goal registry behavior; reuse the established revision-checked `select_goal` request and per-goal run archival.
- [x] Red: add a rendered interaction test for an inactive goal and its revision-bound request; it first failed because the native summary only exposed the active goal and had no selection request.
- [x] Green: list goals when more than one exists, mark the active goal accessibly, disable selection while busy, dispatch selection through the existing controller, and refresh the workspace after success with a localized failure message.
- [x] Verify: 14 focused goal, workspace, layout, and plan tests pass; native-feature all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` pass. Every changed source/test file is at or below 300 lines.
- [x] Verify / full local regression: `cargo test --offline --locked --all-targets` passes with no failed tests when run with the temporary compatibility-library path required by this host's Homebrew LLVM (`libz3.5.1.dylib` is absent at its configured path); native-feature all-target Clippy (`-D warnings`), `cargo fmt --all -- --check`, and `git diff --check` pass. This host setup is not an application dependency.
- [x] Knowledge: update the LLM Wiki and Graphify (6,574 nodes / 10,517 edges / 512 communities); save AgentMemory `mem_mvdssk66_c90c90d5cc70` and send Claude signal `sig_mvdssmvl_a78ab3b6ab94`. CocoIndex is not installed on this host.
- [ ] Acceptance: inspect selection in the exact packaged macOS/Windows apps and keyboard/screen-reader traversal. This widget slice does not change issue #9's accepted-workflow count; keep issue #9 open.

### Native Resume gate preserves integration repair — R-123 / DD-132 / issue #9
- [x] Plan: sync latest `main` failure-resume guard into the #9 branch, compare it with R-123/UC-113/QAS-119, and keep ordinary failed-worker resumes blocked.
- [x] Red: `failed_integration_run_can_resume_the_coordinator_repair_path` failed under the upstream blanket guard with the localized “cannot be resumed” error.
- [x] Green: apply the existing `workflow::repairable` predicate at the native Resume gate; repairable integration failures enter `resume-due`, while non-repairable failures remain rejected before a UI job starts.
- [x] Verify: `ui_failed_resume`, `repair`, and `ui` pass (9 tests); native-feature all-target Clippy (`-D warnings`), formatting, diff checks, and the 300-line source-layout test pass. Local Rust commands require `DYLD_LIBRARY_PATH=/private/tmp/ac-z3-compat` because the host's Homebrew LLVM references a Z3 soname alias absent at its configured path.
- [x] Complete: update DD-132 and the LLM Wiki/Graphify, save AgentMemory `mem_mvdydinm_166904581fe0`, signal Claude `sig_mvdydite_f8f0c5349589`, and commit/push this unit. Post the verification to issue #9; its 0/25 acceptance count and open status remain unchanged.

### Issue #9 current verification — exact macOS source package
- [x] Verify code: all-target offline locked tests pass with zero failures; `source_layout` and both `plan_rules` tests pass. All-target Clippy (`-D warnings`), rustfmt, and diff checks pass.
- [x] Verify package: optimized macOS build and Rust `package-release` succeed for `df591eb80ae2`; package validation confirms 214 dependencies, exact source/binary hashes, and the generated package smoke completes all four approval-gated workflow combinations. Archive SHA-256: `8efa4ec56094f740c4b6d704ba4d577cd6b514d8309509fd95926c072767d7a7`.
- [x] Verify UI: replaced the prior app process before opening the exact packaged app, keeping one AgentConductor process. Visually checked Workspace and MCP settings; keyboard Tab moves from the unchanged JSONC editor to Save. Closed MCP settings without saving.
- [ ] Remaining acceptance: Windows exact-package/UI Automation, current user-safe screenshots for README/manuals, real Korean IME and screen-reader coverage, sustained resource/license review, and end-to-end validation of all 25 workflows. Verified replacement acceptance remains 0/25; keep issue #9 open.

### Issue #9 — pending approval receipt reconciliation and upstream UI integration
- [x] Plan: retain the #9 native UI implementation while integrating latest `main`; keep #9 acceptance gates and issue status open until full workflow parity is proven.
- [x] Red: the native worker regression reproduced two active receipts for one identical task/session/tool request when the scheduler re-verified a changed workspace fingerprint after the worker paused.
- [x] Green: atomically supersede only matching pending/approved receipts with stale context before recording the fresh request; preserve audit rows, consumed/denied decisions, and distinct tool calls.
- [x] Verify: full native-feature offline locked all-target suite passes; focused ledger, worker, coordinator-request, native UI feedback and source-layout tests pass; native-feature all-target Clippy (`-D warnings`), `cargo fmt --all -- --check`, and `git diff --check` pass. Remaining: package/platform UI acceptance.
- [x] Knowledge: update the live tool decision wiki, refresh Graphify (6,603 nodes / 11,455 edges), and save AgentMemory `mem_mvdvmb6z_aec55d53b996`. CocoIndex is not installed on this host.
- [ ] Complete #9: package/platform acceptance and the remaining workflow parity gates listed above stay open.

### Native sidebar provider icons — issue #9 / DD-40
- [x] Plan: preserve the existing bounded provider-icon data URL in native UI snapshots and render installed provider artwork in the eframe sidebar; keep monograms for custom agents and unavailable icons.
- [x] Red: add a native rendered-widget regression proving a valid provider icon produces image geometry and suppresses the provider monogram; it failed before native rendering was implemented.
- [x] Green: decode the existing bounded PNG data URL with the installed image dependency, cache the egui texture by provider URL, and render it in the shared agent avatar while requesting icons in workspace and room snapshots.
- [x] Verify: focused icon/sidebar/workspace/source-layout tests pass (22 passed, one installed-app test ignored); the full native-feature offline all-target suite passes with zero failures. All-target native-feature Clippy with `-D warnings`, formatting, and diff checks pass. On macOS, the optimized Rust app was opened with three fake `/usr/bin/true` profiles; Claude and Codex installed-app artwork appeared in the sidebar and the custom agent retained its monogram.
- [x] Package: committed source `fe9fea5631b5517c0330b5ae57d93be3e774953b` produced a validated macOS archive with 214 dependencies; the packaged executable SHA-256 matches the visually inspected optimized binary (`82ea0135188c92d9be13cb41bdde75b435405e6aab42defec32cc1748c2a55a3`). Package smoke passed all four shared/worktree × Git/folder modes, each with three tasks/tests, approved merge verified and unapproved merge refused. Archive SHA-256: `458aef6cc5331f93ce77101d2d85e0edfa616ed1e243319111989d711b31e2ab`.
- [ ] Platform/issue acceptance: verify Windows icon discovery/rendering on Windows and complete all end-to-end UI workflows. This slice does not change verified workflow acceptance from 0/25; keep issue #9 open.

### Full native regression after latest-main sync — issue #9
- [x] Verify: run the complete offline locked native-feature all-target suite against `0a5b6b1`; the sandboxed attempt exposed four local TCP bind denials in AgentMemory contract tests.
- [x] Verify / TDD environment: rerun the same command with local loopback permission; all 659 tests pass and 14 are ignored across 230 suites (193.74 seconds).
- [x] Verify: native-feature all-target Clippy (`-D warnings`), `cargo fmt --all -- --check`, `git diff --check`, and the 300-line source-layout test pass. This host requires `DYLD_LIBRARY_PATH=/private/tmp/ac-z3-compat` for Homebrew LLVM's missing Z3 soname alias; it is not an app runtime dependency.
- [x] Cleanup: after confirming the test process exited, remove 92 generated mock executables, 21 recent completed workflow fixture folders, and the orphaned 71-minute `AgentConductor-issue9-fixture.app` whose isolated home no longer existed; retain historical failed-test evidence and package archives.
- [x] Cleanup follow-up: remove the remaining 350 `ac-workflow-mock-*` binaries and 61 `ac-flow-*` test repositories after confirming no test or application process was using them; preserve test logs and validated package archives. The repository fixture helper is the source of both prefixes.
- [ ] Acceptance: exact Windows UI/package validation and complete end-to-end parity across all 25 workflows remain open; verified replacement count remains 0/25.

### Native agent-profile popup parity review — issue #9
- [x] Plan: rebuild the current Rust source and compare the agent profile editor directly with the existing MCP settings popup after the popup parity concern.
- [x] Task: build HEAD `d8658d31a662bdd1790bfdb5266560f13fcbcce6` and open the optimized binary in one temporary macOS app bundle with a disposable workspace and a `/usr/bin/true` custom profile.
- [x] Verify: at 1140×772, both centered, resizable settings windows use the same light title-bar frame, close control and application typography. Their body layouts differ to fit their separate forms. Close the editor and MCP window without saving; confirm no AgentConductor/test process remains and remove the temporary app/workspace.
- [x] Complete: no source correction was needed; the previously fixed popup frame matches. This component check does not advance the 0/25 end-to-end workflow acceptance or complete #9.
- [ ] Acceptance: Windows packaged visual comparison and all end-to-end UI workflows remain open.

### Generated plan review journey visibility — issue #9
- [x] Plan: verify the native review screen shows every task returned by a generated plan before the user approves execution.
- [x] Task: extend the existing mock-agent journey to check all three task ID/instruction summaries in the AccessKit tree before exact-plan consent.
- [x] Verify: `cargo test --offline --locked --test native_ui_start_journey --test source_layout` passes locally; the three generated task rows, pending start feedback, and Agent Room transition are exercised in one journey. The 300-line source/test guard passes.
- [ ] Acceptance: this is native-probe journey evidence, not exact packaged macOS/Windows UI acceptance; end-to-end replacement acceptance remains 0/25.

### Agent Room four-stage workflow visibility — issue #9 / DD-161
- [x] Plan: compare the current Rust Agent Room with the v0.1.24 screen; restore its `Plan → Tasks → Verify → Complete` progress context without adding a second source of run state or making steps actionable.
- [x] Red: the AccessKit test failed before the progress steps existed; add current-stage expectations for prepared, active, approval, completed, and failed reports in English and Korean.
- [x] Green: render the four existing localized labels from the report state; distinguish failed task execution from a failed verification after all planned tasks completed.
- [x] Verify: final offline all-target run passes 667 tests, with 14 ignored and 0 failed across 235 test targets; this includes the failed-task and failed-verification stage cases. All-target Clippy (`-D warnings`), changed-file rustfmt, and `git diff --check` pass. The exact isolated macOS app shows `03 Verify` for a three-of-three completed candidate awaiting review; the current label is in AccessKit and the 792×532 layout test keeps all stage labels inside the viewport.
- [x] Knowledge: update the LLM Wiki and Graphify (6,728 nodes / 11,666 edges), save AgentMemory `mem_mvelvv92_2b278ee1838f`, and send Claude signal `sig_mvem4axx_855da1382df5`. CocoIndex is unavailable in this host environment; no dependency was added.
- [x] Cleanup: remove the regenerated Rust incremental cache (2.8 GiB) and temporary compiler wrappers/logs after checks. The observed `/private/tmp` total is 535 MiB, not 41 GiB; the remaining 7.9 GiB is reusable build output and the isolated app fixture/evidence is retained.
- [ ] Acceptance: this focused visual improvement does not prove an end-to-end workflow and does not change #9's verified replacement count (0/25); Windows package/UI, screen-reader/IME, and all workflow acceptance remain open.

### Agent Room integration verification summary — issue #9 / DD-161
- [x] Plan: restore the existing app's visible integration-verification summary below the native Agent Room session list, using only report test records.
- [x] Red: render a report with two passing tests and one failure; the regression initially failed because the Rust sidebar had no verification summary.
- [x] Green: display a localized passed/total count and evidence-derived status; omit raw stdout/stderr paths and avoid a new coordinator state.
- [x] Verify: focused room-render and source-layout tests pass; `cargo test --offline --locked --all-targets` exits 0; all-target Clippy with `-D warnings`, changed-file rustfmt, `git diff --check`, and local binary build pass. The rebuilt isolated macOS app's AccessKit tree shows “Integration verification”, “3 / 3 task tests passed”, and “Completed”; no approval or merge action was triggered.
- [x] Knowledge: update the LLM Wiki and Graphify (6,736 nodes / 11,677 edges), save the AgentMemory slice, and send Claude the signal after commit.
- [x] Cleanup: after verification, remove only this task's temporary app fixture/home and compiler alias plus regenerated `target/debug/incremental`; leave other `/private/tmp` worktrees and user directories untouched.
- [x] Complete subunit: push the focused change and post its evidence to GitHub #9. Do not close #9 or change its 0/25 end-to-end acceptance count.

### Agent Room Human Review rail — issue #9 / DD-161
- [x] Plan: restore the released left-rail review context without duplicating approval/merge controls or report state.
- [x] Red: an AccessKit/pointer regression first failed because no Human Review section or candidate-review button existed in the session rail.
- [x] Green: show the localized candidate-review button, pending tool-request count, and report status. The button switches to Review and asks for evidence only when its revision is not current.
- [x] Verify: 22 focused tests pass across the review-summary interaction/render, verification/stages, feature-gate, existing room review, and source-layout targets; after commit `2ec925b`, the full `cargo test --offline --locked --all-targets --quiet` run also exits 0. All-target Clippy (`-D warnings`), changed-file rustfmt, diff checks and local binary build pass. The exact isolated app's AccessKit tree exposes `HUMAN REVIEW`, `Review candidate`, `Tool requests · 0`, and the report status; the pointer regression proves the link requests only missing/stale evidence and never approves or merges.
- [x] Knowledge: add the LLM Wiki record and refresh Graphify (6,747 nodes / 11,693 edges); save AgentMemory and signal Claude after push.
- [x] Complete subunit: push the focused UI change and update GitHub #9. Overall #9 stays open at 0/25 verified end-to-end.

### Native Workspace source-build visual check — issue #9 / DD-137
- [x] Plan: compare the current Korean Workspace screen with `docs/design-guide.md` in light and dark themes at 1140×772; use the current Rust branch binary, not the installed WebView app.
- [x] Verify: the Rust executable at `2ec925b` rendered one temporary Workspace window. CUA captured the actual window in light and dark themes; title, navigation, form fields, Jira/Confluence references, isolation choice, save action, and settings import/export fit within the viewport and remain readable. The AccessKit tree exposes the same controls. This covers only the empty Workspace screen.
- [x] Knowledge: refresh Graphify (6,749 nodes / 10,780 edges / 527 communities). CocoIndex CLI is unavailable in this host.
- [x] Cleanup: close the temporary app and remove its temporary bundle/workspace; leave shared issue fixtures and build outputs intact.
- [ ] Acceptance: populated agent cards, profile/MCP dialogs, Agent Room pixels, public screenshot assets, Korean IME, Windows UI/package, and end-to-end workflow acceptance remain open. Issue #9 stays at 0/25.

### Agent Room candidate review visual check — issue #9 / DD-161
- [x] Plan: validate the current Rust Agent Room against a fresh, isolated three-task mock run; inspect light and dark themes at 1140×772.
- [x] Verify: the mock run reached `waiting_approval` with 3/3 tasks and 3/3 tests successful. The current Rust app displayed the approval state, four-stage progress, task list, test evidence, candidate review rail, and approval controls in both themes. Loading latest candidate evidence showed the current revision and passing test results; no approval or merge action was taken.
- [x] Cleanup: close the temporary app and remove its 77 MB bundle, isolated run data, and unused compiler alias. Leave reusable build outputs and unrelated temporary directories intact.
- [ ] Acceptance: this is mock-run UI evidence only. Windows/package, real-agent execution, actual human approval/merge, Korean IME, screen-reader coverage, and all 25 end-to-end workflows remain unaccepted; issue #9 stays at 0/25.

### Inline quota refresh spacing — issue #9 / R-64 / UC-54 / DD-137
- [x] Plan: keep each paid agent's quota value and refresh action together, with the specified 5px gap in the compact sidebar.
- [x] Red: `quota_value_and_refresh_keep_the_required_five_pixel_gap` measured an 8px gap at the 248px rail width and failed.
- [x] Green: group the quota value and refresh/spinner controls in a local horizontal row with 5px spacing; preserve quota tooltip/accessibility details and independent refresh behavior.
- [x] Verify: the new rendered AccessKit geometry test passes; sidebar density, roster, icons, quota formatting, and 300-line source-layout suites pass (16 tests total). Focused Clippy (`-D warnings`), rustfmt, and `git diff --check` pass. Changed files are 282 and 176 lines.
- [x] Knowledge: Graphify refreshed to 6,754 nodes / 10,785 edges / 518 communities. CocoIndex structural grep confirmed the sidebar row structure; the repository is not initialized for semantic indexing.
- [ ] Acceptance: inspect the exact packaged macOS and Windows card layout with provider data; this focused spacing fix does not advance #9's verified end-to-end acceptance (0/25).

### Exact-source macOS package — issue #9 / commit 7df4a93
- [x] Build: optimized Rust release build succeeds for `aarch64-apple-darwin`.
- [x] Verify package: `package-release` and `validate-package` pass for exact source `7df4a93d5d347e7a80e523cd2826a170ba646c12`; manifest records one 17,560,864-byte executable, 214 dependency license records, and binary SHA-256 `9d9772bc2e27e23deb588e9e1a46bf2a88cced35c0182ae9a144494fc4500e83`. ZIP SHA-256: `28ce1d33f465af229a955470cd56f4234673a4a58cfec80347ba87f42e20dff3`.
- [x] Smoke: `package-release` completes the four approval-gated shared/worktree × Git/folder cases.
- [x] Cleanup: close the package UI and terminate its CLI launch process; remove this check's temporary HOME, workspace, and compiler compatibility wrapper.
- [x] UI acceptance / empty Workspace only: first GUI attempt was invalid because a separate CUA launch showed the default Workspace; no UI action or Save occurred. Corrected on 2026-10-19 by launching the exact packaged `.app` once with `open -n -a ... --args ui-new <isolated-workspace>`, verifying its process command line, and attaching CUA to that same process. The package displayed a fresh empty Workspace at 1140×772 with setup fields and disabled Coordinator; screenshot and accessibility tree came from the exact package. No user data was touched.
- [x] Verify / populated card spot-check: launched the same exact package once with a disposable HOME/workspace and added a Claude-parser fixture whose executable is `/usr/bin/true`. At 1140×772, the card showed provider icon, status, session count, model refresh, compact quota value/refresh, edit/remove and reorder controls in the exact package screenshot and accessibility tree. Clicking quota refresh returned the trusted-project-folder requirement; no Claude CLI or account query ran.
- [x] Cleanup: remove the task-owned `/private/tmp/ac-package-ui-review-e6d7ca` containing only the isolated HOME/workspace and generated startup/lock files; retain other package archives and test directories.
- [x] Cleanup / card check: close the sole package process and remove `/private/tmp/ac-package-cards-review-c4692a0` after verifying its exact process exited.
- [ ] Acceptance: real provider quota/model content, Windows packaged UI, saved screenshots, and end-to-end workflows remain open. These visual spot-checks do not change #9's 0/25 end-to-end acceptance.

### Availability retry packaged follow-up — issue #9
- [x] Verify: the exact packaged macOS binary emitted a simulated `usage_limit_reached` event and entered `Waiting availability` for all three tasks. The selected-task inspector exposed the restriction reason, timezone-labelled schedule field, and `Schedule retry` action.
- [x] Verify existing regression: `cargo test --features native-ui-probe --test native_ui_task_scheduling` passed all 3 tests, including waiting-state/timezone validation, bilingual accessible controls, and persistence through the real Room dispatcher.
- [x] Cleanup: close the owned package process and availability watcher, then remove the 41 MiB isolated package/HOME/workspace fixture and temporary Z3 compatibility link. Removed 1,145 stale test mock executables from the user temp directory (774 MiB); unrelated temp data and active worktrees were retained.
- [x] TDD: reproduce the fixture leak with `compiled_mock_stays_under_cargo_target_temporary_directory`, then route shared process/workflow test binaries and run roots into Cargo's `target/debug/tmp`.
- [x] Verify: `cargo test --offline --locked --all-targets` completed with all executed tests passing; environment-dependent CLI/account tests remained ignored. The scheduling suite passed 3/3, `git diff --check` passed, and new fixtures were observed under the build temp directory.
- [x] Cleanup: generated test fixtures now stay under the removable Cargo build tree. After the issue #36 worker finished, removed its disposable 3.4 GiB Cargo target cache while preserving the 22 MiB source worktree and PR; `/private/tmp` returned to 466 MiB. The current user temp directory is 804 MiB and no longer contains the prior 774 MiB stale-binary backlog.
- [ ] Acceptance: CUA keyboard entry was translated by the active Korean IME, so this run did not verify the schedule click or due-time resume. Existing Rust integration coverage passes; Korean IME, Windows, and full workflow parity remain open. Do not change #9's 0/25 end-to-end acceptance.

### Native failed-plan recovery — issue #36
- [x] Plan: trace the Rust `Prepare` request through `Room::dispatch`, its follow-up snapshot, the Review error panel, and retry eligibility. Add R-159 → BG-114 → SF-146 → UC-149 → FR-144/NFR-54 → QAS-155 → DD-169.
- [x] Red: add regressions for stale-failure suppression, timeout recovery eligibility, saved/clean input gating, and visible retry action; the first focused build failed because the recovery predicates were absent.
- [x] Green: distinguish `PrepareRefresh`, show localized preparation state, hide stale failure while preparing or reporting the request error, and provide a direct retry button for a failed plan with saved inputs.
- [x] Verify: focused Review regressions (4/4), bilingual localization (1/1), and `cargo test --offline --locked --all-targets --quiet` pass locally. `cargo clippy --offline --locked --all-targets -- -D warnings`, `cargo fmt -- --check`, and `git diff --check` pass. Live subscribed CLI tests remain ignored by their existing environment gates.
- [x] Knowledge: update the planning feedback Wiki and save AgentMemory `mem_mvesb4rq_864b9fc94b1f`. Graphify refreshed the source graph (6,796 nodes / 11,765 edges) and its retry query confirmed the Prepare/Review path. CocoIndex is unavailable because the installed CLI cannot import `mcp.server.fastmcp`.
- [x] Delivery: bug PR #37 merged to `main` as `aa5606b`; issue #36 closed. Ported the fix to `codex/issue-9-native-ui-parity` as `ce1e9ab`. The current full Rust test suite, strict all-target Clippy, rustfmt, and diff checks pass; exact packaged native UI acceptance remains part of #9's 0/25 end-to-end gate.

### Custom CLI JSONL event normalization — issue #9 / Task 2.21 subunit
- [x] Plan: carry the existing custom CLI event contract into the Rust migration branch without changing built-in provider parsing or execution authority.
- [x] Red: a parallel mock-agent workflow using vendor event names first failed to reach review because the terminal event was unknown.
- [x] Green: add bounded custom event-name mappings and a session-ID JSON Pointer to `AgentProfile`; use the profile-aware decoder in live progress, worker completion, recovery, and coordinator parsing.
- [x] Verify: focused protocol, plan, coordinator, progress, recovery, and parallel workflow suites pass; all-target Clippy (`-D warnings`), rustfmt, `git diff --check`, and the source-layout guard pass. A full offline locked all-target run passed before the last input-bound checks; its repeat exposed one `cli_workflow` directory-cleanup race. That test passes by itself. CocoIndex is unavailable on this host.
- [x] Knowledge: record the parser contract in `knowledge/wiki/custom-cli-jsonl-normalization.md`; Graphify AST extraction reports 6,827 nodes and 11,838 edges.
- [ ] Remaining Task 2.21: strict machine-local registry loading, native profile editor fields, and custom quota declarations remain open. This subunit does not change #9's accepted workflow count (0/25).

### Numbered phase action label — issue #9 / native Workspace
- [x] Plan: match the legacy `Plan phase n` action label and keep Rust source files within the 300-line limit.
- [x] Red: an AccessKit regression for `Plan phase 1` failed while the native button used the generic `Plan next phase` label.
- [x] Green: render the localized phase-number label; move the small label helpers into the existing `goals/` module to keep the parent file under 300 lines.
- [x] Verify: goal-list (3), phase-progress (9), native-start-journey (1), source-layout (1), formatting, diff check, and all-target native-feature Clippy (`-D warnings`) pass.
- [x] Knowledge: document the label parity rule; refresh Graphify (7,095 nodes / 11,333 edges / 539 communities); save AgentMemory `mem_mvfik3wp_637a29f437ae`.
- [ ] Acceptance: this component check does not advance issue #9's end-to-end replacement count (0/25); packaged Windows/macOS workflow acceptance remains open.

### Exact-source macOS package and launch — issue #9 / `fd177ea`
- [x] Build/package: `package-release` succeeded on exact source `fd177ea145326407d485d7f18ccd1d916f1fc00e`; `validate-package` confirms 214 dependency licenses and binary SHA-256 `6bd00e6f520e0172a0b1f912c15fe558695d4a3a6c2d80477395b9da7cae6c88`. ZIP SHA-256: `b4d17e6b8ecb152e4056aa2e39d1cabe4e9f7880d8398435c3bb2d132578e6e5`.
- [x] Smoke: four isolated Git/folder × shared/worktree cases passed; each ran three parallel tasks and tests, refused unapproved merge, and accepted approved merge.
- [x] UI launch: opened one exact-package native window with an isolated Workspace, confirmed the Rust Workspace accessibility tree, and closed the app. No authenticated CLI was invoked.
- [x] Focused verification: goal-list (3), phase-progress (9), native start journey (1), source-layout (1), briefing (6 serial), formatting, diff check, and all-target native-feature Clippy pass.
- [x] Broad verification: `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` passes: 719 passed, 14 ignored, 0 failed across 247 test targets. The run used a task-local rustc/Z3 compatibility wrapper for this host's Homebrew linkage.
- [x] Test harness cleanup: egui UI tests now clear unconsumed texture deltas; the stale Korean empty-catalog expectation now says `모델 미조회`; affected native UI integration targets have the required feature gate.
- [x] Knowledge: Graphify refreshed to 7,099 nodes / 11,337 edges / 553 communities; AgentMemory `mem_mvfjlfao_8b19f0823daa`.
- [ ] Acceptance: the launch shows only the empty Workspace; exact packaged end-to-end UI parity, Windows package/UI checks, and issue #9's 0/25 workflow acceptance remain open.

### Native agent-role table width — issue #9
- [x] Plan: keep all five role-table columns inside a standard desktop content width; preserve full agent/model identifiers through hover help.
- [x] Red: the 820px rendered-layout regression overflowed with the English coordinator header and long registered IDs.
- [x] Green: use the shorter existing coordinator label, compact selector labels with full-value tooltips, and size selectors for the available table width.
- [x] Verify UI: all 6 `native_ui_roles` tests pass; the exact Rust preview shows the Korean role table with every column visible at 1140×772. English long-ID geometry and Korean empty-registry geometry both fit the 820px test viewport.
- [x] Verify: `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1`, `cargo clippy --offline --locked --all-targets --features native-ui-probe -- -D warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass; all changed Rust/test files are at or below 300 lines.
- [x] Knowledge: LLM Wiki article and index updated; Graphify refreshed to 7,107 nodes / 11,346 edges / 552 communities; AgentMemory record `mem_mvfkvrsw_dab186986dfd` saved.
- [x] Complete this slice: progress posted to GitHub #9; commit `570f080` pushed. CocoIndex is not installed in this workspace. Windows packaged rendering and issue #9's 0/25 end-to-end acceptance remain open.

### Responsive role selector width — issue #9
- [x] Red: verify desktop selectors meet the legacy 130px minimum; the regression first measured the native 108px width.
- [x] Green: use 130px when the content area is at least 900px and retain 108px on compact layouts so long identifiers still fit.
- [x] Verify: five focused native UI suites pass (28 tests); the full offline all-target suite passes (745 passed, 14 ignored, 0 failed across 252 suites); native-feature all-target Clippy with warnings denied, source-layout guard, rustfmt, and diff checks pass. Both changed Rust files remain under 300 lines.
- [x] Delivery: code commit `1757e32` and subsequent verification/documentation updates are pushed. Issue #9 progress and the complete handoff were recorded; Claude owns the remaining parity work (AgentMemory signal `sig_mvg6khb_9676a82f4cf9`).
- [ ] Verify CI: confirm Platform Validation passes for the current PR #22 head on macOS and Windows; record the result before advancing packaged acceptance.
- [ ] Acceptance: this selector adjustment does not advance issue #9's 1/25 end-to-end UI acceptance.

### Full offline regression on the current issue #9 head — 2026-10-20
- [x] Task: run `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` on `1c9759846eab51ea8be1e1ce2446f427b8c5a30a`.
- [x] Verify: the complete all-target suite exited 0. The host LLVM 23 build expects `libz3.5.1.dylib` while Homebrew provides Z3 5.3; a temporary alias and Rust compiler shim under `/private/tmp/ac-issue9-z3-compat` let Cargo and mock-agent tests that spawn `rustc` run. The first attempt failed only because those direct child processes lacked the shim; the complete retry passed.
- [x] Record: AgentMemory `mem_mvfmtydk_81eea7878957`; LLM Wiki log entry added with the verification boundary.
- [ ] Acceptance: this local test pass does not verify exact packaged macOS/Windows UI behavior. Packaged Windows visual acceptance and the 25 cross-platform end-to-end workflows remain open at `0/25`.

- [x] Plan: trace the repeated wall-timeout path, run report/error banner, retained per-attempt artifacts, scheduler stop gate and terminal blocked status.
- [x] Task / Red: add a one-time timeout worker fixture and assert automatic retry; confirm the test failed because the task was `blocked` instead of reaching review.
- [x] Task / Green: preserve timeout count and attempt artifacts; retry as pending after `min(timeout, 60s)`, audit and surface timeout feedback, report terminal blocked causes, and honor Stop before relaunch.
- [x] Verify: focused timeout/stop regressions, full local Rust tests (180 suites, 542 passed, 0 failed, 14 ignored), source-layout (300-line ceiling), rustfmt and all-target Clippy pass on the integrated branch. The existing JavaScript run banner remains unchanged from `origin/main`; inspect final diff before pushing. Port the same behavior and trace to the issue-linked Rust-native branch.
- [ ] Complete: push issue-linked main PR, integrate it before porting to the issue-linked Rust branch, locally verify and open the Rust PR; update #44 and LLM Wiki/AgentMemory only after both branches are complete.

### GitHub #50 — Continuous finalization progress

- [x] Plan: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166; preserve distinct evidence review, human approval, and final application gates.
- [x] Red: the regression test fails because the run board has no explicit merge stage.
- [x] Green: add Verify → Merge → Workspace stages, localized merge progress, and a visible busy primary action.
- [x] Verify local contracts: focused UI and message tests, JavaScript syntax, formatting, and whitespace checks pass.
- [x] Verify default UI: JS behavior merged to `main` in PR #51.
- [ ] Verify Rust parity: PR #55 targets `rust`; inspect the packaged native UI and reduced-motion behavior before merging; keep #50 open until this separate UI is accepted.
- [ ] Complete: close #50 only after the Rust UI parity review is complete.

## GitHub #52 — recover from overlapping generated phase scopes

- [x] Plan: trace the phase validation error to coordinator output that assigned the same `score:` scope to two tasks; preserve overlap rejection as a safety boundary.
- [x] Red: deterministic coordinator fixture returned duplicate `score:` scopes and reproduced `phase 01-game-selection: unsafe or overlapping scope: score:`.
- [x] Green: retry once with the exact validation error and request literal, disjoint file/directory scopes; keep the first attempt's logs and retain rejection if the correction is still invalid.
- [x] Verify: overlap-repair, schema, phased-planning and MCP-planner suites pass locally; rustfmt and diff checks pass. All touched Rust source and test files are ≤300 lines.
- [x] Complete: PR #53 merged to `main`; PR #54 merged to `rust`; GitHub issue #52 is closed.

## GitHub #56 — wait through transient SQLite writer contention
- [x] Plan: Windows package smoke run `39309528729` showed one of three concurrent tasks failing with `database is locked`; `Store::open` and read-only snapshots both used a two-second timeout.
- [x] Task / Red: add `state_contention`; it failed after 2.5 seconds of a held writer lock with the current timeout.
- [x] Task / Green: use one ten-second SQLite busy timeout for both run connection modes, preserving existing transaction and error semantics.
- [x] Verify local regression and suite: `state_contention` passes; all-target default Rust suite passes (182 suites, 548 passed, 0 failed, 14 ignored); `source_layout` passes.
- [x] Verify structure and formatting: `src/state.rs` 174 lines, `src/state/records.rs` 252 lines, regression test 56 lines; rustfmt and `git diff --check` pass.
- [x] Verify: all-target Clippy with `-D warnings` passes.
- [x] Verify macOS: exact 11,116,528-byte package passes all four shared/worktree × Git/folder smoke cases (three tasks and tests each); approved merge succeeds and unapproved merge is refused. Archive SHA-256: `7c5930b7fec85bba72e45ee3574beb9f077bfb5e9c091d4e713a133fe0945fa4`. Evidence: `docs/evidence/issue56-macos-package.json`.
- [ ] Verify Windows: exact package smoke on the issue-linked PR.
- [ ] Complete: push the issue-linked branch, merge the main PR, port the fix to the issue-linked Rust branch, verify both, and close #56.

## GitHub #59 — Reduced-motion native activity indicators

- [x] Plan: trace the existing `R-17` / `NFR-10` / `QAS-21` / `DD-17` requirement. Use egui's zero `animation_time` as the native UI animation-off signal; make no claim about OS-level preference detection.
- [x] Red: add a repaint-observer regression and confirm reduced-motion task rows still request the 500 ms pulse repaint.
- [x] Green: keep running status green and static when animation is off; gate pulse-only repaints while retaining the independent workspace elapsed-time/output refresh.
- [x] Verify focused native UI regressions (19 passed), source-layout limit, and `git diff --check`. Preserve `RequestWorker`'s existing public path while splitting touched source modules below 300 lines.
- [x] Verify: full native-feature test suite, all-target Clippy with warnings denied, focused reduced-motion regressions, source-layout, rustfmt, and `git diff --check` pass locally. Loopback-only mock tests ran with sandbox network approval.
- [x] Verify scope: egui animation suppression is covered; OS-level reduced-motion preference detection is not claimed or verified.
- [ ] Complete: merge the issue-linked main PR, port to the issue-linked `rust` branch, verify both and update issue #59.

## GitHub #62 — Declarative custom CLI model and result profiles
- [x] Plan: R-160 → BG-115 → SF-147 → UC-150 → FR-145/NFR-55 → QAS-156 → DDR-20 → DD-170; main PR #63 and Rust PR #64 are already merged. This branch ports the same contract into the #9 Rust UI split while preserving quota validation and native provider defaults.
- [x] Red: add profile/workspace JSON round-trip, model restriction, result-pointer/error-condition, option omission, resume-session, and reassignment-validation tests before implementation.
- [x] Task / Green: retain optional discovery, declarative model/option groups, task overrides, result extraction/error detection, and profile/workspace serialization. Port settings editing and effective-model task controls into the #9 modular UI without replacing its fallback, manual catalog, task-card, or quota flows.
- [x] TDD regression coverage: planner model overrides remain visible, model-specific options use profile-default models, profile editing preserves declarations, invalid edits do not partially mutate profiles, and existing quota rules remain validated.
- [x] Verify locally: declarative profile tests (11), profile editor tests (2), native profile UI (10), native task UI (9), profile language regression (1), native UI feature-gate (2), source-layout (1), `cargo fmt --check`, and serial `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` all pass. Every changed Rust source/test file is at or below 300 lines; largest changed files are 298 lines (`src/workflow/assignment.rs`) and 293 lines (`src/models.rs`).
- [x] Verify Clippy and final diff: all-target Clippy with `-D warnings`, serial native-feature all-target tests, formatting, staged/unstaged diff checks, and the 300-line source/test audit pass; no merge markers or unmerged paths remain.
- [x] Complete: document the integration in the LLM-Wiki and commit the verified merge on the existing #9 branch. Record the commit in AgentMemory. Do not push; #62 itself remains closed because PRs #63/#64 already delivered its upstream work.

### Native strong-text colour — GitHub #9 (visual parity)

- [x] Finding: the exact v0.1.30 package showed the sidebar heading "Local by default" almost invisible; `RichText::strong()` uses `widgets.active` text colour, which the theme set to the on-accent colour.
- [x] TDD: `tests/native_ui_strong_text.rs` (strong text ≥ 4.5:1 on panel/window/rail/input/button surfaces; pressed text readable) failed first; fixed in `appearance.rs` (active state uses ink on a neutral fill); `native_ui_appearance` expectations updated.
- [x] Verify: local native suite 746+ passed, clippy `-D warnings` clean, both themes inspected in the native window. Idle resource of the exact package: 54 MB footprint / 110 MiB RSS / 0.00 % CPU, one process, no JS/Python in the archive.
- [x] Task / Green: add explicit optional custom model discovery, generic option templates/defaults/task overrides, standardized result extraction/error detection, and matching Rust settings/task UI. Keep unresolved argument groups omitted.
- [x] Verify locally: focused profile, plan, protocol, native UI, folder execution, and coordinator suites pass; full serial native-feature all-target suite passes; all-target check and Clippy pass; Rust source/test ceiling is 300 lines and `git diff --check` is clean.
- [ ] Complete: finish architecture/wiki/manual documentation, commit and push issue-linked `main` PR, then port the same change to an issue-linked `rust` PR and verify both branches. Keep #62 open until both PRs are reviewed and merged.

### Read-only run loading — GitHub #65

- [x] Plan: PR #55 macOS failure → root cause `workflow::load` write-capable open racing run start-up.
- [x] Task/TDD Red: `tests/workflow_load_readonly.rs` (schema untouched by `load`) failed first; `active_feedback` now reports an early run error instead of a 20 s timeout.
- [x] Task/Green: `load` uses `Store::read_only_snapshot`; `active_feedback` 30/30 (was ~1 failure in 15).
- [ ] Verify: full suite, fmt, Clippy, CI on both platforms; same fix on the `rust` line (separate PR).

### Native session rows — GitHub #9 (visual parity, R-128 / DD-137)

- [x] Finding: the native Agent Room row showed the status tag first, no agent icon, and let a long tag run past a narrow list.
- [x] TDD: `tests/native_ui_session_rows.rs` (task name above agent; 8 pt icon, agent and status on one line at 320 px; tag inside the list at 260/200/160 px) failed first; `task_rows.rs` now renders task name, then a wrapped line of 14 px icon + agent + tag (tag moves down when the line is too short and wraps its own text when a whole line is too short); `avatar_sized` shares the sidebar icon.
- [x] Verify: local native suite 751 passed / 0 failed, clippy `-D warnings` clean, rows inspected in the native window.


### Keep already-started plans out of the approval step — issue #9 / 2026-10-20

- [x] Plan: reproduce the completed-run return path in the packaged v0.1.31 workspace and distinguish a stale pending plan from a plan that has already started.
- [x] Red: `an_already_started_plan_is_not_presented_as_pending_after_returning_to_workspace` asserts the rendered review contains the started-state explanation and omits stale task rows.
- [x] Green: when `proposal.phase` is `starting`, the Review step now gives a localized route to Agent Room or goal editing and hides the obsolete plan digest/task list.
- [x] Verify local: 13 `native_ui_probe` integration tests, all-target tests, the 300-line source/test layout gate, formatting, diff checks, and all-target Clippy (`-D warnings`) pass.
- [x] Verify package: exact source `5c463e6e4feb794c890577e77e4323dcadfe566c` built with the repository release target; `package-release` and `validate-package` pass. Archive SHA-256 `662ef0eeb799428fac7ae2d9546df81d166973fd168ddc13ce23bc0a723fd5b8`; binary SHA-256 `387f0a1ebdaa93910e61ead90c282f8c4c42d34d1683fa67a1903092074cf4d8`; 214 dependency licenses. Package and source binary hashes match. One isolated packaged macOS window shows the localized already-started message and no stale task cards. This display-state regression does not advance acceptance beyond 1/25.
- [ ] Acceptance: this state-transition regression does not increase packaged workflow acceptance; the conservative count remains 1/25.

### Package the executable that built the release command — issue #9 / 2026-10-20

- [x] Plan: trace `package-release` source selection and prove the produced executable matches the running release builder when Cargo uses a non-default target directory.
- [x] Red: add a regression that distinguishes the repository `target/release` cache from the exact executable supplied by the release driver; the test first failed to compile until the explicit-binary path existed.
- [x] Green: pass the running `package-release` executable explicitly into the packager; keep the manifest digest bound to those package bytes while preserving the existing assembler call shape.
- [x] Verify: all-target local suite passes (775 passed, 14 ignored, 257 suites); all-target Clippy (`-D warnings`), rustfmt, diff check, and touched-file 300-line limits pass. Regression asserts the packaged executable and manifest SHA match the explicit binary even when a stale repository target binary exists.
- [x] Complete local slice: commit `78aa530` is pushed to `codex/issue-9-native-ui-parity`; update the issue and handoff signal with this evidence.
- [ ] Platform follow-up: run exact-head macOS/Windows validation; canceled jobs do not count as passes.

### Reopen an already-started run from Workspace — issue #9 / 2026-10-20

- [x] Plan: inspect the exact packaged v0.1.31 flow after returning from Agent Room; its review notice names Agent Room but exposes no action to reopen that run in the same window.
- [x] Red: extend the native start journey to return to Workspace; it failed because the review notice had no Agent Room button.
- [x] Green: add localized Open Agent Room using the workspace snapshot's `run_available` flag and existing `open_run` protocol; the same Probe switches the worker back to Agent Room without launching another window.
- [x] Verify local: the native start journey passes through task completion, candidate approval, merge, return to Workspace, and reopen; the locked all-target suite passes (775 passed, 14 ignored, 257 suites), all-target Clippy (`-D warnings`), rustfmt, diff checks, and the 300-line limit.
- [x] Verify macOS package: source `2fc7bc22264724219f36c0e9a1c3858ffdc38025` built with `package-release`; `validate-package`, packaged `--help`, ZIP integrity, and all four approval-gated shared/worktree × Git/folder smoke cases pass. Manifest binary SHA-256 is `04325483569c54cf2359c002e4066e7b6899280f45d38030bf4b66ceacc267bd`; archive SHA-256 is `1458db4d854b1aff7c05ffbc2d2b3ba09803d179aadc1b9f5e9dec4fc43fbb9e`. A single native v0.1.31 app window was observed with the same executable hash. This validates package/runtime startup, not the Open Agent Room transition itself.
- [ ] Verify platform: no exact-head Actions workflow run was listed after this push; Windows packaged UI remains pending. Do not mark cancelled/stale runs green.
- [x] Acceptance accounting: package smoke and native startup are not separate workflows. The current overall packaged count is **2/25** after exact-package workflow 3 profile management, documented at the start of this plan.
- [x] Complete local slice: commits `4ba9f6f` and `2fc7bc2` pushed to `codex/issue-9-native-ui-parity`; issue #9, AgentMemory, and LLM Wiki updated. Exact-head GitHub validation remains pending.

### Messenger re-notification — GitHub #72

- [x] Red: `tests/messenger_renotify.rs` — `failed → failed → running → failed` must deliver two messages.
- [x] Green: record every reported state; send only for notifiable ones.
- [x] Verify: messenger suites, fmt, clippy `-D warnings`.

### Native manual and updater interactions — GitHub #9 / 2026-10-20

- [x] Audit: the manual and update-panel tests checked rendered labels but did not dispatch pointer input through their user actions.
- [x] Task: add localized English/Korean click tests for manual copy/close and update check/apply/close; verify pending update requests reject duplicate check/apply while keeping Close available.
- [x] Focused verify: `native_ui_manual` and `native_ui_updates` pass (13 tests); `cargo fmt --all` and `git diff --check` pass. The first run exposed unconsumed egui texture deltas in test harness frames; the harness now clears each frame's texture delta.
- [x] Verify: the full native-feature all-target suite passes (766 passed, 14 ignored across 259 suites), including the new interactions; focused manual/update/knowledge/layout suites pass (19 tests). All-target Clippy passes with `-D warnings`; rustfmt, diff checks, and the 300-line limit pass (largest touched Rust test: 267 lines).
- [x] Assetize: `knowledge/wiki/native-ui-interaction-tests.md`; Graphify refreshed to 7,456 nodes / 11,822 edges / 563 communities after the public-doc contract update.
- [ ] CocoIndex mirror: the installed `cocoindex-code` entry point fails at startup because `mcp.server.fastmcp` is unavailable in its environment.
- [x] Complete local slice: commit `191d7c5`, update issue #9 and draft PR #22. Exact-head run `39414744844` was cancelled by the subsequent docs push; Platform validation for current head `7289562` is `39415057947` (in progress at last check). Packaged workflow acceptance remains 2/25; these component interaction tests do not increase it.

### Exact-source macOS package — GitHub #9 / 2026-10-20

- [x] Plan: package the committed branch head `7289562e5de394555b196da75b0ecf3904f3c53c` with the Rust `package-release` driver; preserve release gating and do not publish before product acceptance.
- [x] Task: optimized `aarch64-apple-darwin` package built from the exact source commit. It contains one 17,925,856-byte executable, SQLite bundled, and 214 dependency records.
- [x] Verify: `package-release`, `validate-package`, packaged `--help`, ZIP CRC, and archive SHA-256 passed. The package smoke passed all four shared/worktree × Git/folder combinations with three parallel tasks/tests each; unapproved merge was refused and approved merge verified.
- [x] Evidence: binary SHA-256 `ae6a24792248f973b61a80dfe41805e29581f3bb720378124bb6321df71068db`; 21,600,842-byte archive SHA-256 `abf7a55d3d189a060f3e5434f0af0de5f25fd1f1ebbae16710a124c6a894668d`; evidence in `docs/evidence/issue9-7289562-macos-package.json` and `docs/evidence/issue9-7289562-macos-smoke.json`.
- [ ] Acceptance: the package was not opened as a GUI to protect the active v0.1.31 Workspace; this smoke does not count as a packaged UI workflow. Native UI acceptance remains 2/25. Exact Windows package/UI, current screenshots, accessibility, IME, idle-resource, and license closure remain open.

### Grouped native UI and operator-control audit — issue #9 / 2026-10-20

- [x] Reuse the existing isolated v0.1.31 macOS app window to inspect Workspace settings, Agent Roles, MCP server registration, workspace sharing, Messenger notifications, cache preview, custom CLI model configuration, and completed Agent Room evidence together.
- [x] Confirm the panels expose the expected controls; cache is empty and its destructive action is disabled. Close each panel without saving settings, clearing cache, or calling an external service.
- [x] Run the associated native regressions in one batch: 48 passed, 0 failed across 10 suites (MCP, Messenger, sharing, cache, profiles/fallback, scheduling, task controls, status, and workspaces).
- [x] Record reusable evidence in `docs/evidence/issue9-4a114ec-grouped-native-ui-audit.json`; Graphify/LLM-Wiki and GitHub progress are refreshed for this grouped checkpoint.
- [ ] Platform gate: exact-head validation run `39435401549` for `2ae0b7c` was queued at capture time; record its macOS and Windows outcomes when complete.
- [x] Acceptance accounting: component coverage only; packaged UI remains **4/25**, shipped Rust replacement **0/25**. Windows UI, complete visual parity, accessibility, Korean IME, authenticated provider workflows, idle resource measurement, and final license closure remain open.
- [x] Knowledge indexing: Graphify re-extracted the updated repository graph (7,517 nodes / 12,953 edges). CocoIndex remains unavailable because its installed environment cannot import `mcp.server.fastmcp`; no indexing success is claimed.
- [x] Grouped Git/worktree regression: reproduce the clean-baseline gate failure after switching repositories with the same saved goal; add a red test, clear only the stale `active_slug` when the repository changes, and preserve the historical slug record.
- [x] Focused validation after the fix: 15 tests passed across slug, baseline, project-goal, workspace, and start-journey suites; all-target regression passed 771 tests with 14 ignored across 260 suites.
- [x] All-target Clippy passed with warnings denied; `cargo fmt --check` and `git diff --check` passed.
- [x] Build package from `a1ea942da9db94d68aca81dea7790cf0997d897a`; package-release smoke passed. Binary SHA-256 `8c918bbfaa7a72c075333ee3cc5732dfbbd304ab5d984b755e03746900642980`; ZIP SHA-256 `36a7085d82aacf8fdb5e9bac869c82687752894e9322a763f041f2c306a085eb`.
- [x] One-window isolated app regression: keep the goal unchanged, switch from the dirty reproducer repo to a clean disposable Git repo, regenerate a plan, execute three disjoint tasks in worktrees, pass all task tests, review the exact diff, approve, confirm merge, and verify clean final `main` with the three intended files.
- [x] Acceptance accounting: the distinct packaged Git/worktree flow raises packaged workflow coverage to **5/25 (20%)**; shipped Rust replacement remains **0/25**.
- [x] Update issue #9 progress comment and PR #22 body with evidence `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json` and the 5/25 acceptance count.
- [ ] Exact-head Platform Validation run `39440321768` for `d088fc2` is in progress; record macOS and Windows outcomes when complete.

### Workspace interaction audit follow-up — 2026-10-20

- [x] Re-audit planning cancellation/deletion, return from a running Agent Room, review-step restoration, and narrow Agent Roles layout against current HEAD; confirm existing fixes from `cb49fa1` and rerun their focused regressions (planning stop 2/2, room navigation/start journey 5/5, responsive profile 1/1).
- [x] TDD the missing New Workspace Enter submission: reproduce `None` instead of `Create("Research")`, connect Enter to the existing create action, and add a real egui keyboard-event regression test.
- [x] Focused verification: workspace tests 7/7, planning stop 2/2, source-layout 1/1, native workspace Enter 1/1; `cargo fmt --check` and `git diff --check` pass.
- [x] Complete full local all-target validation after the change: **772 passed, 14 ignored, 0 failed across 261 suites**. All-target Clippy (`-D warnings`), rustfmt, diff, and source-layout checks pass. This source-level regression does not add packaged workflow acceptance.
- [x] Exact package follow-up: build and validate source `6026076`; launch one packaged macOS app process with isolated HOME; create `enter-smoke` from the saved-workspaces menu using Enter; confirm it appears as the current saved workspace. Evidence: `docs/evidence/issue9-6026076-workspace-enter-ui-smoke.json`.
- [x] Acceptance accounting: this exact-package interaction verifies the keyboard regression but is not a distinct complete workflow; packaged acceptance remains **5/25 (20%)**, shipped Rust replacement **0/25**.
- [x] Record `39441872028` as cancelled when the subsequent evidence commit moved the PR head; cancelled checks are not passes.
- [ ] Exact-head Platform Validation run `39442859013` for `39a5ebb` is in progress; macOS is running tests and Windows has completed build and source-layout steps. Record both final platform results.

### Messenger approval parsing — GitHub #81 integration

- [x] Red: add regression cases proving negated and conversational replies such as `don't allow CODE` are ignored; confirm the pre-fix parser incorrectly approved them.
- [x] Green: accept a decision only as the first command after optional mentions, retaining the `inbound::parse` public path through the existing `wire` module.
- [x] Verify focused Messenger suites (4 passed, 1 ignored), full local all-target suite (**794 passed, 14 ignored across 263 suites**), all-target Clippy with `-D warnings`, rustfmt, diff check, and source-layout/300-line validation.
- [ ] Exact-head macOS/Windows Actions for current docs/package commit `0d09b8843fd66bdd55c7fce2f43e5780c1b2dc6a` are not listed yet; prior runs target stale heads and do not count.

### Cross-platform workspace UI regression and exact-source package — issue #9 / 2026-10-21

- [x] TDD Red/Green: reproduce CI run `39488170157` failures for workspace action alignment and MCP settings journey; make the action row use the card width and assert its right edge, and wait for the MCP dialog's stable title.
- [x] Local Verify: native-feature all-target suite passed **801 tests, 14 ignored across 271 suites**; focused UI regressions, warning-denied Clippy, 300-line source/test guard, formatting, and diff checks passed.
- [x] Package exact pushed source `ae2540c496fde4a25be527a468d84114961265a6` for `aarch64-apple-darwin`; `validate-package`, packaged `--help`, ZIP CRC, and four approval-gated shared/worktree × Git/folder smoke cases passed. Evidence: `docs/evidence/issue9-ae2540c-macos-package.json`; binary SHA-256 `dde853795206b7c1c10b5d7f00112d798f2119aa4ba6e6c89ceab8667055d8da`; archive SHA-256 `99eb800644b0910f3268e68494af0c007ec02c1cf9602a2fdcda0f5625de9e43`.
- [x] Exact-package GUI review: in one `ae2540c` window with an isolated temporary HOME, inspect Workspace (light/dark; restore light), Agent Roles, CLI registration, MCP, workspace sharing, Messenger, and cache preview. Save no settings/workspace, register no agent, invoke no provider/service, and clear no cache. No screenshot file was retained.
- [x] Platform Verify: exact-head run `39495359230` passed all four jobs: macOS/Windows validation and native-package build/execute. Actions artifacts: macOS `12298464724` (`2d4897392e12721db809ad7b5678e1939089a6cbefe0006077032071886cb2fe`, 21,180,791 bytes); Windows `12299926070` (`9b977f603024571c591b78f851d4695d6b3f6053760f94fbb0cb1fc397809bda`, 15,415,491 bytes).
- [ ] Acceptance: package smoke and component screenshots do not increment packaged UI acceptance (**6/25, 24%**); shipped/default Rust UI replacement remains **0/25**. Full current-main screenshot parity and Windows package/UI remain open.

### Follow the next active Agent Room session — issue #9 / #109

- [x] Plan: compare main's session-follow behavior with the native Rust room; the Rust selection previously stayed on a task after it stopped running.
- [x] Red: add a state-transition regression and confirm the initial selection was absent (`None`, expected task index `Some(0)`).
- [x] Task: select the first task when opening a run and follow a different `running` or `waiting_tool_approval` task only after the selected running task completes.
- [x] Focused Verify: the new transition test passes for both next-running and tool-approval-waiting sessions; rustfmt and diff checks pass. All affected Rust/test files remain below 300 lines.
- [x] Focus-follow verification: the affected activity suite passes 6/6, source-layout passes 2/2, Clippy passes with warnings denied, and formatting/diff checks pass. The full-suite UI journey issue is tracked in the next checklist. Acceptance remains 8/25.

### Keep workspace return behind in-flight Agent Room requests — issue #9

- [x] Plan: trace the failing “Open Agent Room” return journey through the UI request worker and snapshot polling.
- [x] Red: `tests/native_room_next_step.rs::running_run_disables_return_while_a_request_is_pending` fails because the active-run return button is enabled while a request is pending.
- [x] Task: disable that return action until the current asynchronous request completes, preventing an older room response from being applied as the workspace snapshot.
- [x] Focused Verify: the new regression and all start/return journey tests pass (6 tests across 2 suites).
- [x] CI coverage: include `native_room_next_step` in the opt-in cross-platform native UI test command.
- [x] Verify: return-control, start/return journey, and source-layout tests pass (8 tests across 3 suites); warning-denied all-target Clippy, rustfmt, diff check, and 300-line checks pass.
- [ ] Remaining Verify: the all-target release test suite was stopped after over ten minutes of LTO test-crate recompilation; run exact-head macOS/Windows CI and package jobs with the new regression included. Prior Actions run `39545652734` succeeded on `e0de50b`, before this fix. Acceptance remains 8/25.

### Port coordinator chat to the native Rust Workspace — issue #9 / main #115

- [x] Plan: derive R-166/BG-122/SF-154/UC-157/FR-152/NFR-58/QAS-163/DDR-23/DD-177 from the closed main feature; retain its closed-default, current-window-only conversation and read-only CLI boundary.
- [x] Task: implement and test bounded prompt/history validation and answer decoding in separate Rust source/test files.
- [x] Task: route a chat request through Startup's single coordinator job slot, return its answer only on request, and remove transient prompt artifacts.
- [x] Task: build the native side panel with send/close behavior, visible running/failure states, English/Korean labels, and current-window conversation state.
- [x] Documentation: explain the Workspace entry point, verified read-only CLI boundary, shared planning slot, and current-window conversation in both offline manuals; regenerate their HTML with the Rust renderer.
- [x] Verify locally: native request/job and egui interaction regressions; all-target tests (825 passed, 14 ignored), focused chat/layout regressions (12 passed), warning-denied Clippy, formatting, and source-layout checks.
- [ ] Verify release parity: run exact-head macOS/Windows CI/package jobs and inspect the packaged Workspace interaction.
- [ ] Complete: package-level review after the full Workspace parity workflow; do not count this feature as acceptance until the corresponding packaged interaction is verified. Current issue #9 acceptance remains 8/25.

### Make startup-chat regression locale-independent — issue #9 / 2026-10-21

- [x] Plan: reproduce the exact-head macOS/Windows Actions failure; CI has no saved language preference and correctly defaults to English.
- [x] Red: `large_chat_request_uses_the_existing_startup_job_gate` asserted only the Korean coordinator error and failed under an isolated English-default HOME.
- [x] Task: accept either localized coordinator wording in the test; product behavior is unchanged.
- [x] Verify: startup-chat tests pass under English-default and Korean-preference environments; full all-target tests, rustfmt, warning-denied Clippy, diff check, and 300-line source/test limits pass locally.
- [x] Knowledge: LLM Wiki records the locale-independent assertion rule; Graphify refreshed to 7,925 nodes / 12,496 edges / 581 communities. CocoIndex remains unavailable in its installed runtime because `mcp.server.fastmcp` is missing.
- [ ] Verify CI: push the fix and record the new exact-head macOS/Windows Actions result.
- [x] Acceptance: regression-test portability only; packaged Rust UI acceptance remains 8/25.

### Current Rust Workspace UI spot check — issue #9 / 2026-10-21

- [x] Build and launch the Rust debug executable from the designated issue-9 worktree in one isolated app window; inspect Workspace, Agent Roles, and Review the plan.
- [x] In temporary HOME only, register `/usr/bin/true` as a custom fixture, save a workspace for an empty non-Git project folder with the run-artifact root separate, assign the fixture to development, and save roles. Do not invoke the CLI or generate a plan.
- [x] Evidence: `docs/evidence/issue9-f7584ed-rust-ui-spotcheck.json`; the exact visual screenshots were shown during the session but not retained as repository image assets.
- [x] Knowledge: AgentMemory records the isolated test limits; Graphify refreshed to 7,927 nodes / 12,498 edges / 600 communities.
- [x] No app interaction contacted a provider or external service; temporary workspace/profile data are isolated under `/private/tmp`.
- [x] Acceptance: this debug fixture spot check does not add packaged workflow acceptance; issue #9 remains 8/25.

### Keep Review details from displacing plan phases — issue #9 / 2026-10-21

- [x] Plan: derive R-164/BG-120/SF-152/UC-155/FR-150/NFR-60/QAS-161/DDR-26/DD-175 from the plan-review layout finding.
- [x] Red: render the analysis disclosure and confirm opening it shifts the next phase down by 8 px.
- [x] Red: add an accessibility-tree assertion; it fails because egui does not automatically expose the popup's plain-text child.
- [x] Task: replace the inline analysis disclosure with a bounded egui menu popup anchored to its label and explicitly expose the analysis text to assistive technology.
- [x] Verify: rendered interaction confirms accessible analysis text, popup content, stable phase position, and outside-click dismissal; the workspace and Settings menu regressions pass too. The four affected suites pass 14/14; formatting, diff checks, and 300-line limits pass.
- [x] Knowledge: AgentMemory `mem_mvhxnjkd_a67b0f1b36de` and `mem_mvhxzgz3_5cf355f68aef`; Graphify refreshed to 8,009 nodes / 13,699 edges. CocoIndex is installed but currently fails to start because `mcp.server.fastmcp` is missing from its environment.
- [x] Build: source `147c0daa93ab145da20433ab82b96456834894bb` built in release mode for `aarch64-apple-darwin`; packaged executable `--help` succeeds. Evidence: `docs/evidence/issue9-147c0da-native-build.json`.
- [x] Build: exact source `1743fed29c038fa92c6bf085ecc2e3f6ed8eb761` builds in release mode for `aarch64-apple-darwin`; executable `--help` succeeds. Evidence: `docs/evidence/issue9-1743fed-native-build.json`.
- [x] Package: exact `ce337206891162c3e6598d750c2a065ac7a8e006` macOS archive passes package-manifest validation, bundled `--help`, `Info.plist` lint, archive CRC, and four deterministic shared/worktree × Git/folder smoke cases (3 tasks and 3 tests per case, approved merge accepted, unapproved merge refused). Evidence: `docs/evidence/issue9-ce337-macos-package.json`.
- [x] Follow-up Red/Green: a rendered interaction reproduced the phase-detail disclosure moving the next phase down by 8 px; replace inline expansion with a 480px-wide, 360px-scroll popup and expose its control/content to accessibility.
- [x] Follow-up Verify: phase popup layout/details, phase progress, and coordinator popup suites pass (10/10); source-layout, warning-denied all-target Clippy, rustfmt, diff, and 300-line limits pass. Acceptance stays 8/25.
- [x] Full local verification: native-ui-probe all-target suite passes 832 tests (14 ignored, 283 suites); public_docs 11/11, source_layout 2/2, warning-denied Clippy, rustfmt, and diff checks pass.
- [x] Knowledge: AgentMemory `mem_mvhyjd8s_12fb1fc7778b`, `mem_mvhyt2tp_9d246923164f`; Graphify refreshed to 8,025 nodes / 13,740 edges. CocoIndex CLI was attempted but cannot start because `mcp.server.fastmcp` is missing from its runtime.
- [x] Knowledge update: AgentMemory `mem_mvhz8vy7_ac0aefbddc65` records full-suite and platform CI progress.
- [x] Cross-platform test coverage Red/Green: `release_notes` failed because the new Review popup suites were absent from platform-validation; added them to both native UI test and Clippy target lists. Target-coverage test and the three UI suites pass (10 tests); AgentMemory: `mem_mvhzcqyr_835a6c8572f5`.
- [x] Build/package exact commit `8a2f817b2d918334905a8158df696fcb17242336` for `aarch64-apple-darwin`; manifest, packaged `--help`, ZIP CRC, and the four deterministic approval-gated smoke cases pass. Evidence: `docs/evidence/issue9-8a2f817-macos-package.json`.
- [ ] Verify runtime/package: inspect the exact Rust app after the popup change; the active user's window is left untouched.
- [ ] Complete: package and visual acceptance remain open; packaged Rust workflow acceptance remains 8/25.

### Keep briefing and profile detail popups from moving page controls — issue #9 / 2026-10-21

- [x] Plan: extend R-164 through BG-120, SF-152, UC-155, FR-150/NFR-60, QAS-161, DDR-26, and DD-175 to cover Review task editors and details, Agent Room, and agent-profile detail popups.
- [x] Red: rendered task-card interaction reproduced a 135 px shift in the following task control when the inline editor opened.
- [x] Task: replace task-card, team-briefing, and advanced profile-rules inline disclosures with bounded, scrollable anchored popups; keep editable popups open for inside clicks.
- [x] Verify: rendered egui tests confirm all popup content remains accessible and editable, while the next-task, stop, and save controls retain their positions; affected suites pass.
- [x] Knowledge: extend the native UI interaction wiki with the no-reflow popup rule.
- [ ] Verify: push this follow-up and confirm its exact-source macOS and Windows CI results; the previous pushed head's exact run was queued.
- [ ] Complete: exact packaged app visual inspection remains open; issue #9 acceptance remains 8/25.

### Keep MCP source-import review cards stationary — issue #9 / 2026-10-21

- [x] Plan: apply the existing no-reflow detail-popup rule to each MCP source review card; scope repeated controls by source reference.
- [x] Red: render two sources, open the full-source control, and confirm duplicated egui IDs route the click to the wrong card and hide the selected source.
- [x] Task: replace inline full-source expansion with a bounded menu popup and give each source control a stable per-reference ID/accessibility name.
- [x] Verify: the rendered regression confirms selected source content is visible and the next record's position is unchanged. MCP, popup, task, release-workflow, public-docs, and source-layout suites pass 33 tests; warning-denied all-target Clippy, rustfmt, and diff checks pass.
- [x] Knowledge: update the R-164 to DD-175 trace and `knowledge/wiki/native-ui-interaction-tests.md`; save AgentMemory `mem_mvi0nws0_ae42ffe453a1`, `mem_mvi1cjjb_adb7fddf37cb`, `mem_mvi1y8zj_8ea2e6a3b178`, and lesson `lsn_01e115bb5e421d8d`; Graphify refreshed to 8,051 nodes / 12,715 edges / 613 communities. CocoIndex is unavailable because `mcp.server.fastmcp` is missing.
- [x] CI failure follow-up: exact prior macOS validation found task-popup tests still assuming inline-expanded content; update the tests to open the popup and verify its bounded 560px editor. Focused task, plan-layout, and source-popup tests pass locally.
- [x] Verify exact macOS package from pushed source `733852e`: manifest, binary hash, packaged `--help`, ZIP CRC, Info.plist, and four approval-gated shared/worktree × Git/folder smoke cases pass. The exact package opens in one native app process and loads the saved Workspace plus all three configured agent cards. Evidence: `docs/evidence/issue9-733852e-macos-package.json`.
- [x] Follow-up: TDD found five remaining inline panels—User manual, Agent tools, workspace-action confirmations, update details, and execution consent—which pushed page content down. Move them to centered windows with stable IDs; rendered regressions verify surrounding coordinates remain unchanged. Keep the execution-review button slot in place; preserve manual image/copy controls, update actions, and confirmation/cancel dispatch.
- [x] Verify: the full locked all-target suite with `test-fixtures` and `native-ui-probe` passes (508 tests, 14 ignored across 165 suites); all-target Clippy with warnings denied, formatting, and `git diff --check` pass. Geometry coverage, bilingual consent, update actions, and the source-layout target are included in cross-platform CI. Modified Rust implementation and test files remain below the 300-line project limit.
- [x] Package: build the exact `aarch64-apple-darwin` artifact from `9cea0a4`; packaged `--help`, Info.plist, ZIP CRC, manifest hashes, and all four deterministic shared/worktree × Git/folder smoke cases pass. Evidence: `docs/evidence/issue9-9cea0a4-macos-package.json`.
- [x] Knowledge: record package provenance in the evidence file and AgentMemory `mem_mvi356fy_f98ec10c387e`; Graphify code graph refreshed locally (8,095 nodes, 12,822 edges, 607 communities). CocoIndex is unavailable in this runtime.
- [ ] Verify: inspect the new windows in the packaged app and validate exact-head macOS/Windows CI. Actions run `39597791275` for pushed head `920d426fd0624d847d787f40bd6d24615815b0aa` is still running (Windows native build and macOS all-target tests); CUA timed out both when resolving the app name and its exact bundle path, so no existing window was replaced.
- [ ] Verify the exact-head Windows Actions result and visually inspect the MCP source popup in the packaged UI. macOS screen capture was unavailable; popup content, geometry, and per-record selection are covered by the rendered egui regression.
- [ ] Complete: packaged Rust UI acceptance remains 8/25; issue #9 remains open.

### Keep native help and approval dialogs from moving the page — issue #9 / 2026-10-21

- [x] Plan: extend R-164/BG-120/SF-152/UC-155/FR-150/NFR-60/QAS-161/DDR-26/DD-175 to cover workspace-action confirmations, update details, and execution consent.
- [x] Red/Green: rendered egui regressions caught 95 px, 100 px, and 300 px page shifts for archive confirmation, updates, and execution consent. Centered stable-ID windows keep page anchors fixed.
- [x] Task: preserve existing confirm/cancel and update actions, retain execution-review button geometry, and add visible dismissal and localized confirmation titles.
- [x] Verify: the full locked all-target suite with `test-fixtures` and `native-ui-probe` passes (508 tests, 14 ignored across 165 suites), including geometry, bilingual consent, update actions, and workspace actions. All-target Clippy with warnings denied, rustfmt, and diff checks pass; touched Rust files remain below 300 lines.
- [x] Package: build and smoke-test the exact `aarch64-apple-darwin` package from `9cea0a4`; verify packaged `--help`, Info.plist, ZIP CRC and manifest hashes. Record four passing approval-gated modes in `docs/evidence/issue9-9cea0a4-macos-package.json`.
- [x] Knowledge: update the native overlay rule and its requirement/design trace; save AgentMemory `mem_mvi2brxe_bd3079bc4437` and `mem_mvi2jlo3_e08f773b2344`.
- [ ] Verify: inspect the current packaged app and complete exact-head Actions run `39597791275`; CUA timed out when resolving the existing app by name and exact bundle path, so no window replacement was done.
- [ ] Complete: packaged Rust UI acceptance remains 8/25; issue #9 remains open.

### Exact macOS package UI checkpoint — issue #9 / 2026-10-21

- [x] Build the macOS package from the designated issue-9 worktree commit `34ca477`; its manifest names the same source commit.
- [x] Verify the package binary help, ZIP integrity, SHA-256, and four packaged smoke cases across shared/worktree and Git/folder modes.
- [x] Launch the packaged Rust app in one isolated window and show the current Workspace settings screen; retain the test profile and workspace only under `/private/tmp`.
- [x] Record package and screen provenance in `docs/evidence/issue9-34ca477-macos-package-ui.json` and the LLM Wiki.
- [ ] Verify the exact-head Windows CI result; run `39567796509` is still building Windows at this checkpoint.
- [x] Acceptance: a rendered package screen is a spot check, not a complete UI workflow; issue #9 remains 8/25.
- [x] Use the packaged Save workspace action and verify its name, goal, shared mode, and non-Git project path persisted in the isolated `startup.json`.

### Exact macOS package smoke and menu-overlay recheck — issue #9 / 2026-10-22

- [x] Build and validate the `aarch64-apple-darwin` package from `d23c3c2`; packaged `--help`, archive integrity, manifest and binary hashes pass.
- [x] Run all four packaged mock-agent modes: shared/worktree × Git/folder. Each mode ran three parallel tasks and tests, rejected an unapproved merge, then verified the approved merge.
- [x] Inspect Workspace and Settings menus in the single Rust preview window. Both overlay the existing content; 22 rendered menu/popup layout regressions pass across six suites. The preview binary is `78cc5f0`; the diff to `d23c3c2` contains only the test-feature gate, with no production UI changes.
- [x] Record reproducible package evidence in `docs/evidence/issue9-d23c3c2-macos-package.json` and update the LLM Wiki and AgentMemory.
- [x] Verify Actions run `39687975008` for `d23c3c2`: Windows passed; macOS failed only in the one-frame Settings-dialog dismissal assertion. The follow-up below makes that assertion wait for the rendered close state.
- [ ] Complete: packaged Rust UI acceptance remains 8/25; issue #9 remains open. Menu checks are component evidence, not a new end-to-end workflow.

### Keep MCP import review from pushing Workspace controls down — issue #9 / R-164 / 2026-10-22

- [x] Plan: extend R-164 → BG-120 → SF-152 → UC-155 → FR-150/NFR-60 → QAS-161 → DDR-26 → DD-175 to include the MCP source-selection review, while keeping the existing source-record menu anchored to its record.
- [x] Red: opening the rendered MCP preview moved the fixed Workspace control anchor from y=61 to y=177 (+116 px).
- [x] Green: render the import-selection list in a centered, scrollable, dismissible window; leave the Workspace layout untouched and provide a button to reopen the review.
- [x] Audit: inspect every native `menu_button`, workspace popup, and centered window. Settings, Workspace, Review, Agent Room, profile rules, and individual MCP source details already use overlays; the MCP selection list was the remaining inline detail view that reflowed the page.
- [x] Verify focused: 14 tests pass across eight menu/detail geometry suites; the Settings journey passes five local runs after replacing its one-frame close assertion with a bounded 5-second rendered-state wait. All-target Clippy with warnings denied, source-layout (2), formatting, and diff checks pass. Touched Rust files remain below 300 lines.
- [x] Package: exact source `83f8e619b0fe3197aca91d58f3f6baa94730cebb` passes package manifest, packaged help, ZIP integrity, and four approval-gated shared/worktree × Git/folder mock-agent smoke cases. Evidence: `docs/evidence/issue9-83f8e61-macos-package.json`.
- [x] Knowledge: update the R-164 → DD-175 trace and native interaction Wiki; save AgentMemory `mem_mviq4cnk_b0e9d845a60f`.
- [x] CI diagnosis: run `39687975008` passed Windows and exposed only the macOS one-frame Settings-dialog close assertion. The exact failing journey passes locally after bounded dismissal polling; publish this follow-up for a fresh exact-head run.
- [ ] Verify: run the complete all-target suite (the prior attempt was stopped at 483 passed / 14 ignored) and obtain fresh exact-head macOS/Windows Actions results.
- [ ] Complete: packaged Rust UI acceptance remains 8/25; this popup correction is component evidence and does not close issue #9.

### Port reviewed scope-failure recovery to the native Rust Agent Room — issue #9 / GitHub #122

- [x] Plan: reconcile #122's merged main/Rust behavior with the active native eframe branch. The shared workflow operations exist in main/Rust; the native UI branch lacks the feature. Preserve path-validation, digest approval, non-overwrite restore, audit, and explicit-resume guarantees; do not copy WebView JavaScript.
- [x] Architecture: derive R-165 → BG-121 → SF-153 → UC-156 → FR-151/NFR-61 → QAS-162 → DDR-25 → DD-176; add this trace to the architecture index.
- [x] Red: add Rust-only domain tests for failure details, unchanged scope after feedback, overlap/stale-digest refusal, safe restore/conflict, retained evidence, and completed-work reuse; confirm the new API was absent before implementation.
- [x] Task: implement the workflow recovery API with task ownership validation, digest-bound approval, exclusive non-overwriting restore, preserved quarantine evidence, audit records, and explicit stopped state. Changed files: `src/workflow/scope_recovery.rs`, `src/workflow/scope_recovery/paths.rs`, `tests/scope_recovery.rs`.
- [x] Verify domain: `cargo test --locked --test scope_recovery` passes all 7 focused tests on macOS after final domain edits.
- [x] Task: route preview and approval through the existing CLI/controller boundary; the following item records the dispatcher, revision-bound approval, and existing CLI action path.
- [x] Task: route failure and preview reads through the native dispatcher; send approval via the revision-bound subprocess controller and existing CLI action path.
- [x] Task: render a bounded, scrollable eframe review with offending paths, task ownership, quarantine locations, editable paths/test argv, restore outcomes, and explicit digest-bound approval. Approval keeps the run stopped and clears auto-resume intent.
- [x] Verify locally: 7 domain tests, 2 rendered native UI tests, public-docs (8), source-layout (2), native UI feature check, and warning-denied native Clippy pass; changed Rust files remain at or below 300 lines.
- [ ] Verify: run cross-platform platform-validation tests and inspect an exact-source packaged workflow; keep packaged acceptance at 8/25 until end-to-end evidence is recorded.
- [x] Knowledge: add `knowledge/wiki/scope-failure-recovery.md`; AgentMemory `mem_mvisvkb6_3c2c1af32c07` records focused test evidence and `mem_mvisy4sx_41b2bc4e55d2` records final verification and tooling. Graphify updated to 8,284 nodes / 13,164 edges / 616 communities. CocoIndex is unavailable in this environment (`ModuleNotFoundError: cocoindex`); no new runtime dependency was added.
- [ ] Complete: keep issue #9 at 8/25 until exact packaged user workflow and supported-platform evidence are recorded. Keep PR #22 draft/open until its gates pass.

### Coordinator triage of failed tasks — GitHub #135

- [x] Plan: derive R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173.
- [x] Red: verify a failed task invokes the recorded coordinator only when no availability wait or scope failure blocks triage; verify missing coordinator, blocked verdict, and exhausted repair rounds retain failure evidence.
- [x] Green: widen the existing bounded repair loop to handle eligible failed tasks and resume the run after a revise verdict.
- [x] Verify: focused repair/public-doc/source-layout tests pass 18 tests; the complete `cargo test --locked --all-targets --features native-ui-probe` run passes 866 tests, 14 ignored, 0 failed across 295 suites. Warning-denied repair Clippy, formatting, source-layout, diff checks, and the 300-line source/test guard pass.
- [x] Platform status: run `39722700547` was cancelled after the documentation-only follow-up push; no current-head macOS/Windows result is claimed. Packaged workflow acceptance remains 8/25.
- [x] Knowledge: add `knowledge/wiki/coordinator-failed-task-triage.md`, update the Wiki index/log, save AgentMemory `mem_mvj1jkm5_d628ff3f1d6d` and `mem_mvj2850j_16d51308a1db`, and refresh Graphify to 8,408 nodes / 13,377 edges / 630 communities. CocoIndex is unavailable in this environment (`cocoindex` command not found).
- [x] Cleanup: after all test processes finished, `cargo clean` removed 494,075 generated files / 80.2 GiB from `target`; the 750 MB `dist` package directory was preserved.
- [x] Follow-up plan: apply the R-163 / DD-173 quota-wait review contract without changing automatic task fallback or quota-wait triage.
- [x] Red: verify a shared-quota coordinator is refused before the existing five-hour fallback deadline and leaves no review process; verify an eligible review uses only a registered, unrestricted fallback.
- [x] Green: reuse `fallback_due` and the registered fallback chain, pass its configured model through the read-only CLI invocation, and audit requested/effective coordinator IDs and model.
- [x] Verify: `review_quota_fallback` (2), review/fallback/availability/repair/docs/source-layout suites (43), and unfiltered locked/offline all-target tests (935 passed, 14 ignored, 308 suites) pass. Warning-denied all-target Clippy, rustfmt, diff, and the 300-line source/test limit pass.
- [x] Knowledge: update `knowledge/wiki/coordinator-failed-task-triage.md`, Wiki index/log, refresh Graphify (8,782 nodes / 15,068 edges), and save AgentMemory `mem_mvjtfd7o_0fd6014f4269` plus final verification. CocoIndex is unavailable in this environment.
- [x] Cleanup: `cargo clean` removed 44,958 generated files / 11.8 GiB; preserved the 253 MB `dist` package artifacts and user data.
- [x] Complete this follow-up: quota-wait manual coordinator review uses the registered, policy-eligible fallback and its model; #9 stays open at 8/25 pending packaged UI and platform acceptance. PR #22 remains draft/open.

### Stabilize issue #9 validation and synchronize bilingual manuals — 2026-10-22

- [x] Plan/Red: record exact-head run `39727583952`: Windows exposed a 3-second async MCP worker receive timeout; macOS exposed a stale recovery-message assertion.
- [x] Green: commit `65ab63d3` raises the bounded test receive deadline to 15 seconds and checks localized message substrings; commit `fb004d3d` merges current main `6d0093d3` and makes PR #22 mergeable.
- [x] Documentation: update both manual source headers to v0.1.36 and regenerate English/Korean HTML from Markdown with `cargo run --locked -- manual render`.
- [x] Verify focused: `manual_renderer`, `public_docs`, `native_ui_mcp`, and `scope_recovery_room` pass 26 tests; `source_layout` passes 2; warning-denied targeted Clippy, rustfmt, and diff checks pass. Changed test files are 202 and 84 lines.
- [x] Knowledge: record the generated-HTML/source pairing rule in `knowledge/wiki/offline-html-manual.md`; save AgentMemory checkpoints for CI diagnosis, manual synchronization, and disk cleanup.
- [x] Verify platform: exact-head Actions run `39730774464` for `fb004d3d` passed macOS and Windows validation, including all-target tests, opt-in native Rust UI launch, formatting, and Clippy. Both exact packages built, rendered bilingual manuals with Rust, and executed successfully. Artifacts: macOS `12406617017` (SHA-256 `b88fdf44f4906e2277ca409b01ed1c060c64f82efa0cfc472189d7e776bf53df`); Windows `12406224574` (SHA-256 `21147f9102d0dfee67ad99c1b9bd47c5ed81068db8cca6bc71c35f95d15822ad`). The full local all-target run was interrupted after exposing two manual consistency failures; those were fixed and the focused suites pass, but a full-suite pass is not claimed.
- [ ] Complete: issue #9 remains at 8/25 packaged workflows; keep PR #22 draft until exact-head platform/package checks and user review gates are satisfied.

### Publish stable v0.1.36 packages — 2026-10-22

- [x] Plan/Task: validate tagged source `6d0093d3576e50642bad39fa885b9b5695b0281c`, build macOS and Windows packages, render both offline manuals with the Rust binary, execute each exact package, then publish one stable release.
- [x] Verify: release workflow run `39728295847` completed successfully, including macOS/Windows validation, package execution, and the publish job.
- [x] Verify archive digests against the downloaded `SHA256SUMS`: macOS `7af9cdc0f3631f60a3e8f58a48923c04bca70a01cbaf6ca1cf73322341ecc415`; Windows `c6970e8ffec84a6c2b42a6490c2840130d7ba6aeadc6afaf6f594b17539216bb`.
- [x] Complete release: stable, non-prerelease v0.1.36 is latest at https://github.com/neodamul/AgentConductor/releases/tag/v0.1.36; both platform ZIPs and `SHA256SUMS` are uploaded.
- [x] Knowledge: record release provenance and distinguish shipped v0.1.36 from ongoing native Rust UI migration; issue #9 packaged workflow acceptance remains 8/25.

### Collapsible task detail — GitHub #146

- [x] Red-first tests, green fix, verify (suite, fmt, clippy).

### Coordinator guardrail step 1: decision record — GitHub #149

- [x] Requirement chain R-169/R-170 to DD-180/DD-181, red tests, decision list, room rendering (verified in a browser with a real repaired run).
- [x] Step 2 launch gate; [x] step 3 typed path/dependency amendments with validator, undo and budget. [ ] Step 4 charter and drift check.

### Coordinator guardrail step 2: launch gate — GitHub #149

- [x] R-171 to DD-182, red tests, gate before create_dir, manual.
- [x] Step 3 typed path/dependency amendments with validator, undo and budget. [ ] Step 4 charter and drift check.

### Coordinator guardrail step 3: typed amendments — GitHub #149 / Rust issue branch

- [x] Red: coordinator review schema rejects typed `add_paths` / `add_dependencies` while older review packets remain valid.
- [x] Task: add bounded proposal fields, validate paths/dependencies with existing plan rules, escalate broad/sensitive scopes or exhausted per-run budget, audit successful changes, revoke derived approval on undo, and expose Undo in the native coordinator judgment list.
- [x] Verify: deterministic mock coordinator covers safe amendment → re-run → approval wait → undo; broad, secret, overlapping and budget-exhausted amendments remain unchanged and block for a person. Native UI renders Undo only for active applied entries and a pointer test confirms it emits the selected amendment ID.
- [x] Verify broad regression: offline all-target run passes 925 tests / 14 ignored / 306 suites, with the known blocking `review_approval_shows_start_feedback_then_opens_the_started_run` filtered out. The unfiltered run blocked in that test's `Controller::snapshot → Store::open` call; affected amendment/UI suites, warning-denied all-target Clippy, formatting and source layout pass.
- [x] Review edge cases: explicit dotted top-level directory scopes (for example `assets.v1/`) escalate; already-present dependencies are rejected instead of being recorded as a no-op.
- [ ] Step 4 charter persistence and goal-drift checks; Windows/package acceptance remain open.

### Coordinator guardrail step 4: charter and goal drift — GitHub #149 / issue #9

- [x] Plan: refine R-169 → BG-125 → SF-157 → UC-160 → FR-155/NFR-65 → QAS-166 → DDR-29 → DD-180. Freeze the approved goal, effective planning-rule digest, and ordered task scope/dependency/test contract in each run.
- [x] Verify inherited regression baseline: the later unfiltered offline suite passed 927 tests / 14 ignored / 306 suites; the previously stalled start-feedback test also passed in isolation and in that run.
- [x] Red: test exact contract persistence, legacy plan/report compatibility, drift rejection before scheduling/approval/merge, and contract synchronization only through validated amendment/test-command paths. The initial charter contract test failed to compile before implementation.
- [x] Task: carry effective planning rules in the Plan, persist a compact charter with the Report, validate it at scheduling/review/approval/merge boundaries, and synchronize reversible typed edits and operator-approved scope recovery.
- [x] Verify locally: focused charter/amendment/decision/phase/planner tests pass; unfiltered offline all-target suite passes 932 tests / 14 ignored / 307 suites; warning-denied all-target Clippy, rustfmt, source-layout, `git diff --check`, and all edited source/test files at or below 300 lines pass.
- [x] Code complete: the charter/drift implementation, tests, Wiki, issue-linked Rust branch, and GitHub progress records cover all four agreed steps. Current Windows package/runtime acceptance is tracked separately below.
- [ ] Final acceptance: verify Windows package/runtime and UI behavior, then close #149; keep issue #9 and draft PR #22 open until the full migration gates pass.

### Native integration package checkpoint — issue #9, 2026-10-22

- [x] Verify exact integrated source `ece1ead4c21c`: all-target local tests with `native-ui-probe,test-fixtures` pass 914 tests, 14 ignored, 303 suites, exit 0.
- [x] Package and validate the exact macOS binary; replace the idle previous app with this package and verify one process, v0.1.36, and retained saved Workspace fields. Record hashes in `docs/evidence/issue9-main-e9d1-native-port.json`.
- [x] Preserve the pending disposable fixture without approving or merging it.
- [ ] Verify packaged inspector/coordinator interactions and exact-head Windows results.
- [ ] Fix native periodic per-agent quota scheduling through the existing refresh policy and asynchronous worker. Restart correctly marks old observations stale; manual refresh evidence does not establish automatic refresh acceptance.
- [ ] Complete: keep packaged workflow acceptance at 8/25 and PR #22 draft pending the remaining gates.

### Native automatic quota scheduling — issue #9, R-63/R-67, DD-69/DD-73

- [x] Plan: clarify independent configured deadlines, first-open query, failure throttling, minimized/busy deferral, and reuse of the existing worker in requirements, QAS-56 and DD-69.
- [x] Red: native scheduling tests fail because no next-deadline or due-profile path exists.
- [x] Task: schedule saved supported profiles through the existing metadata request path, retain per-agent attempt times, use delayed native repaints, and keep icon data out of repeated metadata snapshots.
- [x] Verify locally: 34 native library tests and 20 tests across six integration suites pass, covering interval policy, real UI-to-worker admission/failure, draft/icon preservation, manual timing, settings/start journeys and source layout. Warning-denied release Clippy and rustfmt pass. All changed Rust files stay within 300 lines.
- [ ] Package: inspect authenticated automatic collection and subsequent cadence from an exact-source single-window package.
- [x] Package checkpoint `017b51c6`: validation and four smoke cases pass; automatic first-open Codex collection returns a fresh authenticated observation, and the next Claude query starts five minutes after its first attempt with no clicks. Workspace drafts/selection remain intact and controls stay enabled while the CLI runs. Claude itself fails to emit its ready prompt within the existing bound, so successful Claude collection remains unverified.
- [x] Red/Green follow-up: actual package exposed a transient global metadata loading banner. A rendered regression reproduces it; keep background metadata quiet while retaining action errors/save progress. All 35 library tests and warning-denied Clippy pass after the fix.
- [x] Package follow-up `dab1e502`: exact package validation/four smoke cases pass. Single-window screenshot/AX show automatic Claude collection with only the agent-row indicator; no global loading banner, saved fields retained and inputs enabled. Source-layout/pending tests pass 5/5 in addition to 35 library tests and warning-denied Clippy.
- [ ] Complete platform/provider checks: verify Windows and diagnose the actual Claude pre-prompt timeout; keep the full #9 acceptance gate open.
- [ ] Complete: keep #9 acceptance unchanged until the full packaged workflow passes.

### CI test stability — GitHub #156

- [x] Plan — R-172 → BG-127 → SF-160 → UC-163 → FR-158/NFR-68 → QAS-169 → DDR-32 → DD-183.
- [x] Red evidence — issue #156 documents load-sensitive Windows process containment, cooperative worker handoff, and process timeout failures.
- [x] Task — wait on markers and recorded state; use a deterministic Rust process-tree fixture; split touched test files under 300 lines.
- [x] Verify on main — 20 focused tests passed three times locally; macOS and Windows validation and release-package Actions jobs passed; PR #161 merged as `a2ae3ac2`.
- [x] Port — apply only the verified fix to `codex/issue-9-native-ui-parity`, preserving its native UI and existing test fixtures.
- [x] Verify — 19 tests passed across five focused suites with `test-fixtures`; warning-denied Clippy, rustfmt, source layout and diff checks passed. Windows CI passed the Windows-only containment test.
- [x] Complete — main fix `a2ae3ac2` and Rust-branch port `f938a76e` are committed; the Rust branch is pushed and #156 is closed after verification.

### Port credit amount rounding to native quota display — GitHub #162 / issue #9

- [x] Plan: selectively port main's merged display correction to the existing shared Rust quota amount formatter; retain original text for invalid, non-finite, or out-of-range values.
- [x] Red: the new quota-display regression failed on `6667.28056954845/12500`, proving the Rust UI still exposed fractional credit amounts.
- [x] Task: round finite string amounts with absolute value below `1e15` for the Agent Room budget row and badge detail, without changing stored values or percentage calculations.
- [x] Verify locally: unfiltered locked/offline all-target run passes 933 tests, 14 ignored, across 307 suites; quota display 7/7, source layout 2/2, warning-denied all-target Clippy, rustfmt and diff checks pass; touched source/test files are 147/200 lines.
- [x] Package: exact source `97876887c26d` builds an arm64 macOS release package; manifest/binary checksums validate, and the packaged binary passes four mock smoke cases (3 parallel tasks, 3 passing tests, merge refused before approval and verified after approval).
- [ ] Platform/UI: Windows execution and native on-screen inspector/coordinator interaction remain unverified; package smoke is not visual acceptance.
- [x] Evidence: save exact package provenance/checksums in `docs/evidence/issue9-97876887-macos-package.json`, cross-link the quota wiki, and retain the 8/25 acceptance count.
- [x] Complete this code unit: commit `b7b67a0c` is pushed, and the Rust parity handoff/issue #9/PR #22 are updated; keep issue #9 acceptance unchanged.

### Agent Room shared-branding localization regression — issue #9
- [x] Red: a focused five-suite run exposed that `native_ui_room_language` still required the shared app name inside the Agent Room page, contradicting the no-duplicate-branding UI contract.
- [x] Green: change the rendered-text assertion to require the shared app name to be absent; production rendering is unchanged.
- [x] Verify: corrected room-language suite 3/3; related status/language/render suites 19/19; source-layout 2/2; `cargo fmt --all -- --check` passes. Graphify refresh: 8,862 nodes / 14,004 edges / 688 communities.
- [ ] This is a test-contract correction only. It adds no packaged workflow acceptance; #9 remains 10/25 and the Windows/package gates remain open.

### Coordinator amendment payload bounds and test-command repair — issue #149 / #9
- [x] Plan: port the latest mainline 8-entry / 1,024-byte path-and-dependency limits into the Rust review and application validators; keep test-command replacement because issue #149 explicitly requires it.
- [x] Red: new coordinator-contract tests failed for nine paths and 1,025-byte values; an integration test failed to reach approval because test-command replacement had no working fixture response.
- [x] Green: share one bounded validator across review parsing and application; exercise a failed `false` check replaced by `true`, persisted audit/charter state, and exact Undo.
- [x] Verify: 28 tests passed across six amendment/coordinator suites; source-layout 2/2, all-target Clippy with warnings denied, rustfmt and diff checks pass. Changed Rust and test files are each under 300 lines. Graphify: 8,873 nodes / 14,024 edges / 683 communities.
- [ ] Platform/package acceptance remains open; this unit does not change packaged workflow acceptance or close #149.

### Recheck the exact-package manual screenshot blocker — issue #9 / 2026-10-23

- [x] Plan: run the exact v0.1.36 macOS arm64 package only against a user-authorized disposable Git worktree with the deterministic mock CLI; never invoke provider CLIs, network services, MCP servers, or a user repository.
- [x] Task: add the missing `documentation` role mapping after the first planner attempt failed; generate and execute three disjoint mock tasks for `one.txt`, `two.txt`, and `three.txt`.
- [x] Verify: reviewed candidate `94fe3ada21bddf9086c1bd12679bf066b3fd22e6` contains only those three files; all three `git diff --check` commands exited 0. Candidate remains waiting for approval; it was not approved or merged. Evidence: `docs/evidence/issue9-mock-approval-ui-20261023.json`.
- [ ] Screenshot: the exact Agent Room screen was visually inspected in CUA but not saved. macOS `screencapture` failed with `could not create image from display`; find an approved way to persist the window-only image before editing either manual.
- [ ] Acceptance: this duplicate fixture run does not add a distinct workflow; packaged UI remains 14/25 and shipped/default Rust UI remains 0/25.

### Measure the exact v0.1.36 Agent Room idle footprint — issue #9 / NFR-05 / 2026-10-23

- [x] Plan: measure the already-running exact v0.1.36 arm64 app in a visible Agent Room with no active job; do not start another app or any agent.
- [x] Verify: one process, zero children; four five-second `top` samples report 71–75 MiB resident memory and 0.0–0.4% CPU (0.275% average). `footprint` reports 71 MB physical footprint, below the QAS-58 100 MB target. The since-launch peak is 178 MB; `ps` RSS is 123424 KiB, so use physical footprint per QAS-58 rather than RSS inflated by shared pages.
- [x] Evidence: exact binary SHA-256 `f804e8c8406408f901c7103fed4f341fc0ceedfb1a1e44506d2e9ba5457fc3ec`; `docs/evidence/issue9-f438820-idle-resources-20261023.json`.
- [ ] Acceptance: this 15-second visible-idle sample passes only for the exact v0.1.36 macOS package. Startup/active peak, sustained idle and Windows resource use remain open.

### Extend the exact-package idle sample to the QAS 30-second window — issue #9 / NFR-05 / 2026-10-23

- [x] Plan: continue observing the same exact v0.1.36 macOS arm64 Agent Room process in the same no-job state; do not launch an additional app or agent.
- [x] Verify: seven five-second samples over 30 seconds show 0.0–0.4% CPU (0.3286% average); `footprint` reports 73 MB, below the 100 MB QAS-58 target. One app process and no children. RSS reference began at 175 MiB and settled to 73–75 MiB; use physical footprint because shared pages inflate RSS. Since-launch peak reached 180 MB.
- [x] Evidence: `docs/evidence/issue9-f438820-idle-resources-30s-20261023.json`.
- [ ] Acceptance: the exact v0.1.36 macOS visible-idle 30-second target passes; startup/active peak, longer idle and Windows remain unverified.

### Repeat exact-package idle measurement on the c92 package — issue #9 / NFR-05 / 2026-10-23

- [x] Plan: launch only the exact v0.1.36 arm64 package at source `c92e2b49` in an isolated temporary HOME/workspace; no agents or user workspace.
- [x] Verify: seven samples across 31 seconds show 0.0% CPU; physical footprint is 57 MB, below the 100 MB QAS-58 target. There was one app process and no children; since-launch peak was 165 MB.
- [x] Evidence: binary SHA-256 `60c1295a7cae1d4341ceb0370e6654a64ff7144eef12533c74e394d5e56b111d`; `docs/evidence/issue9-c92-idle-resources-30s-20261023.json`.
- [x] Cleanup: closed the isolated package and removed its temporary HOME/workspace. `target/` is 2.2 GB of retained `target/release` build artifacts, not the previously reported 35 GB; no build cache was removed.
- [ ] Acceptance: this visible-idle macOS sample passes only for this package/state; startup/active peak, longer idle, Windows and final-release resource use remain unverified.

### Resume Windows verification in the existing UTM VM — issue #9 / 2026-10-23

- [x] Plan: use the user's existing Windows ARM64 UTM VM and share only a temporary source archive; keep the VM running and avoid its destructive Stop/reset controls.
- [x] Task: reach the signed-in Windows desktop and select the temporary staging folder in UTM.
- [x] Audit: live `gh pr view 22` reports open/draft, head `328faa347a289da44d0a440af79f2f61dd1a8574`, target `main`, merge state `DIRTY`; `docs/09-completion-audit.md` now separates this docs head from the earlier product/test head.
- [x] Red → Green: extend `issue9_current_status` to require the current PR and Windows evidence; it failed before the audit update, then passed with `public_docs` and `source_layout` (14 tests total). Evidence JSON and `git diff --check` pass.
- [ ] Verify: guest file access is blocked. `utmctl ip-address` fails with OSStatus -2700 because QEMU Guest Agent is not installed/running; GUI input/Explorer did not provide reliable access. No source files were read in Windows and no build/package/UI test ran.
- [ ] Complete: resume only after an authorized guest transfer/control path is available. Evidence: `docs/evidence/issue9-windows-utm-login-20261023.json`. AgentMemory: `mem_mvku85c9_b5bc04ac24b1`. GitHub issue #9 updated with the blocked verification state.

### Gate the manual screenshot test by its opt-in feature — issue #9 / 2026-10-23

- [x] Plan: the manual screenshot path is compiled only with `manual-screenshot-capture`; its integration test must follow the same feature boundary.
- [x] Red: the first local `cargo test --workspace` failed because `tests/native_ui_manual_screenshot.rs` imported the feature-gated function in the default feature set. The target guard recognizes the existing crate-level `native-ui-probe` gate pattern. Generated English HTML was stale at 0.1.36 while Markdown described 0.1.37.
- [x] Task: gate the test source on `native-ui-probe`, declare its Cargo target with `required-features = ["manual-screenshot-capture"]`, then regenerate both offline manuals using the repository's Rust `manual render` command.
- [x] Verify: `native_ui_targets`, `manual_renderer`, and the opt-in screenshot test pass 8/8; `cargo test --workspace` passes 959 tests, 14 ignored, across 319 suites. `cargo fmt --all -- --check` and `git diff --check` pass.
- [x] Knowledge: Graphify refreshed the ignored code graph to 9,143 nodes, 14,340 edges and 709 communities; no Graphify LLM key was used.
- [x] Complete: committed and pushed as `4348b9e` on `codex/issue-9-native-ui-parity`; this test-infrastructure correction adds no packaged UI acceptance.

### Diagnose the existing UTM Z: share through the guest-agent boundary — issue #9 / 2026-10-23

- [x] Plan: preserve the user-configured Z: mapping and verify the exact archive in the user-provided host folder; do not change the share or restart the VM.
- [x] Task: rechecked `AgentConductor-d1e1b374.tar.gz` in `/Users/neodamul/Downloads/UTM`; SHA-256 is `6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd`.
- [x] Verify the boundary: UTM `file pull` uses the guest-agent file API. The Windows guest-agent path lookup for Z: returned path-not-found. Windows drive letters are logon-session-specific; this result does not establish whether the signed-in user mapping is available.
- [x] Preserve state: UTM remains started, input capture is off, and no share change, VM restart or shutdown was performed. The Guest Tools restart-apps confirmation is visible; see the live recheck at the top of this plan.
- [ ] Verify: inspect Z: from the logged-in Windows desktop and run the exact-source Windows build/tests. Restart remains gated because the prompt warned that three apps would close.
- [ ] Complete: close the Windows gate only after the interactive Z: file and exact-source build/test results are recorded. Official references: [UTM Windows guest support](https://docs.getutm.app/guest-support/windows/) and [Windows drive mappings across logon sessions](https://learn.microsoft.com/en-us/windows/win32/services/services-and-redirected-drives).

### Diagnose PR #22 native review-step CI failure — issue #9 / 2026-10-24

- [x] Plan: compare the current macOS CI failure with the focused native UI test on exact PR head `f69b474743496589d8324523428cfc12e201d684`; retain issue #9 and PR #22 as open/draft.
- [x] Verify: macOS run `39994106814` fails after 16.16 seconds in `review_approval_shows_start_feedback_then_opens_the_started_run`. The final labels still include the workspace goal editor and step navigation but no review task rows, indicating the failure occurs around the click into Step 3; the log does not prove the precise cause.
- [x] Verify locally: focused journey passes four standalone repetitions. The full offline native-feature test command reported 679 passed and 14 ignored across 216 suites, but the shell ended with exit 130 after an interrupt during teardown, so do not count it as a clean full-suite pass.
- [x] Verify Windows: current-head CI completed native tests, opt-in Rust UI launch, rustfmt, and warning-denied lint successfully.
- [ ] Task: isolate the Step 3 click/navigation under the macOS CI environment and add an unambiguous checkpoint before changing product behavior.
- [ ] Complete: require clean current-head macOS and Windows validation before closing this unit; no UI acceptance credit is added by these tests.
- [x] Plan: review the Google Rust Style Guide against the repository (read-only measurements on `main` at `b7fa27a9`); adopt sixteen review rules for new and changed code, exclude Chrome/Fuchsia-specific, unstable and library-only rules, add no executable gate (every candidate lint fails on many existing sites).
- [x] R-174 to DD-185 added to documents 01 to 06 and the new `docs/13-rust-review-policy.md`; README map and traceability row; CHANGELOG. No source or test file changes, so there is no Red test; the measured baseline is recorded in the policy.
- [x] Verify: `cargo fmt --check`, `cargo test --test source_layout --test public_docs` and `git diff --check` pass locally.
- [ ] Complete: CI green on the issue-linked pull request, merge into `main`, then port the document and entries to the Rust migration line with a `Related to #169` pull request. Open follow-up (not part of #169): a `SAFETY:` comment pass over the 125 `unsafe` blocks before `undocumented_unsafe_blocks` can become a gate.

### Safety comments on every unsafe site — GitHub #172

- [x] Plan: R-175 to DD-186 added; red baseline on macOS with `-W clippy::undocumented_unsafe_blocks`: 85 violations (58 in `src/`, 27 in `tests/`); Windows-only sites are counted by Windows CI.
- [x] Task: safety comments on all `unsafe` blocks and `unsafe impl` in `src/` (cocoa, icons, pickers, process, leases, guard, quota terminal, availability watcher, job object); `icons.rs` split into `icons/macos.rs` and `icons/windows.rs` for the 500-line limit; `geteuid` called once in `leases.rs`; the `BITMAPINFOHEADER` struct update replaced by direct field assignment.
- [x] Decision: test targets are excluded from the gate (26 environment mutations are not sound by a local invariant); the gate is a CI step, not a `Cargo.toml` entry.
- [x] Verify (macOS): the gate command reports zero for `--lib --bins`, with and without `native-ui-probe`; `cargo fmt --check` passes.
- [ ] Verify (Windows): the new CI step, Clippy, tests and package build pass on `windows-latest`.
- [ ] Complete: CI green, merge, port to the Rust line, correct the file count in `docs/13-rust-review-policy.md` (done: 17 files).

### Integrate latest main into the issue #9 Rust branch — 2026-10-24

- [x] Plan: fetch current `origin/main` (`3bcfebf219087c423565f96e1ea3bea9464093da`) and merge it into the issue-linked Rust branch to clear PR #22's GitHub merge conflict.
- [x] Task: preserve main's #172 safety rationale and latest Windows quota/picker behavior while keeping the Rust branch's module splits within 300 lines; retain the native provider-icon decoder and reduced-motion hook needed by the Rust UI. Resolve picker call sites and keep the obsolete WebView Cocoa launcher removed.
- [x] Red: the first merged build exposed missing icon decoding and picker signature mismatches. Green: restore the bounded PNG data-URL decoder and use the current picker API.
- [x] Verify: focused journey/docs/layout tests pass 14/14; `cargo fmt --all -- --check` passes; safety Clippy passes with `-D clippy::undocumented_unsafe_blocks`; full native-feature suite exits 0 with 959 passed and 14 ignored across 319 suites. The failure-era 679-test run above was interrupted; this newer full run supersedes it.
- [ ] Verify: commit and push this merge, then require current-head macOS/Windows CI. Previous CI run `39994106814` predates this merge: Windows passed; macOS Step 3 navigation failed and remains unconfirmed against current main.
- [ ] Complete: keep #9 open and PR #22 draft until native UI acceptance and packaged macOS/Windows end-state gates pass.

### Recheck UTM guest access after the user-approved reset — issue #9 / 2026-10-24

- [x] Plan: honor the user's explicit approval to reset the existing Windows VM after UTM warned that unsaved guest changes would be lost; keep the existing share configuration unchanged.
- [x] Task: restart the existing `Windows` VM. `utmctl status` reports `started`; `utmctl ip-address` returned `192.168.64.6` and IPv6 addresses.
- [x] Verify guest channel: one read-only `utmctl file pull Windows C:\Windows\win.ini` returned the file successfully. A guest `cmd.exe` command writing `dir Z:\` output to a temporary guest file returned undecodable localized text consistent with a missing-drive error; do not treat that as proof about the signed-in user's mapped drive.
- [ ] Verify UI and share: the UTM screen still reads `Display output is not active`; a later `query user` / `net use` guest command and subsequent file pull did not return and were interrupted. Interactive desktop, Z: mapping, exact-source Windows build/tests and package execution remain unverified.
- [ ] Complete: restore a reliable interactive or guest-agent channel, inspect the existing Z: mapping without changing it, and run the exact PR-head Windows checks. No further VM power action or share change was made. Evidence: `docs/evidence/issue9-utm-windows-recheck-20261024.json`.

### Build and verify the exact current-head macOS Rust package — issue #9 / 2026-10-24

- [x] Plan: build the current issue-branch head with the existing Rust package command; do not open another desktop window.
- [x] Task: `cargo run --offline --locked --release -- package-release` built package source `8ef672bd2d94ae38ed6caeab4f1d93d8df52d47c`, version 0.1.37, for `aarch64-apple-darwin`.
- [x] Verify: package validation, ZIP integrity, and the built-in package smoke passed. All four shared/worktree × Git/folder cases ran three parallel tasks and three tests each; approved merges succeeded and unapproved merges were refused.
- [x] Supply chain: the 18,858,304-byte binary SHA-256 is `95746fd5052c7c4fde4472ecce7afa7db2b01ec38a94c612bc9f15b4e0a78e4e`; the 21,672,397-byte archive SHA-256 is `605ee5e9b65e26e41a9a2c5b9a9a4151fe41655bc5ee1d3cd5dc12e4e1bfa519`. Manifest and SBOM each report 242 dependencies; bilingual HTML/Markdown manuals, SBOM, dependency licenses, notices, and project license are included.
- [x] Assetize: record exact package and smoke result in `docs/evidence/issue9-local-macos-package-8ef672bd-20261024.json`, the native-release Wiki, and AgentMemory. This is not a GitHub Release or visual UI acceptance; keep #9 open and acceptance at 15/25 packaged, 0/25 shipped/default Rust UI.
- [ ] Remaining: build/verify Windows package and finish current-head cross-platform UI/runtime gates before release.


### User-approved UTM reset follow-up — issue #9 / 2026-10-24

- [x] Task: performed the user's approved UTM Restart action and accepted its explicit reset warning; the existing share configuration was not changed.
- [x] Verify: the VM remains started, but the display still says `Display output is not active`. Sending Ctrl+Alt+Delete did not restore it. Guest control commands did not return and were interrupted; no further power action was taken.
- [x] Verify: documentation-only package evidence commit `868863a45539bce528088a3c109176fe3547d12e` is pushed. Local macOS package smoke/validation and the 13 public-doc/source-layout tests pass; exact Windows behavior is not established.
- [x] Record remote state: Actions run `40001524380` is still in the Windows build step with no update since 18:17 UTC; its macOS job is cancelled. New head run `40003663262` is queued.
- [ ] Remaining: recover a visible Windows desktop or reliable guest channel, then verify the mapped `Z:` drive and the exact Windows package. Keep issue #9 open; acceptance remains 15/25 packaged workflows and 0/25 shipped/default Rust UI. Evidence: `docs/evidence/issue9-utm-approved-reset-followup-20261024.json`.

### Refresh current 0.1.37 Workspace screenshots and public docs — issue #9 / 2026-10-24

- [x] Plan: align the README version with the current stable release, replace stale Workspace captures with exact current-source Rust UI captures, and keep both offline manuals in sync.
- [x] Task: build the opt-in `manual-screenshot-capture` probe from current source and capture isolated English light, Korean light, and Korean dark Workspace screens; no provider CLI or MCP server was configured or invoked.
- [x] Verify: visually inspect all three 1140×740 PNGs; the blank temporary profile contains no user workspace, agents, goal or company connection. The probe's ordinary update lookup is a GET to the public GitHub latest-release metadata endpoint.
- [x] Docs: set README and Markdown manual version labels to 0.1.37; regenerate both offline HTML manuals with the Rust renderer.
- [x] Tests: `readme_screenshot`, `public_docs`, `workspace_screenshot_assets`, and `native_manual_assets` pass (14 tests across four suites).
- [x] Complete: commit this documentation unit locally; do not claim UI acceptance credit. Windows runtime and Agent Room manual captures remain unverified.

### Recheck current-head native UI journey and Windows guest channel — issue #9 / 2026-10-24

- [x] Plan: revalidate the prior macOS Step 3 failure before changing product code; check the same current source locally and inspect the latest exact-source CI run.
- [x] Local verification: `review_approval_shows_start_feedback_then_opens_the_started_run` passes on the current tree (one focused repetition, 5.31 s); `cargo fmt --all -- --check` and `source_layout` + `public_docs` pass (13 tests).
- [x] Remote macOS verification: Platform Validation run `40003966004`, source `bc9b7d3`, passed native tests, opt-in native UI launch, formatting, Clippy, and unsafe-comment checks. This supersedes the earlier macOS Step 3 failure as a current-head blocker; no product change was justified.
- [ ] Remote Windows verification: run `40003966004` passed build and source-layout checks; its all-target native test step is still in progress. Do not claim Windows success until remaining UI-launch, formatting, Clippy and safety checks finish.
- [x] UTM channel check: VM status is `started` and IP `192.168.64.6`; a read-only `cmd.exe /c ver` launch returns OSStatus `-2700` / child-process-not-found. No VM power action or share change was made.
- [ ] Complete: keep #9 and draft PR #22 open; wait for the active Windows check, then continue remaining packaged/default-Rust UI acceptance and exact Windows runtime verification.

### Build and inspect the exact current macOS Rust package — issue #9 / 2026-10-24

- [x] Plan: package committed source `08a1783476756a7b79b4823a91a370fcfd943135` with the latest bilingual manuals and Workspace assets; validate structure, licenses, smoke behavior, and the exact packaged UI in one isolated window.
- [x] Task: `cargo run --offline --locked --release -- package-release` built v0.1.37 for `aarch64-apple-darwin`; binary SHA-256 `5e3fd1dfe84aa0cb7bec6e50918849b3b56bfc676ebda034dfe5df536fe9fd42`, archive SHA-256 `7cb20690d605ecbf79425d06ec885a08e39c78df58a43ed2d4bb722ac1f4272d`.
- [x] Verify: package manifest, ZIP integrity, `--help`, and four smoke combinations passed (shared/worktree × Git/folder; three parallel tasks and tests each; approved merge verified and unapproved merge refused). Package contains 242 dependency license records and a 242-component SBOM.
- [x] Verify packaged UI: in one isolated HOME, confirmed v0.1.37, Workspace, all six Agent Roles, disabled Review controls without a plan, Settings actions, and Workspace menu. No profile, goal, agent, MCP, approval, or merge was created. Closed the one app process and removed its temporary HOME.
- [x] Cleanup: preserved only this latest local package in `dist/`; removed four older generated package/archive pairs and recovered approximately 190 MB.
- [x] Assetize: evidence `docs/evidence/issue9-local-macos-package-ui-08a1783-20261024.json`, Wiki, and AgentMemory updated. No new packaged workflow acceptance credit; the 0.1.37 candidate is local, not a GitHub Release.
- [ ] Complete: Windows build/runtime and final UI acceptance remain open; keep #9 and PR #22 open/draft.

### Windows native tests passed on current validation run — issue #9 / 2026-10-24

- [x] Verify Windows build and 300-line source/test layout on Actions run `40003966004`.
- [x] Verify all-target native Windows tests on the same run; the long test step completed successfully.
- [x] Verify macOS remains green on this exact remote code head; its full validation job passed.
- [x] Verify Windows opt-in Rust UI launch and formatting on Actions run `40003966004`.
- [ ] Verify Windows warning-denied Clippy; this step is active.
- [ ] Verify Windows unsafe-comment gate; it is pending after Clippy.
- [ ] Complete: retain the existing local evidence and wait for the same active run to finish before claiming cross-platform validation.

### Approved UTM Windows guest reset follow-up — issue #9 / 2026-10-24

- [x] Reset the confirmed `Windows` VM using UTM force-stop/start after the user explicitly approved the warned reset.
- [x] Verify the VM returned to `started` and received `192.168.64.6` again.
- [ ] Verify guest execution: `utmctl exec Windows --cmd 'C:\\Windows\\System32\\cmd.exe /c ver'` still fails with OSStatus `-2700`; the UTM console reports “Display output is not active.”
- [ ] Complete: Windows guest tools/display path remains blocked; do not repeat power actions without a new reason. The UTM reset did not alter the package or repository.

### Windows native UI and formatting checks passed — issue #9 / 2026-10-24

- [x] Verify the opt-in Windows native Rust UI launch on Actions run `40003966004`.
- [x] Verify Windows formatting on the same run.
- [ ] Windows warning-denied Clippy failed: `tests/windows_console.rs` re-exported unused `language`; the safety gate was skipped.
- [x] Remove only the unused test-shim re-export; local formatting and warning-denied all-target native Clippy pass.
- [x] Verify local all-target tests: 378 passed, 13 ignored across 110 suites; commit/push this small fix and require a clean Windows CI rerun.

### Exact pushed-head macOS package and replacement Windows CI — issue #9 / 2026-10-24

- [x] Build exact pushed source `278ad6de798e73e2a71502af03117c0004d61e92` as v0.1.37 macOS arm64; package manifest and ZIP integrity pass. Archive SHA-256: `6b3771dbb6343a2874251da678880a2c5df33b1a3ec79be0b0fb44246c4f1c0e`.
- [x] Verify package smoke: all four shared/worktree × Git/folder cases pass, each with three parallel tasks/tests; approved merge verified and unapproved merge refused.
- [x] Cleanup: remove the superseded `08a1783` package and smoke run; retain only the exact `278ad6d` candidate and evidence.
- [x] Assetize: record package and smoke evidence in `docs/evidence/issue9-local-macos-package-278ad6de-20261024.json`; Graphify refreshed to 9,243 nodes / 15,635 edges.
- [x] Remote exact-head validation run `40008320185` completed: macOS passed; Windows build, source/test layout, all-target tests, native UI validation, formatting, and warning-denied Clippy passed. Its unsafe-comment gate failed at `src/native_ui_probe/motion.rs:54`; fix commit `c6c6052` adds the required rationale.
- [x] Fresh remote validation of `c6c6052` passed on macOS and Windows; exact macOS and Windows packages built, rendered manuals, passed smoke, and were retained as run `40011350882` artifacts.
- [ ] Completion remains open: Windows UTM guest execution, UI acceptance 25/25, shipped/default Rust UI, and stable cross-platform release are not verified.

### Restore Rust Workspace back/discard actions from DD-39 — issue #9 / 2026-10-24

- [x] Plan: reconcile R-33 → BG-09 → SF-28 → UC-25 → FR-19/NFR-10 → QAS-26 → DDR-07 → DD-39 against the Rust Workspace titlebar; legacy documentation described the actions, but the Rust header omitted them.
- [x] Red: `dirty_workspace_header_exposes_discard_changes_in_korean` failed with only the workspace-menu and Settings labels exposed.
- [x] Task: add Back to run when the saved snapshot has a run, plus a confirmed Discard changes action that restores local fields and drafts from the last snapshot without sending save, plan, edit, or start requests; reuse `open_run` after confirmed back navigation.
- [x] Verify: navigation/localization suites pass 27/27; workspace request suite 6/6; the local-discard unit regression passes 1/1 and asserts localized detection of all three dirty-area categories; the complete native UI feature test set passes 967 with 14 ignored across 321 suites; recursive 300-line/source-test separation guard passes 2/2.
- [x] Quality: formatting, focused and all-target warning-denied Clippy, unsafe-block comment Clippy, public-doc tests, and `git diff --check` pass locally.
- [ ] Verify packaged interactions: exercise both confirmation choices in the exact packaged Rust UI and verify no workspace mutation/request is sent; do not count source tests as packaged UI acceptance.
- [ ] Verify Windows runtime/UI: the user reports UTM cannot boot and shows no display. The UTM app was not running in the latest read-only host inventory; guest boot/display was not independently inspected. No VM power, reset, or configuration action was taken.
- [ ] Complete: issue #9, UI acceptance, Windows runtime, and release gates remain open.

### Build the exact d89b3c37 macOS candidate — issue #9 / 2026-10-24

- [x] Plan: build and inspect the package after pushing the DD-39 Rust Workspace navigation change; keep build output separate from UI acceptance.
- [x] Task: `cargo run --locked --offline --release -- package-release` produced the v0.1.37 `aarch64-apple-darwin` archive from `d89b3c37bab80b65570bee358a78c6487f8f6e3f`.
- [x] Verify: package manifest and source commit match; binary SHA-256 `34f42d7cd406c6bbe234dce323a16068685bcff02841e4282dccb0663b712579`, archive SHA-256 `c6251f91411ee52043fa0e009a280b3e2802d14aaa248545128a288a89f990d9`; `validate-package`, ZIP integrity, and package smoke pass.
- [x] Smoke: four shared/worktree × Git/folder configurations each ran three parallel tasks and tests; approved merges were verified and unapproved merges refused.
- [x] Cleanup: removed the superseded 6ca19965 package pair and disposable smoke output; retained only the exact d89b3c37 candidate in local `dist/`.
- [x] Assetize: record package identity and smoke evidence in `docs/evidence/issue9-d89b3c37-macos-package-smoke-20261024.json` and the LLM Wiki. This is not a release and adds no packaged UI acceptance credit.
- [ ] Verify UI: packaged back/discard interaction remains unverified because no exact package window was opened; prior AppKit registration failures remain documented.
- [ ] Complete: Windows package/runtime, shipped/default Rust UI, remaining UI workflows, and release gates remain open.

### Read-only UTM state recheck — issue #9 / 2026-10-24

- [x] Verify host state: CUA inventory reports the UTM application is not running.
- [x] Verify VM state without opening or powering it: `utmctl list` reports the registered `Windows` VM as `stopped`.
- [ ] Verify display/guest boot: not observed because the VM is stopped. The user reports no boot/display; no new cause is inferred.
- [ ] Complete: do not start, reset, or reconfigure the VM as part of this read-only check. Windows package/runtime acceptance remains open.

### Recheck Windows guest display and source-level navigation interactions — issue #9 / 2026-10-24

- [x] Verify current UTM state: `utmctl list` reports `Windows` as `started`; the UTM window shows the Windows desktop. This supersedes the earlier stopped/no-display observation.
- [x] Verify Z: is present in File Explorer's navigation pane. Opening its contents was not confirmed; the user reports it is accessible.
- [x] Add real egui pointer-input regressions for Keep editing, confirmed Discard changes, and Back to run event routing.
- [x] Verify local `native-ui-probe` library tests (47 passed), source layout (2 passed), strict all-target Clippy, formatting, and `git diff --check`.
- [x] Assetize: AgentMemory memory `mem_mvm4o3pr_08ad103b50da` and lesson `lsn_86db5f0ccfc8fa0e` saved; Graphify refreshed to 9,351 nodes / 14,573 edges / 713 communities. CocoIndex is unavailable on this host (`rtk cocoindex --version`: executable not found).
- [x] Verify Z: content access from Windows; the shared folder is open in File Explorer and the exact-head package is visible. Do not power-cycle or reconfigure the VM.
- [ ] Verify direct Windows guest package/runtime behavior; CI package execution is recorded separately.
- [ ] Verify these interactions in the exact packaged Rust UI; source tests do not count as packaged UI acceptance.
- [ ] Complete: #9 and release gates remain open.

### Full local validation and exact-head cross-platform package build — issue #9 / 2026-10-24

- [x] Verify: `cargo test --locked --offline --all-targets --features test-fixtures --no-fail-fast` passes 989 tests, 14 ignored, across 321 suites in 263.15 seconds on head `3b367a28`.
- [x] Confirm the earlier package artifact came from source `70db73e`, which differs from current head in Rust source; remove only the stale ZIP staged by this session from the Z: host share.
- [x] Start workflow dispatch `40078615551` at exact branch head `3b367a28758248661a12d9897da6e72e25574900`; both Windows and macOS validation jobs passed.
- [x] Verify current-head Windows and macOS package jobs and inspect manifests/checksums; stage exact-head Windows ZIP to the accessible Z: share.
- [ ] Verify Windows guest execution and packaged UI interaction; source tests and Actions runner smoke are distinct evidence.
- [ ] Complete: #9 and release gates remain open.
