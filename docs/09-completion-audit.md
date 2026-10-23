# Full-product completion audit

## Current checkpoint — issue #9 / 2026-10-23

Packaged native UI operator acceptance is **12/25 (48%)**; shipped/default Rust replacement remains **0/25**. The latest product-code commit is `9845db79af260b631bde7ade2104bae2400cc649`. Its Messenger first-use JSON-template fix passes focused regression, docs/layout, warning-denied Clippy, formatting, and diff checks; the broad all-target run was stopped at 637 passed / 14 ignored and is incomplete. The exact macOS arm64 package at `9845db79` passes `package-release` smoke, `validate-package`, packaged `--help`, ZIP integrity, and checksum validation. Its binary is 18,606,512 bytes (SHA-256 `0c4e1fa888cc18d13da40f81683032686829a79ea5411250a2b4ab59c8397a23`); archive SHA-256 is `6bba2a52980147a01402949749789d6d49a615406a9399b5bdd9b449724b1570`. The built-in deterministic package smoke does not exercise the Messenger UI; exact-package loopback transport verification remains open, so packaged UI acceptance stays 12/25. Earlier package evidence for quota fallback, review, human approval, and merge is `docs/evidence/issue9-ca9495b2-quota-fallback-ui.json`. The bundle still contains historical manual screenshots.

Windows remains unverified. The host has no Windows Rust `core`/`std` target. A hidden disposable UTM Windows boot reached `started`, but its QEMU guest agent was unavailable, so IP discovery and `utmctl exec` failed; the VM was stopped and the probe-created window closed. Keep #9 open and PR #22 in draft until current screenshots, Windows package/UI, accessibility/IME, resource measurements, and final-source license review are complete. No release is authorized by this package checkpoint.

The earlier single-window, read-only inspection used the older `cff95dd` bundle; it is historical and does not establish exact-package workflow acceptance. The exact `ca9495b2` package UI journey is recorded above. Before rebuilding from `9845db79`, the Codex worktree `target/` measured 11 GB; idle `debug/incremental` and `debug/tmp` were removed, then the exact-source release build and package were produced. The latest measured target size is 9.6 GB: 8.8 GB debug dependencies/build outputs, 731 MB release artifacts, and 184 KB for the unavailable Windows target. The packaged 21 MB archive lives in `dist/` and is not part of `target/`.

### Previously verified v0.1.36 package visuals and acceptance details

The following source and UI observations are historical component evidence from `f1602e7d`/v0.1.36. They do not describe the latest packaged binary or change the current 11/25 acceptance count.

Packaged native UI operator acceptance is **11/25 (44%)**; the shipped Rust replacement remains **0/25**. Workflow 10, coordinator-directed operator feedback, was verified in the exact v0.1.36 macOS package at `b298ae79774e`. Workflow 11 switches the same package's Agent Room from Korean to English and back while preserving the pending candidate, task statuses, and test evidence; evidence is `docs/evidence/issue9-6f1111c-language-status-workflow.json`. The package archive SHA-256 is `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`. The current Rust source `f1602e7d` passes 929 all-target tests (14 ignored across 310 suites), warning-denied Clippy, rustfmt, diff check, and the 300-line source/test guard. Its exact macOS arm64 package validates and passes package smoke; manifest lists 214 dependencies, bundled SQLite, and Git plus selected authenticated agent CLIs as runtime requirements. The package was opened in one window and visually checked: the goal appears once in the wide Agent Room header, while the left column retains task status and verification. The same package's Korean Agent Room was also inspected in light theme: three task rows, candidate SHA, and three passing test results were visible; candidate `3db8840b` remained `waiting_approval`, with no approval, rejection, or merge. Dark theme and the saved user workspace were restored without changing workspace settings. Evidence: `docs/evidence/issue9-f1602e7-agent-room-light-theme.json`. Graphify's retained curated graph has 8,916 nodes, 14,081 edges, and zero dangling endpoints; the no-cluster extraction was not retained because it had 1,271 dangling edges. CocoIndex CLI is unavailable. These component checks do not add workflows. Fourteen packaged UI workflows remain. Windows UI, accessibility/IME, resource measurements, and final-source license review remain open. Keep issue #9 open and PR #22 in draft.

The native MCP registration dialog now follows the existing Settings → MCP servers eyebrow + registration-title hierarchy, with distinct English and Korean window titles. The exact v0.1.36 macOS package from `cff95dd02677519ebbb7c359f5ca9e3da0c504ed` was visually checked in Korean/dark, English/dark, and English/light; archive SHA-256 is `dafc056361c6e68a1165f30db1f3bd8a0cf99d24b59876d7d6eb7d2aa1ad44d8`. The saved workspace was left unchanged, and the candidate was not approved or merged. Evidence: `docs/evidence/issue9-cff95dd-mcp-dialog.json`. The screenshot was not retained. This component correction adds no workflow; acceptance remains 11/25. Windows verification remains open.
Packaged native UI operator acceptance is **11/25 (44%)**; the shipped Rust replacement remains **0/25**. Workflow 10, coordinator-directed operator feedback, was verified in the exact v0.1.36 macOS package at `b298ae79774e`. Workflow 11 switches the same package's Agent Room from Korean to English and back while preserving the pending candidate, task statuses, and test evidence; evidence is `docs/evidence/issue9-6f1111c-language-status-workflow.json`. The package archive SHA-256 is `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`. The current Rust source `f1602e7d` passes 929 all-target tests (14 ignored across 310 suites), warning-denied Clippy, rustfmt, diff check, and the 300-line source/test guard. Its exact macOS arm64 package validates and passes package smoke; manifest lists 214 dependencies, bundled SQLite, and Git plus selected authenticated agent CLIs as runtime requirements. The package was opened in one window and visually checked: the goal appears once in the wide Agent Room header, while the left column retains task status and verification. The same package's Korean Agent Room was also inspected in light theme: three task rows, candidate SHA, and three passing test results were visible; candidate `3db8840b` remained `waiting_approval`, with no approval, rejection, or merge. Dark theme and the saved user workspace were restored without changing workspace settings. Evidence: `docs/evidence/issue9-f1602e7-agent-room-light-theme.json`. Graphify's retained curated graph has 8,916 nodes, 14,081 edges, and zero dangling endpoints; the no-cluster extraction was not retained because it had 1,271 dangling edges. CocoIndex CLI is unavailable. README image embeds were removed after confirming the assets showed v0.1.21/v0.1.24 or a prepared demo. Release packaging now discovers local image assets directly from both manuals instead of relying on README links; the retained manual screenshots are still historical and remain due for exact-package refresh. These component/documentation checks do not add workflows. Fourteen packaged UI workflows remain. Windows UI, accessibility/IME, resource measurements, and final-source license review remain open. Keep issue #9 open and PR #22 in draft.

## Exact v0.1.36 post-approval drift refusal — issue #9 / 2026-10-23

The exact macOS package from `aa8090405239` approved the exact candidate in a disposable copy
of the pending mock run. A matching baseline charter existed only in that copy; changing the
stored plan goal then caused the packaged merge command to exit 1 with `[goal_drift]`. The
package persisted one coordinator judgment and did not change the fixture repository HEAD. In
the single Agent Room window, the localized Korean blocked judgment appeared with the completed
3/3 task and test evidence, and Merge was disabled. The original app-managed candidate remained
pending approval with zero drift records and was restored; the temporary run copy was removed.
No provider CLI or external service was invoked. This is a distinct post-approval failure
workflow, advancing packaged UI acceptance from 8/25 to **9/25 (36%)**. Windows package/UI,
accessibility, Korean IME, resource measurements, and final cross-platform release gates remain
open. Evidence: `docs/evidence/issue9-aa80904-goal-drift-workflow.json`.


## v0.1.36 CI package-license audit — issue #9 / 2026-10-22

