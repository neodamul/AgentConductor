## [2026-10-25] verify | Windows schedule UIA window-focus regression — issue #9

Exact-head run `40170014908` passed all macOS checks but failed the Windows schedule workflow: pre-edit UIA, refreshed UIA, and physical click attempts sent input without persisting `manual_schedule`; no coordinator action appeared. Comparison with the last passing package driver (`40119945464`) found it called `window.SetFocus()` immediately after locating the controls. The local source contract failed before restoring that call and passes afterward. Focused schedule, retry, and source-layout suites pass 7 tests across 4 suites; rustfmt, diff checks, and 300-line limits pass. Exact Windows verification is pending, and package acceptance stays 21/25. Evidence: `docs/evidence/issue9-windows-schedule-focus-40170014908-20261025.json`.

## [2026-10-25] verify | Three fresh UIA retries did not fix Windows schedule dispatch — issue #9

Exact Windows run `40168107124` disproved repeating the fresh-element UIA attempt: all three emitted `method=uia`, but the report stayed `waiting_availability` / `manual_required`, with no timestamp or coordinator action; Windows validation failed and package validation was skipped. The last passing package run `40119945464` used the pre-edit button element, whereas failed runs `40162937724` and `40168107124` used a refreshed element. The current driver tries pre-edit UIA, refreshed UIA, and physical click once each, and requires durable `manual_schedule` after every attempt. The macOS validation job passed. The updated locked/offline all-target suite passes 999 tests / 14 ignored / 328 suites; final focused tests pass 9 / 8 suites. Rustfmt, warning-denied Clippy, diff and 300-line checks pass. Graphify refreshed to 9,450 nodes / 15,912 edges; CocoIndex CLI unavailable. Acceptance remains 21/25. Evidence: `docs/evidence/issue9-windows-schedule-40168107124-20261025.json`.

## [2026-10-24] fix | Keep Agent Room snapshots responsive during execution — issue #9 / QAS-158

TDD first reproduced a live-task UI snapshot exceeding its two-second bound. Exact package ea672184 showed the rows refreshing but revealed a disabled feedback editor. The AccessKit red test failed before the UI enabled feedback only for running tasks. Package 6aa687a3 verified that running-task feedback is editable, pending-task feedback is disabled, and an alternate-agent selection enables Assign; neither action was submitted. Read-only snapshot access, report-derived watcher restoration, and feedback editing for a running task are implemented; 44 focused tests, all-target Clippy, formatting, and 300-line checks pass. Graphify refreshed at 9,278 nodes / 15,677 edges. CocoIndex could not start because `mcp.server.fastmcp` is missing from its CLI environment. Packaged UI count is 18/25 (72%); shipped/default Rust UI stays 0/25. Evidence: `docs/evidence/issue9-qas158-live-snapshot-20261024.json`.

## [2026-10-23] verify | Transfer current #9 PR head through UTM Z: — issue #9

Archived exact PR head `d1e1b3746b3ea08cbaa09c1ac167e0c569c9810d` and copied it to the user's existing `/Users/neodamul/Downloads/UTM` share folder. The source and host-copy SHA-256 values match. Windows still shows an app-restart prompt, so guest-side Z: access and Windows build/test remain unverified; no VM restart or shutdown was selected. Evidence: `docs/evidence/issue9-utm-z-share-head-20261023.json`.

## [2026-10-23] verify | Refresh issue #9 PR conflicts and UTM transfer state

At branch head `8b50d0bd`, GitHub reports PR #22 open/draft, `DIRTY`, with no checks; `origin/main` is `602ec118` and the latest stable release is v0.1.37. A read-only merge-tree now reports 39 conflict paths, so the earlier changelog-only summary was incomplete. UTM Windows remains started but has no available QEMU Guest Agent. A source-only archive for the exact Rust branch head is staged on the host and not transferred; Windows build/package/UI verification remains unclaimed while Guest Tools action-time authorization is pending. Acceptance stays 15/25 packaged and 0/25 shipped/default Rust UI. Evidence: `docs/evidence/issue9-utm-build-transfer-20261023.json`.

## [2026-10-23] verify | Reconcile latest main feature changes with Rust — issue #9

At the earlier parity checkpoint, the current Rust branch passed 42 tests across ten coordinator/amendment/charter/quota/process suites. `origin/main` was `b7fa27a9f033b91df5787e05833c30db471645e5`; its ten main-only commits touched 56 files. The non-mutating merge-tree analysis showed broad source/design/manual/Wiki conflicts and three shipped JavaScript UI paths deleted by Rust, so no blind merge was attempted. Issue #9 acceptance was 15/25 packaged workflows and 0/25 shipped/default Rust UI workflows. Evidence: `docs/evidence/issue9-main-feature-parity-20261023.json`.

# Knowledge and development log

## [2026-10-24] verify | Recheck QAS-29 in exact v0.1.37 package — issue #9

The exact Actions macOS package ran under an isolated HOME and disposable repository. Generate plan stayed disabled without a coordinator; manual planning created three empty tasks immediately; confirming the incomplete plan was refused before the registered executable ran. Evidence: `docs/evidence/issue9-qas29-plan-gate-source-20261024.json`; AgentMemory `mem_mvlwz2up_715f5973b7b8`. Packaged acceptance remains 17/25 because QAS-29 was already counted. The temporary app process and test root were removed.

## [2026-10-24] user report | UTM Windows no longer boots — issue #9

The user reports that after the prior VM reset Windows does not boot and the display is unavailable. Earlier `utmctl` reachability and file reads are superseded as current-state evidence. Do not power-cycle, reset, change UTM settings, or modify the Z: share; further work requires a read-only diagnosis authorized by the user.

## [2026-10-24] verify | Cross-platform release package and smoke — issue #9

Run `40011350882` passed macOS and Windows validation, release builds, Rust bilingual manual rendering, exact-package smoke, and artifact retention. The macOS artifact is `12527489015` (21,164,379 bytes; artifact SHA-256 `faf6d2a0acea970638e4c59697d77175737c44e8dd79fc51ccc8971ca0703a79`; package archive SHA-256 `47075ce7a6908926d14d049e8613a02bce862170ff58bf3992b1c17f41ef7629`). The Windows artifact is `12528158860` (15,301,355 bytes; artifact SHA-256 `02bc04f6cd86b4e9bdaa96c6d05bac58d46a94b5ee6b377dbaa1adf546b7e856`; package archive SHA-256 `9927f872a25505804c3f07ccff70ef33e318c49fdab83d2e15fe599c5a19e8d1`). Both are Actions artifacts expiring 2026-11-06, not a published GitHub Release. Exact packaged UI acceptance remains 16/25; shipped/default Rust UI is 0/25. Evidence: `docs/evidence/issue9-platform-packages-40011350882-20261024.json`.

## [2026-10-24] fix | Document Windows motion-query safety — issue #9

Actions run `40008320185` passed Windows build/layout, all-target tests, native UI validation, formatting, and warning-denied Clippy, then found a missing `SAFETY` comment on the Windows-only `SystemParametersInfoW` call in `src/native_ui_probe/motion.rs`. Commit `c6c6052` documents the valid writable BOOL result pointer without changing behavior. Local rustfmt, host Clippy, source layout (2/2), and diff checks pass. Windows-specific verification requires a fresh Actions run. Evidence: `docs/evidence/issue9-windows-unsafe-comment-fix-20261024.json`.

## [2026-10-24] verify | Exact-package create and archive a non-Git workspace — issue #9

The exact v0.1.37 macOS arm64 package from pushed source `278ad6de798e73e2a71502af03117c0004d61e92` saved a plain-folder workspace, created a replacement, and completed the confirmed archive flow. The registry dropped the inactive workspace while the external project directory remained unchanged and had no `.git`. No agent or plan ran; the app and disposable HOME were removed. Packaged UI acceptance advances to 16/25; shipped/default Rust UI remains 0/25. Evidence: `docs/evidence/issue9-workspace-create-archive-278ad6de-20261024.json`.

## [2026-10-23] fix | Keep native manual table columns readable — issue #9

At `d051c8d4d7cdcb583b6591a8aec0f37db214b246`, the built-in native manual now sizes table columns to available window width and lays cell segments out vertically so wrapped text contributes to row height. TDD captured the initial 48 px column gap and a 77 px row overlap before the fix. Focused manual/layout tests pass 13/13; the exact v0.1.36 macOS arm64 package validated, passed all four smoke modes, and was visually inspected. Acceptance remains 14/25; screenshot refresh and Windows remain open. Evidence: `docs/evidence/issue9-d051-native-manual-table-package-20261023.json`.

## [2026-10-23] verify | Full native-feature regression after Claude quota fix — issue #9

The exact Rust product code at `f43882095aa0308dc2870bd34269e0ea240bab8d` passes the locked/offline release-profile all-target suite: 938 passed, 14 ignored, 0 failed across 310 test binaries in 351.22 seconds. The checked-out head was `8a46c8a0181a5d27440a309d9f68e9420037c858`. This does not add packaged workflow credit; acceptance remains 14/25. Evidence: `docs/evidence/issue9-full-regression-f438820-release-20261023.json`.

## [2026-10-23] verify | Exact-package quota refresh UI follow-up — issue #9

Followed up on the earlier headless-only check: in the exact v0.1.36 macOS arm64 package from `f4388209`, the Claude quota refresh button showed progress, advanced the observation timestamp, refreshed the summary, and returned enabled. The screenshot was not retained because it exposed account usage and a local project path. This component result does not add a distinct packaged workflow; acceptance remains 14/25. See `docs/evidence/issue9-claude-quota-f438820-macos-package-20261023.json`.

## [2026-10-23] verify | Package Claude quota CLI path fix — issue #9

Built pushed source `f43882095aa0308dc2870bd34269e0ea240bab8d` as a v0.1.36 macOS arm64 candidate. Rust package validation passes with 214 dependency records and no missing license expressions; packaged `--help` succeeds; built-in smoke passes four isolated shared/worktree × Git/folder cases, each with three parallel tasks/tests, approved merge success, and unapproved merge refusal. Binary SHA-256: `f804e8c8406408f901c7103fed4f341fc0ceedfb1a1e44506d2e9ba5457fc3ec`; archive SHA-256: `8355d7bf0a86c7497ec9cc9775e095e5294133607defca07a1a5820a88887972`. The app was not opened; its existing running windows and workspaces remain untouched. The matching-hash executable then queried the signed-in Claude CLI with a desktop-style minimal PATH; it persisted two windows without recording usage values, and no generation or tools ran. The GUI refresh button remains unverified, and this candidate retains historical manual screenshots. Evidence: `docs/evidence/issue9-claude-quota-f438820-macos-package-20261023.json`.

## [2026-10-23] fix | Resolve Claude quota CLI for desktop launch — issue #9

The exact v0.1.36 package showed Claude quota refresh failing with `No such file or directory` while the configured executable is the bare name `claude`. TDD reproduced the desktop-PATH mismatch with a deterministic terminal fixture: the test failed before the change and passes after the macOS collector resolves the executable using the shared filesystem-only lookup. No real provider CLI, account, or generation request was used. Formatting, diff checks, and the 300-line source/test bound pass. Exact-package rebuild and UI refresh remain open; packaged workflow acceptance stays 14/25. AgentMemory: `mem_mvkgfbu4_c8be2c1ac746`.

## [2026-10-23] verify | Diagnose issue #9 PR mergeability against main

Fetched `origin/main` at `b7fa27a9`; Rust branch `fe34aa0d` is 656 commits ahead and 15 behind from shared base `e9d1f1ff`. The non-mutating merge-tree check found conflicts across docs, generated manuals/Wiki, Rust coordinator/UI/workflow modules and tests, plus files removed or split to keep the branch Rust-only. No merge/rebase was attempted; worktree remains clean. Keep PR #22 draft until exact conflict resolution preserves modular Rust and the release version. Evidence: `docs/evidence/issue9-main-mergeability-b7fa-20261023.json`.

## [2026-10-23] verify | Reconcile current main with Rust issue branch — issue #9

Compared current main `b7fa27a9` against shared base `e9d1f1ff`; 15 upstream commits follow the base, four after the prior `c9c575a7` audit. The only new behavior is the #165 bounded coordinator lease-retry change already present in Rust as `f7dccadb`; the other changes are v0.1.37 release metadata/manual updates. Six focused Rust suites pass 24 tests at `2fed27cc`. Kept Rust version at 0.1.36 until the migration is release-ready. No behavior or packaged workflow acceptance changed. Evidence: `docs/evidence/issue9-main-parity-b7fa-20261023.json`.

## [2026-10-23] verify / stop — Windows VM access probe, issue #9

Booted the registered UTM Windows ARM64 VM from its stopped state. It reached the Windows lock screen but no authenticated desktop was available; no credentials or project files were accessed. Stopped the VM after the probe and verified that UTM shows Start enabled and Stop disabled, returning its configured 10 GB RAM to the host. No Windows build, package, or UI verification is claimed. Evidence: `docs/evidence/issue9-windows-vm-access-20261023.json`. AgentMemory: `mem_mvkev46a_870b27e35e91`.

## [2026-10-23] maintain | Remove idle Rust debug cache — issue #9

After confirming no Cargo/Rust build was active, removed only the generated `target/debug` cache (9.0 GB). Preserved the 731 MB `target/release`, the extracted 42 MB macOS app under `dist/`, and its release ZIP. The executable remains present and packaged `--help` succeeds. The next debug build must regenerate dependencies. Refreshed Graphify to 8,970 nodes / 14,140 edges / 692 communities; its output is ignored generated data. CocoIndex CLI is not installed, so no CocoIndex result is claimed. This changes no source, app data, or workflow acceptance. Evidence: `docs/evidence/issue9-target-debug-cache-cleanup-20261023.json`.

## [2026-10-23] verify | Probe local UTM Windows VM — issue #9

UTM listed a stopped Windows VM. A hidden disposable boot succeeded, but UTM reported that the QEMU guest agent was unavailable; IP discovery and guest command execution both failed. The host also lacks the `x86_64-pc-windows-gnu` Rust `core`/`std` target. Stopped the disposable VM, verified its stopped status, and closed the VM window created by this probe. No Windows build or runtime pass is claimed; the next local option requires a guest agent, otherwise use an exact-source Windows runner for the release gate.

## [2026-10-23] verify | Build exact-source macOS package — issue #9

Built and packaged committed source `ca9495b2` as an arm64 macOS archive. Release build, Rust packaging, manifest validation, binary help, four mocked CLI smoke cases, ZIP integrity, and archive checksum passed. The packaged executable is 18,606,512 bytes with 214 dependency license records. No GUI was launched and no workflow was counted; the bundled manual screenshots remain historical. Local Windows cross-compilation failed because `x86_64-pc-windows-gnu` core/std are absent. Evidence: `docs/evidence/issue9-ca9495b2-macos-package.json`. Graphify confirms the package driver owns its packaging, smoke and checksum steps; CocoIndex CLI remains unavailable.

## [2026-10-23] task | Bound coordinator lease-race retries — issue #9 / GitHub #165

Latest merged `main` commit `4195f64c` bounded the post-preflight coordinator lease retry to 240 × 100 ms. Rust already retried only `lease is held` and respected the shared deadline, but could retry throughout a long configured timeout. Added the same finite contention budget while preserving deadline and fail-fast behavior. A new bounded-retry regression first failed to compile because the helper lacked a retry-budget parameter. Trace remains R-173 → UC-164 → DD-184; full native-feature verification passed 938 tests (14 ignored, 310 suites) and warning-denied all-target Clippy passed. This is a parity bug fix, not a new workflow.

## [2026-10-23] verify | Inspect exact-package MCP settings hierarchy — issue #9

Opened the exact v0.1.36 macOS package built from `cff95dd02677519ebbb7c359f5ca9e3da0c504ed` in the existing single app window. Checked Korean/dark, English/dark, and English/light: each showed the Settings · MCP servers eyebrow, localized registration heading, and distinct window title. The saved workspace was not modified; no provider, approval, or merge action occurred. Archive SHA-256: `dafc056361c6e68a1165f30db1f3bd8a0cf99d24b59876d7d6eb7d2aa1ad44d8`. Evidence: `docs/evidence/issue9-cff95dd-mcp-dialog.json`. No screenshot file was retained. This component check does not add a distinct packaged workflow; acceptance remains 11/25. Windows verification remains open.
AgentMemory: `mem_mvk7geva_5b15ec7aa847`. Graphify confirmed the MCP editor state/event module. CocoIndex is unavailable; no index result is claimed.

## [2026-10-23] verify | Match MCP settings dialog hierarchy — issue #9

Compared the native MCP registration window with the existing settings dialog. The new bilingual
rendered-text regression first failed because the native view omitted the Settings · MCP servers
eyebrow and the Korean window title duplicated the content heading. The native dialog now shows
the eyebrow and registration heading while naming the window MCP server settings. Eleven focused
tests pass across two suites; warning-denied native-feature Clippy and rustfmt pass. This is a
component parity fix, not an accepted packaged workflow. Exact-package visual verification remains
open because the user's existing application window was not replaced. AgentMemory:
`mem_mvk73wbj_5f33afab5ffd`. Graphify confirmed the editor module's state/event ownership; its
read-only integrity check found 8,916 nodes, 14,081 edges and no dangling endpoints. CocoIndex is
not installed in this environment.
See [native settings dialogs](native-settings-dialogs.md).

## [2026-10-23] verify | Remove duplicate Agent Room goal — issue #9

The exact macOS arm64 package from `f1602e7d` showed the workspace goal once in the wide header;
the narrow task list retained task rows, status, verification, and review controls. The red
rendered AccessKit regression caught the duplicated goal before the fix. Locked/offline serial
all-target tests passed 929 (14 ignored, 310 suites); warning-denied Clippy, rustfmt, diff check,
the 300-line source/test guard, optimized build, package manifest validation, and package smoke
passed. Candidate `3db8840b` stayed `waiting_approval` with 3/3 tasks/tests; no approval or merge
occurred. This is a component correction, not a new packaged workflow; acceptance remains 11/25.
Graphify reports 8,893 nodes / 14,043 edges / 683 communities; CocoIndex CLI is unavailable.
AgentMemory: `mem_mvk4r2u5_d3f75533fb77`. Evidence:
`docs/evidence/issue9-f1602e7-agent-room-goal-layout.json`. Windows UI, accessibility/Korean IME,
resource measurements, and release/license closure remain open.

## [2026-10-23] verify | Keep one packaged app process during UI review — issue #9

Process inspection found the older `dist` v0.1.36 package and the exact candidate package running simultaneously. Saved the older window's current workspace and quit it. A display-name CUA lookup then relaunched the default `dist` package; it was immediately quit using its exact bundle path. Final process inspection found only the candidate-preserving package. Its Agent Room still shows candidate `3db8840b` at `waiting_approval`, with 3/3 tasks and tests complete; no approval or merge occurred. Use exact app paths in CUA, not the display name, when multiple package bundles exist. AgentMemory lesson: `lsn_d8fe0b84cfadeb2c`; session record: `mem_mvk2kxvu_12aaf6979508`.

## [2026-10-23] verify | Recheck Windows CI readiness on the Rust branch — issue #9

On source `43463f4b`, locked/offline serial runs of `cooperative_worker`, `process_timeout`, and `windows_containment` pass on macOS. `public_docs` and `source_layout` pass 13 tests; rustfmt and diff checks pass. This is source-level revalidation, not a Windows run. The Mac lacks the Windows Rust standard library and MinGW linker, and the UTM Windows VM is paused. Graphify: 8,880 nodes / 15,211 edges. The platform gate remains open. AgentMemory: `mem_mvk2es1e_5573fb907b0c`.

## [2026-10-23] verify | Packaged native language/status workflow — issue #9

Verified the v0.1.36 macOS package in one window: Settings switches Agent Room labels and task/status tags Korean → English → Korean while the candidate remains `waiting_approval` (3/3 tasks/tests). No approval, rejection, or merge occurred. Current Rust source passes 948 tests / 14 ignored across 310 suites, warning-denied Clippy, rustfmt, and 13 public-doc/source-layout tests. Graphify reports 8,879 nodes / 15,210 edges; CocoIndex is unavailable. Evidence: `docs/evidence/issue9-6f1111c-language-status-workflow.json`; acceptance is 11/25 (44%), with 14 workflows remaining. Windows UI, Korean IME, and shipped/default replacement remain open.

## [2026-10-23] fix | Explain goal-drift judgments in the Agent Room — issue #9

A red bilingual regression found that the audit pipeline persisted and listed
`goal_drift`, but the native Agent Room displayed the raw kind through its
generic fallback. Added a dedicated localized judgment label and a rendered
view assertion for the blocked-run explanation and charter-change evidence.
The five charter, four coordinator-decision and seven native-judgment tests pass;
the source-layout/300-line test, warning-denied focused Clippy, formatting and diff checks pass. Graphify refreshed to 8,788 nodes / 13,906 edges / 678 communities. Exact macOS arm64 package `aa8090405239` passed manifest validation, ZIP integrity and four package smoke cases (3 parallel tasks/tests each, merge refused before approval and successful after approval). Replaced the idle older package process in the same window and inspected the saved Issue 9 Mock Workflow: 3/3 tasks and tests passed, candidate remains pending. This run contains no goal-drift event, so live display of that label remains open. Removed the superseded package bundle and archive after confirming the old process had exited. Evidence: `docs/evidence/issue9-aa80904-macos-package.json`. Packaged workflow acceptance remains 8/25.

## [2026-10-23] verify | Persisted charter and goal-drift guard — issue #9

The plan's step-4 checklist was too broad: implementation and Rust integration
coverage already existed, while exact packaged UI acceptance remained open.
Verified the current source with `cargo test --test charter_contract` (5 passed).
Coverage includes charter round-trip persistence, goal drift before approval and
after approval before merge, audit recording, and legacy report compatibility.
The approval-pending disposable GUI fixture was left unchanged. Packaged
workflow acceptance remains 8/25; no merge was approved or performed.

