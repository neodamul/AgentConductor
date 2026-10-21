# Full-product completion audit

## Windows folder-picker compile regression — issue #9 / 2026-10-21

Actions run `39541153826` exposed a Windows-only compile failure: the localized picker change
removed the `executable` parameter but the Windows file filter still used it. Restore that intent
parameter and pass `false` for folder/Markdown selection and `true` for executable selection at
each caller. The existing Windows CI build is the platform compile regression gate; local macOS
tests do not compile this `cfg(windows)` module. Focused local tests and macOS target check pass;
the exact-head Windows rerun is pending.

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
| R-12 Rust/lightweight | Exact source package `cf92f39`: one 18,008,720-byte macOS executable, bundled SQLite, 214 dependency records; four approval-gated package smoke cases, packaged `--help`, ZIP CRC, and checksum pass | Windows exact-head package/UI and cross-platform idle resource measurements; exact-current visual UI review |
| R-13 Workflow/TDD | Architecture trace documents and Red/Green tests; current native-feature all-target suite passes **800 tests, 14 ignored, 0 failed across 271 suites**; focused public-doc/source-layout checks, all-target Clippy (`-D warnings`), rustfmt, diff, and authored-file ≤300-line guard pass | Complete the requirement-by-requirement audit |
| R-14 Development tools | RTK and Ponytail used; Graphify re-extracted 7,557 nodes / 11,988 edges on 2026-10-21 | CocoIndex Code remains unavailable: startup fails because `mcp.server.fastmcp` is missing from its Python environment; use it when repaired and retain evidence |
| R-15 Distribution licenses | Exact macOS package manifest records 214 dependencies; package validation and license bundle checks pass | Recheck final dependency closure and every bundled/optional boundary for both platforms |
| R-16 English/public docs | Current English documents and public README | Keep status accurate and exclude internal decisions from README |
| R-17 UI | Eframe-native source surfaces for 25/25 workflows; 7/25 packaged end-to-end workflows accepted, including workspace settings export/import; source-level interaction tests do not add to packaged acceptance; direct native transport with no UI HTTP listener | 18 packaged workflows; exact current GUI parity/retained screenshots; Windows rendering; full accessibility and Korean IME; resource measurements |
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
| R-23 Windows continuity | CI 38039813985 on 47eb2d7 passed native Windows build, applicable tests, Job containment, WebView2 smoke, layout and Clippy | Partial: packaged runtime and full native workflow acceptance |
| R-24 platform releases | Exact macOS issue-head package from `7419557`: manifest, 214 dependencies, SBOM, checksum, packaged `--help`, archive CRC and four approval-gated smoke combinations pass | Windows exact-head package/UI, complete packaged UI acceptance, final license audit; do not release before all acceptance gates pass |
| R-25 subscription-independent supervision | Deterministic execution loop; optional structured review | Partial: idle policies/designated AI lifecycle |
| R-26 per-agent fallback | Eight tests; registered policy/model UI implemented | Partial: native policy interaction/chain restart acceptance |
| R-27 cooperative memory steering | UC-21/DD-29; ledger/REST and daemon fixture; six worker tests cover bootstrap, polling, quiescent exits, unresolved directives, killed-coordinator pause/handoff recovery and explicit resume | Partial: automatic memory discovery/import, worker authentication, company LLM acceptance, interrupted pending/accepted instruction continuation and automatic reviewed-target handoff |
| R-28 status briefing | Six tests; CLI/native factual query; actual three-session read-only UI; exact-ID agentmemory progress import verified against the configured daemon | Partial: automatic live worker report collection, active-provider and expired approval acceptance |
| R-29 status lights | Actual three-state native pulse and Stop/cleanup; mapping tests | Partial: live reduced-motion/resource acceptance |


The 2026-10-21 grouped settings inspection found the workspace-sharing dialog opening at the viewport's upper-left with egui's compact default width, unlike the centered MCP, Messenger, and cache dialogs. A rendered regression failed at x=30 before the fix. The share dialog now anchors to the center with a 480px initial width; 18 focused tests across six Settings/MCP/Messenger/cache/theme suites pass. The exact `97a16bb` macOS package was then opened in one isolated temporary workspace: share was visually verified centered/readable in light and dark themes, MCP, Messenger, and cache dialogs were inspected, profile registration and all five role rows rendered, and the empty Review stage correctly disabled approval. No settings, workspace, or cache data was changed, and no screenshot artifact was retained. This component correction does not increment packaged workflow acceptance: it remains 5/25, with 0/25 shipped Rust replacements. Public screenshots, Windows UI, and the remaining 20 packaged workflows remain open.

A grouped Git/worktree smoke exposed and fixed a repository-switch bug: when the saved goal text was unchanged, saving a different repository retained the old active slug, leaving new plan metadata untracked and blocking the clean-baseline gate. The red test `changing_repository_detaches_the_active_goal_before_planning_again` reproduces this and verifies that creating the next slug leaves the new repository clean. The fix clears only the active slug on repository changes and retains the historical goal record. Focused validation passed 15 tests across five suites; all-target regression passed 771 tests with 14 ignored across 260 suites; all-target Clippy passed with warnings denied. The exact a1ea942 macOS package passed its runtime smoke. One isolated UI run confirmed goal-preserving repository switch, three parallel worktree tasks, 3/3 tests, exact diff review, human approval, merge, and a clean final main worktree. Evidence: `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Packaged acceptance is now 5/25; shipped Rust replacement remains 0/25. Windows UI, complete visual parity, accessibility, Korean IME, authenticated provider workflows, idle-resource measurement, final license closure, and exact-head CI remain open.

## Exact-head UI regression and package checkpoint — 2026-10-21

The latest product source is `ae2540c496fde4a25be527a468d84114961265a6`. It fixes two cross-platform UI test failures from run `39488170157`: full-width right-aligned workspace actions and stable MCP dialog identification. Full local native-feature tests passed 801 with 14 ignored across 271 suites; Clippy (`-D warnings`), source/test layout, formatting, and diff checks passed. The exact-source macOS release package passed manifest validation, packaged `--help`, ZIP integrity, and all four approval-gated smoke combinations. In one window with an isolated temporary HOME, the package GUI was inspected on Workspace (light/dark), Agent Roles, CLI registration (including dark-theme Save contrast), MCP, workspace sharing, Messenger, and cache preview. Light theme was restored; no workspace/settings were saved, no provider or external service ran, and no screenshot file was retained. Evidence: `docs/evidence/issue9-ae2540c-macos-package.json`. Exact-head Actions run `39495359230` passed all four macOS/Windows validation and native-package jobs, with both platform artifacts retained. Packaged UI acceptance is **6/25 (24%)**; shipped/default Rust replacement is **0/25**. Full visual parity, Windows package UI screenshot, full product acceptance, accessibility, Korean IME, provider-authenticated flows, idle-resource data, and final license review remain open.