The paired v0.1.36 macOS and Windows packages from exact PR-validation run `39740270740` both record GitHub merge source `eff9f44858ada378ac230a71dbf040f4c47455bc` (PR head `cd9935de`, base `3fecd21d`). Their manifest dependency counts match the SBOMs: 214 macOS components and 201 Windows components, with no missing license expressions. Each archive checksum and executable SHA-256 matches its embedded checksum file; both packages include dependency notices and license texts. The sole GPL-optional expression is `self_cell 1.3.0`'s `Apache-2.0 OR GPL-2.0-only`; both license texts are bundled, and the permissive Apache-2.0 option is available. Evidence: `docs/evidence/issue9-eff9f448-license-audit.json`.

This package review does not replace legal review or close issue #9. Repeat dependency-closure and license review against the final release source after issue #9 acceptance; the Windows executable was hash-checked but not run on Windows in this audit.

## Full native-feature regression after merging #135 — issue #9 / 2026-10-22

At code commit `d4338e5b` (the later `21f5e6b` changes only the plan checklist),
`cargo test --locked --all-targets --features native-ui-probe` passes **866 tests, 14 ignored,
0 failed across 295 suites** (228.37 seconds). The additional focused repair, public-doc, and
source-layout batch passes 18 tests; repair Clippy with warnings denied, formatting, diff checks,
and the 300-line source/test guard pass. Graphify reports 8,408 nodes / 13,377 edges / 630
communities. CocoIndex is unavailable because its command is not installed. The exact-head
Platform Validation run `39722700547` was cancelled after a documentation-only follow-up push;
there is no current-head platform result. This is source-level verification only and adds no
packaged workflow acceptance: issue #9 remains 8/25, and the Rust UI remains unshipped.
After testing, `cargo clean` removed 494,075 generated files totaling 80.2 GiB from the Codex
worktree. The 750 MB `dist` package directory was preserved.

## Exact v0.1.34 packaged native plan-to-merge UI revalidation — issue #9 / 2026-10-22

The exact `aarch64-apple-darwin` v0.1.34 package from source `76d8082b02a722ab18cc4e2f0e8383b3c1a3fdd5`
was opened in one AgentConductor window. Its manifest records an 18,356,880-byte executable,
214 dependency notices, and binary SHA-256
`6076725213cde19f24b29cc5111da9ac6e885cb70c78e377a122376e290c0e05`; the archive SHA-256 is
`8b33b1378b4e3e145b90518609ecc1b70a9c5a67440e53c44d3df098dfa5ccab`.

In a disposable Git project, the UI reviewed a manual three-task plan using worktree isolation.
Three deterministic custom CLI processes completed in separate worktrees; each owned only its
fixture file, and all three test commands exited 0. The UI loaded the exact candidate diff, showed
all test output, required candidate approval and merge confirmation, then reached Completed.
The final disposable project's `main` points to candidate `2c071b8fbc887f139f0804206c417023a653a312`
and has no uncommitted changes. No provider CLI, external service, or product-repository path was
used. Evidence: `docs/evidence/issue9-76d8082-macos-package-ui-workflow.json`.

This repeats the already accepted packaged plan-to-merge journey patterns and adds no distinct
workflow: acceptance remains **8/25**. Windows GUI, authenticated provider workflows, full
accessibility parity, Korean IME, and idle-resource measurements remain unverified. During fixture
setup, one direct-plan editor draft showed three summary tasks but two editable rows; adding the
missing row restored three. A newly generated manual plan then showed all three. Reproduce this UI
observation before treating it as a confirmed product defect.

## Native scope-recovery button interaction evidence — issue #9 / 2026-10-22

Added a focused rendered interaction test for the recovery dialog's Preview changes and Approve and
apply buttons. It dispatches pointer movement before press and release, matching egui's hit-testing
sequence, then checks the exact task, owned paths, test argv, and reviewed digest in the emitted
events. The first test draft omitted the hover frame and failed to activate the button; correcting
the test harness made both button paths pass. `native_scope_recovery_actions` passes (1), existing
`native_scope_recovery` passes (2), with source-layout and public-doc checks passing. This closes a
component interaction-test gap only: no packaged end-to-end workflow is added; acceptance remains
8/25 and exact-head Windows/macOS package validation is still open.

## Previous local native-feature suite — issue #9 / 2026-10-22

At branch head `696b2f8` (product code `214e359` plus documentation-only commits),
`cargo test --locked --all-targets --features native-ui-probe` passed 845 tests, 14 ignored,
0 failed across 289 suites in 224.71 seconds. This covers the stabilized Settings dismissal test
locally but does not provide Windows runtime/package evidence or increase packaged UI acceptance
beyond 8/25. Graphify's local no-cluster code refresh reports 8,170 nodes and 14,029 edges.
CocoIndex Code remains unavailable: its installed launcher fails to import
`mcp.server.fastmcp`; no CocoIndex results are claimed.

## Current native-feature suite — issue #9 / 2026-10-22

At issue-9 branch head `630b73d` plus the current scope-recovery interaction-test unit,
`cargo test --locked --all-targets --features native-ui-probe` passes **855 tests, 14 ignored,
0 failed across 292 suites** (223.63 seconds). The new rendered pointer test activates both preview
and approval buttons and checks the dispatched task, paths, test argv, and digest; it is listed in
the macOS/Windows platform validation and Clippy commands. Focused source-layout (2) and public-doc
(11) tests, formatting, warning-denied all-target Clippy, and the 300-line guard pass. Graphify's
local no-cluster code refresh reports **8,301 nodes and 14,302 edges**. CocoIndex remains unavailable
in this runtime and was not installed. This is component evidence only: packaged workflow
acceptance remains 8/25; exact-head macOS/Windows package and complete native workflow gates remain
open.

## Native UI test feature-gate repair — issue #9 / 2026-10-22

Actions run `39634545852` failed on macOS and Windows because the newly added
`tests/native_room_briefing_layout.rs` integration test was auto-discovered without requiring
`native-ui-probe`. The test now has a crate-level feature gate. The no-default-feature target audit,
focused Team Briefing popup test, source layout, public-doc tests, formatting, and diff checks pass
locally. The workflow-equivalent all-target run completed; one unrelated `cli_workflow` case failed
once with a `DirectoryNotEmpty` cleanup race and passed on isolated rerun. All other targets passed.
PR merge-result run `39687975008` passed Windows but failed macOS in
`settings_dialogs_remain_accessible_across_sequential_open_close`: the Cache cleanup dialog was
still visible after the one-frame close check. Commit `214e359` replaced that fragile check with
bounded dismissal polling; the focused journey test passes on the current local head. The failing
run predates that fix, and no exact-current-head macOS/Windows result is available. No issue #9
workflow acceptance changes.

## Exact-head native UI package and menu review — issue #9 / 2026-10-22

Actions run `39613489337` passed all four jobs at `dbf893a18c4d73bc611c984fcc031684346ef67a`:
Windows/macOS platform validation and Windows/macOS native-package builds, manual generation,
package validation, and execution. In the exact-source v0.1.33 macOS app at
`/private/tmp/AgentConductor-1f75758.app`, the Workspace menu, Settings menu, and coordinator
selection popup were opened and visually confirmed to overlay the page without shifting its panels.
The earlier screenshot showing a vertical shift came from the stale 16:29 package under `dist/`;
the current packaged executable was built at 22:06 after the popup implementation. Focused local
layout verification passed 14 tests across five suites; formatting and diff checks pass. No source
change or new packaged workflow acceptance is claimed. Workflow acceptance remains 8/25 (32%), and
the stable shipped UI remains WebView.