## [2026-10-22] task/verify | Native ports of main #146 and #149

Integrated main `e9d1f1ff` while retaining removed WebView sources. Preserved
both requirement histories by assigning new native R-168–R-171/DD-179–DD-182.
Rendered tests first failed for the fold control, task judgment text and wrong
coordinator-row order. The native inspector now separates saved preference from
decision opening; the coordinator/count row follows tasks and reuses the central
activity/log/lesson view through the existing asynchronous worker. Focused
validation passes 49 tests across eight suites. Full validation and exact-package
acceptance remain open; no new packaged workflow is counted yet.

The broader checkpoint passed 914 tests, 14 ignored, across 303 suites. Review
also caught an unwanted automatic Activity-tab change, count/list reads from
different audit versions, and a new-field filter hiding original tool inputs.
These were corrected; the final rich-tool-context follow-up passes ten affected
tests and warning-denied all-target Clippy, formatting and diff checks.
AgentMemory: `mem_mvjhxm23_011f01ecb05d`.

## [2026-10-22] completed | Shared-sidebar metadata polling — issue #9

Fixed a one-condition native UI omission: metadata completion was polled only
on Team, leaving Workspace settings and Review spinners stale after a collector
had exited. Verified the cause in the v0.1.36 macOS window by switching to Team;
the completed Claude observation and refresh button immediately appeared. A
rendered real-worker regression fails on step 0 before the fix and passes on all
three steps after it, preserving unsaved name/goal and avoiding idle fast polling.
Local validation: 39 tests across six suites, warning-denied focused Clippy,
rustfmt and diff checks pass. AgentMemory: `mem_mvjayz0d_02d7203a86d2`.
This component fix does not add a packaged workflow to the 8/25 acceptance count.

## [2026-10-22] verify | Partial package-license audit — issue #9

Audited the paired v0.1.36 GitHub Actions package artifacts from run `39740270740`, built from merge source `eff9f448` (PR head `cd9935de`, base `3fecd21d`). macOS/Windows SBOMs contain 214/201 components, with no missing license expressions. Embedded archive and executable hashes match; both packages include dependency notices and both license texts for the sole GPL-optional dependency, `self_cell 1.3.0` (`Apache-2.0 OR GPL-2.0-only`). The permissive Apache option is available; final-release-source closure remains pending. Evidence: `docs/evidence/issue9-eff9f448-license-audit.json`. AgentMemory `mem_mvja8ga0_477e45de583d`. Graphify refreshed to 8,496 nodes / 13,475 edges / 645 communities; CocoIndex failed to import `mcp.server.fastmcp`.

## [2026-10-22] verify | Exact merged worktree-cleanup package — issue #9

Pushed merge commit `cd9935de0d598612af1f52c1a11b699c1714de26` after full native-feature tests (873 passed, 14 ignored), warning-denied Clippy, rustfmt, and diff checks. The exact macOS package's manifest, checksum, help command, and deterministic fixture smoke passed in shared/worktree × Git/folder modes. Each of four cases completed three parallel tasks and tests, rejected pre-approval merge, and verified post-approval merge; no real CLI or GUI was used. Package evidence: `docs/evidence/issue9-cd9935de-macos-package.json`.

Graphify: 8,479 nodes / 14,574 edges. `cargo clean` removed 38,456 files / 10.8 GiB, and the superseded `ead895e` macOS package pair was removed while retaining the exact `cd9935de` package. GitHub run `39740270740` passed all validation and package-execution jobs on macOS and Windows. Artifacts: macOS `12409469483` (`sha256:c1a1b6efda2c88180815aafb38f07b05c94dbd6f9298dbf61236083f4104724f`), Windows `12409824890` (`sha256:5d2e0cf637a586cf1b3a05cbed2d4c93a509917e00074533d1515b7a357d18ed`). Packaged UI acceptance remains 8/25. AgentMemory: `mem_mvj800lg_5c940c9a959d`.


## [2026-10-22] fix | Release run-owned Git worktrees safely — GitHub #143 / issue #9

Merged main `3fecd21d` so merge/reject finalization removes checkouts under the run folder, while
cache clearing and workspace archival release them before deleting run metadata. The repository
helper preserves unrelated worktrees and records cleanup failure without undoing a final run. A
red source-layout check found the incoming method made `src/git.rs` 329 lines; it was moved into
`src/git/worktrees.rs` without changing its public caller contract. Focused cleanup/layout tests
pass 6/6; full native-feature all-target tests pass 873 with 14 ignored across 298 suites; combined
feature Clippy with warnings denied, rustfmt, and `git diff --check` pass. Packaged UI acceptance
remains 8/25. Details: `knowledge/wiki/native-rust-ui-migration.md`.


## [2026-10-22] verify | Exact PR #22 head passes macOS and Windows package execution — issue #9

