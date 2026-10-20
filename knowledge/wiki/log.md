# Knowledge and development log

## 2026-10-20 issue #9 grouped workspace and settings audit

Audited Workspace settings, Agent Roles, Review the plan, the Settings menu, MCP registration, workspace sharing, Messenger notifications, and cache cleanup together in the exact isolated v0.1.31 package. The saved-state plan button still navigated without a registered coordinator and surfaced the backend error late. Gated it on a nonempty saved goal and a coordinator ID present in the profile list; added localized guidance and a UI regression. Also stabilized the settings journey to wait for dialog content and the start journey to wait for the previous UI action to settle before merging. Focused planning tests pass (2); settings/start/docs/layout suites pass (10). The all-target local run was stopped after five minutes at 363 passed, 14 ignored across 120 suites without a reported failure; full-suite verification remains open. Actions run `39424348558` failed on both OSes in settings journey, plus a Windows merge race; its head predates these local fixes. Packaged acceptance remains 3/25; no additional workflow is claimed.

Commit `4a114ec6a76141c48ebe8138694398572cae1acd` was packaged and validated locally. The exact macOS package's help command and manifest validation pass; its single isolated UI shows Generate plan disabled with “coordinator not registered” after a goal is saved without profiles. UI evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`. GitHub Actions run `39428920229` is pending; packaged acceptance remains 3/25.

Continued the same issue #9 batch across Agent Roles, empty Review, MCP server registration, workspace-sharing controls, Messenger notifications, and cache preview in the exact package. All expected controls rendered; Approve remained disabled without a reviewed plan, the empty cache preview disabled Clear, and no settings or workspace data changed. Evidence: `docs/evidence/issue9-4a114ec-settings-batch.json`. The grouped native UI suite batch passed 122 tests across 23 suites; the 3/25 packaged workflow denominator is unchanged.

Completed a positive-path exact-package UI workflow with one local Rust fixture CLI: register and select the coordinator, generate/review the plan, approve execution, complete the task, pass 1/1 test, review and approve the diff, then confirm merge. The run reached Completed and created `one.txt` only in the disposable project. Initial fixture typo `/usr/bin/test` was corrected to macOS `/bin/test`; a fresh project run passed. Evidence: `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`. This adds Workflow 4 and advances packaged acceptance to 4/25 (16%); shipped Rust replacement stays 0/25. Follow-up public-doc/layout tests pass (8 tests / 2 suites), and rustfmt/diff checks pass. Actions run `39429494121` was cancelled during native build/test on both OSes. Graphify re-extraction reports 7,509 nodes / 12,943 links with no source-graph changes; CocoIndex remains unavailable due missing `mcp.server.fastmcp`.

Grouped a further exact-package read-only inspection across Workspace phase analysis, Agent Room candidate review, and Updates. The accessibility tree showed the expanded analysis state and phase/task context, exact candidate revision/diff and passing test, plus a release-notes link and scrollable full notes. Reused the one existing app process; the public release metadata check was requested, but the link was not opened. Five source suites passed 23 tests and native-feature all-target Clippy is clean. This component batch does not change the 4/25 workflow count. Evidence: `docs/evidence/issue9-4a114ec-review-updates-batch.json`.

Inspected the same exact package in dark mode at 1140×772, then restored light mode. Completed status, test result, diff tab/body and inspector were readable. The completed run has no enabled primary green action, so button text contrast remains open. Five grouped theme/layout suites passed 15 tests; rustfmt and diff checks pass. Actions `39431587615` was cancelled on both OSes; no run is reported for docs head `459ba84`. Evidence: `docs/evidence/issue9-4a114ec-dark-theme-audit.json`.

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