The follow-up audit also opened a populated Team Briefing disclosure in the Rust room view. The
session details paint in the popup and the following Stop button remains at the same y-coordinate.
`tests/native_room_briefing_layout.rs` now guards this path; the focused popup audit passes 21 tests
across six suites, the layout/doc batch passes 19 tests across five suites, and warning-denied
Clippy passes for the new test. This is component evidence only; packaged workflow acceptance remains 8/25. Graphify refreshed locally to 8,136 nodes
and 13,973 edges. CocoIndex has no available CLI or MCP tool in this runtime, so no CocoIndex result
is claimed.

The follow-up menu audit also covers the Profile Rules JSON editor and task-detail editor: opening
either popup leaves the Save action or next task at its original y-coordinate. Both regressions pass
locally and are registered in the macOS/Windows native UI validation commands. They confirm popup
behavior only; no complete packaged workflow is added to the 8/25 acceptance count.

## Current package, exact-window review, and disk cleanup — issue #9 / 2026-10-21

The exact source at `de032e016323dda18ca369a08ab3207f38118f82` built as a macOS release package.
Package validation and the built-in four-case deterministic smoke passed. Manifest SHA-256 is
`36dd3e934b75fe278307815e9555ab78f72f65a01e270bc3cdf67ce7b6662695` with 214 dependency records.
The exact bundle was selected by full path and inspected in one isolated window: Workspace, Agent
Roles, and Review empty states rendered; no setting was saved, and this spot check does not add a
packaged workflow. The test window and isolated HOME/workspace were closed and removed.

The worktree contained 61 GB of Rust build output. With no Cargo/Rust compiler process active, only
the reproducible `target/debug/deps` and `target/debug/incremental` caches were removed. Existing
executables, source, release dependencies, and package artifacts were preserved; `target/` is now
2.2 GB. Issue #9 now reports 8/25 packaged UI workflows (17 remaining). Actions run `39543488348` passed macOS validation, Windows build/layout, and applicable Windows tests,
then failed only `public_docs`: a multiline assertion assumed LF while the Windows checkout used CRLF.
The assertion now normalizes CRLF before matching; exact-head Windows rerun is pending. No new workflow
acceptance is claimed. AgentMemory checkpoint: `mem_mvhmis60_5fa5b73d16bd`.

## OpenCode MCP server registration and discovery — issue #9 / 2026-10-21

The exact v0.1.32 macOS package registered a local server through Settings → MCP servers using
the OpenCode JSON/JSONC shape, saved it, reopened the editor, and discovered `get_issue` from a
deterministic Rust stdio fixture. Its request trace contained initialization and `tools/list`
only; no tool call, provider CLI, external service, or user workspace was involved. The isolated
HOME was removed. Evidence: `docs/evidence/issue9-256db1a-mcp-server-registration-ui.json`.
This is distinct from importing Jira/Confluence records, so acceptance advances to
**8/25 (32%)**, with 17 workflows open. Actions run `39541731070` exposed a second Windows-only
picker compile error; the intent is now passed into the dialog and a new exact-head run is pending.

## Windows folder-picker compile regression — issue #9 / 2026-10-21

Actions run `39541153826` exposed a Windows-only compile failure: the localized picker change
removed the `executable` parameter but the Windows file filter still used it. Actions run
`39541731070` then found the restored value was not passed from `pick` into the inner `dialog`.
Run `39542757398` compiled that production path successfully, then found `tests/windows_console.rs`
did not re-export `ui::language` into its isolated test shim. The shim now exports the existing
module. The Windows CI build remains the platform compile regression gate; local macOS tests do
not compile this `cfg(windows)` test. Focused local tests and macOS target check pass; rerun is pending.

## UC-79 recoverable workspace archive recheck — issue #9 / 2026-10-21

The exact v0.1.32 package from source `256db1a` was tested under a disposable HOME. It archived
an incomplete workspace only after switching to another workspace, displayed the exact saved
path and project-preservation warning, removed the entry from the active registry, and wrote a
recovery record under `workspace-trash`. The disposable project folder remained present. No
provider, external service, or user workspace was involved; the package process was closed.
Evidence: `docs/evidence/issue9-256db1a-uc79-archive-ui.json`. This revalidates Workflow 2 and
does not change acceptance from **7/25 (28%)**. Actions run `39541153826` was in progress for
the source checkpoint at the time of this validation.

## Workspace settings export/import package UI — issue #9 / 2026-10-21

The exact macOS package built from source `256db1a5b536650edf4bbe791f573cac716638ff`
passed package validation and deterministic release smoke. In an isolated HOME, the UI exported
`agentconductor.workspace.json`, previewed the import, required explicit “Import and replace
settings” confirmation, and restored the exported workspace name. No provider CLI or external
service ran and the user's workspace was untouched. Evidence:
`docs/evidence/issue9-256db1a-settings-share-ui.json`. This is a distinct packaged workflow, so
acceptance advances to **7/25 (28%)**, with 18 workflows open. No screenshot was retained. The
focused title/share tests, formatting, macOS-target compilation, and diff checks pass; exact-head
Actions run `39540102836` remains pending.

## Workspace settings picker prompts — issue #9 / R-52 / UC-42 / 2026-10-21

The Settings → Workspace settings export and import actions now pass distinct localized purposes
through the existing async picker worker. The regression covers project/export/import titles in
English and export/import titles in Korean. `native_folder_picker_purpose`, `native_ui_probe`, and
`workspace_share` pass 15 tests across three suites; formatting, a macOS-target `cargo check`, and
`git diff --check` pass. A whole-repository all-target test run reported 30 tests across four suites
before stalling and was interrupted, so it is incomplete. Windows cross-compilation and exact-package UI
round-trip remain unverified. The UI attach selected the installed app with the user's saved
workspace; it was quit without interacting with or saving that workspace. Only the two disposable
test directories created for this check were removed. Packaged workflow acceptance remains
**6/25 (24%)**; this component fix does not add an accepted workflow.
Commit `7c668a0` is pushed to the issue branch; issue #9 comment `6307473584` and AgentMemory
`mem_mvhkfr8u_67de9db17f0b` preserve the checkpoint.

## Exact PR-package revalidation and cross-platform packages — issue #9 / 2026-10-21

Actions run `39533269033` passed PR-head validation and release-package jobs for head
`d799ad6fe4199516c6344f9f63676eb2708d9157` and synthetic merge result
`d386e6a3cdd0bb3f7484b0578d61ebdb867b67d0`. macOS and Windows packages built, rendered both
offline manuals, executed, and were retained. Artifact `12317625213` is the macOS package
(21,193,358 bytes; artifact digest `ed43e0e0a37f71e00ddd91337b88a2f69f7479f88d2090f240f4bd709220c522`);
artifact `12318045860` is Windows (15,422,921 bytes; digest
`05a89173b6fc69655d34d42b46a722adbac31e47648708b140b42321888cd91a`). Per-platform executable
and archive checksums are in `docs/evidence/issue9-d386e6a-pr-merge-cross-platform-packages.json`.

The exact merge-result macOS app revalidated the previously accepted shared-folder plan-to-merge
flow: three independent Rust fixture tasks wrote disjoint files, all tests passed, the operator
inspected the exact diff and approved the candidate, and the app merged it and displayed Completed.
The disposable Git repository ended at candidate `517e82d46cfe047589fd0164f5d42f6c1933549a` with
only the three expected files. No real provider CLI, API, external service, or user workspace was
involved. This repeats Workflow 1 and does not add a distinct acceptance workflow. Evidence:
`docs/evidence/issue9-d386e6a-native-ui-plan-to-merge.json`.

Packaged native UI workflow acceptance remains **6/25 (24%)**, with 19 workflows open. The Rust UI
has not shipped publicly. Screenshots, Windows GUI execution, Korean IME/accessibility, and full
visual parity remain unverified.

## Pull-request merge-result macOS and Windows packages — issue #9 / 2026-10-21