Actions run [39735141070](https://github.com/neodamul/AgentConductor/actions/runs/39735141070)
for exact pushed head `287e9f2608bbacb9dd9d7d1983e00dba2d7ef8d2` passed both platform validation
jobs and both native-package jobs. Each platform built the Rust executable, rendered the bilingual
manuals with that binary, executed the exact package smoke workflow, and retained an artifact. macOS
artifact `12407514908`: 21,600,253 bytes, SHA-256
`1151a58d91fafa17bb3900bd34edf24418ae98857a45e64ac642cbc271adcd08`. Windows artifact
`12408746101`: 15,718,557 bytes, SHA-256
`efda584bf92e48290e478ab2ccb820c55cf6bf9dbd0da28b62f806e04b3eabb8`. Source product code is
`ead895e`; the exact pushed head adds documentation only. Packaged UI acceptance remains 8/25, so
this CI pass does not complete issue #9 or authorize a release. Graphify no-cluster refresh reports
8,456 nodes / 14,546 edges. CocoIndex Code could not start because `mcp.server.fastmcp` is
missing. Machine evidence:
`docs/evidence/issue9-287e9f2-exact-head-ci.json`.

## [2026-10-22] cleanup | Remove superseded native package build outputs

Removed twelve ignored macOS package directories and ZIPs for superseded source revisions, reclaiming
about 750 MiB. Kept the latest `ead895e` package, archive, checksum, and tracked source evidence.
`dist` decreased from 813 MiB to 63 MiB. No source, user data, or active test fixture was removed.


## [2026-10-22] release | Publish stable macOS and Windows v0.1.36 packages

Release workflow `39728295847` passed both platform validation jobs, exact package execution,
and publication for source `6d0093d3576e50642bad39fa885b9b5695b0281c`. The latest stable
release is [v0.1.36](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.36).
The downloaded `SHA256SUMS` matches both uploaded ZIP digests: macOS Apple Silicon
`7af9cdc0f3631f60a3e8f58a48923c04bca70a01cbaf6ca1cf73322341ecc415`, Windows x86_64
`c6970e8ffec84a6c2b42a6490c2840130d7ba6aeadc6afaf6f594b17539216bb`. Both package jobs
rendered the bilingual manuals with the Rust binary and executed the exact package. The
release does not close issue #9: native UI acceptance remains 8/25 and PR #22 is still draft.

## [2026-10-22] verify | Distinguish stale UI bundles from current popup behavior — issue #9

At commit `dbf893a18c4d73bc611c984fcc031684346ef67a`, exact-head Actions run `39613489337` passed
all Windows/macOS platform and native-package jobs. The v0.1.33 current-source package at
`/private/tmp/AgentConductor-1f75758.app` was opened directly; Workspace, Settings, and coordinator
selection popups float over the page without moving the roster or main panel. The screenshot that
appeared to push content came from the older `dist` bundle built at 16:29, before the popup code
was built at 18:06. The focused layout suites passed 14 tests across five suites, plus formatting
and diff checks. This is component/runtime evidence, not another end-to-end workflow; acceptance
remains 8/25 and the stable shipped UI remains WebView. AgentMemory lesson `lsn_ece6924167d840c3`
records the build-provenance check for future visual QA.

## [2026-10-21] verify | Guard asynchronous Agent Room workspace return — issue #9

The native all-target run exposed a stale-response race after returning from Agent Room. Trace showed the header allowed view switching while a Room request was pending; its response could then be mistaken for the Workspace refresh. Added a failing UI test for the enabled return action under pending work, then disabled the header action until the request completes. Return-control, start/return journey, and source-layout suites pass 8 tests; Clippy with warnings denied, Rustfmt, diff check, and 300-line checks pass. The platform workflow now runs `native_room_next_step` on macOS and Windows. The full release all-target suite was stopped after more than ten minutes of LTO test-crate recompilation and is not claimed. Prior Actions run `39545652734` passed all four jobs on `e0de50b`; exact-head CI/package verification remains required. Packaged workflow acceptance is unchanged at 8/25. AgentMemory `mem_mvhoklpd_31cd94816a25`.

## [2026-10-21] verify | Check Agent Room header geometry — issue #9

Added a regression over rendered text bounds for the Agent Room context header in English and
Korean at 792px and 1140px. An initial AccessKit-node assertion was inappropriate for passive labels,
so the test now checks the visible `TextShape` bounds without adding accessibility metadata or
changing product code. The header, layout, and rendering suites pass 17/17; format and diff checks
pass; the 129-line test stays within the 300-line limit. Packaged workflow acceptance remains 6/25.
AgentMemory `mem_mvhb36q0_2892617eb052` was saved and Graphify refreshed to 7,724 nodes / 12,215 edges.

## 2026-10-21 issue #9 packaged stage-marker parity

Inspected the exact v0.1.31 package from `ca5c3f4` under a disposable `HOME`. Workspace, Agent Roles, and Review each show the selected step marker in both the page eyebrow and active card. The checked-in reference has the same layout, but its `v0.1.24` label is historical: `native-screenshot-provenance.md` records it as an earlier Rust build, so it cannot prove parity with the published WebView UI. Corrected the design guide, implementation checklist, completion audit, and package evidence record to preserve that limit. Focused UI/docs/source-layout tests pass 27 tests across four suites, as do formatting and diff checks. The exact package scan found no JavaScript/TypeScript/Python files. The new source-layout guard rejects non-Rust UI/build sources and its first red run exposed 20 ignored `.pyc` cache files under `tests/__pycache__`; those generated files were removed and the guard passes 2/2. The full native-feature all-target suite passes 797 tests, 14 ignored, 0 failed across 263 suites (220.67s); all-target Clippy is clean. Graphify re-extracted the repository at 7,557 nodes / 11,988 edges. CocoIndex is unavailable in this environment because its Python environment lacks `mcp.server.fastmcp`; no dependency was installed. AgentMemory record `mem_mvgzgobf_a05025be5c13` predates the completed suite. Packaged acceptance remains 6/25 and shipped Rust UI remains 0/25. The current screenshots were inspected but not retained.

## 2026-10-20 issue #9 grouped workspace and settings audit

Audited Workspace settings, Agent Roles, Review the plan, the Settings menu, MCP registration, workspace sharing, Messenger notifications, and cache cleanup together in the exact isolated v0.1.31 package. The saved-state plan button still navigated without a registered coordinator and surfaced the backend error late. Gated it on a nonempty saved goal and a coordinator ID present in the profile list; added localized guidance and a UI regression. Also stabilized the settings journey to wait for dialog content and the start journey to wait for the previous UI action to settle before merging. Focused planning tests pass (2); settings/start/docs/layout suites pass (10). The all-target local run was stopped after five minutes at 363 passed, 14 ignored across 120 suites without a reported failure; full-suite verification remains open. Actions run `39424348558` failed on both OSes in settings journey, plus a Windows merge race; its head predates these local fixes. Packaged acceptance remains 3/25; no additional workflow is claimed.

Commit `4a114ec6a76141c48ebe8138694398572cae1acd` was packaged and validated locally. The exact macOS package's help command and manifest validation pass; its single isolated UI shows Generate plan disabled with “coordinator not registered” after a goal is saved without profiles. UI evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`. GitHub Actions run `39428920229` is pending; packaged acceptance remains 3/25.

Continued the same issue #9 batch across Agent Roles, empty Review, MCP server registration, workspace-sharing controls, Messenger notifications, and cache preview in the exact package. All expected controls rendered; Approve remained disabled without a reviewed plan, the empty cache preview disabled Clear, and no settings or workspace data changed. Evidence: `docs/evidence/issue9-4a114ec-settings-batch.json`. The grouped native UI suite batch passed 122 tests across 23 suites; the 3/25 packaged workflow denominator is unchanged.

Completed a positive-path exact-package UI workflow with one local Rust fixture CLI: register and select the coordinator, generate/review the plan, approve execution, complete the task, pass 1/1 test, review and approve the diff, then confirm merge. The run reached Completed and created `one.txt` only in the disposable project. Initial fixture typo `/usr/bin/test` was corrected to macOS `/bin/test`; a fresh project run passed. Evidence: `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`. This adds Workflow 4 and advances packaged acceptance to 4/25 (16%); shipped Rust replacement stays 0/25. Follow-up public-doc/layout tests pass (8 tests / 2 suites), and rustfmt/diff checks pass. Actions run `39429494121` was cancelled during native build/test on both OSes. Graphify re-extraction reports 7,509 nodes / 12,943 links with no source-graph changes; CocoIndex remains unavailable due missing `mcp.server.fastmcp`.

Grouped a further exact-package read-only inspection across Workspace phase analysis, Agent Room candidate review, and Updates. The accessibility tree showed the expanded analysis state and phase/task context, exact candidate revision/diff and passing test, plus a release-notes link and scrollable full notes. Reused the one existing app process; the public release metadata check was requested, but the link was not opened. Five source suites passed 23 tests and native-feature all-target Clippy is clean. This component batch does not change the 4/25 workflow count. Evidence: `docs/evidence/issue9-4a114ec-review-updates-batch.json`.

Inspected the same exact package in dark mode at 1140×772, then restored light mode. Completed status, test result, diff tab/body and inspector were readable. The completed run has no enabled primary green action, so button text contrast remains open. Five grouped theme/layout suites passed 15 tests; rustfmt and diff checks pass. Actions `39431587615` was cancelled on both OSes; no run is reported for docs head `459ba84`. Evidence: `docs/evidence/issue9-4a114ec-dark-theme-audit.json`.

Rechecked the full parallel UI path in the same exact v0.1.31 app process and an isolated workspace. A three-session Rust fixture produced three independent tasks with disjoint paths; run records show overlapping execution intervals, all tasks completed, and all three tests passed. Reviewed the exact diff, approved the candidate, confirmed app-level integration, and observed Completed. In dark mode, the enabled approval action and Completed panel remained readable; restored light mode afterward. This revalidates the existing 4/25 parallel workflow rather than adding another acceptance item. The target was a non-Git folder, so no Git merge is claimed. Evidence: `docs/evidence/issue9-4a114ec-parallel-ui-plan-to-merge.json`.

In the same run, inspected Agent Room Activity for each task, Raw logs, Lessons, and the read-only team briefing. Each task showed the matching result; the selected task exposed a JSON result and no stderr errors; Lessons explained draft promotion; the briefing showed all three Completed with no action required. Native session ID/model are unknown for this mock CLI, so real provider-session metadata still needs separate verification.

The complete local native-feature all-target suite at this product source now passes 770 tests, with 14 ignored and 0 failed across 260 suites (216.85 seconds). All-target Clippy with warnings denied passes. Exact package tests are macOS only; Actions returned no run for the docs-only head and Windows packaged UI remains open.

## 2026-10-20 issue #9 packaged workspace workflow

Ran the exact v0.1.31 macOS package from `a8be85f` under an isolated `HOME` so the menu contained no real-user workspace names or paths. Verified UC-30/31 as one flow: create named workspaces, save settings and role stage, confirm unsaved-state cancellation, switch and reload persisted values, remove from the recent list and reopen, then archive an inactive workspace with confirmation. One app process; no provider or external service. Evidence: `docs/evidence/issue9-a8be85f-workspace-ui.json`; binary SHA-256 `74204cbc9492379a71769a6f679d19ee28012b823e280e8a6fdd03af89300057`. Added distinct Workflow 2; packaged acceptance is 3/25, shipped Rust remains 0/25. Windows, screenshots and remaining workflows are still open.

## 2026-10-20 issue #9 grouped UI continuation

Confirmed Claude's latest AgentMemory handoff signal (`sig_mvgo0qng_9ea40889a2f2`) and continued issue #9 only. A grouped native UI scan found no TODO/FIXME/unimplemented markers; all six native custom dialogs are already covered by the current grouped close-action batch. Full offline native-feature verification at PR head `27e307e`: 768 passed, 14 ignored across 260 suites; Clippy (`-D warnings`), formatting, and diff checks pass. Exact-head GitHub run `39423261706` is pending on macOS and Windows. No new packaged UI workflow is claimed; the accepted count remains 2/25. The active saved workspace was read-only and unchanged.

## [2026-10-20] fix | Restore sequential Settings dialog accessibility — issue #9 / R-17 / DD-160

A connected native-probe journey found that Settings → MCP had no accessible in-content Close action: clicking the expected Close target left Save visible. Added a localized Close button beside Save, preserving the save request path. The grouped journey opens and closes MCP and cache dialogs, then verifies workspace-sharing remains accessible. Local all-target tests pass (767 passed, 14 ignored across 260 suites); focused related suites pass (54 tests/12 suites); public-doc/source-layout gates pass (8 tests/2 suites); Clippy, formatting, and diff checks pass. Packaged UI acceptance remains 2/25 until exact-build native review.

## [2026-10-20] fix | Add close actions to remaining Settings dialogs — issue #9 / R-17 / DD-160

Extended the dialog audit across all six native custom windows. The profile editor already had Cancel and cache already had Close; added localized Close actions to the MCP source-reference editor, workspace-sharing dialog, and Messenger settings. Updated sharing's open-state merge so the in-content action is not overwritten by the title-bar state. Native pointer regressions cover the source-reference and sequential sharing/Messenger paths. The 260-suite local run passes 768 tests (14 ignored); Clippy, rustfmt, diff, public-doc, source-layout, and 300-line checks pass. This source-level dialog coverage does not change packaged UI acceptance (2/25).

## [2026-10-20] verify | Build current macOS package from issue #9 branch

Built optimized `aarch64-apple-darwin` binary and exact-source package from `a8be85fc2392821fab345d3e1b879a6aa9bc6714`. Package validation, packaged `--help`, ZIP integrity, and four isolated shared/worktree × Git/folder mock workflow cases passed (12 tasks/tests; all unapproved merges refused and approved merges verified). Binary SHA-256 `74204cbc9492379a71769a6f679d19ee28012b823e280e8a6fdd03af89300057`; archive SHA-256 `1965a123dcd5988c24ffeee4d88bfc2b84a62696faf16e6c893c6d49e9c60547`; evidence is `docs/evidence/issue9-a8be85f-macos-package.json`. The app GUI was not launched; acceptance stays 2/25. Graphify refreshed to 7,492 nodes / 12,924 edges. CocoIndex remains unavailable because its installed environment lacks `mcp.server.fastmcp`.

Refreshing the completion audit exposed stale baseline-hash and CI-status assertions in `tests/public_docs.rs`. Updated the regression to require the current product-code batch and exact package source while dropping superseded run-status requirements; `public_docs` and `source_layout` now pass (8 tests across 2 suites).

## [2026-10-20] verify | Inspect packaged Settings dialog dismissal

Launched the exact `a8be85` macOS package once with a disposable empty workspace and verified Korean/light-mode in-content Close and actual dismissal for MCP server settings, Jira/Confluence reference entry, workspace sharing, Messenger, and cache. No agent or external service ran; the original workspace remained unchanged. Closed the package app and removed its temporary workspace. Evidence: `docs/evidence/issue9-a8be85f-packaged-dialog-ui.json`. This component review does not raise packaged workflow acceptance beyond 2/25; no screenshot file was retained.

## [2026-10-20] docs | Reconcile current issue #9 progress reports

Found stale `bb0ff39` head and over-broad “latest full suite” claims in the completion audit. Updated `docs/09-completion-audit.md` and the native Rust UI Wiki to identify exact current head `b226404`, distinguish historical 759-pass/package evidence at `7419557` from targeted timeout results and the partial integrated suite, and record pending Platform validation run `39412569165`. Refreshed Graphify without clustering: 7,422 nodes / 12,771 edges. CocoIndex Code is installed but cannot start because `mcp.server.fastmcp` is missing from its environment. This does not change packaged UI acceptance (2/25).

Grouped native UI regression verification across Room language/stages, MCP registration/sources/keyboard, the offline manual, and release updates: 56 tests across 17 suites pass locally. This is automated source-level evidence only and does not advance packaged acceptance beyond 2/25.

## [2026-10-20] docs | Align timeout retry guidance across issue #9

Reconciled the timeout contract across DD-07/DD-104, QAS-92/QAS-149, UC-143, Rust recovery comments, both Markdown manuals and their generated HTML, and the Wiki index. Wall-timeouts retry automatically after bounded backoff unless Stop is requested; Resume is for stopped or terminally blocked runs. A new public-doc regression first failed against the stale manual text. The Rust manual renderer regenerated both HTML files; public-doc and source-layout tests pass (7 tests), as do rustfmt and diff checks. This documentation correction does not change packaged UI acceptance (2/25); the remaining 23 workflows and platform/accessibility/resource gates remain open.

## [2026-10-20] fix | Port timeout auto-resume and Stop-aware backoff — issue #9

Integrated the latest `main` reliability batch for per-root Git index isolation (#70), timeout-stop test coverage (#71), and messenger state re-notification (#72). The Rust scheduler had lost timeout counting and retry-deadline enforcement during its module split, so a timed-out task was not reported as retryable; the new regression failed before the fix. Ported timeout evidence counting, auto-resume with a rounded-up `min(timeout, 60s)` deadline, and interruptible Stop polling into the split worker scheduler. Added deterministic slow-task coverage to the existing Stop/resume test. Targeted workflow, Stop/resume, availability, read-only loading, messenger, Git, and source-layout verification passes: 24 passed, 1 ignored across 8 suites. All-target native-feature Clippy, formatting, diff checks, and 300-line layout pass. Full suite, package, Windows UI, and end-to-end acceptance remain open.

## [2026-10-20] verify | Batched exact-package UI audit — issue #9 / `7419557`

Inspected the exact macOS Rust package in one isolated app window and temporary HOME. Visual checks covered Workspace, Settings/MCP, profile registration, and Agent Roles. Functional checks created two saved workspaces and archived an inactive one; a Rust-only delayed mock coordinator then entered planning, exposed the stop-and-delete action, stopped, returned to Ready, and archived the active workspace into a fresh workspace. No provider CLI, network service, or task execution ran. Evidence: `docs/evidence/issue9-7419557-packaged-ui-audit.json`. This is component/interaction verification, not a full end-to-end workflow; packaged acceptance remains 2/25. Screenshot bytes were inspected live but not persisted as a public asset. The disposable package window is the only AgentConductor window left running.

## [2026-10-20] verify | Package current issue #9 head — `7419557`

The exact-source Rust macOS package validates with 214 dependencies, bundled SQLite, manifest checksum, packaged `--help`, archive SHA-256, and ZIP CRC. All four shared/worktree × Git/folder smoke cases passed with three parallel tasks/tests, unapproved-merge refusal, and approved-merge verification. Machine-readable package and smoke results are tracked under `docs/evidence/issue9-7419557-macos-*.json`. No GUI or provider CLI was opened; Windows and native UI acceptance remain unverified, and the count remains 2/25. Graphify refreshed the source graph without clustering to 7,399 nodes / 12,744 edges. The CocoIndex executable is not installed on this host; RTK and Graphify were used for structural discovery and source inspection.

## [2026-10-20] fix | Batch native UI regressions — issue #9 / `cb49fa1`

Four native UI findings were fixed together: return to Workspace from a running Agent Room without stopping the run, preserve the selected Review step, stop-and-delete a running plan only after its work and metadata settle, and fit the profile editor at 640px. The local all-target suite passes 759 tests / 14 ignored across 258 suites; focused tests and Clippy/layout/format checks pass. Commit `cb49fa1` and checklist commit `7419557` are pushed; packaged UI acceptance remains 2/25. AgentMemory records the handoff and Claude received the continuation state.

## [2026-10-20] docs | Refresh full-product completion audit — issue #9

Updated the stale 2026-10-10 audit checkpoint with current exact-source macOS package and all-target test evidence. R-12–R-17 and R-24 now distinguish native UI source coverage (25/25), packaged UI acceptance (2/25), and shipped Rust replacement (0/25), with Windows, GUI, accessibility/IME, resource, authenticated-provider, and final license gates still open. Graphify was refreshed without clustering to 7,400 nodes / 12,745 edges. CocoIndex was checked and is unavailable on this host.

## [2026-10-20] fix | Protect the active workspace from archive — issue #9 / UC-79

The exact packaged v0.1.31 UI exposed Archive for the active workspace; confirming it archived the current workspace and switched to a new default. Added an AccessKit regression that first failed on the active row. The Rust navigation view now exposes Archive only for inactive saved workspaces. The focused native workspace suite passes 7/7, targeted Clippy (`-D warnings`), rustfmt, and diff checks pass, and both changed Rust files are under 300 lines. Exact macOS source `4de454a3e605cc5f8f368eb264cec8fc601145e6` packaged and passed manifest validation, help, ZIP CRC, and four approval-gated shared/worktree × Git/folder smoke cases. In the package UI the active row has no Archive action while an inactive row retains it; the user's real workspace was observed without edits. The full all-target run was stopped after 250 tests across 80 suites, so it is not claimed as complete. Packaged native UI acceptance remains 2/25; Windows and exact-head CI remain open. Graphify traced the navigation view and workspace confirmation path. CocoIndex is unavailable on this host.

## [2026-10-20] verify | Packaged agent-profile management UI — issue #9 / workflow 3

Launched the exact packaged executable in one native macOS window with a disposable HOME/workspace. Through the packaged UI, registered a custom profile (`ui-smoke`, `/usr/bin/true`), edited its ID to `ui-smoke-edited`, opened the explicit removal confirmation, confirmed, and observed the empty-agent state. No agent task or provider command ran. The package executable SHA-256 was `04325483569c54cf2359c002e4066e7b6899280f45d38030bf4b66ceacc267bd`, matching the manifest for source `2fc7bc22264724219f36c0e9a1c3858ffdc38025`. This closes packaged UI workflow 3; with the previously accepted shared-folder execution journey, verified packaged native UI acceptance is now 2/25 (8%). It does not certify Windows behavior or the shipped Rust replacement.

## [2026-10-20] verify | Package the started-run navigation slice — issue #9

Built exact pushed source `2fc7bc22264724219f36c0e9a1c3858ffdc38025` with the Rust release packager. `validate-package`, packaged `--help`, ZIP integrity, and the four shared/worktree × Git/folder approval-gated smoke cases pass. The manifest executable SHA-256 is `04325483569c54cf2359c002e4066e7b6899280f45d38030bf4b66ceacc267bd`; archive SHA-256 is `1458db4d854b1aff7c05ffbc2d2b3ba09803d179aadc1b9f5e9dec4fc43fbb9e`. One native v0.1.31 window launched with the same executable hash. This verifies packaging and startup, not the packaged UI transition; it does not change packaged end-to-end acceptance from 1/25. No exact-head GitHub workflow run was returned and Windows packaged UI remains open.

## [2026-10-20] verify | Reopen a started run from Workspace — issue #9

After returning to Workspace, the plan-review notice explained that a run had already started but provided no way to reopen its Agent Room. Added a localized **Open Agent Room** action through the existing `open_run` request and enabled it only while the saved run is available. The same window and worker are reused. The native journey regression now completes mock tasks, approves and merges the candidate, returns to Workspace, and reopens the run. Focused journey, all-target Clippy (`-D warnings`), rustfmt, diff checks, and the 300-line layout test pass; the locked all-target suite also passed locally (775 passed, 14 ignored, 257 suites). This navigation subpath does not add another end-to-end acceptance; packaged workflow acceptance remains 1/25. Exact-head CI and Windows packaged UI remain pending.

## [2026-10-20] verify | Bind release packaging to its running executable — issue #9

The release command previously assembled `root/target/release/agentconductor` regardless
of which executable was running, so a custom Cargo target directory could package a stale
binary. Added an explicit-binary assembly path used by `package-release`, retaining the
existing assembler entry point for other callers. The regression distinguishes stale and
current binary bytes and checks the packaged executable and manifest SHA. Full locked
all-target tests pass (775 passed, 14 ignored, 257 suites); all-target Clippy with
warnings denied, rustfmt, diff checks, and the touched-file 300-line check pass. Exact
new-head macOS/Windows CI is still pending; native workflow acceptance remains 1/25.

## [2026-10-20] verify | Stabilize issue #9 start journey readiness

Platform Validation run `39385260992` passed Windows build, all-target tests, the 300-line layout
gate, opt-in native launch, rustfmt, and Clippy. macOS failed the start-journey test while its
synthetic click moved from Workspace settings to plan review. The readiness wait matched a static
navigation label before the asynchronously loaded workspace snapshot had rendered both saved
agents; that snapshot can populate/reflow the roster during the synthetic click. Tightened the
test precondition to wait for the planner and worker edit controls before clicking. No product code
or timeout changed. The focused journey passed five consecutive local runs; the complete locked
all-target suite, source-layout guard, all-target Clippy (`-D warnings`), rustfmt, and diff check
passed locally. The prior run's macOS failure still needs exact-new-head platform validation.

## [2026-10-20] verify | Exact v0.1.31 package runtime on issue #9

Built the clean issue #9 source commit `d884d7c769b51c8478fa3919777628b328621d4f` as an
optimized macOS package. Package assembly, manifest/checksum validation, packaged `--help`, ZIP
integrity, and four deterministic shared/worktree × Git/folder smoke cases pass. Each case runs
three parallel tasks and tests, rejects unapproved merge, and verifies approved merge. The
archive contains no project-owned `.js` or `.py` files; its single native executable is
17,909,232 bytes with 214 dependency notices. No GUI or external agent CLI/service was run, so
this adds no end-to-end UI workflow acceptance beyond 1/25. Full evidence:
[`issue9-d884d7c-macos-package.json`](../../docs/evidence/issue9-d884d7c-macos-package.json).
Graphify query of the release/package path traversed 276 graph nodes (7,522-node graph).
CocoIndex could not be run because no `cocoindex` command is installed in this environment.

## [2026-10-20] audit | Current native package and public screenshot provenance — issue #9

Read the current app without editing its fields: the existing README/manual workspace-light,
workspace-dark, agent-settings, and Agent Room image assets all visibly say v0.1.24. Reused the
existing single window by closing package `1ff6814` and launching exact package source `d884d7c`;
the accessibility tree confirms v0.1.31 and restores the saved `헬스체크 기능 개발` workspace.
No duplicate app window remains. CUA now provides an in-session screenshot; the exact v0.1.31
Workspace view was inspected at 1140×772 in both themes against the design guide. Selected tabs,
surface colors, input boundaries, Korean labels, and the three-agent rail were visible without
clipping in those views. The original light preference was restored. This does not certify the
other workflows, keyboard/IME, or accessibility. Shell `screencapture` still fails with
`could not create image from display`, and the in-session image includes a local path, goal, and
quota state, so no image was written to public docs. Capture an isolated/anonymized exact-package
view before replacing the stale v0.1.24 assets.

The exact-source opt-in native launch-path check also passed locally: 86 tests across 11 suites,
both native binaries pass `cargo check`, and the focused native-feature Clippy run has no warnings.
No desktop app process was started by this validation. This is source-level evidence only and does
not supersede the macOS full-suite UI failure recorded below.

The exact PR #22 validation run `39385260992` built successfully and passed the macOS 300-line
layout gate, then failed `native_ui_start_journey` while waiting for the plan-review UI; its final
AX labels still showed Workspace settings and an `Under review` phase without the expected review
controls. The current local full suite passed once (772 passed, 14 ignored across 257 suites),
then reproduced this failure on the next full-suite invocation. Four isolated journey runs passed.
This remains an unresolved suite-context interaction; no timeout-only fix is claimed. Windows
validation was still in progress at this checkpoint.

## [2026-10-20] verify | Correct native UI design-guide test command — issue #9

The design guide referenced the removed `room_task_rows` target, producing an immediate cargo
error. Updated it to the current native appearance, bilingual, room-layout, and task-selection
targets. The replacement command passed 16 tests across seven suites. Exact packaged UI review
showed the Workspace layout in both themes and restored the user's light preference; it did not
compare to a JavaScript baseline or produce a screenshot. No UI acceptance count change. Current
PR run `39366430380` passed macOS and was still building on Windows.

## [2026-10-20] verify | Restore desktop role selector width — issue #9

The role-table selectors were narrower than the legacy UI at standard desktop width. A new
AccessKit regression failed at 108px; the native selectors now use 130px at content widths of
900px and above, while keeping the compact width for narrower layouts. Five focused native UI
suites passed (28 tests), as did `source_layout`, rustfmt, and diff checks. Both changed Rust files
remain below the 300-line limit. Commit `1757e32` is pushed. Workflow run `39367609082` for old
head `7f86cce` was cancelled after newer commits arrived; it did not validate this change. This
component-level check does not change the 1/25 packaged acceptance count.

The full offline native-feature all-target suite then passed on code HEAD `311533f`: 745 passed,
14 ignored, 0 failed across 252 suites. Native-feature all-target Clippy with `-D warnings`,
rustfmt, `source_layout`, and `git diff --check` passed. Platform Validation run `39368429666` for
code head `311533f` was cancelled after the documentation-only update. Run `39369276017` then
started on PR head `2ba31c4`; the next documentation-only commit supersedes it without changing
Rust code. Check PR #22 for the current Windows/macOS run before recording cross-platform success.

# [2026-10-20] verify | Exact-head macOS package smoke — issue #9

Built the optimized macOS package from exact clean issue #9 HEAD `1ff6814` with the Rust
`package-release` command. Package validation and packaged `--help` passed. The package's
deterministic smoke passed four shared/worktree × Git/folder cases with three parallel tasks and
tests per case, unapproved merge refusal, and approved merge verification. Evidence is in
`docs/evidence/issue9-1ff6814-macos-package.json`. This does not count as native UI acceptance;
the UI workflow checkpoint remains 1/25. Exact-head macOS/Windows run `39365586146` was still
pending when recorded.

## [2026-10-20] verify | First packaged native workflow acceptance — issue #9

Executed the exact v0.1.30 Rust package from `35a5c09` against a disposable shared-folder Git repository with a Rust mock coordinator and three concurrent stdin/JSONL mock worker sessions. Planning produced disjoint paths, all three `git diff --check` tests passed, the operator inspected the task results and diff and approved review and merge, the UI reported Completed, and the repository ended clean. This is the first accepted end-to-end operator workflow (1/25); the evidence records that no provider CLI or external service was called and that no screenshot asset was saved. See [native package workflow evidence](../../docs/evidence/issue9-35a5c09-native-workflow-01.json) and [packaged workflow acceptance](native-release-packaging.md). The app was closed and disposable package-test files were removed after evidence capture.

## [2026-10-20] complete | Reconcile the packaged workflow Wiki with current evidence

Updated `knowledge/wiki/packaged-workflow.md` to put the accepted 1/25 packaged UI journey at the top and link its committed evidence. The 2026-10-19 consent-only setup is labeled historical and superseded, removing its stale “waiting” status from current context. `docs/07-implementation-plan.md` records the Wiki sync. The source coverage (25/25), packaged workflow acceptance (1/25), and shipped replacement (0/25) remain separate measures. No screenshot export was created; no workflow beyond the accepted first journey is claimed.

## [2026-10-20] verify | Stabilize the native start-journey test — issue #9

Platform Validation run `39363867674` and the local full suite both timed out after 15 seconds while waiting for the approved run to open Agent Room. The focused test passed repeatedly, and its final test-only wait was raised to 30 seconds; application code and behavior are unchanged. The full serialized native-feature suite then passed 744 tests with 14 ignored across 252 suites. `source_layout`, rustfmt, diff checks, and all-target native-feature Clippy with warnings denied pass. Exact-head macOS/Windows CI for this test adjustment is still required before counting it as complete.

## [2026-10-20] verify | Stabilize native start-journey wait — issue #9

Platform Validation run `39345540466` exposed a macOS timing failure in `native_ui_start_journey`: the execution-review UI state was not observed within the test helper's five-second deadline. Raised the test-only wait bound to 15 seconds. The exact CI-style local all-target suite passed 749 tests (14 ignored), and the affected journey passed three consecutive focused runs. Rust application code is unchanged; cross-platform workflow acceptance remains 0/25 pending the platform rerun and exact UI review.

## [2026-10-20] verify | Make native role-table headers visible — issue #9

The exact light-theme app screenshot showed role-table headings missing although they remained in the accessibility tree. A TDD regression failed for all four header labels in both theme palettes because bold `RichText` kept placeholder section colors. Setting the current `ui.visuals().text_color()` made the test pass. `native_ui_roles` (7/7), `native_ui_workspace_style` (4/4), `source_layout` (1/1), all-target native-feature Clippy with warnings denied, rustfmt, and diff checks passed. Release binary SHA-256 `159c1ff98ec489083e5deadeb1bfe905ce5d1af8def62e2831a21fafe0ede81f` showed all five Korean headers in the isolated macOS app. Commit `a41f16e` was pushed to the issue-linked branch. Graphify structurally refreshed to 7,109 nodes / 12,318 edges. CocoIndex was checked but cannot start because `mcp.server.fastmcp` is missing from its installed environment. Packaged Windows visual acceptance and full issue #9 workflow acceptance remain open (0/25). AgentMemory: `mem_mvfme8hn_dd113ffc4c4b`.

## [2026-10-20] verify | Full serial native Rust suite — issue #9

Full `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` passes: 719 passed, 14 ignored, 0 failed across 247 targets. The first full runs exposed test-harness texture deltas left unconsumed, an outdated expected Korean empty-catalog label, and missing feature gates on native UI integration tests. Fixed those test-only issues; focused profile UI targets (5/5) and the feature-gate audit (2/2) pass. Formatting and diff checks pass. A task-local rustc/Z3 wrapper was needed for the host's Homebrew LLVM/Z3 linkage. Graphify: 7,099 nodes / 11,337 edges / 553 communities. #9 remains 0/25 end-to-end workflow acceptance. AgentMemory: `mem_mvfjlfao_8b19f0823daa`.

## [2026-10-20] verify | Exact-source macOS package on issue #9

Built and validated commit `fd177ea145326407d485d7f18ccd1d916f1fc00e`. The optimized package has 214 dependency license records, binary SHA-256 `6bd00e6f520e0172a0b1f912c15fe558695d4a3a6c2d80477395b9da7cae6c88`, and ZIP SHA-256 `b4d17e6b8ecb152e4056aa2e39d1cabe4e9f7880d8398435c3bb2d132578e6e5`. Its local `package-release` workflow completed four isolated Git/folder × shared/worktree mock cases with three parallel tasks/tests and approval-gated merge. One exact package app window showed the empty Workspace and was closed. Goal-list 3/3, phase-progress 9/9, native start journey 1/1, source-layout 1/1, briefing 6/6 serial, native-feature Clippy, formatting and diff checks pass. A full all-target run is not green: the sandbox first blocked loopback tests, then an escalated run hit one intermittent briefing revision mismatch; isolated relevant suites pass. AgentMemory: `mem_mvfius14_41fb107ca8d4`. #9 remains 0/25 pending complete cross-platform UI acceptance.

## [2026-10-20] verify | Number the native phase action

Matched the released Workspace's `Plan phase n` label in the Rust UI using the existing localized message. An AccessKit test failed first with the generic label, then passed after the change. Goal-list (3), phase-progress (9), native start journey (1), source-layout (1), formatting, diff check, and all-target native-feature Clippy pass. The label helpers live in a small submodule so the goal view remains within 300 lines. Graphify: 7,095 nodes / 11,333 edges / 539 communities. AgentMemory: `mem_mvfik3wp_637a29f437ae`. Issue #9 remains at 0/25 end-to-end acceptance. See [native goal phase labels](native-goal-phase-labels.md).

## [2026-10-20] verify | Label proposed phases as under review

Compared native Goals & Progress with the v0.1.30 phase panel. A non-completed proposal should label its phase `Reviewing phase n/N`; Rust still displayed the same phase as `Next`. A rendered regression failed first, then passed after Rust used the localized reviewing heading and marked a proposed phase under review regardless of the current-phase field. Two goal-list tests, nine phase-progress tests, the native start journey, source-layout, formatting, and warning-denied native-feature Clippy pass. Graphify refreshed to 7,083 nodes / 11,320 edges / 539 communities. Exact packaged UI/accessibility and overall 0/25 acceptance remain open.

## [2026-10-20] verify | Show goal identity and stage in the native goal list

Compared v0.1.30 `renderGoals()` with native Workspace. Legacy saved-goal rows display slug, localized stage, goal description, and a distinct selection action. Rust only displayed the active goal heading and rendered other descriptions as direct buttons. A rendered regression failed because slug/stage were absent; Rust now displays them for active and inactive goals while preserving the revision-bound select event. Goal-list and nine phase-progress tests, native start journey, review copy, source-layout, formatting, and warning-denied native-feature Clippy pass. Graphify refreshed to 7,079 nodes / 11,314 edges / 555 communities. Exact packaged layout/accessibility and full end-to-end acceptance remain open (0/25).

## [2026-10-20] verify | Preserve mismatches with empty phase lists

`slug_progress::progress()` reports unreadable `plan/plan.json` as a mismatch with an empty `phases` array. The native Goals & Progress early return suppressed that warning. A rendered test failed on this case; Rust now keeps the card visible when mismatches exist and shows a planned state instead of claiming all zero phases are complete. Nine phase-progress tests, source-layout, formatting, and warning-denied all-target native-feature Clippy pass. Packaged UI/accessibility and 0/25 end-to-end acceptance remain open.

## [2026-10-20] verify | Show goal-progress mismatch warnings

Compared the v0.1.30 `renderGoals()` goal card with native Rust. The legacy UI showed every mismatch in phase/task state, while Rust silently omitted the warning list. A rendered regression failed first; Rust now shows every mismatch with the existing warning foreground color and a warning marker. Eight phase-progress tests, review-copy/source-layout checks, formatting, and all-target native-feature Clippy with warnings denied pass. Exact packaged UI and accessibility checks remain open; workflow acceptance remains 0/25.

## [2026-10-20] verify | Restore coordinator analysis disclosure

The v0.1.30 `renderPlanPhases()` shows saved whole-goal coordinator analysis in a collapsed disclosure before phase rows. The native Goals & Progress card omitted it. A rendered test failed on the missing localized disclosure; Rust now shows the saved non-empty analysis through the existing localized collapsible widget. All seven phase-progress tests, `native_ui_review`, `native_ui_start_journey`, `source_layout`, and the three-parallel-task shared/worktree approval-and-merge workflow pass. Formatting and all-target native-feature Clippy with warnings denied pass. Graphify refreshed to 7,059 nodes / 11,281 edges / 539 communities. CocoIndex remains unavailable because its installed environment cannot import `mcp.server.fastmcp`. Exact packaged UI and keyboard/accessibility checks remain open; native workflow acceptance stays 0/25.

## [2026-10-20] verify | Restore Workspace phase briefs and planned task details

Compared the v0.1.30 `renderPlanPhases()` behavior with the Rust Goals & Progress card. Rust showed phase names and aggregate counts but omitted each phase's brief and planned task ID, first-line instruction summary, agent, and state. A rendered test failed first; expandable phase rows now show those details and open the current/reviewed phase by default. A reviewed phase shows the existing edit-below notice without duplicating its tasks. The native start-journey test also exposed stale expectations for merged task labels and a test click that did not scroll; it now follows separate accessible task heading/summary labels and scrolls to controls. All six phase-progress tests, review copy, the UI journey, the three-task shared/worktree approval-and-merge test, source layout, formatting, warning-denied native-feature Clippy, and diff checks pass. Graphify refreshed to 7,055 nodes / 11,275 edges / 532 communities. CocoIndex remains unavailable because its installed environment cannot import `mcp.server.fastmcp`. Packaged macOS/Windows UI and accessibility checks remain open; native workflow acceptance stays 0/25.

## [2026-10-20] verify | Restore Review Plan snapshot-base cue

Compared actual v0.1.30 `startup.js::tasks()` with Rust. The shipped review screen shows the captured repository-base prefix and reminds the operator to inspect executable paths, permission modes, scope, and tests before the start-consent dialog. The Rust view omitted that cue. A rendered AccessKit test first failed; the native view now uses the existing localized message and the proposal's captured base. Full repository-base evidence remains in the existing consent dialog. Five Review Plan unit tests and ten focused task-card/edit/review-copy/source-layout tests pass; Clippy with warnings denied, formatting, diff checks, and source-layout guard pass. Exact packaged macOS/Windows review and end-to-end parity remain open at 0/25. CocoIndex Code cannot start here because `mcp.server.fastmcp` is missing.

## [2026-10-20] verify | Restore task role labels in native Review Plan

The v0.1.24 shipped task summary puts a two-digit ID and optional role badge on the first line, with the instruction summary below, and labels its removal action `Remove task`. Native Rust omitted the role, combined both lines, silently clipped long text, and shortened the action label. Rendered regressions failed on each mismatch. Rust now restores the two-line hierarchy and localized role/remove labels for English and Korean, keeps unknown roles visible, and appends an ellipsis after the 160-character summary limit. AccessKit confirms the role badge stays within 480px. Focused task-card, plan-edit, review-copy and source-layout suites pass (10 tests); Rust formatting, warning-denied Clippy, and diff checks pass. This covers visible R-47/UC-39/UC-106 details only; packaged parity and 0/25 workflow acceptance remain unchanged. CocoIndex is unavailable in the current environment, and exact macOS/Windows package comparison was not run for this slice.

## [2026-10-17] verify | Native screenshot provenance correction and Workspace header

The public-doc regression first failed because README/manual captions called existing images
“Native Rust” based on filenames; those names did not prove the image source. Captions are now
neutral, and the test explicitly rejects unsupported Rust-provenance claims. Manual-render
checks still prove local images and alt text, not source provenance. A second TDD regression
found two rendered Workspace stage eyebrows; removing the card-level duplicate leaves one
header label. The exact Rust preview was inspected with CUA. Saving its window image through
macOS `screencapture` failed, so no replacement screenshot is claimed.

Focused tests passed: 22 across native UI probe, public docs, native manual, and source-layout
targets. The exact rebuilt Rust release binary was visually inspected in English/Korean and
light/dark modes; placeholders and the empty-agent state now show. Release build, all-target
Clippy, rustfmt, and diff checks pass. Current source has no tracked JavaScript/TypeScript/Python
application files and no Rust source/test file above 300 lines. The CUA capture is visual evidence
only; no image was saved. Issue #9 and PR #18 remain open/draft; screenshot and platform acceptance
are incomplete.
Graphify structural refresh completed without LLM: 6,105 nodes / 10,582 edges. The installed
`cocoindex-code` entry point cannot start because `mcp.server.fastmcp` is missing; no CocoIndex
index is claimed. AgentMemory screenshot-provenance memory: `mem_mvcfugk4_10e89bf62871`;
Claude handoff: `sig_mvcfy8tw_53ef98b68ce4`.

## [2026-10-10] verify | Native Codex subscription/session contracts

Installed Codex 0.162.0 with existing ChatGPT login passed actual stdin/JSONL generation and exact-ID context resume (7.74 seconds); two native sessions ran concurrently, retained distinct IDs and each recalled its own marker on concurrent resume (7.13 seconds). API key environment variables were removed; no model tool calls appeared. Nonterminal host role-definition error items are now visible warning events; terminal failures remain separate.

Regression: 135 passed, four ignored, 37 suites in 52.91 seconds; both new ignored generation tests ran explicitly above. Clippy/format/whitespace pass. Graphify 377 nodes/1,199 edges; CocoIndex located item normalization. Raw evidence remains local; hashes are in [native Codex evidence](native-codex-sessions.md). Agentmemory: mem_mv1ldpjn_b96c9112eb44. Native containment, scoped implementation, all-provider proof, functional UI and final packaging remain open.

## [2026-10-10] verify | Reviewed runtime lesson assets

Red → Green covers exact typed promotion, source-hash revalidation, offline next-attempt task-scoped handoff, changed-source exclusion, sensitive/alias refusal and wiki alias failure. SQLite remains authoritative when replica materialization fails; retry repairs mirrors without duplicating acceptance log entries. Compiled digest/accept CLI controls pass.

Regression: 134 passed, two ignored, 36 suites in 51.90 seconds. Clippy/format/whitespace pass. Graphify 377 nodes/1,199 edges, no LLM; CocoIndex located worker lesson injection. No runtime dependency added. Agentmemory decision: mem_mv1l3l0i_ea3708c1e9b9. Native lesson integration, confidence-aware/cross-run context, functional UI and native containment remain open. See [runtime lessons](runtime-lessons.md).

## [2026-10-10] verify | Live normalized intermediate reports

Red → Green established pre-completion SQLite queries, malformed-stream process cancellation, final unterminated record draining, immutable replay, separate attempts, compact memory summaries and bounded recent-event queries. Both worker transports are connected. Parallel native-worker rework exposed a deferred SQLite lock upgrade; immediate writer transactions fixed it.

Regression: 131 passed, two ignored, 35 suites in 50.14 seconds. Clippy/format/whitespace pass. Graphify 352 nodes/1,110 edges; CocoIndex located both observer calls. No runtime dependency added. Agentmemory bug/decision: mem_mv1kvcr1_7c00066988b4. Native authenticated execution, OS containment, functional UI, runtime lessons and knowledge revalidation remain open. See [live progress](live-progress.md).

## [2026-10-10] verify | Worker-bound human decisions

Opt-in Claude worker binding and CLI list/approve/reject/refresh controls now have
compiled mock-protocol evidence. Persisted session UUIDs, approval epochs, paused
workspace revalidation, single consumption and separate merge approval are verified.
Red → Green covered exact-ID lookup beyond 1,000 history records, payload tampering,
foreign/replayed callbacks, expiry/refresh, rejection/revision and stopped explicit
resume. Shared and optional worktree modes pass.

Regression: 127 passed, two ignored, 34 suites in 49.61 seconds; Clippy, formatting
and whitespace checks pass. Actual Claude metadata-only initialization: 0.90 seconds,
no generation prompt. Graphify: 341 nodes/1,083 edges, no LLM. CocoIndex found the
operator/scheduler receipt reads. Agentmemory decision: mem_mv1klk1z_adfd73a70fd3.
No runtime dependency added. Native enforcement, OS containment, functional UI and
runtime knowledge remain open. See [worker decisions](native-worker-decisions.md).

## [2026-10-10] verify | Asynchronous context verification

Red → Green established asynchronous callback cancellation and normal final-frame
draining. Owned verifier descendants terminate even when they do not observe the
stop marker. The compiled internal command derives binding from current stored
plan/session and Git state; both workspace modes pass. Ignored sensitive targets
use metadata rather than content reads. Verification never grants permission.

Regression: 121 passed, two ignored, 34 suites in 43.06 seconds; Clippy, formatting
and whitespace checks passed. Actual Claude metadata-only initialization passed
separately in 0.90 seconds without a generation prompt. Graphify: 321 nodes/987 edges;
CocoIndex located the verifier. No new runtime dependencies or API keys.
agentmemory records the design reason and cancellation/ordinary-exit distinction.
See [verifier evidence](tool-context-verification.md). Native worker/operator
integration, runtime knowledge, actual native enforcement and functional UI remain open.

## [2026-10-10] verify | Bounded worker policy components

Red → Green exposed approval consumption before duplicate tool identity validation
and evaluation after input closure. The corrected controller rejects both first.
An explicit bounded file classifier protects owned edits, metadata/settings, sensitive
paths and filesystem aliases; a replaced workspace directory also invalidates it.
The classifier bridge keeps commands/searches behind the human evaluator and refuses
unsupported inputs before evaluation. Mock control integration confirms retained raw
logs, terminal/quiescent ownership, no pending side effect and one-shot consumption.

Regression: 116 passed, two ignored, 32 suites in 38.76 seconds; Clippy, formatting
and whitespace checks passed. Graphify: 308 nodes/941 edges; CocoIndex located the
classifier and evaluator bridge. agentmemory recorded replay/lifecycle and policy
decisions with reasons. No new runtime dependency, API key or LLM indexing call.
Ponytail guidance remains applied; no callable tool/CLI is available locally.
See [policy evidence](tool-approval-policy.md) for source hashes and limitations.
Production worker/context/operator integration, live native proof, runtime knowledge
and functional UI remain open; the full goal is active.

## [2026-10-10] verify | Pre-execution control components

Installed Claude 2.1.286 predates the documented fail-blocking command-hook option;
ordinary hook failure is not enforcement proof. Bounded bidirectional subprocess
control retains stdin until explicit idle, spools raw output before decisions, and
kills/reaps the owned group on malformed output or callback errors. Typed approvals
start pending, bind exact context/input, expire within 900 seconds, and have atomic
single consumption with audit/outbox updates. Mock subprocess integration proves no
command side effect before approval and no replay after its first use. A Red native
controller test exposed loss of pending permission decisions after a result while work
was active; preserving them repairs the late callback without reevaluating approval.

Regression: 107 tests pass, two opt-in/helper tests ignored, 30 suites in 37.64 seconds; Clippy/formatting clean. Native metadata-only SDK initialization
also passes with Claude 2.1.286, registered PreToolUse callback IDs, zero user frames and
terminal/quiescent ownership. Raw stdout SHA-256:
`935b6d736d8562ab968ac4a6fa440f34f46f0a00f3d3753e09e488a84857f5f1`.
The current Claude CLI reports no login. Native generation/control tests require the
existing subscription login; no API-key fallback is introduced. Production worker
permission integration, all-provider enforcement, runtime knowledge and functional UI
remain open. Graphify: 284 nodes/869 edges without LLM; CocoIndex verified consumption.

## [2026-10-10] verify | Structured coordinator review

A registered CLI coordinator receives bounded diff/test/log excerpts with exact source
hashes. Typed final documents reject invented task/evidence IDs and generated command
fields. Saved proposals bind to current task state and evidence; exact-hash application
makes no additional model call. Acceptance never creates merge approval. Stop preserves
raw review logs, interrupted ownership remains recoverable, and unexpected edits revoke
the candidate without rebaselining. TDD also exposed stopped reviews consuming a round;
they now refuse launch before changing state. Regression: 96 tests, one intentionally
ignored subprocess helper, 27 suites in 37.42 seconds; Clippy and formatting clean.

Graphify: 234 nodes/737 edges, no LLM. CocoIndex structural grep verified CLI/workflow
saved-review integration. No runtime dependency added. Native provider contracts and
pre-execution approval enforcement remain unverified; full-product acceptance is open.

## [2026-10-10] verify | Review feedback and dependency refresh

The worktree rework test first failed because dependent retries retained old predecessor
contents. Versioned dependency context and preserved-checkout refresh now propagate a
reviewed upstream revision through a three-task chain in both workspace modes. Scoped
partial files and old native session evidence remain available; refreshed worktrees use
fresh sessions. Patch conflicts leave destination bytes/index unchanged and block launch.
Compiled `revise` control, failure repair, transitive invalidation, preserved unaffected
tasks and fresh candidate approval pass. Regression: 82 tests; Clippy clean.

## [2026-10-10] verify | Durable ownership and checkpoint boundaries

TDD exposed orphaned child processes after coordinator SIGKILL, a claim created too
early during dirty-repository validation, and mock session IDs reused across independent
workspaces. Kernel leases now keep stable inodes; durable repository/process claims
require explicit recovery when ownership is unknown. Native session leases use the
provider/account/session identity, and paused working trees and indexes are checked
before resuming. All 78 tests pass (one subprocess helper is intentionally ignored in
normal discovery and invoked by its parent test); Clippy reports no warnings.

Graphify extracted 206 nodes/612 edges without an LLM. CocoIndex structural grep
confirmed SessionLease declaration and workflow integration. No new runtime dependency
or model API key was introduced. Full-product acceptance remains open in
[the audit](../../docs/09-completion-audit.md); this is an incremental verification checkpoint.

## [2026-10-09] plan | Requirements and OSS research

Investigated CLI subprocesses, existing authentication, worktrees, human approval.
Added Rust/TDD, agentmemory, LLM-wiki, RTK/Ponytail/Graphify/CocoIndex constraints.
Recorded OSS comparison and headless/resume contracts in [protocols](cli-protocols.md).

## [2026-10-09] task | Design documents

Linked requirements → BG → SF → UC → QAS → DD → TDD implementation plan.
This was a documentation task, not the start of product execution implementation.
Initial architecture: Rust/SQLite/CLI-pager; optional memory/indexing sidecars.

## [2026-10-09] verify | Licenses and contracts

Excluded ao code reuse for unconfirmed license. Distinguished agentmemory Apache-2.0
from iii Apache-2.0 AND Elastic-2.0. Recorded conduct text output and Vibe Kanban
app-server/serve differences from requested exec/run. At this stage, product execution,
TDD tests, performance and transitive distribution checks had not been performed.

## [2026-10-09] complete | Research and design

Verified 15 Markdown files, zero broken local links, seven stage documents.
Research/design complete; T-01–08 implementation not complete. Saved findings and
outstanding verification to shared agentmemory.

## [2026-10-09] complete | T-01 Profile and plan foundation

Red: five failed, three passed. Green: eight passed. cargo test and clippy passed.
Commit 6391f7d; design commit 33a3edf. Created private neodamul/AgentConductor repository.

## [2026-10-09] verify | T-02 State foundation

Red: three failed, one passed. Green: four passed. Transitions and audit events share
one transaction. Verified resume transitions, duplicate events, database reopen.
Single-executable requirement: bundled SQLite, separate optional sidecars.
Commit 37e7860 pushed; total 12 tests passed. Full recovery scope remains pending.

## [2026-10-09] plan | Documentation and interface preview

User requested README UI, English documents/Korean communication, and exclusion of
internal decisions/development plans from README. Scope: translate all Markdown, retain
source references/IDs and evidence, add a labeled mockup. No implemented UI is claimed.

## [2026-10-09] task | English migration and public README

Translated documentation. Replaced README with product-facing content. Added original
SVG and a PNG preview. An intermediate README referenced an image before its file existed;
corrected the broken reference. Git history preserves pre-migration log/source wording.

## [2026-10-09] verify | Preview and document checks

Rendered the SVG locally and inspected the PNG. Initial Quick Look aspect handling cropped
the image; rendering on a square canvas and cropping blank padding preserved the full design.
Verify local links, English-only text, XML structure, image dimensions, and Git diff before commit.

## [2026-10-09] complete | Documentation and preview

Verified 15 English Markdown files, zero broken local links, valid SVG XML, 1280×880 PNG,
and clean Git diff whitespace. Visually inspected all preview panels. Saved requirements and
progress to shared agentmemory mem_mv0xsakb_73793144f244. This completes the documentation
change, not the UI or orchestration implementation.

## [2026-10-09] plan | Optional worktrees and human controls

User changed worktree separation to an option and requested a less web-like interface with
visible intervention controls. Shared is the default; worktree is explicit. Shared-mode
policy requires disjoint scopes, a clean baseline, no worker Git mutations, quiescent tests,
and aggregate review with unverified per-worker attribution.

## [2026-10-09] task | Workspace plan schema and console concept

Added workspace_mode (shared/worktree), backward-compatible shared default, and unknown-mode
rejection. Redesigned the preview as a terminal console with task selection, goal/feedback
entry, pause/resume/revise, workspace selection, and inspect/approve/reject controls.

## [2026-10-09] verify | Workspace schema

Red: two new tests failed while eight existing tests passed. Green: 14 tests passed overall;
clippy passed. Actual workspace execution/scheduling and UI controls remain unimplemented.

## [2026-10-09] complete | Workspace option and revised preview

Plan schema, English documentation, and revised console illustration verified. All local
links resolve; SVG parses; PNG is 1280×940 and all interaction controls are visible.
Shared/worktree execution and interactive controls still require later implementation.

## [2026-10-09] plan | Additional interface concepts

User requested more design mockups and approved the addition of intervention controls.
Produce three visually distinct layouts retaining targeted feedback and human approval.

## [2026-10-09] task | Three interface mockups

Created Agent room (conversation/inspector), Dispatch desk (stage/assignment), and Review
cockpit (diff/evidence). Added original SVGs, PNG previews, gallery, and README comparison.
All artifacts explicitly label illustrative data and unimplemented controls.

## [2026-10-09] verify | Visual review

Rendered each SVG and inspected all three PNGs for clipping, overlaps, readable labels,
and visible intervention controls. Validate XML, dimensions, English text, and local links.
No runtime code changed; visual/document checks apply instead of new unit tests.

## [2026-10-09] complete | Interface gallery

Three 1280×860 PNGs and corresponding valid SVGs verified. English-document and local-link
checks passed. Gallery and README updated. These mockups do not complete runtime UI work.

## [2026-10-09] complete | Push cadence preference

User clarified that pushing every time is unnecessary. Keep small commits; batch pushes
at meaningful checkpoints or for remote review. Updated the implementation workflow.

## [2026-10-09] complete | Preferred interface palette

User preferred the second mockup's colors. Applied the Dispatch desk light/sage/forest
palette to Agent room, Review cockpit, and console overview. Preserved feedback, pause,
resume, revision, and approval controls. Inspected rendered previews and corrected pale
action/status text for contrast. Saved preference to agentmemory; commits stay local
until a meaningful push checkpoint.

## [2026-10-09] plan | T-03 Process supervisor

Implement direct argv execution, inherited auth environment/cwd, stdin EOF, separate disk
spooled stdout/stderr, exit capture, and bounded POSIX process-group termination. No retries
or real paid-model calls. Explicit user stop and parser normalization remain later tasks.

## [2026-10-09] task | T-03 Foundation

Red: process-contract test target failed because supervisor API did not exist. Green: six
process cases pass, including multi-megabyte stdin/stderr, nonzero exit, spawn failure,
timeout of descendants, unread stdin, and ignored interrupt/termination escalation.

## [2026-10-09] verify | T-03 Foundation

20 tests passed; clippy passed. CocoIndex structural grep inspected ProcessGroup; Graphify
code-only extraction found 41 nodes/72 edges across four source files. Derived graph moved
to temporary storage. Inventory captured 33 host-target dependency license declarations;
full source-notice and multi-platform release checks remain pending.

## [2026-10-09] complete | T-03 Foundation

Supervisor foundation complete on Unix. Windows containment and explicit user-stop API
remain unimplemented; no full MVP completion is claimed.

## [2026-10-09] complete | Initial single-binary CLI

Red: all three CLI tests failed with an empty entry point. Green: help, plan validation,
and invalid-input handling pass. Full suite: 23 tests passed after fixing parallel test
temporary-directory collisions with an atomic sequence. Release build and help smoke pass.
otool shows only host system libraries (libiconv/libSystem), no dynamic SQLite dependency.
The CLI currently validates plans; it does not yet run the orchestration workflow.

## [2026-10-09] plan | T-04 JSONL normalization foundation

Normalize stored Claude/Codex/OpenCode/custom events without launching providers. Preserve
unknown payloads, session IDs, sequence and byte offsets; bound line size and reject malformed
JSON/UTF-8. OpenCode step completion must not imply whole-task completion.

## [2026-10-09] task | Parser and normalize command

Red: missing protocol module; normalize CLI test failed before command implementation.
Green: six synthetic parser tests and four CLI tests pass. Preserve raw provider payloads
and expose normalized JSONL through the single native executable.

## [2026-10-09] verify | Normalization

30 tests passed overall; clippy passed. Fixtures are synthetic contracts, not captured or
version-pinned live CLI evidence. Live provider smoke and resume capability checks remain
pending. No API keys or paid-model calls were used.

## [2026-10-09] complete | T-04 Foundation checkpoint

Added five adapter tests for literal file argv, native initial/resume flags, exact session
selection, invalid-session rejection, and custom-profile registration requirement. Full suite:
35 tests passed; clippy and release build passed. Release normalize smoke preserved session
and terminal result. Current CLI commands are validate and normalize only. Scheduling,
workspaces, approval/merge execution, recovery, and live-provider checks remain unfinished.

## [2026-10-09] complete | Progress checklist

User requested stage-by-stage progress in the plan. Added evidence-based checklists for
T-01–08 with plan/task/verify/complete and explicit distinction between foundations and
full completion. Current Git tests pass; scheduler/approval E2E is still in the red stage.

## 2026-10-10 — Availability/session extension

- Added R-18–20 traceability from goals to design decisions and T-09.
- TDD verified explicit reset parsing, manual scheduling across restart, role transfer, model approval invalidation, concurrency limits, and bounded retries.
- Native Claude/Codex metadata discovery succeeded with inherited authentication; OpenCode returned an empty configured catalog.
- Graphify extracted 173 nodes/482 edges; recovery links to drive, permission validation, and exclusive locking. CocoIndex structural grep verified all session-edit entry points use the run lock.
- No new framework, API key, indexing daemon, or provider generation request was introduced. Ponytail's small-scope verification discipline continues to guide implementation.

- Release binary E2E: three mock agents in optional worktrees, 1.148 seconds, all integration tests pass, unapproved merge denied, exact approved SHA merged. Artifact is 3,170,736 bytes with startup/help RSS 2,195,456 bytes. Live CLI workloads are not included in that memory figure.
- Packaged 42 pinned Rust dependency license texts, SQLite disclaimer, SBOM, checksum and manifest. Broader product backlog remains unchecked in the implementation plan.

- 2026-10-10: Embedded Agent Room checkpoint with 141 passing tests, four ignored; real browser fixture feedback/reassignment/model/resume/diff/test/approval/merge. Native window acceptance and full product remain open.

- 2026-10-10: Native WKWebView review and Cmd-W close verified; duplicate-window lease added after user cleanup correction. Quota UI fixture outcome remains unresolved and retained.

- 2026-10-10: Assetized DD-23 direct native Room transport and source/artifact fingerprints. Historical HTTP checkpoint retained as superseded evidence. Actual native feedback/resume/review/approval/merge verified; both owned desktop windows exited 0. Availability fixture failure remains open.

- 2026-10-10: DD-24 process outcome TDD and 33 targeted regressions passed. Replaced ps-based absence checks with kernel probes after sandbox denial. Historical fixture failure did not reproduce in a new run; retained old evidence.

- 2026-10-10: DD-24 full regression passed 144 tests with zero failures and four explicit ignores (70.88 seconds); Clippy/rustfmt clean. No runtime dependency added.

## [2026-10-10] verify | Native startup workspace

See [source-hashed native startup evidence](native-startup-workspace.md). Full regression: 148 passed, four ignored, zero failed. Real native fixture completed setup, planning, edits, consent, execution, review, approval and merge without a UI web server. Two owned windows exited zero sequentially; no overlapping verification windows were retained. Authenticated all-provider acceptance and dialog accessibility remain open.

## [2026-10-10] ingest | Claude CLI and native app acceptance

Source: [raw observations](../raw/2026-10-10-claude-native-acceptance.md) (sha256 `bbef81bd95104db3b467aa8673844edaedc0c936fea4539ba2a53d45ca1b568c`). Live Claude session/resume/concurrency tests and a three-worker guarded workflow passed against a subscribed CLI. Findings fed R-30–R-32, DD-32–DD-34 and the guard missing-file deny fix.

## [2026-10-10] verify | Live tool decisions (DD-32)

See [live tool decisions](live-tool-decisions.md). Commit 6fc77b6. Full regression 203 passed, 0 failed, 8 ignored.

## [2026-10-10] verify | Isolated planning snapshot (DD-33)

See [planning snapshot](planning-snapshot.md). Commit cbba143. Hostile planner left the target untouched; full regression 203 passed.

## [2026-10-10] verify | Korean input (DD-34)

See [Korean input](korean-input.md). Commit 75b121d. Native IME acceptance with the system input method remains open.

## [2026-10-10] update | Method documents

Added functional/non-functional analysis (`docs/04b`), design drivers with architecture view (`docs/05b`) and the engineering document index with traceability matrix (`docs/README.md`).

## [2026-10-10] verify | Workspace back navigation (DD-39)

Operator report ingested as R-33. See [workspace navigation](workspace-navigation.md). Native discard acceptance and full regression (208 passed) recorded. Windows WebView2 shell (DD-38) is pending native CI evidence.

## [2026-10-10] reconcile | Codex–Claude continuity

Reconciled 19 committed changes through 2211ba3 and preserved the older Codex working changes in a named stash. Windows CI 38037224504 passed compilation, containment and UI smoke but failed gate-test directory cleanup with an open SQLite store. Explicit close added; focused macOS gate regression passed 4/0/0. Native Windows correction verification remains pending. See [development continuity](development-continuity.md); agentmemory mem_mv25cgtz_2a8a07bee3cc.

## [2026-10-10] verify | DD-41 source layout

Layout Red on eight oversized files and three inline tests; split Green, maximum 485 lines. Full macOS regression 213/0/9 across 57 suites before canonical checkout migration; focused layout and format passed in Development checkout. Windows verification remains pending. See [source layout](source-layout.md); agentmemory mem_mv25wmpa_f63ae0e2a551.

## [2026-10-10] verify | Native packaging contract

Two Python tests cover platform-specific packages, exact binary checksums, public asset selection, SBOM/notices and fail-closed license checks. Source layout also passed. Target-filtered license declarations/text fingerprints retained locally. Real artifacts and Release acceptance remain pending; see [packaging contract](native-release-packaging.md).

## [2026-10-10] verify | Packaged workflow

Actual macOS da746b9 packaged binary passed parallel three-worker, diff/tests, refused unapproved merge and exact approved merge in shared/worktree modes. Integrity contract tests passed; native release CI artifact gates added. Windows execution still pending. See [packaged workflow](packaged-workflow.md).

## [2026-10-10] fix | Release archive timestamps

Actual archive failed on pre-1980 registry file timestamps. Red/Green regression preserves bytes and executable mode while clamping ZIP dates. Five contracts passed. macOS archive generated with all CRCs and binary SHA validated; Windows packaged acceptance remains pending.

- 2026-10-10: Windows default startup HOME/USERPROFILE selection, Red/Green evidence and sequential native test lifecycle recorded in [native startup workspace](native-startup-workspace.md). Native Windows acceptance remains pending.

- 2026-10-10: DD-40 runtime icon asset added; five icon tests passed including actual macOS installed applications. Windows path/extraction/rendering acceptance remains open.

- 2026-10-10: Windows CI 38040929721 failed on omitted WebView2 crate license files after release compilation. Exact official MIT notices retained and bound to version/VCS/hash; seven packaging tests and actual 42/108 dependency closure pass.

- 2026-10-10: 0308003 (Windows home selection), 6646f2e (icon knowledge) and 6ccd4bf (upstream license texts) pushed. CI 38041844312 validates source 6ccd4bfe2684f5c7d7dbe32cf7b1aab6ffa35efc; full release acceptance remains open.

- 2026-10-10: native package prerequisite Red/Green completed; public Windows support notes and WebView2/WebKit manifest requirements updated. Seven package/smoke tests pass; source CI 38041844312 still in progress.

- 2026-10-10: Claude resumed after the Codex token stop: autocrlf license-byte fix (a78d5f5) and tag-driven release workflow (9ffc124) assetized in [native release packaging](native-release-packaging.md). Native Korean IME acceptance attempt found the app window in an English input source; not verified.

## [2026-10-10] verify | Native end-to-end with Claude workers

Source: [raw observations](../raw/2026-10-10-claude-native-e2e.md). Three DD-32 gaps found and fixed by TDD (Room listing, per-call binding, synchronous Room decisions); full native run merged candidate 917074f. See [live tool decisions](live-tool-decisions.md).

- 2026-10-10: First pre-release `v0.1.0-preview.1` published through the tag workflow; CRLF checksum defect found by downloading it and fixed (d9501f5). See [native release packaging](native-release-packaging.md).

- 2026-10-10: `v0.1.0-preview.2` published from d84b876; downloaded macOS and Windows archives and both inner binaries verify with `shasum -c`.

- 2026-10-10: QAS-25 native Korean IME acceptance passed (composition across polling, exact UTF-8 save). See [Korean input](korean-input.md).

## 2026-10-11 public docs (DD-42)

README rewritten as desktop-first introduction; docs/user-manual.md holds the full reference. public_docs 3/3, release contracts 13/13, package ships manual and screenshots. Commit 39a55f0.

## 2026-10-11 v0.1.0-preview.3

Release run 38069707257 green on macOS and Windows; archives contain README, user manual and screenshots; SHA256SUMS verified locally. Regression 221/9/0.

## 2026-10-11 reviewed handoff (DD-43)

Red→Green cooperative_handoff; regression 224/10/0.

## 2026-10-11 v0.1.0-preview.4

DD-43 shipped; Release 38072717471 green on both platforms; checksums verified.

## 2026-10-11 memory discovery (DD-44)

Red→Green discovery fixture + real daemon; regression 225/10/0.

## 2026-10-11 v0.1.0-preview.5

DD-44 shipped. CI 38074008190 failed on a cold PowerShell start exceeding the 8 s containment-test timeout (test timing, not product); raised to 30 s; CI 38074563237 and Release 38075480798 green; checksums verified.

## 2026-10-11 workspace UX and planning

DD-45…DD-60 assetized: manual plans, workspace settings/switching/names, release notes, baseline dialog, OS-file ignore, progress, readable instructions, role rules with models, immediate agent save, model lists, plan retention, compact agents, settings export/import, 1–12 tasks, Read plan. Regression 260/11/0.

- 2026-10-11: Codex reconciled clean Development checkout to Claude source 318f930, reviewed R-36–59/DD-42–65, confirmed 16 focused tests and actual preview.6 assets; latest CI jobs did not start due GitHub account billing/spending restriction.

- 2026-10-11: DD-66 Claude quota stream collection; six regressions and local full-suite evidence recorded in [Claude subscription quota](claude-quota.md).

- 2026-10-11: DD-67 inline quota badges and retained workspace observation tests.

- 2026-10-11: R-62/DD-68 planned MCP goal imports; workspace quota collection regression reproduced and corrected.

- 2026-10-11: DD-69 per-agent refresh/action icons and independent workspace collection; live account and cleanup evidence in [Workspace quota collection](workspace-quota-refresh.md).

- 2026-10-11: DD-70 compact multi-window quota presentation and deterministic cache-initialization race correction.

- 2026-10-11: resource constraints reaffirmed, saved to shared agentmemory and signaled to Claude; final compact quota runtime replaced.
## 2026-10-11 slug work trees and idle budget

DD-61–DD-65, DD-71, DD-72 assetized. Regression 287/12/0 on 51f5bb6.

## 2026-10-11 agent preferences and native AC identity

R-67/DD-73/QAS-60 assetized with [native acceptance evidence](../raw/2026-10-11-native-settings-acceptance.md). Default five-minute per-agent configurable refresh, executable discovery/Browse, consistent fields, silent quota jobs and AC application identity are implemented. Actual operator screenshots and English manual updated. Final integrated regression: 293 passed, 12 ignored, 0 failed; 22 Python checks and macOS shared/worktree packaged smoke passed. The OS picker opened outside the app-scoped capture; preserve the working chooser. Windows verification and stable publication remain blocked by GitHub runner billing.

## 2026-10-11 provider-driven identity and quota reset time

R-68/R-69, DD-74/DD-75 assetized in [agent-identity-and-reset](agent-identity-and-reset.md). Repository made public (user decision) so GitHub Actions runs again; first real Windows run exposed a 3 s timing assumption in `claude_quota` stream test, now 20 s with the observer's own error reported. Regression 299/0.

## 2026-10-11 interface language and UTF-8

R-70/DD-76 assetized in [interface-language](interface-language.md). Windows CI exposed cp1252 decoding in Python tests and a one-second race in the Claude quota stream test; both fixed. Regression 303 Rust / 26 Python.

## 2026-10-11 v0.1.1 released (stable)

First stable release after Codex's v0.1.0, published by GitHub Actions from 3cea042 with verified macOS (aarch64) and Windows (x86_64) packages and SHA256SUMS. Lesson: the first real Windows CI run surfaced four Windows-only problems (cp1252 Python I/O, release notes encoding, a one-second test race, an unused `mut` that only exists without the Unix block); validate on both platforms before tagging.

## 2026-10-11 messenger notifications phase 1

R-43 activated by the user; DD-79 implemented and assetized in [messenger-notifications](messenger-notifications.md). Live delivery verified locally over HTTPS. Regression 310 Rust / Python OK. Inbound decisions remain phase 2 (company messenger API to be chosen).


## 2026-10-11 stable v0.1.2: MCP, Windows metadata and custom catalogs

DD-68, DD-78 and DD-80 assetized; concurrent DD-79 messenger changes preserved. Source 5cefa6c passed all four [native CI jobs](https://github.com/neodamul/AgentConductor/actions/runs/38134378978): macOS 320 Rust tests / Windows 135, no failures, 32 Python contracts each. Exact packaged binaries passed shared/worktree parallel execution, tests and approval/merge checks. Both archives, SHA256SUMS, license notices and SBOM published as stable v0.1.2. Existing app replaced with the verified macOS binary and the user workspace/plan preserved. Company MCP auth and real Windows subscription acceptance remain operator checks.

Lessons: a functioning custom CLI does not imply catalog discovery; keep explicit manual model provenance. Windows cmd.exe argv[0] must use native separators/literal cmd.exe; terminal output needs explicit null standard handles under ConPTY. Compile and lint platform-specific tests on the actual platform before release.

## 2026-10-11 messenger decisions phase 2

DD-81 implemented: Slack replies or signed company relay decisions answer live tool requests through the exact DD-32 path. Live round trip verified locally over HTTPS with the system curl. Regression 323 Rust / Python OK. Disk was full during the session; both build caches were cleaned (user approved for the main checkout).

## 2026-10-11 automatic slug names

R-74/DD-82: slug input removed; Hangul romanized for readable unique folder names. Regression 323/0. Lesson: chain doc scripts and commits with `&&` so a failed edit never ships.

## 2026-10-11 v0.1.3 released (stable)

Published by GitHub Actions from 12448c9 after macOS/Windows validation and native packaging: messenger decisions (phase 2) and automatic slug names. Windows MCP remote test was flaky once before the release run (rerun passed).


## 2026-10-12 session communication and optional memory

Assetized the distinction between CLI execution interfaces, A2A-style message exchange, authoritative local state and LLM-wiki knowledge. Local checkpoint operation already works without agentmemory. Documented a general durable signal inbox as a proposal, with receipt/acceptance separation and remote transport limits; no runtime transport was added. Public README contains only product behavior. Source observation pins local hashes and official source links. Another Claude machine owns the in-progress v0.1.9 build; this documentation does not change its version or publish a release.

Follow-up requirement R-92/DD-100: selectable Local Rust or agentmemory-preferred signals, automatic local fallback on memory server/MCP failure, bounded retry and duplicate/late-reply handling. Full BG/SF/UC/QAS/design trace and unchecked TDD implementation checklist added. Runtime support remains pending.

## 2026-10-12 build identity, storage and blocked planning

R-93–R-95/DD-101–DD-103: compile-time version beside both titles; narrowly owned disposable metadata cache preview/cleanup and bounded idle pruning; stop-marker planning cancellation and same-window current-workspace deletion. Focused fixtures and Python48 pass before final follow-up; full compact regression/native/platform verification is pending. Initial full debug compilation exhausted disk; cargo clean removed 22 GiB of generated target artifacts. User workspace/project evidence was preserved.

- 2026-10-12: audited and published stable v0.1.10; verified downloaded platform packages and retained existing reviewed workspace. Reviewed the subsequent signal adapter, reproduced uncertain-restart duplication and added a durable attempt claim plus kernel-owned lease (post-release work).

## 2026-10-13 delivery closed at v0.1.3

The user declared the goal complete with v0.1.3. Open follow-ups need the operator's environment: a real Windows PC run and a live Slack/company messenger round trip.

## 2026-10-18 native sidebar provider icons

Native eframe now decodes the existing bounded provider PNG data URLs, caches textures per provider URL and renders installed Claude/Codex artwork; custom profiles and unresolved icons keep monograms. The rendered regression was red before implementation; 22 focused tests and the full native-feature offline all-target suite pass with zero failures. All-target native-feature Clippy passes with warnings denied. Graphify refreshed the source graph (6,617 nodes / 11,478 edges). CocoIndex is not installed on this host. An optimized macOS app showed installed provider icons with isolated fake CLI profiles. The committed macOS package binary hash matches the inspected build; package validation passed four shared/worktree × Git/folder smoke cases (three tasks and tests each, approved merge verified, unapproved merge refused). Archive SHA-256: `458aef6cc5331f93ce77101d2d85e0edfa616ed1e243319111989d711b31e2ab`. Windows runtime discovery/rendering remains pending. Verified workflow acceptance stays 0/25; issue #9 remains open.

## 2026-10-19 native Resume and integration-repair boundary

Latest `main` (`5993970`, failed-run resume guard) was merged into the #9 Rust branch. Requirement review found the blanket `failed` rejection conflicted with R-123/UC-113: repairable integration failures must re-enter the coordinator's bounded repair loop. A regression first failed under the blanket guard, then passed after the UI guard reused `workflow::repairable`; unrelated failures remain blocked. Nine focused UI/repair tests, native-feature all-target Clippy, rustfmt, diff checks and the source-layout test pass. #9 remains 0/25 pending full packaged operator-workflow acceptance.

## 2026-10-19 full native-feature regression

After latest-main synchronization, the locked offline all-target native UI suite passed 659 tests, ignored 14, across 230 suites. Four local AgentMemory TCP fixtures failed under sandbox restrictions, then passed when the same suite ran with loopback permission. Clippy, formatting, diff checks and the 300-line source-layout test pass. Removed this suite's 92 mock executables and 21 recent fixture folders after it exited; kept older failed-test evidence and validated release archives. This does not change #9's 0/25 accepted workflows.

## 2026-10-19 completed test-fixture cleanup

Removed the remaining 350 `ac-workflow-mock-*` test binaries and 61 `ac-flow-*` temporary repositories from `/private/tmp/agentconductor-tmp` after confirming no AgentConductor or test process was using them. Both prefixes are created by `tests/support/workflow.rs`; logs, failure reports outside these fixture paths, package archives, and the active issue worktree were preserved. This cleanup does not change issue #9's 0/25 workflow acceptance or platform verification status.

## 2026-10-19 issue #16 short-window start feedback

The start status was nested at the end of scrollable Workspace content and disappeared at 700px height during plan approval. A native UI journey test failed against that placement, then passed after moving the existing pending/error text into a bottom panel outside the scroll area. The test drives plan generation, review, exact consent, the pending interval, resize to 700px, and the actual Room transition; it confirms the pending text bounds fit the short viewport. The locked offline native-feature all-target suite passed with loopback permission (the sandbox-only attempt denied four local AgentMemory fixture binds). After the final regression strengthening, the focused journey and source-layout guard passed again; all-target Clippy (`-D warnings`), rustfmt, and `git diff --check` passed. Graphify refreshed the code graph to 6,666 nodes / 11,574 edges. Removed 117 recent generated `ac-flow-*` and `ac-workflow-mock-*` test fixtures after confirming no test process remained. CocoIndex is unavailable in this host environment. Issue #16 remains open until the Rust UI is integrated and verified in packaged macOS and Windows builds; #9 acceptance remains 0/25.

## 2026-10-19 native profile popup visual comparison

Rebuilt the current optimized Rust binary at HEAD `d8658d31a662bdd1790bfdb5266560f13fcbcce6` (SHA-256 `7588325b4cbd4937d20d1c35766b618400a930d815ad1140ad089f7c631654bd`) and compared the agent profile editor with MCP server settings in one disposable 1140×772 macOS app window. Both use the same centered, resizable settings-window frame, title bar, close control and application typography; body layouts differ for their distinct forms. The only registered test profile used `/usr/bin/true`; its workspace and temporary signed app bundle were removed after review. No user workspace/settings were edited. This visual component check does not change issue #9's 0/25 accepted end-to-end workflows; Windows/package and full workflow acceptance remain open.

## 2026-10-19 generated-plan review visibility

Extended the existing native UI journey to require all three generated task ID/instruction summaries in the Review Plan accessibility tree before approval. The first run caught an incorrect test expectation (fixture IDs are `task-0` through `task-2`); corrected it to match the fixture and the journey passes. The 300-line layout guard initially caught the test at 302 lines; compacted it to fit and reran both checks successfully. This adds journey evidence only: no workflow count increase, packaged UI acceptance, or source behavior change.

## 2026-10-19 isolated issue #9 settings-window review

Launched the release build from exact branch head `f9eaf26` as one temporary macOS app bundle with an isolated HOME. Inspected the initial Workspace, Team role assignment, Register CLI agent window, and MCP server registration window. Both settings windows use the same centered native frame, title bar, close affordance, and resize behavior; the MCP window displays the OpenCode JSONC sample. No workspace, agent profile, or MCP settings were saved. Closed the review window and confirmed the process exited. This is a component visual check only; it does not change the 0/25 accepted end-to-end workflow count or the pending platform/package gates.

The exact source at `1dbfc2be14a55de5d41abe01c03c6919a2512ec3` then passed the local macOS `package-release` command. The package manifest records 214 dependency license records and binary SHA-256 `02781d8408ca0fc10edf64b6c737eedbb949d5b47368ac4fbdac1e25063964a7`; the ZIP SHA-256 is `0e3250708aba4474ca94aa9a2526ce881b60f0e22b1cbdadc425974c9801d736`. All four packaged workflow smoke cases passed (shared/worktree × Git/folder; three parallel tasks and tests per case; approved merge accepted, unapproved merge refused). This is local macOS packaging evidence, not a published release, Windows validation, or 25-workflow acceptance.

PR #31's Actions run `39143064496` passed build and source-layout on macOS and Windows but failed the full test step on each OS. macOS failed `native_worker` at the exact request-count assertion (2 observed, 1 expected); the same result was reproduced on the exact `main` baseline and is assigned bug #32. Windows failed four cases in the unchanged `mcp_auto_sources` target; macOS passed all four of those cases, and Windows baseline comparison is tracked as bug #33. Both bugs precede merging planning-rules enhancement #8; PR #31 remains open and unmerged. Source-layout passed on both platforms.

Graphify was refreshed after the issue #9 verification updates using local AST extraction only: 6,671 nodes and 11,579 edges. CocoIndex Code CLI is installed but this repository is not initialized for its index; `ccc status` reported that fact and no CocoIndex index result is claimed.

## 2026-10-19 issue #9 progress metrics

Corrected the progress report to keep the historical source-coverage inventory (24/25 rows marked Complete; Workflow 3 partial) separate from verified native end-to-end parity (0/25) and shipped Rust replacement acceptance (0/25). GitHub issue #9, the implementation plan, and the native Rust UI Wiki now use the same definitions. Graphify local AST extraction refreshed `graphify-out` to 6,671 nodes and 11,579 edges. CocoIndex Code CLI is present, but this repository is not initialized for its index; structural grep returned no matches for the attempted Rust function pattern, so no CocoIndex verification is claimed. `git diff --check` passed; docs-only commit `dbef9f1` was pushed to `codex/issue-9-native-ui-parity`.


## 2026-10-19 issue #9 Workflow 3 source runtime

Ran the exact Rust binary built after source commit `74d3872` (SHA-256 `2e4bcc2ff02576f09c54afd0182389eb23d4a4411b632cd4f447c7ff85905cc0`) in one ad-hoc signed macOS app window with isolated HOME and a disposable workspace. Registered a custom `/usr/bin/true` profile, edited and saved its ID, verified the delete confirmation and removed it; the sidebar returned to its empty state. No agent task was invoked. This completes the source-coverage inventory for Workflow 3, bringing it to 25/25 implemented. The exact released UI was not compared and Windows/package/accessibility gates remain open; verified end-to-end parity remains 0/25. The temporary bundle, HOME and workspace were removed. AgentMemory: `mem_mvedk95b_ffa5371a4f10`.


## 2026-10-19 issue #9 sidebar footer parity

Restored the existing English/Korean local-first heading and approval summary at the bottom of the native CLI-agent sidebar. The sidebar list remains independently scrollable and reserves space for the footer. A rendered-text test using the production left-panel layout failed before the change and passes afterward; existing sidebar tests pass 5/5 and the source-layout test passes. Graphify local AST extraction updated the graph to 6,680 nodes and 11,590 edges. CocoIndex Code's structural grep found the sidebar entry point without initialization; the semantic index remains unavailable. AgentMemory: `mem_mvedynhg_5787cdf797cd`. This component check does not change accepted workflow parity (0/25); packaged macOS/Windows and exact v0.1.24 comparison remain open.


## 2026-10-19 issue #32 bug port to Rust branch

Ported merged PR #34 to `codex/issue-9-native-ui-parity` as `d4b7734`. Preserved the branch’s atomic ledger reconciliation helper, moved the provider tool-call ID into the human-visible request reason exactly once, and added stale-ID approval rejection to the parallel-worker regression. `native_worker` 4/4, `gates` 5/5, and source-layout 1/1 pass locally; focused offline Clippy (`-D warnings`), rustfmt, and diff checks pass. Touched Rust and test files stay below 300 lines. The upstream PR’s macOS/Windows validation and packaging checks were green before merge. AgentMemory: `mem_mvee4h1v_1606fc039071`. Native UI acceptance remains 0/25.

## 2026-10-19 custom CLI event normalization — issue #9

Added bounded profile declarations for custom JSONL event names and session-ID JSON Pointers.
The profile-aware decoder now feeds live progress, worker completion, interrupted-run recovery,
and coordinator-document parsing while preserving built-in provider behavior. A mock CLI test
reproduced the unrecognized vendor completion event blocking the run; after the fix, all three
parallel tasks complete and the run correctly waits for human approval. An all-target offline
locked suite passed before the final cardinality and JSON-Pointer boundary checks. Its repeat after
those checks exposed one `cli_workflow` temporary-directory cleanup race; that test passed when
run alone. Relevant protocol, plan, coordinator, progress, recovery, and workflow suites pass, as
do all-target Clippy (`-D warnings`), formatting, diff, and source-layout checks. Graphify AST
extraction now reports 6,827 nodes and 11,838 edges. CocoIndex is not installed in this host
environment.
This is runtime compatibility coverage only and does not change issue #9's accepted end-to-end
workflow count (0/25). The registry, custom quota declarations, UI editor, and Windows package
acceptance remain open.

## 2026-10-19 issue #9 exact 0.1.30 macOS package

Merged latest main into `codex/issue-9-native-ui-parity`, then built committed HEAD `dbad31d` with
the Rust release packager. The four shared/worktree × Git/folder smoke cases passed; manifest
validation accepted 214 dependency license records. Binary SHA-256 `d2b66628f69d248e5dc92b1244c035afb783ab4280057831fa20174ae0e38e85`; ZIP SHA-256
`07b8956372b0f10ddbbacbfd74e645d1ac345a1c1eeebf2e971acd57a740f9eb`. Opened one isolated native
window, saved a disposable Workspace and confirmed persisted settings. Closed the app and removed
its temporary bundle/HOME/workspace. This package/Workspace check does not advance end-to-end
workflow parity beyond 0/25. Windows and comparison against the legacy shipped UI remain open.

## 2026-10-20 issue #9 full offline test suite

The exact current Rust branch HEAD `1c9759846eab51ea8be1e1ce2446f427b8c5a30a` passed `cargo test --offline --locked --all-targets --features native-ui-probe -- --test-threads=1` with exit code 0. Homebrew LLVM 23 expected a Z3 5.1 soname that was absent while Z3 5.3 was installed; a temporary alias and Rust shim in `/private/tmp/ac-issue9-z3-compat` were needed for Cargo and integration tests that spawn `rustc`. The initial run without the child-process shim failed only while compiling mock agents; the complete retry passed. AgentMemory record: `mem_mvfmtydk_81eea7878957`.

This local regression pass does not establish exact packaged macOS/Windows UI acceptance. Current-head macOS package validation, Windows visual acceptance, and the 25-workflow end-to-end parity audit remain open; accepted workflow count remains 0/25.

## 2026-10-20 issue #9 native finalization verification

On the exact issue #9 branch HEAD, the focused finalization, rendered-room, and source-layout suites passed (15 tests). `cargo fmt --all -- --check`, `git diff --check`, the changed Rust/test file 300-line check (maximum 288 lines), and the optimized release build plus `--help` entry point passed. PR #22 still has no check run for its current HEAD and conflicts with `main`; cross-platform exact-package and end-to-end native UI acceptance remain open at 0/25. Issue #50's default JavaScript behavior remains in merged PR #51; Rust work is additional parity for #9.

## 2026-10-20 issue #9 port of bug #52

Ported the bounded correction attempt for invalid overlapping phase scopes from the already-merged Rust fix to the #9 migration branch. The regression was confirmed red before implementation: duplicate `score:` scopes failed with `unsafe or overlapping scope`. The dedicated regression and source-layout tests now pass after one retry; invalid second responses remain rejected. This preserves the behavior from GitHub #52 when the Rust migration branch replaces the current implementation.

The complete serialized suite then passed 725 tests, 14 ignored, and 0 failed across 249 suites with debug symbols and incremental artifacts disabled; all-target Clippy, rustfmt, diff checks, and the exact optimized release build/CLI help check also passed. The first default-debug test build exhausted the host disk, so only its generated `target/debug` cache was removed before the compact full-suite retry.

## 2026-10-20 issue #9 port of SQLite contention fix

The #9 branch reproduced GitHub #56's 2.5-second `BEGIN IMMEDIATE` writer lock and failed with `database is locked` under its two-second timeout. A shared ten-second timeout was added to writable and read-only SQLite connections without changing transaction semantics. The focused regression passed after 2.53 seconds. Full verification passed: 726 tests, 14 ignored, 0 failed across 250 suites; all-target Clippy (`-D warnings`), rustfmt, diff and 300-line checks, and the optimized release build/CLI help check passed. Binary SHA-256: `9313d3adfa62682d1b85930eaffe9956ca364dc352587c586204ae19282d8562`.

R-153 / DD-163: keep a wall-timeout visible while automatically retrying from retained artifacts; cap the backoff at 60 seconds; honor Stop before relaunch; expose terminal blocked causes through the report-level error used by the existing banner. Focused timeout and Stop regressions pass locally. Full verification on integrated #12 branch: 180 suites, 542 passed, 0 failed, 14 ignored; Clippy, rustfmt, and 300-line source-layout checks pass. The separate Rust-native port remains in progress.

## GitHub #52 generated phase overlap recovery

The coordinator returned duplicate `score:` ownership paths in phase `01-game-selection`, which correctly failed validation but ended planning without a correction attempt. Added one bounded re-plan with the exact validation feedback; overlap remains rejected if the corrected response is still invalid. Deterministic retry, phased planning, schema and MCP planner tests pass. See [planning overlap repair](planning-overlap-repair.md). Main-first merge and Rust-native port are still pending.

## 2026-10-20 read paths must not open SQLite write-capable (#65)

PR #55's macOS failure (`active_feedback` timing out after 20 s) was `workflow::run` ending at start-up with `Err("database is locked")`. `workflow::load` used `Store::open`, which runs `PRAGMA journal_mode=WAL` and `CREATE TABLE IF NOT EXISTS` on every call; polled while a run starts, those statements race the run's own database set-up and SQLite returns BUSY at once (the busy timeout does not apply to lock-upgrade conflicts). `load` now uses `Store::read_only_snapshot`. Lessons: a read must never open the database write-capable; a test that waits for a state must also watch the task that should produce it, or an early error looks like a timeout; keep failed-run directories (`$TMPDIR/ac-flow-*`) as evidence; four threads opening and closing the same database in a tight loop made SQLite mutexes stall for minutes, so concurrency tests need bounded polling.

## 2026-10-20 v0.1.31 release preparation (#68)

Prepared the v0.1.31 release from `main` (WebView line): CHANGELOG section, English/Korean manuals (finalization progress, overlapping-scope retry, #62 profiles, SQLite contention, read-only polling), README and Cargo version. Local gate: fmt, clippy `-D warnings`, full suite; `timed_out_worker_resumes_automatically_and_keeps_timeout_evidence` (1 s timeout) failed once under full-suite load and passes alone. Release is gated on PR #55 (Rust port of #50) being merged into `rust` and #50 closed; tag `v0.1.31` only after that. The native #9 UI is excluded until its gates pass.

## 2026-10-20 messenger de-duplication reset on state change (#72)

`messenger::notify_run` remembered only the last notifiable state, so `failed → running → failed` sent one message. It now records every reported state (when the link is enabled) and sends only for notifiable ones; repeated saves of one state remain silent. Red-first test: `tests/messenger_renotify.rs`; existing messenger suites unchanged.

## 2026-10-20 grouped native UI/operator-surface review (#9)

One existing isolated v0.1.31 app window covered workspace settings/roles, MCP registration, sharing, Messenger, cache preview, custom model configuration, and completed Agent Room evidence. No settings were saved and no cache or external state changed. The associated ten native UI suites passed 48 tests. Evidence: `docs/evidence/issue9-4a114ec-grouped-native-ui-audit.json`. Packaged acceptance stays 4/25; source/UI checks do not count as a new end-to-end workflow. Exact-head run `39435401549` was queued, and Windows/accessibility/IME/provider/resource/license gates remain open.

Graphify re-extracted the issue #9 checkpoint after the regression fix: 7,517 nodes and 12,953 edges. CocoIndex Code remains unavailable because its environment lacks `mcp.server.fastmcp`; no CocoIndex result is claimed.

## 2026-10-20 repository-switch clean-baseline regression (#9)

The grouped Git/worktree smoke found that switching a workspace to another repository while keeping identical goal text retained the old `active_slug`. The next plan wrote metadata into an unexcluded goal folder, so the clean-baseline approval gate correctly refused to start. Red-first regression `changing_repository_detaches_the_active_goal_before_planning_again` failed before the fix. Saving now clears only the active slug when the repository path changes and preserves prior slug records. Focused validation passed 15 tests across five suites; all-target regression passed 771 tests with 14 ignored across 260 suites; Clippy passed with warnings denied.

The exact `a1ea942` macOS package passed package-release smoke and a single-window isolated UI run: switching to a clean Git repository while retaining the goal cleared the old active slug, planning created three disjoint worktree tasks, 3/3 task tests passed, the exact diff was reviewed and approved, merge completed, and the fixture's `main` worktree was clean with only the three intended files. Evidence: `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Packaged issue #9 acceptance advances to 5/25 (20%); shipped Rust replacement remains 0/25. Windows, accessibility, IME, authenticated CLI, resources, license closure and exact-head CI remain open.

## 2026-10-21 explicit messenger decision commands (#81)

The main-branch parser accepted an allow/deny word anywhere in a reply, so conversational or negated text could approve a request. Regression tests first reproduced `don't allow CODE` as an approval. `inbound::parse` now checks only the first command after optional mentions and still accepts explicit allow/approve/deny/reject forms. The Rust #9 branch keeps the parser in `src/messenger/inbound/wire.rs` and preserves its public re-export; this avoids undoing the issue branch's module split. Focused tests passed (4 passed, 1 ignored); the full local all-target suite passed (794 passed, 14 ignored across 263 suites); all-target Clippy passed with warnings denied; rustfmt, diff, and source-layout passed. GitHub Actions has not listed an exact-head run for merge commit `1b1a92058b8f2219626a386cecf6eb8963b155cc`; the prior run targets the pre-merge commit and is stale.

The current exact-source macOS package at `0d09b88` passed `package-release`, manifest validation, packaged `--help`, ZIP CRC, and archive scans for JavaScript/Python files and inline scripts. It contains one 17,942,384-byte executable and 214 dependency records. Binary SHA-256 `1425a6384fd7b24382acc91226de007fe8646c6b579e5d9d632d4b3f427b51db`; archive SHA-256 `3c06a3d3ea4407359a0fdc48fdfb04341a3627ad892c2b7872f81701fd81f009`. Evidence: `docs/evidence/issue9-0d09b88-macos-package.json`. The package was not opened; it adds no UI acceptance. Windows package/UI and exact-head Actions remain open.

## 2026-10-21 restore compact legacy step navigation (#9)

Compared the isolated native v0.1.31 Workspace with a checked-in same-size capture carrying a v0.1.24 label. Screenshot provenance identifies that reference as an earlier Rust build, not the published WebView UI. It showed compact content-sized tabs while the native three-step row used equal-width, 48px-high cards; this supported a Rust-component layout correction but did not establish WebView parity. Red-first bounds regression failed at 48px; the row now uses content-sized localized buttons that wrap only when needed. The exact package from `227854d3` validates, its `--help` exits 0, and ZIP CRC passes; its 17,942,384-byte executable SHA-256 is `8568714474af52d04e7cf2f678dd69d27a8f95c4149c59c51b59993fa34ea97a`. One exact-package window visually confirmed compact tabs and navigation through Roles and Review at 1140×772. Header/compact-header/start-journey suites pass (5 tests / 3 suites); after the exact-head macOS CI failure exposed a test helper's language lookup mismatch, the settings journey passed three consecutive runs with the helper fixed, and the grouped four-suite batch passed 6 tests. Focused Clippy, rustfmt, source-layout, diff, and 300-line checks pass. A full all-target attempt was stopped after 666 passed / 14 ignored across 213 suites (exit 130), so it is incomplete. Graphify refreshed to 7,532 nodes / 12,968 edges; CocoIndex Code remains unavailable because `mcp.server.fastmcp` cannot be imported. No screenshot file was retained; README/manual screenshot parity, native IME, Windows rendering, remaining workflows and full suite remain open. No packaged workflow acceptance is added: 5/25 packaged and 0/25 shipped.

Graphify was re-extracted without clustering after this record: 7,532 nodes and 12,969 edges. CocoIndex Code remains unavailable because its environment cannot import `mcp.server.fastmcp`.

Graphify re-extracted the current tree locally without clustering (7,526 nodes / 12,963 edges); a local query located the packaged-workflow, exact-package and completion-audit assets. CocoIndex Code is installed but its entry point still fails before command handling because `mcp.server.fastmcp` is missing. This is an environment failure, not successful indexing.

## 2026-10-21 center workspace-sharing settings dialog (#9)

The native workspace-sharing settings dialog used egui's compact default placement and opened at the viewport's upper-left, unlike the centered MCP, Messenger, and cache dialogs. A rendered regression first failed with the Export action at x=30 in a 1140px viewport. The dialog now uses a centered anchor and 480px initial width, matching the cache dialog. Six related native suites pass 18 tests. Component evidence does not change packaged workflow acceptance (5/25) or shipped Rust replacement (0/25); exact current-package screenshots and Windows UI remain open.

The exact `97a16bb4ca40070760d6052e5bd095ef6b1c4a3c` macOS package also passed `package-release`, manifest validation, packaged `--help`, and ZIP CRC. Binary SHA-256: `2f9402e0f14b9c90083a7c897211e0e7968be08039543178d12b43ac4ea94eaf`; archive SHA-256: `5380a52ad58763375a168324b928db26cf95d787f57e884ed6da2e3aa83b49e6`. Evidence: `docs/evidence/issue9-97a16bb-macos-package.json`. The package was not opened, and this does not add workflow acceptance.

Correction: the package was opened in one isolated temporary HOME after old package processes were stopped. The current exact build visually shows the sharing dialog centered in light/dark themes; MCP, Messenger, and empty-cache dialogs were also reviewed. No workspace/settings/cache mutations occurred. No screenshot was saved as a repository artifact, and component review does not increase the 5/25 workflow count.

The same isolated build's profile-registration form, five-role stage, and empty Review stage were checked as a group. Registration Cancel preserved the empty profile list; the empty Review page showed no reviewed plan and kept approval disabled until settings and roles are saved. These are component checks only.
# 2026-10-21 v0.1.32 parity port into issue #9

Merged the current main source into the Rust UI branch, resolved module-boundary conflicts without retaining its WebView/CJS implementation, and added two system-architecture traces (R-161/R-162 through DD-171/DD-172). TDD regressions were first red for fail-closed shell forms, case-insensitive task scopes, MCP environment headers, IANA quota zones, pending-task reassignment, and update-download cleanup. The Rust integration now connects those contracts plus queued agent/model edits, running-task feedback, and factual Agent Room activity.

Focused local verification passed 36 tests across 12 suites. Full local verification passed 800 tests with 14 ignored across 271 suites. The 300-line source-layout test, all-target warning-denied Clippy, rustfmt, diff checks, and offline manuals passed. The optimized macOS release package for `cf92f39` passes a four-case packaged smoke across shared/worktree and Git/folder combinations, with approved merges verified and unapproved merges refused; archive CRC passes. Archive SHA-256 is `fd17557d0be3a27470fccbbf75c372c279ed4d93bde905eb447d54f65755fc51`; evidence is `docs/evidence/issue9-cf92f39-packaged-smoke.json`. Exact-package UI review, Windows packaging/UI, and exact-head Actions completion remain open. Packaged issue #9 acceptance remains 6/25 and shipped Rust UI remains 0/25. Graphify re-extracted the tree at 7,699 nodes / 13,223 edges. CocoIndex remains unavailable because the configured Python environment lacks `mcp.server.fastmcp`.

See Current-main Rust parity for architecture, test scope, and source map.

## 2026-10-21 Agent Room focus-follow port — issue #109 / issue #9

Ported the WebView's completed-session focus rule into the native Rust Agent Room: a selected task that transitions from running to terminal yields focus to the next running or `waiting_tool_approval` task. TDD first reproduced missing selection, then the new activity suite passed 6/6. Source layout (2/2), warning-denied all-target Clippy, rustfmt, and diff checks pass. The full all-target run surfaced an unrelated, independently reproducible “Open Agent Room” workspace UI failure after a completed run; the backend still marks its run available, so this remains an open #9 gate. Packaged workflow acceptance stays 8/25. AgentMemory: `mem_mvhn80be_75cd291be699`. See [current-main Rust parity](current-main-rust-parity.md).
## 2026-10-21 coordinator chat manual parity — issue #9 / main #115

The Rust Workspace coordinator-chat feature was present, but the public English/Korean manuals and generated HTML did not describe it. Added the Workspace entry point, read-only CLI scope, shared planning slot, and current-window-only conversation note. The first public-doc regression failed on the missing English label; after updating the Markdown and regenerating both HTML manuals with `agentconductor manual render`, `public_docs` and `source_layout` passed (13 tests). This is documentation parity only; packaged acceptance remains 8/25.

## 2026-10-21 fixed-slot Agent Room activity — main #117 / issue #9

Ported main's stable single-row live activity layout into the Rust Agent Room. A rendered test first reproduced a 57px vertical shift between empty and populated activity; the fix now keeps a localized, truncated 18px row in both states. Seven native task-activity tests, two source-layout tests, warning-denied native-feature Clippy, Rustfmt, and diff checks pass. Packaged workflow acceptance remains 8/25 pending exact-package verification.
# 2026-10-21 — Keep Stop-during-backoff test stable (#112)

Aligned the Rust migration branch's stop-during-backoff regression test with main: retry delay 5s and stop-response bound 2s. The focused test and full workflow suite passed (8 tests total). No production behavior changed; packaged native UI acceptance remains 8/25.

## 2026-10-21 Main integration and test-temp cleanup — issue #9

Integrated the current-main documentation and non-WebView changes into the native Rust migration branch. Removed duplicate chat declarations and the upstream WebView UI/CJS tests while retaining Rust-embedded locale JSON. Regenerated the English and Korean HTML manuals, restored screenshot links referenced by the public manuals, and made the room-header version assertion follow `CARGO_PKG_VERSION`. Full local verification passed 828 tests with 14 ignored across 278 suites; all-target Clippy, formatting, source-layout, and public-doc checks passed. A duplicate unconditional chat-result read was removed after caller tracing; focused `startup_chat` and `chat_ask` tests pass. Removed 148 MB of test-generated `target/debug/tmp` files. The optimized package for exact merge HEAD and Windows package verification remain pending. Packaged UI acceptance remains 8/25; current screenshots and complete Rust UI parity remain unverified.

## Exact merge package verification — issue #9 / 2026-10-21

The `9be8e88` exact-source `aarch64-apple-darwin` package passed packaged `validate-package`, `--help`, archive CRC, and the four approval-gated smoke cases: shared/worktree × Git/folder. Each case ran three concurrent deterministic mock tasks and tests; approved merge was verified and unapproved merge refused. Binary SHA-256: `86c72bf263fa32b4affe652b38b7d4d80d8125d754372252ae84509b63993d30`; archive SHA-256: `e82f2a852215413a946ef54829237725e8c2b5b47779171d5ac301f1b2c16916`. Evidence: `docs/evidence/issue9-9be8e88-macos-smoke/run-36772-1792567832764762000-0`. Windows exact-source package remains pending; this is not visual UI acceptance, which remains 8/25.

## 2026-10-21 Coordinator analysis popup layout — issue #9

A rendered regression reproduced the Review page's whole-goal analysis disclosure moving the next phase down by 8 px. Replaced the inline collapsing section with an anchored, 480 px-bounded menu popup; explicit accessibility metadata keeps the analysis in the assistive-technology tree while phase geometry stays fixed, and an outside click closes it. The new geometry/accessibility/popup-content regression and workspace/Settings menu, goal-list, and phase-progress suites pass (14 tests); adding source-layout checks brings focused verification to 16 tests across five suites. Warning-denied all-target Clippy, formatting, diff checks, and the 300-line limit pass. The exact 1743fed release binary builds for `aarch64-apple-darwin` and passes `--help`; SHA-256 is `3dd649d07d6ca7f311922fe5a48f3a8576bcb1a08852842a62e5e7fd0688b76e`. Graphify refreshed to 8,009 nodes / 13,699 edges. CocoIndex is installed but cannot start because its environment lacks `mcp.server.fastmcp`. Package-level visual verification remains open; packaged workflow acceptance is unchanged at 8/25.

The exact `ce33720` macOS release package then passed manifest validation, `Info.plist` lint, archive CRC, packaged `--help`, and four deterministic shared/worktree × Git/folder smoke cases. Each case ran three tasks and three tests; the harness verified approved merge and refused unapproved merge. The package contains one native executable, SQLite, an SBOM, and dependency license texts for 214 crates. This is CLI/package smoke evidence, not visual UI acceptance; issue #9 remains 8/25 and the live user window was not replaced. Archive SHA-256: `7f773a96d81b3b02838bef622548cc83a3b08f9e75c64a4de5351c2e9d1749c6`. Evidence: `docs/evidence/issue9-ce337-macos-package.json`.

The next rendered pass found the same vertical reflow in phase-detail rows: opening a completed phase moved the following phase down by 8 px. Phase rows now open bounded, scrollable popups (480 px wide, 360 px high) with accessible names, preserving other phase and action positions. TDD passed the phase-popup interaction/details tests (3), phase-progress tests (6), and coordinator-analysis popup test (1); source-layout (2), warning-denied all-target Clippy, rustfmt, diff, and 300-line limits pass. AgentMemory: `mem_mvhyjd8s_12fb1fc7778b`. Graphify refreshed to 8,025 nodes / 13,740 edges. CocoIndex CLI was attempted but cannot start because `mcp.server.fastmcp` is missing from its runtime. No live app window was replaced. Packaged acceptance remains 8/25 until the exact build is reviewed.

The exact `8a2f817` macOS package then passed `package-release`, manifest/source/target/binary-hash validation, packaged `--help`, ZIP CRC, and all four shared/worktree × Git/folder smoke cases (three tasks and tests per case, approved merge verified and unapproved merge refused). Its 18,158,448-byte binary SHA-256 is `b75e5ce72c817fb91a14ac10b90a25baf57d17f8b31365c26249be8909c90e9e`; archive SHA-256 is `941b45048b6e18d344939f3d56050219a849254d732fda4ee2b1bc769a4a2a7d`. The native-ui-probe all-target suite passed 832 tests (14 ignored, 283 suites); public-docs, source-layout, Clippy, formatting, and diff checks passed. Evidence: `docs/evidence/issue9-8a2f817-macos-package.json`. AgentMemory: `mem_mvhyt2tp_9d246923164f`, `mem_mvhz8vy7_ac0aefbddc65`. The prior exact-base Actions run `39581794809` passed macOS/Windows validation and its macOS package; Windows package execution remains open. No app window was launched; exact-head visual review and Windows CI remain open. Packaged UI workflow acceptance remains 8/25.

## 2026-10-22 Team Briefing popup geometry — issue #9

Added a rendered Rust UI regression for the Team Briefing disclosure. With a populated session fixture, the popup paints its session details and the following Stop control retains its original y-coordinate. The focused regression passes; the broader popup audit passes 21 tests across six suites, and layout/docs coverage passes 19 tests across five suites. This is component-layout evidence only: packaged workflow acceptance remains 8/25, and no product UI behavior changed.

## 2026-10-22 Remaining native menu layout checks — issue #9

Audited every `menu_button` and popup call site after another reported vertical shift. The remaining untested Profile Rules JSON and task-detail editors already use anchored egui popups; rendered tests now verify that the Save action and following task keep their original y-coordinates. Focused popup/UI regressions pass (27 tests across nine suites), warning-denied focused Clippy, `source_layout`, formatting, and diff checks pass. The profile editor regression is included in macOS/Windows native UI CI. No application-source fix was needed and packaged acceptance remains 8/25.

## 2026-10-22 Cargo feature gate for Team Briefing popup regression — issue #9

Actions run `39634545852` failed macOS and Windows because Cargo auto-discovered the new
`tests/native_room_briefing_layout.rs` target without the `native-ui-probe` feature gate. The test
now has `#![cfg(feature = "native-ui-probe")]`, matching other UI integration tests. The
no-default-features target-gate audit and focused Team Briefing popup test pass, as do rustfmt and
diff checks. The local all-target run completed; one unrelated `cli_workflow` test reported a
`DirectoryNotEmpty` cleanup error but passed when rerun alone. All other targets passed. No UI
workflow acceptance changes. Exact-head macOS/Windows Actions verification remains pending.

## [2026-10-22] verify | Recheck Rust menu overlays and exact macOS package — issue #9

The single Rust preview window showed Workspace and Settings menus overlaying the page. Twenty-two rendered menu/popup layout tests passed across six suites. The exact `d23c3c2` macOS package passed help, package/archive validation, and four mock-agent smoke cases across shared/worktree and Git/folder modes; each ran three parallel tasks/tests, refused a pre-approval merge, and verified the approved merge. Evidence: `docs/evidence/issue9-d23c3c2-macos-package.json`. This does not add an accepted packaged UI workflow (8/25). Graphify refreshed without clustering (8,159 nodes, 14,009 edges). Actions run `39687975008` completed: Windows passed and macOS failed in the immediate Settings-dialog dismissal assertion.

## [2026-10-22] fix | Keep MCP import-review results out of Workspace flow — issue #9

The final disclosure audit found one more page reflow: opening the MCP source-selection preview inserted its result cards in the Workspace scroll area and moved the execution controls down by 116 px (anchor y=61 → y=177). A rendered regression reproduced the shift first; the results now use a centered, scrollable window and a reopen control. All other `menu_button` details and long-form dialogs already overlay the page. Fourteen tests across eight focused suites pass, as do warning-denied all-target Clippy, the two-test source-layout target, public-doc checks, formatting and diff checks. The Settings journey passes five local runs after changing the one-frame close check to bounded polling. The exact `83f8e61` macOS package passed validation and four approval-gated smoke modes; evidence is `docs/evidence/issue9-83f8e61-macos-package.json`. Graphify refreshed after the docs update with no additional code-graph topology changes. CocoIndex Code remains unavailable because `cocoindex-code --help` fails to import `mcp.server.fastmcp`. A full all-target attempt was stopped after 483 passed / 14 ignored and is not reported as complete. The previous exact-head CI passed Windows but failed the old one-frame macOS assertion; the current PR has no new checks reported. Packaged workflow acceptance remains 8/25, so issue #9 stays open.


## 2026-10-22 | verify | Exact v0.1.34 packaged native plan-to-merge UI — issue #9

Opened the package built from exact source `76d8082b` in one window and completed three concurrent
mock worktree tasks against a disposable Git project. All task tests passed; the exact candidate
diff was reviewed; candidate approval and merge confirmation were required; the app reached
Completed and the disposable project was clean. The custom parser requires `type=result` unless a
profile mapping declares another terminal event. Evidence:
`docs/evidence/issue9-76d8082-macos-package-ui-workflow.json`. This repeats accepted workflows 4/5;
count remains 8/25. AgentMemory: `mem_mvj06ed6_a5514176f478`.

## 2026-10-22 | fix | Match native update indicator to GitHub #127

The upstream v0.1.35 requirement specifies a downward-arrow update control, but the Rust UI still
painted `↑`. A rendered regression failed with the actual glyph while confirming the localized
accessible name; changing the glyph to `⬇` made the regression pass. The focused native update and
source-layout suites pass 11 tests, with formatting, warning-denied focused Clippy, and diff checks
clean. This is component parity only; issue #9 acceptance remains 8/25. See
`knowledge/wiki/native-update-indicator.md`.

## 2026-10-22 | feat | Port update repository Settings to the Rust UI — GitHub #130

Added an accessible native Settings entry and a localized editor backed by the existing asynchronous
Room worker. The rendered UI journey enters `acme/tool`, saves it, closes the editor, and reads the
persisted value back; backend tests continue to cover GitHub Enterprise URLs and rejection without
overwriting a valid setting. The manual HTML files were regenerated by the Rust renderer. This
feature is integrated into issue #9; its packaged workflow acceptance count remains 8/25. See
`knowledge/wiki/native-update-source-settings.md`.

## 2026-10-22 | feat | Coordinator triage of failed tasks — GitHub #135

The existing bounded coordinator repair loop now also handles an eligible failed run with a
failed task, unless the task is waiting for availability or the failure is out of scope. A revise
verdict resets that task and its dependents and resumes the run; a missing coordinator, blocked
verdict, or unusable review leaves the failure for the operator. Coordinator decisions during
quota exhaustion remain a follow-up.
See [coordinator failed-task triage](coordinator-failed-task-triage.md).
## 2026-10-22 | feat | Port step lessons into the native Agent Room — GitHub #138

The native task-detail bridge returns bounded run lessons and the Agent Room Lessons view renders each lesson's source task, cause, and response in English and Korean. Coordinator revisions and replaced test commands record lessons through the existing run store and memory outbox; prompt and scope-recovery integrations remain in the shared Rust workflow engine. This adds no store, process, or permission path. Packaged UI acceptance remains 8/25 pending exact-package end-to-end evidence.

### Exact-source macOS package checkpoint — 2026-10-22

The pushed `ead895e51bcf92b6d843447cc8c3c00790958a8b` head packages as `aarch64-apple-darwin` with the binary and manifest naming the same source commit. Package validation, packaged `--help`, internal binary SHA-256, and archive CRC pass. The archive hash is recorded in `docs/evidence/issue9-ead895e-macos-package.json`.

The GUI was not replaced because the current application showed an unsaved-workspace warning. Windows exact-head package/runtime and a complete packaged GUI workflow remain unverified; issue #9 stays at 8/25. This checkpoint does not count as UC-28 acceptance.

Existing reviewed lessons are manual, hash-pinned and only reach the same task. New `workflow::step_lessons` records bounded lessons from coordinator revisions, replaced test commands, and approved scope changes, shows them in the Agent Room, adds them to worker prompts, and uses the memory outbox plus best-effort synchronization when AgentMemory is preferred.

## 2026-10-20 Worktree cleanup after merge or reject (#143)

`Repository::remove_worktrees_under` removes linked worktrees contained by a run folder and prunes dangling entries while leaving unrelated worktrees alone. Merge and reject call it after saving final state; cache clearing and workspace deletion call `workflow::release_run_worktrees` first. Cleanup failures are recorded as `worktree_cleanup` and do not fail the run.

## 2026-10-22 Three-task Review row visibility (#9)

Added a full-screen rendered regression for the prior observation that a manual plan summary showed
three tasks but only two editor rows seemed visible. At 1140×772, all three localized task controls
are present and inside the viewport. The initial English-only expectation was invalid under the
Korean host locale; no current-source UI defect was reproduced. This test does not claim exact
package, Windows, or new workflow acceptance. See `docs/evidence/issue9-review-three-task-rows.json`.

## 2026-10-20 Collapsible task detail with decision-driven opening (#146)

room.js gets a pure inspectorStep reducer (preference, open, decision key) and decisionKey derived from the snapshot; the workbench gets an insp-closed class that narrows the third column; the heading holds the toggle and a warning dot whose opacity animation stops under reduced motion. The choice is kept in local storage with a safe fallback. Verified in a browser with a real repaired run in dark and light themes and at 800px.

## 2026-10-22 Coordinator decision record (#149)

workflow::decisions derives one list of coordinator judgments from existing audit records (no second log); Store::audit_of and count_events feed it; the snapshot carries decision_count, the coordinator request judgments and the task request coordinator_decisions; the Agent Room renders judgment entries and the count. First step of #149 (guardrail: decision record, launch gate, typed amendments, charter).

## 2026-10-22 Launch gate (#149 step 2)

workflow::guardrail::launch_gate resolves every agent executable and task test command (PATH, absolute, project-relative, or a script a task writes inside its owned paths) before the run folder is created and refuses with typed violations; a passed gate is a launch_gate audit record shown as a gate judgment. No command is executed: a dry run would mutate the project and tests legitimately fail before the work exists.
## 2026-10-23 Native typed coordinator amendments (#149 step 3)

TDD: the new Revision contract test first failed to compile because typed `add_paths` and
`add_dependencies` were absent. Rust now accepts those bounded fields while preserving old review
records. The existing plan validator rejects malformed, overlapping and cyclic scope/dependency
changes; broad top-level directories, sensitive paths and the five-change budget escalate without
mutating the plan. Safe amendments are audited and can be undone from the native Agent Room; undo
invalidates derived evidence and stops the run for explicit resume. The mock coordinator tests the
complete failed-scope → amendment → re-execution → waiting-for-approval flow and the human undo.
Focused Rust/UI tests pass (22 tests across five suites); after review, explicit dotted directory
scopes and duplicate dependencies received additional regressions and pass. All touched source and
test files are under 300 lines. Warning-denied all-target Clippy, formatting, source layout and
manual rendering pass. Offline all-target tests pass 925 / 14 ignored / 306 suites when one unrelated
UI-start test is filtered; the unfiltered run blocks inside Controller::snapshot → Store::open.
Graphify was refreshed against the current tree (8,737 nodes / 13,793 edges) to confirm guardrail
and amendment ownership; generated graph output is ignored local data. Step 4 charter/drift checks
and Windows/package acceptance remain open.

## 2026-10-23 Charter persistence and goal-drift gates (#149 step 4)

TDD started with a failing compile test for Plan rules, Report charter, drift checks and typed
amendments. The implementation persists the approved goal, planning-rule SHA-256 and ordered task
ID/path/dependency/test contract. Scheduler, review, approval and merge boundaries reject
unrecorded changes as `goal_drift`; the existing decisions list exposes the audit event. Old Plan
and Report JSON remains readable. Coordinator amendments, test-command replacements and
operator-approved scope recovery update the charter; Undo restores the prior contract. A first
full regression caught scope recovery omitting the charter update; that flow was fixed and the
unfiltered offline suite then passed 932 tests / 14 ignored / 307 suites. Warning-denied Clippy,
source-layout, format and diff checks pass; changed Rust files remain at or below 300 lines.
Graphify indexed the current source (8,762 nodes / 15,015 edges). `/private/tmp` was 463 MB and
contained other-project files, so it was preserved; this worktree's 14 GB Cargo `target/` is a
rebuildable cache to remove after final local checks. CocoIndex is not installed in this environment.

## 2026-10-23 Rust port of quota credit rounding (#162)

The Rust quota display helper now rounds finite amount strings below the `1e15` magnitude bound,
matching the merged main fix for issue #162. Invalid, non-finite, and out-of-range strings remain
verbatim; quota snapshots and percentages are unchanged. Red reproduced the long fractional amount.
Quota display tests pass 7/7, source-layout 2/2, warning-denied all-target Clippy, rustfmt, and diff
checks pass. Changed Rust files are 147/200 lines. The exact-source package and Windows display have
not been checked; issue #9 remains at 8/25 acceptance pending packaged UI verification. The
unfiltered locked/offline all-target suite subsequently passed 933 tests / 14 ignored / 307 suites
on exact source `b7b67a0c`. Source `97876887c26d` then built a local arm64 macOS release
package. Manifest, binary and archive checksums validate; the packaged binary passed four mock
smoke cases for parallel task execution, test results, approval refusal and post-approval merge.
The package evidence is in `docs/evidence/issue9-97876887-macos-package.json`. The native screen,
Windows execution and 8/25 packaged UI acceptance remain unchanged.

## 2026-10-23 Manual coordinator fallback during quota waits (#9)

Implemented the pending R-163/DD-173 follow-up: a manual evidence review uses the selected
coordinator when its shared account has reset; otherwise it waits for the existing `fallback_due`
deadline and follows only its registered, unrestricted fallback chain. The fallback model reaches
the read-only CLI invocation. Audit receipts include requested/effective coordinator IDs and the
selected model. A missing restriction fails closed; before eligibility there are no review process
artifacts. This does not auto-triage quota waits or alter task retries/assignment.

TDD first reproduced the old refusal, then passed both deadline/fallback cases. Review, fallback,
availability, repair, docs and source-layout suites pass 43 tests. The full locked/offline all-target
run passes 935 tests / 14 ignored / 308 suites; warning-denied Clippy, rustfmt, diff checks and the
300-line limit pass. Graphify updated to 8,782 nodes / 15,068 edges; CocoIndex is unavailable.
`cargo clean` removed 44,958 files / 11.8 GiB and preserved `dist`. #9 remains at 8/25 pending
packaged UI and exact platform acceptance.

## 2026-10-23 Exact-package Agent Room reinspection (#9)

Built and launched the v0.1.36 macOS package from `01f6443b804918ee23b4bd12d84e9e3e64674a16`.
In the retained disposable mock workspace, the Agent Room showed three completed tasks and three
passing tests, candidate diff, per-task session IDs/raw JSONL, zero coordinator judgments, and a
read-only team briefing with absent processes and stale results. The candidate remains pending
human approval; no approval or merge occurred. This repeats an existing fixture and does not change
the 8/25 workflow acceptance count. Windows UI and later plan charter/drift checks remain open.


## 2026-10-23 Exact-package goal-drift label display (#9)

The exact macOS package from `aa8090405239` rendered the Korean goal-drift coordinator
judgment in its native Agent Room. To preserve the user's approval candidate, the event was
inserted only into a disposable copy of the pending run under `/private/tmp`; the original
database still has zero goal-drift events. The copied UI showed 3/3 tasks and 3/3 tests. No
approval, rejection, or merge was performed. The original single-window view was restored and
the copy removed. This verifies the packaged label rendering only; workflow acceptance remains
8/25 and Windows package/UI verification remains open.


## 2026-10-23 Exact-package Workflow 9: post-approval drift refusal (#9)

The exact v0.1.36 package approved the exact candidate in an isolated run copy. After a matching
baseline charter was added only to that copy, the plan goal was changed. The packaged merge
command rejected the operation with `[goal_drift]`, persisted one coordinator judgment, and
left the fixture repository HEAD unchanged. The Agent Room showed the Korean blocked label and
disabled Merge while retaining 3/3 task/test evidence. The original app-managed run remained
pending approval with zero drift rows, was restored, and the copy was removed. This distinct
failure-path workflow advances packaged acceptance from 8/25 to 9/25; Windows UI remains open.

## 2026-10-23 Exact-package Workflow 10 and branding review (#9)

Workflow 10 used the exact v0.1.36 package at `b298ae79774e` to send a coordinator-directed
operator request in a disposable copied run. The queued, processing and completed states rendered,
and the read-only response cited evidence. All tasks/tests remained complete; the candidate stayed
pending approval and the original run was untouched. Evidence:
`docs/evidence/issue9-b298ae7-packaged-ui-coordinator-request.json`. Acceptance is **10/25 (40%)**.

The exact package from source `6f1111c5ae61c91933e03a20fb04a4df513193de` was then opened in the
single app process. The Agent Room showed the shared AC mark/name/version once; candidate review
displayed three diffs and three passing tests. The temporary candidate remained
`waiting_approval`, and approval/merge were not triggered. This component verification adds no
workflow. Docs commits `c02ce6a0` and `4c5714d9` record the screen check; archive SHA-256 is
`6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`.
## 2026-10-23 Upstream main parity audit for the Rust branch (#9)

Fetched `origin/main` at `c9c575a7` and compared its seven commits since shared base `e9d1f1ff`.
The #149 amendments/launch gate/charter, #156 CI readiness, #162 quota rounding, and #165
coordinator feedback have equivalent Rust implementations on the issue branch. Six focused
suites pass 22 tests. The Rust branch intentionally does not merge main wholesale: main has 47
project-owned JS/CJS/Python files and consolidates Rust modules, while the issue branch contains
none of those extensions and maintains the 300-line limit. Main's R-172 renumber for #165 would
also collide with the existing #156 R-172 trace, so Rust keeps #165 at R-173. This audit changes
no packaged workflow acceptance and keeps #9/#149 open pending review and release acceptance.

## 2026-10-23 — Agent Room shared-branding contract

The selected-language Agent Room regression suite still expected the app name inside the page,
although the shared native shell owns that branding. Corrected the test to assert that the app name
is absent from the page. The initial related run caught this stale expectation; after the test-only
fix, the room-language suite passed 3/3, related status/language/render suites passed 19/19,
source-layout passed 2/2, and formatting passed. Graphify refreshed to 8,862 nodes, 14,004 edges,
and 688 communities. CocoIndex is unavailable in this environment (`cocoindex` command missing).
No UI behavior or packaged acceptance changed.

## 2026-10-23 — Coordinator amendment bounds and test-command repair (#149)

Compared the latest seven commits on `origin/main` against the Rust port. The Rust review parser
accepted up to 32 paths / 12 dependencies and 4,096-byte path values, while main's current
deterministic gate is eight entries per type and 1,024 bytes per entry. A red coordinator-contract
test reproduced the wider acceptance. Added one shared bounds check at review and application
boundaries. Added a full failed-check → coordinator test-command replacement → audit/charter update
→ exact Undo test; it first failed because the fixture only proposed scope changes. The corrected
fixture and test pass. Six related suites pass 28 tests; source-layout 2/2, all-target warning-denied
Clippy, formatting and diff checks pass. Files remain below 300 lines. Graphify refreshed to 8,873
nodes / 14,024 edges / 683 communities. The upstream test-command feature remains required by the
open #149 issue; this port keeps it inside the typed, validated, auditable amendment path.

## 2026-10-23 — Agent Room review and activity layout (#9)

At the 1140 px reference viewport, the previous Rust layout gave the center review column about 428 px and let activity cards shrink to 257 px despite unused review space. Red tests now require a 550 px review span, a bounded inspector, and activity frames at least 540 px in a 620 px test surface. The summary and inspector widths were balanced, and each activity group now takes the parent review width.

The exact macOS arm64 package from `1f152729e0c3babd0beb2b6338536b667ea1671c` was validated and inspected in one v0.1.36 window. The activity cards span the center column; the inspector stays bounded. Candidate `3db8840b02ce28558701698c327a5820b7dc7311` remained `waiting_approval` with 3/3 tasks/tests; no approval, rejection, or merge occurred. The full locked/offline suite passed 929 tests (14 ignored), and Clippy, release build, formatting, and the 300-line source check passed. Graphify reports 8,886 nodes / 14,036 edges / 689 communities; CocoIndex is unavailable. This component visual check keeps packaged acceptance at 11/25. Package evidence is in `docs/evidence/issue9-1f152729-agent-room-layout.json`.
## 2026-10-23 — Fix first-use company Messenger test payload (#9 / workflow 13)

The exact v0.1.36 package at `2ef5c3b` reproduced a failure in Settings → Messenger: changing
from Slack to company webhook caused the view to turn an unset template into `Some("")`; the
explicit test action then failed with `message body is not JSON` before the curl transport ran.
A new AccessKit-backed test failed red on the empty editor value. The view now seeds the editor
with the same `{"text":"{{text}}"}` default that the Rust payload builder already uses, and the
regression verifies that the production renderer produces a JSON object. The focused Messenger
suite passes 2/2, source-layout 2/2, public-docs 11/11, all-target native-feature Clippy with
warnings denied, rustfmt, and diff checks pass; changed Rust/test files are below 300 lines.

The exact-source UI Send test now passes in one isolated macOS package window: the JSON body is
visible and the UI reports “Test message sent.” The loopback-only fake curl captured the HTTPS URL
and JSON request body; no external request was sent, and the disposable settings were not saved.
This accepts Workflow 13 and raises packaged UI acceptance to 13/25 (52%); shipped/default Rust
UI remains 0/25. The broader local all-target test run was stopped at 637 passed / 14 ignored and
is incomplete. Graphify's read-only integrity check reports 8,916 nodes, 14,081 edges, and no
dangling endpoints; the source module is present, but Graphify found no path from that module to
the integration test, so no source/test link is claimed. `ccc status` confirms CocoIndex has not
been initialized for this project; no index result is claimed.

The fixed source `9845db79af26` now builds and passes package-release smoke, package validation,
packaged help, ZIP integrity, and checksum verification for macOS arm64. Binary SHA-256 is
`0c4e1fa888cc18d13da40f81683032686829a79ea5411250a2b4ab59c8397a23`; archive SHA-256 is
`6bba2a52980147a01402949749789d6d49a615406a9399b5bdd9b449724b1570`. Idle Cargo incremental/temp
caches were removed before this build; the post-package target size is 9.6 GB (8.8 GB debug,
731 MB release). The CUA screenshot was inspected but not saved as a repository artifact.

## 2026-10-23 — Complete the current-head native-feature regression (#9)

The earlier broad test attempt at the same product source was stopped at 637 passes and 14 ignored; it was incomplete. The full rerun at docs head `12d6d808` completes with 938 passed, 14 ignored and no failures across 310 test binaries in 363.89 seconds. Warning-denied all-target Clippy, rustfmt, and diff checks also pass. This closes the source-level regression gate only: packaged UI acceptance stays 14/25, shipped/default Rust UI remains 0/25, and Windows/package/UI, accessibility/IME, resource, current-manual screenshot, and final-source license checks remain open. Evidence: `docs/evidence/issue9-full-regression-20261023.json`.

## 2026-10-23 — Review workspace and settings overlays (#9)

The existing v0.1.36 macOS package was inspected read-only in one window. The workspace menu does not shift the underlying page; MCP registration and workspace-sharing dialogs show no visible overlap at 1140×768 in Korean dark theme. No settings were saved and no workspace was switched. This does not add packaged workflow acceptance. AgentMemory: `mem_mvkd4i5n_771f4210cdac`.

## 2026-10-23 — Verify packaged native cache maintenance (#9 / workflow 14)

The exact macOS arm64 v0.1.36 package from `9845db79af26` ran in one isolated app window with a temporary HOME. A synthetic completed quota-query cache entry appeared in Settings → Preview and clear local cache as the only eligible entry; the protected/uncertain count was zero. Clearing the reviewed digest reported one removed entry. Filesystem checks confirmed the fixture was gone while the 938-byte `startup.json` remained. The temporary app and HOME were removed, and the user's existing package window was restored with its saved workspace intact.

`native_ui_cache` passes 7 tests, `cache_cleanup` 4, `public_docs` 11, and `source_layout` 2. The UI screenshot was inspected but not retained; this validates only the macOS package. CocoIndex is unavailable in the current runtime (`cocoindex` is not installed); no index result is claimed. Graphify diagnostic reports 8,916 nodes and 14,081 edges with no dangling endpoints. A narrow query finds `preview()`, `RequestWorker`, and the native worker fixture, but the module-to-test path remains indirect, so no direct edge is claimed. Evidence: `docs/evidence/issue9-9845db79-cache-cleanup-ui.json`.


## 2026-10-23 — Check current-source bilingual manual assets in the macOS package (#9)

The v0.1.36 package built from issue-9 source `575fef74` passed package validation with 242 dependency records. English/Korean HTML manuals and Agent Room PNG hashes match the current source assets. The generic CUA selector showed a saved workspace outside the isolated test root, so exact-package manual display was not accepted and no UI action was taken. Packaged UI acceptance stays at 15/25. Evidence: `docs/evidence/issue9-current-source-package-manual-20261023.json`; AgentMemory: `mem_mvksk77o_52dd00979543`. CocoIndex was unavailable; no CocoIndex result is claimed.

## 2026-10-23 — Confirm exact-package manual rendering (#9)

Retrying with Launch Services arguments opened the exact v0.1.36 package under an isolated `HOME` and workspace. The in-app User manual rendered its dark Korean Workspace screenshot and its English Agent Room screenshot of three completed tasks, passing tests, and pending human review. The first generic app-selector mismatch remains recorded as a discarded attempt. No task, approval, or merge occurred, and workflow acceptance remains 15/25. Evidence: `docs/evidence/issue9-current-source-package-manual-20261023.json`.

## 2026-10-23 — Port the Google Rust review policy to the migration branch (#169)

Ported the selected review policy and R-174 → DD-185 architecture trace from `main` into the Rust migration branch. The policy adds no lint gates or source cleanup; stable rustfmt, warning-denied Clippy, and the 300-line changed Rust-file guard remain authoritative. TDD contract `rust_review_policy` failed before the missing document was ported. The scoped Graphify query and final tests are recorded in the implementation plan; CocoIndex is not installed on this host.

## 2026-10-23 — Refresh issue #9 PR and release-version audit

TDD refreshed the PR #22/current-release audit after push `02cc8eb`: GitHub reported OPEN/DRAFT, base `main`, merge state `UNKNOWN`, and no checks; `origin/main` was `602ec118`, v0.1.37 was published, and the Rust branch remained v0.1.36. The changelog merge conflict is unresolved. Recheck the release and align to the next stable version before packaging. Focused status/docs/layout tests passed 14/14, formatting and diff checks passed. AgentMemory: `mem_mvkv768g_ad00e2a409b0`.
## [2026-10-23] verify | Copy exact #9 source archive to existing UTM Z: share

The user clarified that Windows already has `Z:` mapped and provided its host folder `/Users/neodamul/Downloads/UTM`. Copied the source archive for branch head `c4cbd2a6c13b5ab59f50b1efa37842a54a9c7a9c` into that folder; Finder shows the archive. UTM stayed started and input capture was restored off. The Windows restart-apps confirmation remains visible, so guest-side file access, build, package and UI remain unverified. No VM restart or shutdown was selected. Packaged UI acceptance remains 15/25, shipped/default Rust UI 0/25. Evidence: `docs/evidence/issue9-utm-z-share-transfer-20261023.json`.

## 2026-10-24 — Integrate latest main into issue #9 Rust branch (#169 / #172)

Merged current main `3bcfebf21908` into the issue-linked Rust branch as `4a53cf66516e`. Preserved #172 safety rationale and Windows quota/picker updates while retaining Rust-only native UI helpers and the 300-line file split. The initial compile failure for provider icon decoding and picker arguments was fixed. Local native-feature tests pass 959 with 14 ignored across 319 suites; focused journey/docs/layout 14/14, public docs/source layout 13/13, safety Clippy, rustfmt, and diff checks pass. Graphify: 9,201 nodes / 14,401 edges / 701 communities. CocoIndex unavailable in this environment. Current CI run 39999933920 is pending; no UI acceptance credit. See [current-main Rust parity](current-main-rust-parity.md) and evidence `docs/evidence/issue9-main-merge-20261024.json`.

## 2026-10-24 — Recheck UTM Windows guest channel after approved reset

Recorded that the VM reset completed and guest-agent file access partly recovered: one `C:\Windows\win.ini` pull succeeded, while the UTM display remains inactive and the guest `Z:` probe does not verify the interactive user's mapping. A later session query hung. No share change or second power action occurred. Plan, completion audit and evidence JSON are updated; 13 public-doc/source-layout tests pass. GitHub run `40000373810` continued on the previous documentation head while this follow-up was staged. See [UTM Windows guest access](utm-windows-guest-access.md) and `docs/evidence/issue9-utm-windows-recheck-20261024.json`.

## 2026-10-24 — Recheck UTM Windows guest channel after approved reset

Recorded that the VM reset completed and guest-agent file access partly recovered: one `C:\Windows\win.ini` pull succeeded, while the UTM display remains inactive and the guest `Z:` probe does not verify the interactive user's mapping. A later session query hung. No share change or second power action occurred. Plan, completion audit and evidence JSON are updated; 13 public-doc/source-layout tests pass. GitHub run `40000373810` completed only partially and was cancelled by the later checkpoint push. See [UTM Windows guest access](utm-windows-guest-access.md) and `docs/evidence/issue9-utm-windows-recheck-20261024.json`.

## 2026-10-24 — Build exact issue-head local macOS Rust package

Built and smoke-tested the current issue branch head `8ef672bd` using `cargo run --offline --locked --release -- package-release`. Package validation and ZIP integrity passed; all four shared/worktree × Git/folder smoke cases passed with three parallel tasks/tests, approved merge, and unapproved-merge refusal. The 18,858,304-byte binary SHA-256 is `95746fd5052c7c4fde4472ecce7afa7db2b01ec38a94c612bc9f15b4e0a78e4e`; the archive SHA-256 is `605ee5e9b65e26e41a9a2c5b9a9a4151fe41655bc5ee1d3cd5dc12e4e1bfa519`. Package includes the 242-component SBOM, dependency license bundle, notices, and bilingual manuals. Evidence: `docs/evidence/issue9-local-macos-package-8ef672bd-20261024.json`; see [native release packaging](native-release-packaging.md). No release or visual workflow credit is claimed.



## 2026-10-24 — Existing-package Settings gear spot-check

Bound the already-running `f23381cf` v0.1.37 app by full bundle path, avoiding a second process. Visual and accessibility inspection confirmed the gear-only `설정` button and Settings menu actions; Escape closed the menu. The Settings renderer and its regression test have no source diff from that package commit to the current branch. The existing app displayed saved workspace/profile data, so no isolation or workflow credit is claimed; nothing was changed, and a screenshot was not committed to avoid exposing workspace context. Evidence: `docs/evidence/issue9-settings-gear-existing-package-20261024.json`.


Graphify refreshed after this observation: 9,223 nodes, 14,423 edges, 697 communities; 369 code files extracted and 380 unclassified paths skipped. CocoIndex CLI is unavailable.


Read-only package screen review covered Agent Roles and Plan Review. Six roles and primary/fallback model controls were visible; Plan Review reported no reviewed plan with save/run disabled. Returned to Workspace; no plan execution or data mutation. No isolated acceptance credit.


Focused current-source regression tests for Settings appearance, Agent Roles, Plan Review, workspaces, and planning stop passed 23/23 across six suites without launching a desktop process. This strengthens component verification but adds no packaged UI acceptance.

## 2026-10-24 — Refresh 0.1.37 public Workspace documentation captures

Replaced the stale README screenshot and refreshed the bilingual manuals' light/dark Workspace captures using the current-source Rust UI probe in fresh temporary homes. Set README and Markdown manual version labels to the latest published 0.1.37; regenerated both offline HTML manuals through the Rust renderer. Focused docs/manual/screenshot tests pass 14/14. The update lookup is the public GitHub release metadata GET; no provider CLI/MCP or user profile data was used. Agent Room screenshots, Windows UI and packaged UI acceptance were not checked or credited. See `native-screenshot-provenance.md` and `docs/evidence/issue9-current-workspace-docs-20261024.json`.

## 2026-10-24 — Revalidate #9 current-head platform journey

The previously failing macOS Step 3 test passes locally on the current source; the current Actions run 40003966004 also passes the entire macOS job. Do not make a speculative fix based on the older failure. The Windows job passed build and layout and is still in native tests. Local formatting and public-doc/source-layout checks pass. UTM remains started with IP 192.168.64.6, but guest process launch fails with OSStatus -2700; no power/share change was made. Acceptance stays 15/25 packaged and 0/25 shipped/default Rust UI. Evidence: `docs/evidence/issue9-current-head-platform-recheck-20261024.json`.

## 2026-10-24 — Exact current-source macOS package and UI inspection

Built and validated local v0.1.37 macOS arm64 package from commit 08a1783. Package smoke passes four shared/worktree × Git/folder cases with three concurrent tasks/tests; approved merges verify and unapproved merges are refused. Exact package UI was inspected once under an isolated HOME; only read-only navigation was used. Package includes current bilingual docs/screenshots, 242 dependency license records and SBOM components. Removed older generated dist packages, retaining the current candidate. This is not a release or new workflow acceptance. Windows remains open. See `native-release-packaging.md` and `docs/evidence/issue9-local-macos-package-ui-08a1783-20261024.json`.

## 2026-10-24 — Windows native test step completed on current CI

Actions run 40003966004 has advanced: Windows build, source/test layout and all-target native tests passed; opt-in native UI launch is active, formatting/lint/safety are pending. macOS full validation remains passed. Do not claim cross-platform validation until the active Windows job completes. This is remote test evidence only; UTM guest execution and `Z:` remain unverified.

## 2026-10-24 — Approved UTM Windows reset follow-up

User approved the warned VM reset. Force-stopped and restarted only the confirmed `Windows` VM; it returned to `started` with IP `192.168.64.6`. Guest execution still fails with OSStatus `-2700`, and UTM says “Display output is not active.” The reset did not restore the guest agent; do not repeat power/share changes without a new reason. Actions 40003966004 had passed Windows all-target native tests, with UI launch still running at observation. See the issue #9 Windows progress evidence.

## 2026-10-24 — Windows UI and formatting checks passed

Actions run 40003966004 passed Windows opt-in native UI launch and formatting, in addition to build/layout and all-target tests, then failed warning-denied Clippy because `tests/windows_console.rs` re-exported unused `language`; the unsafe-comment gate was skipped. Removed that unused export. Local all-target Clippy and formatting pass; the full local test suite is running. UTM reset result is unchanged: VM started at 192.168.64.6 but guest process execution still fails with OSStatus -2700 and display output is inactive.

## 2026-10-24 — Exact pushed-head package and CI rerun

Built and validated the exact pushed head `278ad6d` as local v0.1.37 macOS arm64. Manifest and ZIP integrity pass; four embedded shared/worktree × Git/folder smoke cases pass. Retained the newest candidate and removed the superseded package/smoke run. Platform Validation run 40008320185 passed all macOS checks and Windows build/layout/all-target tests; the Windows native UI launch step is now in progress, with later checks pending. No new packaged UI acceptance. See `docs/evidence/issue9-local-macos-package-278ad6de-20261024.json`.

## 2026-10-24 — workflow 19 session/model package verification

Exact v0.1.37 macOS package at source `6ca19965` displayed `fixture-session-19`, exposed only two fixture-observed models, saved `fixture-large`, and persisted it through the native Rust action. A deterministic local JSONL fixture intentionally failed; model editing invalidated its evidence and the run was stopped. Four focused task-control tests, package validation, and ZIP integrity pass. Packaged acceptance is 19/25; shipped/default Rust UI remains 0/25. No provider CLI, MCP, or external service ran. Evidence: `docs/evidence/issue9-workflow19-session-model-20261024.json`.

## 2026-10-24 — workflow 21 task activity, raw logs, and lessons

Exact v0.1.37 macOS arm64 package `6ca19965` displayed normalized Activity, separate JSONL Raw logs, and task-specific Lessons from deterministic mock state. The run stayed at `waiting_approval`; no decision or merge was made. Focused tests passed 9/9. Packaged acceptance is 20/25; shipped/default Rust UI remains 0/25. The package window and temporary fixture were closed/removed. Windows display and package runtime remain unverified. See `docs/evidence/issue9-workflow21-task-activity-20261024.json`.

## 2026-10-24 — native manual AccessKit text regression

Added a source test proving the manual's headings, prose, command-table cells, code and screenshot alternative text are represented in AccessKit updates. Text uses `TextRun.value`, not `node.label`; no product-source fix was warranted. Exact-package visual inspection showed current tables and Workspace/Agent Room illustrations, but VoiceOver remains unverified because the interactive CUA tree omitted text runs. The check adds no packaged acceptance. See `docs/evidence/issue9-manual-accesskit-text-20261024.json`.

## 2026-10-24 — workflow 20 package UI attempt stopped safely

A disposable package CLI run reached `waiting_availability` using only a local Rust mock, and `native_ui_task_scheduling` passed 3/3. The CUA binding by display name showed an existing Workspace with saved profiles instead of the requested Agent Room, so no schedule or other control was touched. The run and temporary tree were removed. Automatic quota queries may have occurred because the visible Workspace had refresh timestamps; this is not verified. No packaged workflow acceptance is claimed. A follow-up unique-bundle launch aborted with signal 6 before CUA exposed a window; its disposable run/root were stopped and removed. Resolve reliable launch/window binding before another UI attempt. See `docs/evidence/issue9-workflow20-package-ui-attempt-20261024.json`.


## 2026-10-24 — Windows waiting-task schedule regression

Actions run `40098597709` passed macOS validation but found that the Windows UI request guard rejected schedule actions while the top-level workflow remained running. UI Automation successfully set the timestamp and invoked the enabled button; the app did not persist it. Added a regression reproducing the rejection, then allowed schedule actions through the active-job guard while retaining waiting-task and timezone checks. Focused schedule suites, source layout, rustfmt, and Clippy pass locally. Exact Windows package rerun remains required. Graphify refreshed to 9,404 nodes / 15,859 edges; CocoIndex CLI is unavailable. Evidence: `docs/evidence/issue9-windows-schedule-regression-20261024.json`. UTM remains at Windows sign-in; no guest reset/restart was performed.

## 2026-10-25 — Windows schedule driver marker contract

A focused source-level regression test exposed a guaranteed false failure in `tests/windows_schedule.rs`: the UI script emitted `SCHEDULE_ACTION_INPUT_SENT`, while the Rust driver accepted only `SCHEDULE_CLICK_SENT`. The new seven-line test failed against the mismatched driver and passed after aligning the success check. Focused marker, schedule, briefing, native scheduling interaction, and source-layout suites pass (5 suites); formatting passes; the schedule test remains exactly 300 lines. Actions run `40160742328` targets the pre-fix commit and is not Windows package acceptance for this correction; exact-head run `40161383352` is now in progress on `07b988cd3cdc2b285dc3d49e993f68ec576745ab`. Keep the package gate and issue #9 acceptance count pending until an exact-head Windows package test persists `manual_schedule` and displays the briefing. CocoIndex CLI is not installed in this environment (`rtk which cocoindex` returned no path).

## 2026-10-25 — Exact-head Windows validation progress

Run `40161383352` on `07b988cd3cdc2b285dc3d49e993f68ec576745ab` passed every macOS validation step, including all-target tests, native UI checks, formatting, Clippy, and the unsafe-block gate. The Windows job remains in `cargo build --all-targets`; Windows tests and package acceptance have not started. Keep issue #9's package count unchanged.

## 2026-10-25 — Windows desktop goal input dropped the final character

Exact-head run `40161383352` on `07b988cd3cdc2b285dc3d49e993f68ec576745ab` passed macOS validation, Windows build/layout, `windows_briefing`, and `windows_schedule` (2 tests). Windows `windows_desktop` failed because `SendKeys` persisted `Windows UIA go` instead of `Windows UIA goal`; the Windows package job was skipped. Added a source regression test that failed while the driver used SendKeys, then replaced it with UIA `ValuePattern.SetValue` and a bounded read-back check before Save. The regression, Windows desktop harness compile, and source-layout suites pass locally (3 suites); rustfmt passes. Exact-head Windows runtime/package retest remains required; package acceptance remains 21/25. Graphify: 9,433 nodes / 15,895 edges. CocoIndex CLI is unavailable.