Actions run `39524574228` completed successfully for PR head
`806bf3204e63fa829b13d8e4bf09168da6b0e0c1`, checking out and building its synthetic merge result
`9e3617545e6f0c6ad8357bda7a15c541302ce8dc` (the uploaded artifact names carry this merge SHA).
Both platform-validation jobs passed native builds, source/test layout enforcement, applicable
tests, the native UI launch path, formatting, and warning-denied Clippy. Both package jobs passed
optimized release builds, Rust-rendered English and Korean manuals, exact package execution, and
artifact retention. macOS artifact `12312859028` is 21,190,192 bytes with artifact digest
`7c66e34aa06e7ccf441f0a7fdd9a41ab734ba87b9b53b22f300376df027c47ed`; Windows artifact
`12313174827` is 15,417,423 bytes with digest
`e650695d4dfcd56adad9778ca5b79ce4b922b4d889de16b9436ef0358e6c49e3`. Evidence:
`docs/evidence/issue9-9e36175-pr-merge-cross-platform-packages.json`.

This verifies packaged CLI orchestration on both platforms, not another native UI journey. Verified
packaged UI acceptance remains **6/25 (24%)**, 19 workflows remain, and the public release still
does not ship the Rust UI. Exact packaged GUI parity and the remaining user journeys are open.
AgentMemory records: `mem_mvhg8qd7_f7ff48237803`; lesson `lsn_215b3738fa9a9cdd`. Graphify
re-extraction: 7,743 nodes / 13,278 edges. CocoIndex Code could not start because its environment
lacks `mcp.server.fastmcp`.

## Exact-package quota fallback and platform validation — issue #9 / 2026-10-21

The exact v0.1.32 macOS release binary from product source `b4ad2f5` was exercised against a
disposable Git repository and deterministic Rust CLI fixtures. Three independent tasks each
received a quota-limit event with a reset time, transferred to the configured company fallback on
attempt 2, and passed their tests. The run reached `waiting_approval`; an unapproved merge was
refused, approval of the exact candidate allowed merge, and the packaged CLI then reported
`completed` with the three expected files and a clean repository. No external provider or service
ran. Evidence: `docs/evidence/issue9-b4ad2f5-quota-fallback-cli.json`.

Exact-head Actions run `39520326287` passes both macOS and Windows platform-validation jobs,
including native build, source-layout enforcement, applicable tests, native UI launch-path,
formatting and warning-denied Clippy. The macOS native-package job also passed the release build,
bilingual manual rendering, package execution and artifact upload; artifact `12312373026` is
retained (21,189,925 bytes; SHA-256 `c5f8dce7a06e04ff2f59f7208b8fbaf1b4e094e6e2aa94d5937791c72ae61624`).
The Windows native-package job also passed release build, bilingual manual rendering, package
execution and artifact upload; artifact `12311274352` is retained (15,417,457 bytes; SHA-256
`8415762398af24ebf21d1830a4027fcff6d6b11647292235eae12b83456ab363`). Both validation and both
package jobs are green. Evidence: `docs/evidence/issue9-b714887-cross-platform-packages.json`.
Local scheduling/start and fallback/availability/profile regression batches pass (4 tests and
21 tests respectively; the overlapping scheduling suite is not double-counted). This is packaged
CLI and component evidence, not a new native UI journey: acceptance remains **6/25 (24%)**, 19
workflows remain, and the public Rust UI is not yet shipped. AgentMemory records:
`mem_mvhe2xb3_6ad4618a9535`, `mem_mvhe77xu_5497db2b04a4`.

## Latest local verification — issue #9 / 2026-10-21

At branch head `7a6256d`, the complete local `rtk cargo test --all-targets --no-fail-fast`
run passes **805 tests, 14 ignored, 0 failed across 272 suites**; Cargo reports 216.14 seconds
of test execution. All-target Clippy with `-D warnings`, `cargo fmt --all -- --check`, and
`git diff --check` pass. The earlier macOS start-journey timeout was not reproduced in this
complete run. Exact-head GitHub Actions run `39517629909` remains in progress; its predecessor
`39517056024` was cancelled after a later push. Packaged UI workflow acceptance remains **6/25
(24%)**; this verification does not count as another workflow or as cross-platform UI acceptance.

## Current UI parity checkpoint — 2026-10-21 / macOS navigation CI triage

Exact-head Actions run `39507118252` is complete: Windows passed and macOS failed only
`review_approval_shows_start_feedback_then_opens_the_started_run`, timing out while waiting for
the enabled `Open Agent Room` button after returning from a completed run to Workspace. The test
passes ten consecutive local executions on the current source; a focused run also passes. This
does not establish the failure as harmless or fixed. The UI wait now reports accessible button
states and labels on timeout so the next exact-head result identifies the last visible surface.
Formatting, diff checks, focused native UI tests, and warning-denied Clippy pass. This is test
diagnostics only; packaged acceptance remains **6/25 (24%)**, with 19 workflows open.

An optimized exact-source macOS package was built from `0935f481fba1b89b329bca604ebb1c0131035992`.
`package-release`, manifest validation, packaged `--help`, ZIP integrity, and the packaged smoke
passed. The binary is 18,008,720 bytes (SHA-256
`a5887a30c8475f9700c1e7e853a5e9b739ff07de219cc630fbfb8fac44e86ba4`); the archive SHA-256 is
`07d41c8297da13cbe4067b53a024caf812c913aa53cd0507a8c69e05d4591ab1`, with 214 dependency
records. Evidence: `docs/evidence/issue9-0935f48-macos-package.json`. Visual review is still open:
the existing AgentConductor window is on the user's saved workspace, so no second window was
opened. This package build does not add a packaged UI workflow.

## Historical UI parity checkpoint — 2026-10-21 / Agent Room header context

The stable v0.1.32 WebView titlebar shows app/version, repository, linked phase, and local CLI-auth context. Rust redundantly rendered “Agent Room” under its native window title and omitted those context fields. Red-first English/Korean rendered assertions reproduced both defects; the Rust header now shows the app/version, localized workspace/phase/auth context, and repository name while removing the repeated title. The 21 focused room/layout/source-layout tests pass across five suites; all-target Clippy (`-D warnings`), formatting, diff checks, and the 300-line source/test guard pass. Graphify refreshed to 7,719 nodes / 12,210 edges. The exact-head Actions run `39507118252` failed macOS native tests at prior HEAD `4ba268e`; Windows was still running when recorded. This is a component correction, not an accepted end-to-end workflow; packaged workflow acceptance remains 6/25 (24%). Current-package visual parity and cross-platform screenshot checks remain open.

## Current verification checkpoint — 2026-10-21 / issue #9 profile quota editor

Commit `dbf249f` on the issue #9 branch fixes the custom CLI profile editor's loss of `quota_command` and `quota_rules`. Red-first profile-save coverage reproduced the dropped fields; the editor now serializes and applies both, labels the advanced JSON section accordingly, and renders existing quota configuration accessibly. Focused profile/quota tests pass 15/15 across six suites. Full native-feature tests pass 803, with 14 ignored across 272 suites; warning-denied all-target Clippy, `cargo fmt --check`, `git diff --check`, and `source_layout` (including the 300-line guard) pass. The macOS package from source `016451b` passes `package-release`, `validate-package`, packaged `--help`, ZIP integrity, and all four shared/worktree × Git/folder smoke cases; binary SHA-256 is `2b0a4b03aac1bc87ef75a97039187ec60179cb84a834e6a8a39e04dbc0d34b92`, archive SHA-256 is `5e69ac1306d77eee5e6a7ac91897e7774ccaf8c003adfbb50a137476dffacea9`. In one isolated package window, AccessKit confirmed the quota editor fields survived profile save and reopen; the test profile was removed. No screenshot was retained and no quota command or external service ran. The exact packaged Workspace and Agent Roles screens were also rechecked in one isolated window without saving data. Evidence: `docs/evidence/issue9-016451b-macos-package.json` and `docs/evidence/issue9-016451b-macos-smoke.json`. Actions run `39505888033` is in progress on both platforms: macOS passed compilation and is checking source layout; Windows is compiling. This component fix is not a packaged end-to-end workflow: acceptance remains **6/25 (24%)**. The native Rust/eframe UI is already the default package UI; full visual parity is not yet accepted.

## Current verification checkpoint — 2026-10-21 / PR #22

Product source commit `cf92f39315c12922915109073e6e1a43365f5a14` is the Rust integration of main v0.1.32, merged from `3123ece`; its package evidence is retained below. The latest local branch includes a cross-platform Workspace action-row fix and a stable MCP settings journey check. Its native-feature all-target run passes **801 tests, 14 ignored, 0 failed across 271 suites**. Warning-denied all-target Clippy, `cargo fmt --check`, diff checks, the authored source/test ≤300-line guard, offline English/Korean manual rendering, focused public-doc tests, and the CI-feature focused regressions pass. The new UI fix has not yet been rebuilt into a package.

The optimized `aarch64-apple-darwin` release package for `cf92f39` passes package smoke in all four shared/worktree × Git/folder combinations (three tasks and tests each), verifies approved merges and refuses unapproved merges, and passes ZIP CRC. The package contains one 18,008,720-byte Rust executable and 214 dependency license records. Binary SHA-256: `1a7831dc567c5fc93859d5949dcb6cb9d7643fb26e0fe65ea65d81894805f741`; archive SHA-256: `fd17557d0be3a27470fccbbf75c372c279ed4d93bde905eb447d54f65755fc51`. Evidence: `docs/evidence/issue9-cf92f39-packaged-smoke.json`.

Packaged workflow acceptance remains **6/25 (24%)**; shipped/default Rust UI replacement remains **0/25**. GitHub issue #9 is open and PR #22 remains draft/mergeable. Platform validation run `39488170157` (#600) finished with native-test failures on macOS and Windows; both jobs passed build/layout, and the native-package job was skipped. The failing tests were the Agent Roles action-row coordinate assertion and the sequential MCP-settings journey's missing duplicate heading. The action group now fills the available width before right alignment, and its regression checks the rightmost action against the card edge; the settings journey now anchors to the dialog title and Save control. Both focused regressions pass locally, including the test-fixture feature set; the full post-fix local suite and Clippy also pass. The exact `cf92f39` v0.1.32 package remains open in one window; its Workspace step row, active underline, and right-aligned save control were visually checked without changing or saving fields. No screenshot artifact was retained, so this does not prove current-main screenshot parity. A new exact-head macOS/Windows Actions run, a package rebuilt with the UI fix, Windows package/UI, published-WebView parity, Korean IME/accessibility, authenticated provider workflows, idle-resource measurements, and final license/release review remain open. Run #598 was cancelled before either OS build completed; #597 was also cancelled before Windows validation completed.

## Stage card marker parity — 2026-10-21

The optimized v0.1.31 macOS package from `ca5c3f4dd8e48d5bef89c533153a4d01e92f1c02` (binary SHA-256 `0163e285900b005b6b9a3cc241cad253be8a5ca22ff0ffe9eea3aea7dd10f0fd`) was run directly with a disposable `HOME`. In one 1140×772 window, Workspace, Agent Roles and Review each showed the selected step marker in both the page eyebrow and active card. This matches a checked-in same-size capture carrying a v0.1.24 label, but `knowledge/wiki/native-screenshot-provenance.md` identifies that capture as an earlier Rust build; it does not establish parity with the published WebView UI. The check supersedes the earlier “show the step marker only once” rule. The inspection did not save a workspace, change settings, or run an agent. Accessibility-tree observations are retained in `docs/evidence/issue9-ca5c3f4-stage-card-markers.json`; current screenshots were inspected in-session but not saved as repository files. The exact package contains no JavaScript, TypeScript or Python source files; the package inventory is one native executable, bundled SQLite, and 214 dependency license records. No end-to-end workflow was added: packaged acceptance remains **6/25 (24%)**, while the shipped/default Rust UI replacement remains **0/25**. Windows UI, Korean IME/accessibility, comparison against a verified WebView capture, and the remaining packaged workflows are still open.

## Native MCP source import — 2026-10-21

The exact v0.1.31 macOS Rust package (`2fd8e57`, binary SHA-256 `818f4d137127e1d650d520796ebe97d7a41be41d5f0f786eea529b9607cc5f8a`) completed the full Workflow 12 path in one window under a disposable `HOME`: register an OpenCode-shaped local stdio MCP fixture, discover its Jira and Confluence tools, preview `jira:AC-1` and `confluence:53553565`, and separately approve applying both source records. The UI showed each record's content, URL, and hash, followed by `Applied sources: 2`. The coordinator and MCP server were deterministic local Rust fixtures; no vendor CLI, external MCP server, or credentials were used. Focused offline verification passed 17 tests across three suites. Evidence: `docs/evidence/issue9-2fd8e57-mcp-import-ui.json`. Packaged workflow acceptance is now **6/25 (24%)**; the shipped/default Rust replacement remains **0/25**. Windows UI, Korean IME/accessibility, full parity, and 19 packaged workflows remain open.

## Native Workspace heading correction — 2026-10-21

The selected step now supplies the bilingual page eyebrow, while each section card keeps its title and description without repeating the step marker. Red/Green header and AccessKit checks pass 17 tests across two suites; source-layout, rustfmt, diff checks, all-target native-feature Clippy, and an optimized release build pass. The exact `2fd8e57` macOS package validates with 214 dependency records and bundled licenses; packaged `--help` and ZIP CRC pass. The serial offline native-feature all-target suite passes **775 tests, 14 ignored, 0 failed across 262 suites** (300.95s). The exact package smoke passes all four shared/worktree × Git/folder modes: 12 fixture tasks and tests, verified concurrency, refusal of unapproved merges, and successful approved merges; retained evidence is `docs/evidence/issue9-2fd8e57-macos-smoke/`. A subsequent exact-package visual audit inspected Workspace, Agent Roles, empty Review, MCP, sharing, Messenger, and cache in Korean dark and English light themes at 1140×772; fourteen screenshots are retained under `docs/evidence/issue9-2fd8e57-visual-audit/`. It used one disposable profile (`fixture`, `/usr/bin/true`) and did not save settings, send messages, clear cache, or run an agent. This component inspection adds no workflow: packaged acceptance remains **5/25 (20%)**; shipped Rust replacement remains **0/25**. Windows rendering, Korean IME, full UI parity, and the remaining workflows are open.

## Exact-package visual batch — 2026-10-21

The v0.1.31 package built from `2fd8e57f27d5f5c7b5d4a63c2eae7b20ea7b1a14` was inspected in one window at 1140×772 using a disposable profile. Fourteen retained captures cover Workspace, Agent Roles, empty Review, MCP registration, workspace sharing, Messenger notifications, and cache cleanup in Korean dark and English light themes; each file's SHA-256 is recorded in `docs/evidence/issue9-2fd8e57-visual-audit.json`. The screenshots show the saved fixture profile only. No real workspace or credentials were loaded; no settings were saved, no message was sent, no cache was deleted, and no agent was run. The inspected forms and dialogs fit the viewport without visible overlap; this is a component visual check and not acceptance of an end-to-end workflow. The metric remains **5/25 (20%)**, with shipped Rust replacement **0/25**. Windows, IME, accessibility on hardware, and the remaining workflows remain unverified.

## Historical branch checkpoint — 2026-10-21 (before the 2fd8e57 package)

PR #22 is open and remains draft; the native-session test-fixture correction is `6e6ef4d`. Packaged native UI acceptance is **5/25 (20%)**; the shipped Rust replacement remains **0/25**. The compact step-navigation product change is `227854d`; its exact macOS package passed package validation, packaged `--help`, and ZIP CRC, and Workspace → Agent Roles → Review was checked at 1140×772. Test-only commit `b204eb5` stabilizes the Settings journey's selected-language wait. On that source, the grouped Settings/header/start-journey tests pass 6 tests across 4 suites. The all-target run first found one `native_session` fixture failure because it wrote a live owner record before initial lease acquisition; after correcting the fixture, `native_session` passed 3/3 and the complete suite passed **794 tests, 14 ignored, 0 failed across 263 suites** in 258.50 seconds. Source layout, public-doc, rustfmt, all-target Clippy with `-D warnings`, and diff checks also pass. Actions run `39452437163` tested merge commit `11588965` from PR head `c5dac47` into base `fdfd6d3`, not the current head; both OS jobs failed only `native_ui_settings_journey`. No exact-head Actions result has been observed. The older 794-test result below is from product code `1b1a920` and does not verify this branch head. Keep the migration open and do not merge or release until current-head platform validation and all remaining acceptance gates pass.

## Historical checkpoint — 2026-10-21 (before the current issue #9 progress)

The full user-requested product remains in development. Issue #9 is open and PR #22 remains draft. The current packaged workflow evidence is from the v0.1.31 macOS build at product commit `a1ea942`: **5/25 (20%)** distinct packaged end-to-end UI workflows accepted; the shipped Rust replacement remains **0/25** until an accepted release ships. The exact package and Git/worktree workflow evidence are in `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Source `6026076` builds and validates as a v0.1.31 macOS package; its one-window UI check confirms Enter creates and selects a saved workspace (component evidence only: `docs/evidence/issue9-6026076-workspace-enter-ui-smoke.json`). Current product code `1b1a920` passed the complete local native-feature all-target suite: **794 passed, 14 ignored, 0 failed across 263 suites** (222.61s); all-target Clippy (`-D warnings`), formatting, diff, public-doc and source-layout checks pass. The current exact-source macOS package at docs-only commit `0d09b88` also passes manifest validation, packaged `--help`, ZIP CRC, and package-content scans; its 17,942,384-byte binary SHA-256 is `1425a6384fd7b24382acc91226de007fe8646c6b579e5d9d632d4b3f427b51db`, and evidence is in `docs/evidence/issue9-0d09b88-macos-package.json`. The checked-in Workspace, dark Workspace, Agent Settings, and Agent Room screenshots still show v0.1.24; no replacement was saved from the current exact Rust package, so the screenshot refresh remains an explicit release gate. No exact-head Actions run is listed for current head `0d09b88`; earlier run `39448772842` targets stale commit `275faf5`.

The grouped UI batch (`cb49fa1`) fixed four audited defects: return from a running Agent Room without stopping it, preserve the selected Review step, safely stop-and-delete a workspace with an active plan only after plan/metadata work settles, and fit the agent editor at 640px. The subsequent Rust scheduler batch (`fdc3528`) restores timeout accounting, bounded automatic retry, and Stop-aware backoff. Current exact issue head `27e307e` passes the complete offline native-feature all-target suite: 768 passed, 14 ignored, 0 failed across 260 suites (211.51s); all-target Clippy with warnings denied, rustfmt, and diff checks pass. The grouped exact-package Settings dialog audit also passes. This full suite validates source behavior, not additional packaged UI acceptance.

The exact macOS package from code source `7419557` was opened in one isolated app window for a grouped screen/control audit: Workspace Settings, Settings/MCP registration, profile registration, and Agent Roles. Two temporary workspaces were created and one inactive workspace archived; a delayed Rust mock plan showed Running/Stop planning and then returned the coordinator to Ready after stop-and-delete. Evidence: `docs/evidence/issue9-7419557-packaged-ui-audit.json`. No provider CLI or external service ran. This spot-check does not constitute a complete packaged workflow. On 2026-10-20, exact source `7289562e5de394555b196da75b0ecf3904f3c53c` built a 17,925,856-byte macOS binary and passed package and smoke validation; evidence is in `docs/evidence/issue9-7289562-macos-*.json`. A newer local package from source `a8be85fc2392821fab345d3e1b879a6aa9bc6714` built a 17,942,384-byte binary, included 214 dependency notices, passed package validation, packaged `--help`, ZIP integrity, and four mock workflow cases (12/12 tasks and tests; rejected unapproved merges; verified approved merges). Evidence: `docs/evidence/issue9-a8be85f-macos-package.json`. The exact package's dialogs were checked and dismissed in a disposable workspace; evidence: `docs/evidence/issue9-a8be85f-packaged-dialog-ui.json`. A separate exact-package workflow exercised UC-30/31 with an isolated `HOME`: create and save named workspaces, save role-stage state, switch and verify persisted settings, remove from recent and reopen, and archive an inactive workspace after confirmation. Evidence: `docs/evidence/issue9-a8be85f-workspace-ui.json`. This added Workflow 2 and advanced packaged UI acceptance to **3/25** at that checkpoint; shipped Rust replacement remained **0/25**. At previously tested code head `27e307e`, the full native-feature suite passed 768 tests / 14 ignored across 260 suites; all-target Clippy, rustfmt and diff checks passed. Exact PR head `604579c` Actions run `39424348558` completed with test failures on both OSes: the settings journey sampled before MCP registration became visible, and Windows also attempted a merge before the previous UI action settled. Local fixes are in this batch. A later full local run stopped after five minutes at 363 passed / 14 ignored across 120 suites without a reported test failure. Focused planning, settings/start, documentation and source-layout checks passed (10 tests across 4 suites); full-suite verification remains open. Exact pushed source `4a114ec6a76141c48ebe8138694398572cae1acd` built a 17,942,384-byte macOS package (binary SHA-256 `f288ff81542870c49a40a73824a5fe70b0d10fe917370dfa66c3f13251d2ec11`); `validate-package` and packaged `--help` passed. In one isolated app window, a saved disposable goal without an agent coordinator showed Generate plan disabled with coordinator guidance; evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`. A follow-on grouped audit inspected Agent Roles, empty Review, MCP, workspace sharing, Messenger, and cache preview together without saving or deleting data; evidence: `docs/evidence/issue9-4a114ec-settings-batch.json`. The native UI suite batch passed 122 tests across 23 suites, and feature Clippy, rustfmt, and diff checks pass. These remain component checks and do not add a workflow. Actions run `39429494121` was cancelled on both Windows and macOS during the native build/test step; it provides no platform validation. The subsequent positive-path package workflow adds Workflow 4, bringing current acceptance to **4/25 (16%)** with **21 packaged workflows** still open. Exact screenshots, Windows UI/package, accessibility/IME, provider integration, resources, and final license closure remain open.

This checkpoint is not completion evidence for Windows, the other workflows, authenticated providers, accessibility/IME, idle resources, or final license closure. Do not merge PR #22 or publish a release until the remaining criteria below are verified. See the current issue #9 checklist in [the implementation plan](07-implementation-plan.md).

The subsequent positive-path exact-package UI workflow registered a local Rust fixture CLI, generated and reviewed a plan, approved execution, passed its 1/1 test, reviewed and approved the diff, confirmed merge, and reached Completed in one isolated process. This adds **Workflow 4**, advancing packaged acceptance to **4/25 (16%)**; 21 packaged workflows and the shipped Rust replacement remain open. Evidence: `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`. The first fixture attempt referenced `/usr/bin/test`, which does not exist on macOS; the fixture was corrected to `/bin/test` and the complete flow passed in a fresh disposable folder.

A subsequent same-build recheck used a three-session local Rust fixture and a fresh non-Git shared folder. The exact reviewed plan had three independent tasks with distinct paths and overlapping process intervals; all three task tests passed, the combined diff was reviewed, candidate approval and app-level workspace integration completed, and the run reached Completed. The enabled approval action and green Completed panel remained readable in dark mode; light mode was restored. Activity, Raw logs, Lessons, and the read-only team briefing were inspected together; each task's result and clean stderr were visible, while provider-native session/model metadata remained unavailable for the mock CLI. This strengthens the already-counted parallel workflow evidence and does not add a fifth workflow. The target folder was not a Git repository, so no Git commit or branch merge is claimed. Evidence: `docs/evidence/issue9-4a114ec-parallel-ui-plan-to-merge.json`.

A follow-on exact-package component batch reused the same app process to inspect Workspace phase analysis, Agent Room candidate evidence, and Updates release notes. The accessible analysis disclosure expanded; candidate revision, passing test, and exact diff were visible; the release-notes link and scrollable full notes were present. The release metadata check ran, but the link was not opened and no screenshot was retained. Five grouped UI suites pass (23 tests) and all-target native-feature Clippy is clean. This adds no workflow acceptance; the total remains **4/25**. Evidence: `docs/evidence/issue9-4a114ec-review-updates-batch.json`.

The same packaged process was also checked in dark mode at 1140×772, then restored to light mode. Completed status, passing-test count, selected diff tab, diff text, and inspector remained readable; no defect reproduced. That run had no enabled primary action; a later three-task run in the same package verified the enabled approval action and green Completed panel (see the follow-up evidence below). Five related theme/layout suites pass (15 tests), as do rustfmt and diff checks. No screenshot artifact was captured. Evidence: `docs/evidence/issue9-4a114ec-dark-theme-audit.json`. Component evidence does not change the 4/25 acceptance count. Actions run `39431587615` was cancelled on Windows and macOS during native build/test; current docs head `459ba84` has no reported Actions run.

At the exact product source `4a114ec`, the complete offline native-feature all-target suite now passes: 770 passed, 14 ignored, 0 failed across 260 suites (216.85s); all-target Clippy with warnings denied also passes. These source checks do not add packaged workflow acceptance. Exact macOS/Windows Actions results, Windows UI, and the remaining acceptance gates are still open.

Historical audit baseline: 2026-10-10 KST. The detailed table below began as an audit of requirements through R-59/DD-65; rows are updated where current evidence is now authoritative, while remaining work is explicit. See [requirements reconciliation](../knowledge/wiki/requirements-reconciliation-2026-10-11.md) for the reviewed historical baseline.

| Requirement | Current authoritative evidence | Remaining acceptance work |
|---|---|---|
| R-01 Native authentication | Native Claude/Codex metadata; installed Codex 0.162.0 ChatGPT-authenticated stdin/JSONL generation and exact-ID context resume with API-key environment variables removed | Claude login and OpenCode configuration; native implementation and all-provider task execution |
| R-02 Subprocess control | Process groups/raw spools, exit/timeout/recovery tests, live bounded normalized SQLite attempt events and fail-fast malformed-stream tests | Native all-provider progress fixtures and sustained resource bounds |
| R-03 Registered providers | Templates, custom contract, parser/argv tests | Real provider contract fixtures and end-to-end registration/execution |
| R-04 Planning | Planner mock tests and plan validation | Goal -> native planner -> registered agents -> verified result end-to-end |
| R-05 Isolation/parallelism | Shared/worktree mock E2E; leases/checkpoints and transitive dependency refresh | Native concurrency evidence |
| R-06 Evidence | Git/test integration, snapshots, reviewed rework, retained workspaces, bounded typed coordinator proposals; distinct persisted Unix signal/exit outcomes | Normalized summaries/failure inventory and native review contract evidence |
| R-07 Approval | Exact candidate binding and mock E2E | Full negative tests for target/index/diff/test changes; native execution review |
| R-08 Dangerous actions | Typed SQLite approval ledger, opt-in Claude worker binding and human CLI controls, compiled shared/worktree approval/expiry/rejection/stop/staleness tests, owned verifier and async cancellation/alias/replay/closed-channel negatives, native metadata initialization | Actual native denial/approval/failure containment for every provider; ignored/external command dependencies and OS containment; profile labels alone do not prove containment |
| R-09 Lifecycle | Quota, CLI stop/resume/revise/review, leases, checkpoints, crash recovery and rework tests | Native contract evidence and interrupted native review recovery |
| R-10 Memory | Outbox/offline recall tests, live intermediate summaries, reviewed task-scoped runtime lesson handoff with current source hashes, development saves | Native lesson service integration, confidence-aware retrieval, cross-run/project context and native session proof |
| R-11 Wiki | English raw/schema/wiki files, reviewed runtime assets with source hashes/staleness and derived pages/index/log; mirror alias/failure tests | Native ingestion/review flow, source revalidation UI and all asset integration evidence |
| R-12 Rust/lightweight | Exact source package `ca9495b21c77886df2f75b1672f965dfe75d53fc`: one 18,606,512-byte macOS executable, bundled SQLite, 214 dependency records; four approval-gated package smoke cases, packaged `--help`, ZIP CRC, and checksum pass | Windows exact-head package/UI and cross-platform idle-resource measurements; retained exact-current screenshots |
| R-13 Workflow/TDD | At branch head `084089fb`, locked/offline native-feature all-target suite passes **938 tests, 14 ignored, 0 failed across 310 suites**; warning-denied all-target Clippy, rustfmt, diff check, focused public-doc/source-layout suites (13 tests), and the authored-file ≤300-line guard pass | Complete the requirement-by-requirement audit and final macOS/Windows release verification |
| R-14 Development tools | RTK and Ponytail used; Graphify queried for quota-fallback evidence on its curated snapshot (8,916 nodes / 14,081 edges) | CocoIndex executable is unavailable in this environment; no CocoIndex result is claimed. Use it when available and retain evidence |
| R-15 Distribution licenses | Exact macOS package manifest records 214 dependencies; package validation and bundled-notice checks pass | Recheck final dependency closure and every bundled/optional license boundary for both final release packages |
| R-16 English/public docs | English requirements, design, plan, and user manuals; public README remains free of internal decisions | Replace historical manual screenshots only after exact-current UI captures pass review; revalidate both manuals in the release package |
| R-17 UI | Eframe-native source coverage is 25/25; distinct packaged end-to-end workflows accepted are **12/25 (48%)**; source-level interaction tests do not add to package acceptance; direct native transport has no UI HTTP listener | 13 packaged workflows; exact current GUI parity and retained screenshots; Windows rendering; full accessibility and Korean IME; idle-resource measurements; public Rust replacement remains unreleased |
| R-18 Availability | Parsing, schedule, bounded retries, watch tests | Native restriction fixtures, crash-safe schedule ownership and runtime UI |
| R-19 Multi-session transfer | Mock concurrency/reassignment, native session exclusion and refreshed dependency context; installed Codex two concurrent independent contexts with exact-ID resumes | Claude/OpenCode/custom native concurrency, reassignment and interrupted task evidence |
| R-20 Session models | Native Claude/Codex catalogs; model edit tests | OpenCode/self-hosted catalogs, native model changes and UI controls |

| R-21 Native startup | Private revision-bound registry, read-only planner, three-task review, exact consent/baseline lease tests; actual native fixture setup through merge | Provider-authenticated planning, dialog accessibility and final packaging |

## Execution order

- [x] Restore full completion criteria and record the user's correction in agentmemory.
- [x] Finish T-03 explicit stop/resume, including integration-test interruption and CLI verification.
- [x] Finish T-02 crash/ownership recovery and cross-run session/working-file ownership.
- [x] Complete human feedback/rework and dependency propagation.
- [x] Implement and verify coordinator-generated review feedback from bounded diff/test evidence using mock subprocess contracts; native contracts remain open above.
- [ ] Enforce dangerous-action capabilities and human gates before native execution.
- [ ] Complete runtime memory/wiki/lessons and source revalidation.
- [ ] Implement the lightweight human-intervention UI using the approved visual direction.
- [ ] Perform real native CLI end-to-end execution/model/session verification.
- [ ] Verify final licenses, resource usage, artifact checksum and complete-product acceptance.

Proceed autonomously under existing authorization while resources permit. Preserve the
user's original scope. Keep the goal active until this audit is proven, rather than
presenting the checkpoint as a finished product.

## New requirement acceptance (R-22–29)

| Requirement | Current evidence | Acceptance |
| --- | --- | --- |
| R-22 quota | Codex authenticated stdio/native refresh; normalized window tests | Partial: Claude capture/resources remain |
| R-23 Windows continuity | Historical CI `38039813985` on `47eb2d7` passed native Windows checks. At this checkpoint the local Windows Rust target is absent (`cargo check --target x86_64-pc-windows-gnu` reports missing `core`/`std`), and the UTM Windows VM is stopped | Verify exact current source on Windows, including package/runtime/UI, containment, and user-visible behavior; no current Windows pass is claimed |
| R-24 platform releases | Exact macOS issue-head package at `ca9495b21c77886df2f75b1672f965dfe75d53fc`: manifest, 214 dependencies, checksum, packaged `--help`, archive integrity and four approval-gated smoke combinations pass | Windows exact-head package/UI, 25/25 packaged UI acceptance, final-source license closure; do not release before every gate passes |
| R-25 subscription-independent supervision | Deterministic execution loop; optional structured review | Partial: idle policies/designated AI lifecycle |
| R-26 per-agent fallback | Eight tests; registered policy/model UI implemented | Partial: native policy interaction/chain restart acceptance |
| R-27 cooperative memory steering | UC-21/DD-29; ledger/REST and daemon fixture; six worker tests cover bootstrap, polling, quiescent exits, unresolved directives, killed-coordinator pause/handoff recovery and explicit resume | Partial: automatic memory discovery/import, worker authentication, company LLM acceptance, interrupted pending/accepted instruction continuation and automatic reviewed-target handoff |
| R-28 status briefing | Six tests; CLI/native factual query; actual three-session read-only UI; exact-ID agentmemory progress import verified against the configured daemon | Partial: automatic live worker report collection, active-provider and expired approval acceptance |
| R-29 status lights | Actual three-state native pulse and Stop/cleanup; mapping tests | Partial: live reduced-motion/resource acceptance |


The 2026-10-21 grouped settings inspection found the workspace-sharing dialog opening at the viewport's upper-left with egui's compact default width, unlike the centered MCP, Messenger, and cache dialogs. A rendered regression failed at x=30 before the fix. The share dialog now anchors to the center with a 480px initial width; 18 focused tests across six Settings/MCP/Messenger/cache/theme suites pass. The exact `97a16bb` macOS package was then opened in one isolated temporary workspace: share was visually verified centered/readable in light and dark themes, MCP, Messenger, and cache dialogs were inspected, profile registration and all five role rows rendered, and the empty Review stage correctly disabled approval. No settings, workspace, or cache data was changed, and no screenshot artifact was retained. This component correction does not increment packaged workflow acceptance: it remains 5/25, with 0/25 shipped Rust replacements. Public screenshots, Windows UI, and the remaining 20 packaged workflows remain open.

A grouped Git/worktree smoke exposed and fixed a repository-switch bug: when the saved goal text was unchanged, saving a different repository retained the old active slug, leaving new plan metadata untracked and blocking the clean-baseline gate. The red test `changing_repository_detaches_the_active_goal_before_planning_again` reproduces this and verifies that creating the next slug leaves the new repository clean. The fix clears only the active slug on repository changes and retains the historical goal record. Focused validation passed 15 tests across five suites; all-target regression passed 771 tests with 14 ignored across 260 suites; all-target Clippy passed with warnings denied. The exact a1ea942 macOS package passed its runtime smoke. One isolated UI run confirmed goal-preserving repository switch, three parallel worktree tasks, 3/3 tests, exact diff review, human approval, merge, and a clean final main worktree. Evidence: `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Packaged acceptance is now 5/25; shipped Rust replacement remains 0/25. Windows UI, complete visual parity, accessibility, Korean IME, authenticated provider workflows, idle-resource measurement, final license closure, and exact-head CI remain open.

## Exact-head UI regression and package checkpoint — 2026-10-21


## MCP source-review popup layout — 2026-10-21

The remaining native inline disclosure was the full-source control on MCP import-review cards. A rendered two-record regression exposed an egui widget-ID collision: clicking one source could open the other source's body. The control now uses a per-reference ID and a bounded menu popup; the test verifies the selected source body is visible and the next record stays at the same screen position. The exact earlier Actions run also exposed stale task-popup tests that expected inline content; those now click the popup and verify the bounded editor. Focused MCP/UI/workflow/document/layout coverage passed 33 tests, warning-denied all-target Clippy, formatting, and diff checks. Graphify refreshed to 8,051 nodes / 12,715 edges / 613 communities.

The pushed source `733852e` builds a validated macOS ARM package with SHA-256 `156d40a118ab990fc699b7f0358026ceac2032a33b5952d2666b9cff6d0e4ddd`; ZIP SHA-256 is `aac823d80d230c9435ef3b9ff3025e35edc15936e035314c97530204f7eaa63a`. Package validation, help, archive CRC, Info.plist, and all four approval-gated smoke modes pass. The exact package launched in one process and loaded the Workspace and three configured agent cards. macOS screenshot capture failed, and the MCP-source popup was not opened in that native package window; rendered egui interaction coverage passes. Exact-head cross-platform run `39592532512` remains active. Packaged workflow acceptance remains 8/25; issue #9 stays open.

A further scan found five page-reflow panels: menu-launched User manual, Agent tools guide, workspace-action confirmation, update details, and execution consent. They now use centered egui windows with stable IDs and visible dismissal; the execution-review button remains in place. Rendered regressions confirm surrounding controls retain their coordinates and existing actions remain covered. The full locked all-target suite with `test-fixtures` and `native-ui-probe` passes (508 tests, 14 ignored across 165 suites); all-target Clippy with warnings denied, rustfmt, and diff checks pass. An exact current-source `aarch64-apple-darwin` package from `9cea0a4` passes package validation, four approval-gated smoke cases, packaged help, Info.plist lint, ZIP CRC, and manifest hash checks; evidence is `docs/evidence/issue9-9cea0a4-macos-package.json`. Graphify refreshed locally to 8,095 nodes, 12,822 edges, and 607 communities; CocoIndex is unavailable in this runtime. Actions run `39597791275` for `920d426` remains in progress; its macOS all-target tests and Windows native build are running. CUA timed out when resolving the existing app by name and exact bundle path, so native visual inspection remains incomplete. Packaged workflow acceptance remains 8/25; issue #9 stays open.

## Exact-head package smoke and menu-layout recheck — 2026-10-22

The exact `d23c3c2` macOS package passes manifest validation, packaged `--help`, archive integrity, and all four deterministic shared/worktree × Git/folder smoke cases. Each case runs three overlapping mock tasks and tests, refuses a merge before approval, and verifies the exact candidate after approval. Evidence: `docs/evidence/issue9-d23c3c2-macos-package.json`. In the one-window Rust preview (`78cc5f0`), Workspace and Settings menus were opened and both overlay page content; the production UI is unchanged between that preview and `d23c3c2`. Twenty-two rendered menu/popup layout tests pass across six suites. This remains component evidence; packaged acceptance is 8/25. Actions run `39687975008` is still in progress: macOS native tests are running and Windows is building.
