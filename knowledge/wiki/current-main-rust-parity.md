# Current-main behavior carried into the Rust UI branch

## Native event-driven refresh

Successful mutating Room requests publish a content-free state invalidation to subscribed peer windows; the source view is suppressed. Each listener coalesces invalidations, requests an authoritative asynchronous snapshot, and wakes the native renderer. Periodic snapshots run on a dedicated worker and reply lane, separate from interactive dispatch, so a delayed refresh cannot consume foreground queue capacity or overwrite a foreground pending result. The active/idle fallback cadence is one second / thirty seconds. Verification includes a deterministic barrier proving a chat request completes while the background snapshot is blocked, plus event fan-out, source-suppression, and read-only-filter tests. Local targeted suites and warning-denied Clippy pass at `153d152f`; full-suite, packaged macOS/Windows, and visual acceptance remain separate gates.

## Full local Rust test suite — issue #9 / 2026-10-29

At exact branch head `355d6256d358533bfd182712ce1fa99d174b4aa4`, `cargo test --all-targets --locked --offline --features test-fixtures --no-fail-fast --quiet` exited 0 with no failures. The local run is macOS; authenticated provider and exact packaged-binary tests keep their explicit prerequisites. This does not replace the pending exact Windows package check or award packaged UI acceptance. Evidence: `docs/evidence/issue9-full-suite-355d625-20261029.json`.

## Exact-current-head macOS package — issue #9 / 2026-10-29

Built the optimized Apple Silicon archive from source `355d6256d358533bfd182712ce1fa99d174b4aa4`. The archive CRC passed; its manifest identifies that exact commit and a 19,456,032-byte binary, whose SHA-256 matches the embedded `SHA256SUMS`. The packaged binary's `--help` ran successfully. Bilingual HTML manuals, licenses, SBOM, and UI screenshot assets are present. Archive SHA-256: `560e138b9ce2543157caed57fedda16e73fa92000295b96e16f2f26fba2cd73e`. GUI launch and visual acceptance were not performed, so this adds no packaged UI workflow credit. Exact Windows package is still pending. Evidence: `docs/evidence/issue9-macos-package-355d625-20261029.json`.

## Verify Windows scheduled retries through controller dispatch and persistence — issue #9 / 2026-10-29

Exact run `40821720696` passed both platform validation jobs and the macOS package, but the Windows package stopped in the schedule journey after UI Automation focus and invoke failed. A foreground-checked pointer click was sent, but the test driver treated the still-enabled Schedule button as proof that no action was accepted and exited before inspecting the controller owner record or persisted timestamp. The driver now receives the isolated run directory and treats either the button's disabled transition or a process-owned `coordinator-ui-*` owner record as an action transition; the package test still requires the controller dispatch record and exact persisted `manual_schedule` time. The source-contract regression was red before the change. Local verification passes: schedule driver 7/7, source layout 2/2, native schedule/action tests 6/6, warning-denied Clippy, rustfmt, diff check, and the 300-line source/test limit. Exact package rerun is pending. No acceptance credit is added; issue #9 remains **22/25 packaged workflows, 0/25 shipped Rust replacements, 22/50 aggregate**.

## Reacquire Windows Signal Settings automation windows — issue #9 / 2026-10-29

Exact package run `40817071475` passed macOS and Windows validation plus the macOS package, then failed in the Windows Signal Settings journey. The `agentmemory preferred` control was present in a second top-level window owned by the packaged process, while the test driver kept searching only the window captured at startup. The driver now enumerates current process-owned top-level windows on each control lookup and uses the matched window handle for focus and pointer input. The regression was red before this change; focused local verification passes 35/35 tests across six suites, warning-denied Clippy, rustfmt, diff check, and the 300-line source/test limit. Exact packaged rerun is pending. Acceptance remains **22/25 packaged workflows, 0/25 shipped Rust replacements, 22/50 aggregate**.

## Keep Windows tool-decision confirmation actionable — issue #9 / 2026-10-29

Exact package run `40811281029` passed the macOS native archive and both platform validation jobs, but the Windows package reached `Deny this exact tool request?` with both confirmation controls disabled. The active-run one-second background snapshot poll could overlap the human decision; a pending tool receipt now suppresses that poll until the operator decides or defers, and Cancel remains available if an earlier refresh is already in flight. The new state/poll and modal regressions pass in the focused native-room suites (20/20); warning-denied all-target Clippy and formatting pass. The first full local test attempt caught that the new test file lacked the `native-ui-probe` feature gate; the gate was added and the targeted source-contract suite passes. Exact Windows package verification is still required. No packaged UI acceptance credit is added; current issue #9 acceptance remains **22/25 packaged workflows, 0/25 shipped Rust replacements, 22/50 aggregate**.

## Refresh Windows UI Automation controls for modal tool confirmation — 2026-10-29

Exact package runs `40803567094` and `40805049482` both passed validation and preceding Windows journeys, then failed to observe the `Confirm tool decision` control after activating `Allow once`. The failure dump showed that the confirmation prompt and button were present, while the approval request remained pending. The test driver retained a UI Automation element from before the modal state change and searched only that window subtree. It now enumerates the current top-level UI Automation windows owned by the packaged process and looks up the exact accessible button in each. The new red/green source-contract test passes with the 12 existing dispatch regressions; source-layout, warning-denied Clippy, rustfmt, diff checks, and the 300-line limit pass. Exact Windows package rerun is pending, so workflow 23 gets no acceptance credit yet. Evidence: `docs/evidence/issue9-tool-confirmation-live-uia-20261029.json`.

## Default executable opens the native Workspace — 2026-10-29

The `agentconductor` binary now routes an empty argument list to the native desktop flow and keeps explicit CLI commands on their existing path. The optimized binary was assembled as a temporary macOS `.app` and launched from the current Rust branch. CUA confirmed one standard window and rendered Workspace, Agent Roles, and Plan Review; navigation did not save or execute any workspace action. The screenshot was shown in-session and intentionally not retained because it contains private workspace details. This local bundle is not the GitHub Release archive; Windows exact-package verification and packaged workflow acceptance remain pending.

## Main #226 launch behavior in the Rust desktop — 2026-10-29

The Rust default app now calls `ui::workspaces::launch_root()` before creating `Room`. It reopens the most recently registered, still-saved workspace, falls back to another saved workspace if that entry is stale, and uses the default data folder only when no workspace has been saved. An isolated regression failed before the function existed, then passed after implementation. The test covers a fresh install, creating two workspaces, deleting the default workspace, and reopening the surviving one without recreating the deleted folder. Verification: launch regression 1/1; native-default, workspace-settings, and source-layout tests 6/6; desktop routing 3/3; warning-denied Clippy and rustfmt pass. The regression is platform-neutral and awaits exact Windows package execution.

## Main #184 help entries in Rust Settings — 2026-10-29

The Rust Settings menu now exposes both the agent tools guide and user manual. Selecting them opens the existing native dialogs; the workspace menu keeps its existing shortcuts. The UI regression failed before the Settings actions existed and passes after routing. Verification: settings help 1/1, appearance 5/5, manual 7/7, workspace menu 6/6, source layout 2/2; warning-denied Clippy and rustfmt pass. A locally packaged Rust app opened both dialogs in a single macOS window. Windows exact-package behavior remains pending.

## Coordinator plan prompt structure — 2026-10-29

Rust planner prompts now persist one ordered `# Document`, `# Task`, and `# Format` sequence. Existing plan material and approved Jira/Confluence source evidence are untrusted Document content; the goal, agent catalog, role and plan rules, phase instructions, task types, repository capabilities, and cooperation guidance stay in Task; the bound response JSON Schema is last in Format. Exact heading lines inside evidence are escaped. Invalid-scope repair uses the same structure and places its validation feedback in Task. Existing source-plan/context sidecar files and schema-constrained CLI arguments are unchanged. Verification: coordinator planning contracts 15/15 over eight suites, worker/review/recovery regressions 50/50 over ten, Claude quota 6/6, timeout resume and stop 1/1 each, focused warning-denied Clippy and rustfmt pass. This does not claim chat prompt parity, full-suite verification, or packaged UI acceptance.

## Worker prompt structure — 2026-10-29

The Rust worker persists every CLI prompt with exactly one `# Document`, `# Task`, and `# Format` section in that order. Declared dependency evidence, checkpoint continuation, reviewed lessons, and step lessons are document material; task scope, human feedback, direct-folder rules, cooperative context, and checkpoint instructions stay in the task; evidence reporting is the output format. Heading-like lines in supplied material are escaped so they cannot create extra structural sections. The focused suite checks ordering, neutralization, worker scope, retained `.aipro/plans/` cache guidance, and the prompt written by a real workflow fixture. Local evidence: 4 focused tests pass, focused Clippy and rustfmt pass. This covers worker prompts only; it makes no packaged UI acceptance claim and does not establish planner/chat prompt parity or full-suite status.

## Candidate confirmation outside the main-window rectangle — issue #9 / 2026-10-29

Run `40745298702` passed the briefing journey after the bounded retry update and reached the Windows candidate approve/merge flow. The final `Confirm candidate decision` control was rejected because its UIA bounds fell outside the primary app window rectangle. The driver now permits an enabled, visible confirmation control at those coordinates only if the foreground window and `WindowFromPoint` both belong to the expected AgentConductor process. Empty or offscreen controls still fail closed. Local candidate-driver, briefing-driver, and source-layout contracts pass 9 tests; exact packaged Windows confirmation and persisted Git/run assertions remain required. No issue #9 acceptance credit is added.

## Windows briefing activation follow-up — issue #9 / 2026-10-29

Exact package run `40741509631` failed before the candidate-review journey: UIA Invoke on `Load read-only team briefing` returned `Unrecognized error`; a process-checked pointer click returned success but produced neither a read-only briefing menu nor a disabled/pending action, and the run stayed `Waiting approval`. PR #230's earlier head did not include a briefing fix; its newer base changes candidate-review activation only. The `windows_briefing` driver now waits for observable state and retries at most three times, reacquiring the button and rechecking its bounds and owning process each time. Retries stop when the request is pending or the menu appears. Local contract/layout tests pass 3/3, formatting, diff, and line-limit checks pass. This is test-driver hardening only; exact Windows package verification and candidate-review acceptance remain pending. Do not credit workflow 24. The exact failure and follow-up checklist are recorded in `docs/07-implementation-plan.md`.

## Live ACIO worker integration — issue #9 / 2026-10-29

Commit 447afce112c44e37e2de0fbacb8ebb6bc73eb0fc wires profiles explicitly configured with protocol ACIO through the live Rust worker. The scheduler assigns a host-owned session ID; the worker sends schema-validated structured task input as stdin's first line, then consumes and persists ordered ACIO JSONL events through the existing process observer. Event session identity must match the assigned ID, and a mismatch is rejected before a tool event can create an approval request. A needs-decision tool event creates the existing workspace/task/session-bound approval receipt and pauses worker timeout accounting; timeout leaves the request pending and does not authorize the tool. Resume identity is sent only after the CLI establishes the session. Non-ACIO profiles keep their prior path. This is protocol integration, not an operating-system sandbox or packaged UI acceptance. AgentMemory record: mem_mvsg09ok_a99229b36e7b.

## Rust ACIO runtime and CLI conformance — 2026-10-29

The Rust branch now exposes `io schema`, `io describe`, `io validate`, and `io conformance`; the last command checks a custom executable's ACIO description, read-only response stream, and invalid-input exit code. The optional `AgentProfile.protocol` field preserves older profile JSON when omitted. ACIO event types are normalized by the existing Rust decoder, and ordered `user_message`/`cancel` control lines share the split process runner's stdin channel. Focused verification passes 38 tests across seven suites; the complete locked/offline suite passes 1,077 tests with 14 ignored across 344 suites, warning-denied Clippy, rustfmt, diff, and source-layout checks all pass. The conformance command executes the chosen CLI with the user's OS permissions: its `read_only` request is not sandbox enforcement. The live `workflow::execution::worker` still sends plain text through `process::run_until` and only applies the ACIO decoder after process exit; it does not call `acio::run`, so setting `protocol: acio` is not yet an end-to-end worker integration. The next port must construct and validate `acio.input/1` from task scope/profile state, bridge decision-required tool events to the existing approval ledger without auto-allow, and preserve process ownership/cancel behavior. Task-mode, cancellation timing, and oversized-output conformance also remain unimplemented; this adds no packaged UI acceptance.

## Issue #9 current integration audit — 2026-10-28

Current source branch: `codex/issue-9-native-ui-parity` at `c51998cb`. Current `main` is
`d324caca`; the shared base is `e32295b3`. The branch is 56 commits ahead and 971 behind `main`.
GitHub marks PR #22 `DIRTY`. A read-only merge-tree calculation reports 43 conflicts: 24 content
conflicts in Rust source/tests and documentation, and 19 modify/delete conflicts where the Rust
migration removed legacy JavaScript/CJS/Python UI files that `main` continued to modify. The branch
diff spans 1,546 files. Do not resolve this by selecting one side wholesale; preserve the Rust-only
removals while reviewing each upstream behavior that still needs a Rust port.

The read-only parity scan also found 1,371 lines in 11 ACIO/CLI-I/O/task-profile source and test
files on `main` that are absent from this Rust branch. They implement process-I/O conformance and
control channels, Document/Task/Format prompt assembly, task types with per-model effort, and
configurable agent characteristics. These contracts map to the original CLI-first system goal and
must be traced and ported before claiming whole-product Rust parity. The exact package/UI acceptance
ledger is separate and does not prove these contracts.

Exact-head run `40701439500` on `c51998cb`: macOS validation passed; Windows validation failed before
package jobs because a driver-source test assumed LF line endings, while the Windows checkout uses
CRLF. The assertion now normalizes CRLF. The preceding Windows candidate-review package failure led
to a driver change that reacquires the process-owned window before reading controls. Candidate
workflow 24 remains uncredited until the exact Windows package completes approval, rejection,
persistence, and Git HEAD assertions. Current issue #9 acceptance remains **22/25 packaged
workflows**, **0/25 shipped Rust replacement**, **22/50 aggregate**. Open bug issues #189 and #221
have separate Rust PRs under cross-platform validation.

The Windows package in run `40697161465` (issue #221 branch) then exercised the shared candidate
review script and failed at `Merge approved candidate`. Its final UIA tree showed the approval
confirmation still open and the run still `Waiting approval`; the driver had trusted `InvokePattern`
without verifying a state transition. A new source-contract regression was confirmed red before the
driver change. The driver now checks the expected visible state after each decisive click and falls
back to `SendInput` only after checking that both the foreground window and `WindowFromPoint` belong
to the packaged process. Local `windows_desktop_driver` and `source_layout` pass 6 tests across 2
suites; the Rust test and embedded PowerShell file are 60 and 178 lines. A regression assertion also
checks that pointer release occurs before a post-click foreground failure can return. This is not Windows UI
verification: run `40701439500` tested the prior source, and a fresh exact package is required before
candidate acceptance.

No product code or acceptance status is claimed by this integration audit. Evidence is tracked in
the issue #9 comments and AgentMemory entries `mem_mvs74d17_0470bf274584` and
`mem_mvs79a4p_cf3929822fee`.

## Prior integration checkpoint — 2026-10-27

At `85c322d`, PR #22 was open/draft and conflicting. A read-only merge-tree preview found nine conflicts; the full branch-to-branch tree difference was initially mistaken for the merge delta. The resolved conflicts preserved Rust-specific module splits because #176's serialized test-environment safety change was already ported in `30e0fdaa`. Ancestry merge `aef00f5a97fb96ba2b5c4df0d87ee3b72edf0ca3` is now pushed; GitHub reports PR #22 `MERGEABLE` / `UNSTABLE`. No new CI run was triggered.

Local verification: the combined nine-target check passes 35 tests with 1 ignored; formatting, warning-denied all-target Clippy, unsafe-comment Clippy over test targets, and diff checks pass. Graphify refresh: 9,484 nodes, 14,748 edges, 728 communities. CocoIndex is unavailable in this runtime. These checks do not verify Windows UI Automation or the exact release package.

## Upstream `main` reconciliation — 2026-10-23

Fetched current `origin/main` at `b7fa27a9`; the shared base with the issue #9 branch is
`e9d1f1ff`, with 15 upstream commits after that point. Four commits arrived after the prior
`c9c575a7` audit: one bounded coordinator lease-retry fix, followed by v0.1.37 release metadata
and manual updates. The retry behavior is already present in Rust commit `f7dccadb`. The #149
amendment, launch-gate and charter/drift contracts, #156 readiness fix, and #162 quota rounding
are also ported (`f271f239`, `8eef5856`, `c7d9bbf6`, `f938a76e`, and `b7b67a0c`). Six focused
suites pass 24 tests at Rust head `2fed27cc`. Evidence: `docs/evidence/issue9-main-parity-b7fa-20261023.json`.

Keep the Rust package version at 0.1.36 until migration acceptance; choose the next stable version
when preparing the release. Do not import the upstream JavaScript UI or release-only metadata in
advance. This selective parity audit does not change packaged UI acceptance or Windows readiness.

### Latest main integration — 2026-10-24 (#169 / #172)

The 2026-10-23 mergeability warning above described an earlier base and is superseded. After the
Rust branch and `main` converged at `602ec118`, main added only the #169 Rust review policy and #172
unsafe-site safety comments. Current main `3bcfebf219087c423565f96e1ea3bea9464093da` is now merged
into issue #9 as `4a53cf66516eda9a78432b1a05bec574816e7aaf`; GitHub reports PR #22 mergeable.

The conflict resolution retained the native Rust provider-icon decoder and reduced-motion hook,
the latest picker contract and Windows quota code, and all #172 safety comments. `native_guard` and
the Windows terminal quota collector remain split below 300 lines. The unused legacy Cocoa WebView
launcher stays removed. The first merged compile exposed the lost icon decoder and changed picker
arguments; restoring the bounded decoder and adapting callers fixed the failure.

Verification on the merged head: native-feature tests pass 959/959 with 14 ignored across 319
suites; focused native journey/docs/layout pass 14/14; public docs/source layout pass 13/13;
rustfmt, warning-denied Clippy including `undocumented_unsafe_blocks`, and diff checks pass.
Graphify re-extraction reports 9,201 nodes, 14,401 edges, and 701 communities. CocoIndex is not
installed in this environment. Current Platform Validation run `39999933920` is still running;
the old-base macOS Step 3 UI failure is not yet confirmed or cleared on this head. This integration
adds no packaged UI acceptance; issue #9 remains open and PR #22 remains draft.

The earlier #165 trace warning remains applicable: keep the unique R-173 trace for #165 because
R-172 is already used by #156 on this branch. Evidence: `docs/evidence/issue9-main-merge-20261024.json`.

## Scope

This checkpoint ports the v0.1.32 main-branch contracts for issues #79, #80, #82, #83, #84, #95, #100, and #103 into issue #9's Rust-only branch. The port keeps application behavior in Rust and excludes the upstream JavaScript UI and CJS tests.

## Architecture

- Task-scope path matching reuses plan::overlaps, so path casing follows the supported filesystem semantics while writes still require the task's owned scope.
- Shell command forms use the shared fail-closed classifier. Unknown or concatenated forms continue through the human-approval gate.
- Remote MCP authorization headers accept validated environment references; resolved secrets are not written into diagnostics.
- Quota reset timestamps use the IANA timezone database, including daylight-saving transitions.
- A successful self-update clears the update download root after replacing the installed app. Failed replacement preserves its files for diagnosis.
- Normalized task events update one bounded task-activity document. Agent Room reads a snapshot containing the current tool, elapsed time, age of the last event, and owned-process observation. Silence is not described as thinking or failure.
- Running-task feedback uses the existing active-feedback queue. Agent/model changes for pending tasks use a run-database document and are applied by the scheduler before launch. A running task cannot be reassigned.

No API-key provider integration, local UI web server, extra polling service, or agent-to-agent free-form channel is introduced.

## TDD and verification

Rust regressions for the upstream contracts were added before porting and failed on the missing behaviors. Native UI regressions cover active feedback, pending reassignment/model changes, immutable running ownership, activity text in English/Korean, and the snapshot contract. The focused local batch passed 36 tests across 12 suites. The full local run passed 800 tests with 14 ignored across 271 suites. Formatting, diff checks, public-document checks, the authored-file source-layout test, and warning-denied all-target Clippy passed.

The release-profile macOS package for commit `cf92f39` passed its packaged workflow smoke in shared/worktree modes with Git and folder sources (four cases, three tasks each); approved merge succeeded and unapproved merge was refused. ZIP CRC passed. Evidence: `docs/evidence/issue9-cf92f39-packaged-smoke.json`; archive SHA-256 `fd17557d0be3a27470fccbbf75c372c279ed4d93bde905eb447d54f65755fc51`; binary SHA-256 `1a7831dc567c5fc93859d5949dcb6cb9d7643fb26e0fe65ea65d81894805f741`. Exact-package UI, Windows package/UI, and GitHub Actions completion remain open. These code, test, and smoke results do not increase packaged issue #9 acceptance or indicate that the Rust UI is ready to ship.

## Source map

src/guard/policy.rs, src/guard/forms.rs, src/mcp/config.rs, src/quota/zones.rs, src/update/install.rs, src/activity.rs, src/state/history.rs, src/briefing.rs, src/native_ui_probe/room/task_activity/, src/native_ui_probe/room/task_controls/, and src/workflow/pending_edit.rs.

## Cross-platform Workspace regression follow-up — 2026-10-21

Platform Validation run `39488170157` (#600) exposed two native UI test failures on both macOS and Windows. The Agent Roles assertion checked the Save button's left x-coordinate, which changed with locale/font metrics even when the action group should be judged by its rightmost action. The settings journey waited for a duplicate heading that was not exposed in the platform accessibility tree, although the MCP window title and Save control were present. The Rust action row now lays out over the full available width; the test checks the rightmost action edge. The MCP journey waits on the stable dialog title and verifies its Save control. Red/Green was recorded in the plan; both focused tests pass locally with CI fixtures, and the full native-feature suite passes 801 tests (14 ignored) across 271 suites. Warning-denied Clippy, source layout, formatting, and diff checks pass. A new exact-head Actions run and rebuilt package are still required; acceptance remains 6/25 and shipped/default Rust UI remains 0/25.

## Agent Room focus-follow parity — issue #109 / 2026-10-21

The main WebView follows the active session when the selected running task stops. The native Rust port now records the selected task's observed status and, on its running-to-nonrunning transition, selects the next running or `waiting_tool_approval` session. Rust's human-review controls remain visible in the fixed summary pane, so there is no separate coordinator-selection target. A red transition regression reproduced the missing initial selection before the implementation; the focused activity suite passes 6/6, including next-running and approval-waiting sessions. Source layout passes 2/2, warning-denied all-target Clippy, Rustfmt, and diff checks pass. AgentMemory decision: `mem_mvhn80be_75cd291be699`.

The all-target suite then exposed a Workspace return race. The running-room header exposed “Return to Workspace” while an asynchronous room request was pending, allowing the old reply to replace the workspace snapshot and hide “Open Agent Room.” A new UI regression reproduced the enabled button while pending; the button now waits for request completion. Red/Green is verified: the regression failed with `[false]` before the UI guard and passes afterward. The return-control, start/return journey, and source-layout suites pass 8 tests; warning-denied all-target Clippy, Rustfmt, diff, and 300-line checks pass. The full release all-target suite was stopped after over ten minutes spent recompiling test crates with LTO; that broad check is not claimed. The cross-platform native UI CI command now includes `native_room_next_step`, so the regression will run on both Windows and macOS. Actions run `39545652734` completed all four jobs successfully on prior source `e0de50b`; it did not include these local commits. Packaged workflow acceptance remains 8/25. AgentMemory finding: `mem_mvhoklpd_31cd94816a25`.

Issue #117's WebView layout fix reserves a stable row above the Activity/Diff/Tests tab row. The native Rust room places its Activity/Review/Logs/Lessons tabs above the activity panel inside a fixed column; changing live output cannot move those tabs. The same CSS fix is therefore not applicable to the native layout.

Graphify's code-only re-extraction after this Rust change reports 7,812 nodes and 13,382 raw edges; multigraph diagnosis found no malformed or missing-endpoint edges. CocoIndex is not installed as a CLI in this environment, so no CocoIndex result is claimed.

## Visual evidence status

The checked-in Workspace image is a historical Rust capture from `392aa20`, not a current-main screenshot. The exact `cf92f39` v0.1.32 package was visually checked in one live Workspace window, but no screenshot artifact was retained. No current-main screenshot was retained or reviewed in this check. Do not claim current-main screenshot parity from the checked-in images or this component check. Packaged workflow acceptance remains 6/25, and shipped/default Rust UI replacement remains 0/25.

## Provenance

The behaviors were compared against origin/main v0.1.32 commit 3123ece. Cargo.toml and Cargo.lock changed only for the v0.1.32 package version; this port adds no external dependency or license obligation. Graphify's code-only extraction contains 7,699 nodes and 13,223 edges. CocoIndex could not run because its configured Python environment lacks mcp.server.fastmcp. The integration is tracked by GitHub issue #9 and its draft PR #22.

## Exact-head CI regression and package evidence — 2026-10-21

CI run `39488170157` exposed two cross-platform native UI test assumptions. The workspace action test used the Save button's left edge, which varies with layout/font metrics; it now checks the right edge of the complete right-aligned action group. The settings journey waited for a duplicate heading absent from the accessibility tree; it now waits for the stable MCP dialog title and verifies the Save control. Both focused regressions pass locally; the native-feature all-target suite passes 801 tests with 14 ignored across 271 suites. Exact pushed source `ae2540c496fde4a25be527a468d84114961265a6` has a verified macOS package and four approval-gated Git/folder × shared/worktree smoke passes. In an isolated temporary HOME, the package UI was inspected on Workspace (light/dark), Agent Roles, CLI registration, MCP, sharing, Messenger, and cache preview; theme was restored to light, no state was saved, and no screenshot files were retained. Run `39495359230` passed all four macOS/Windows validation and native-package jobs. Artifacts: macOS `12298464724`, SHA-256 `2d4897392e12721db809ad7b5678e1939089a6cbefe0006077032071886cb2fe`; Windows `12299926070`, SHA-256 `9b977f603024571c591b78f851d4695d6b3f6053760f94fbb0cb1fc397809bda`. Graphify re-extraction reports 7,708 nodes, 12,192 edges, and 587 communities. This evidence does not increase packaged UI acceptance (6/25) or shipped Rust UI replacement (0/25). Full visual parity and Windows GUI review remain unverified.

## Plan import and manual drafting actions — 2026-10-21

The native Agent Roles stage now exposes the existing “Read plan” and “Write plan manually” actions. Import uses the platform-native file picker on the request worker; both actions first save changed workspace settings, then call the existing `read_plan` or `draft` backend request. The manual-draft integration sends pointer input to the rendered button with an unsaved goal, asserts that the async Save happens first, then verifies a three-task review proposal uses the newly saved goal without starting an agent. Its Drop guard removes the temporary repository on both success and assertion failure. The AccessKit rendering regression failed before the buttons were restored; the focused workspace batch passes 20 tests, source-layout passes 2, and all-target Clippy passes with warnings denied. These controls do not constitute a complete packaged workflow, so acceptance remains 6/25 and shipped Rust UI replacement remains 0/25. Exact-package behavior and the complete cross-platform workflow remain unverified.

## Fixed-slot live activity — main #117 / issue #9

Main reserves one 18px `.task-activity` row, truncates long text, and hides an empty row with visibility so content below does not move. The Rust panel previously returned without a row when idle and rendered up to three separate labels while active. A rendered-position regression failed first: the following content moved from y=111 to y=168. The fix combines available localized facts into one clipped 18px row and reserves it when no facts exist. The native task-activity suite passes 7/7; source-layout passes 2/2; warning-denied native-feature Clippy, rustfmt, and diff checks pass. This parity fix does not count as a packaged workflow; acceptance remains 8/25 and shipped Rust UI remains 0/25. Exact-package visual verification is still open.
# Stop-during-backoff timing

Main #112 widens the retry backoff in `stop_during_timeout_backoff_prevents_automatic_relaunch` from 1s to 5s and its stop-response bound from 500ms to 2s. This gives the test enough time to observe a stop request under host load without allowing an automatic retry. The Rust migration branch now carries those same values; the focused test passed locally in 5.67s. This is test reliability work only and does not increment packaged native UI acceptance (still 8/25).

## 2026-10-21 Current-main documentation and chat integration — issue #9

The branch now contains main's v0.1.33 release/manual updates while keeping the native Rust UI and Rust-only application/test sources. Both offline manuals were regenerated and all referenced screenshot links restored in README. Full source/test verification passed 828 tests (14 ignored, 278 suites); warning-denied Clippy, formatting, and source-layout checks passed. The chat result snapshot retains the one-shot `chat_result` gate; an upstream duplicate unconditional read was removed and the two focused chat suites pass. This remains source-level evidence, not a new packaged workflow. Packaged acceptance is 8/25, and current-source screenshot parity remains unverified.

## Exact-source macOS package — issue #9 / 2026-10-21

Package source `9be8e880ad96d01d83d826c6ac843e195706861e` passed validation, packaged help, ZIP integrity, and four deterministic shared/worktree × Git/folder smoke cases. All cases ran three parallel tasks/tests, verified approved merge, and refused unapproved merge. Binary SHA-256 `86c72bf263fa32b4affe652b38b7d4d80d8125d754372252ae84509b63993d30`; archive SHA-256 `e82f2a852215413a946ef54829237725e8c2b5b47779171d5ac301f1b2c16916`; evidence in `docs/evidence/issue9-9be8e88-macos-smoke/run-36772-1792567832764762000-0`. Windows exact-source package and visual acceptance remain open; package smoke does not increase 8/25 UI acceptance.

## Later validation and exact-head package — 2026-10-24

The merge-era runs `39999933920`, `40000373810`, and exact-doc-head run `40001727894` were cancelled before complete macOS/Windows validation. Run `40001524380` is associated with code/docs commit `b52e3d75`; its Windows validation job was restarted and is now running, while its macOS job is cancelled. The pushed `8ef672bd` change after `b52e3d75` is documentation-only and passed local public-doc/source-layout tests. An exact `8ef672bd` macOS arm64 package was built offline and passed package validation, ZIP integrity, and all four built-in mock smoke modes. This is local package evidence, not a completed remote run, UI acceptance, or GitHub release. Windows remains unverified. See `docs/evidence/issue9-local-macos-package-8ef672bd-20261024.json` and `docs/evidence/issue9-utm-windows-recheck-20261024.json`.



## Settings gear existing-package spot-check — 2026-10-24

The already-running exact bundle for source `f23381cf` was bound without launching another process. The v0.1.37 screenshot and accessibility tree confirmed the gear-only control named `설정` and Settings menu actions. Its renderer/test are unchanged in the current source. Since saved workspace/profile data was visible, the check stopped before any mutation; it earns no workflow acceptance. See `docs/evidence/issue9-settings-gear-existing-package-20261024.json`. Current-head Actions run `40003966004` is in progress; it is validation only, not acceptance evidence.

## Stop, resume, and recover a run — issue #9 / workflow 25 / 2026-10-29

`tests/package_run_recovery.rs` exercises the Rust executable through a disposable Git repository and three mock tasks. A bounded mock blocks until the coordinator stops it; explicit resume completes the work; a simulated stale owner record then exercises recovery followed by another explicit resume. The candidate diff is asserted to contain exactly the three owned file paths, once each. The exact local binary journey passes. Full offline regression passes 1,103 tests (15 ignored across 350 suites); focused source/layout tests, formatting, and warning-denied all-target Clippy pass. The exact-package Windows and macOS run is pending; this gives no workflow acceptance credit. Evidence: `docs/evidence/issue9-package-stop-resume-recovery-20261029.json`.

## Full regression and current main conflict checkpoint — issue #9 / 2026-10-29

At Rust head `09fa3482daae68f5728e1610eec9b77b632cd736`, the complete command `cargo test --all-targets --locked --offline --features test-fixtures --no-fail-fast` exited successfully, including the coordinator-format-retry regression. Warning-denied all-target Clippy, rustfmt, diff hygiene, and the 300-line source/test guard pass. Authenticated live CLI tests and exact-package acceptance tests are explicitly ignored in this local run and remain separate.

Against fetched `origin/main` `9a8b3258169a0e5834bc141f1bf5941660deb7a1` (merge base `e32295b3bdb0d056adb7e40ec6b04a0adb01957e`), a read-only merge-tree reports 88 conflicted paths. The branch has 1,037 Rust-only and 73 main-only commits, with 1,675 changed paths from the merge base. Conflicts span shared Rust implementation/tests and architecture documents, plus legacy JavaScript UI paths intentionally removed on Rust. PR #22 is still open, draft, and conflicting. No merge, package, or workflow acceptance is claimed. AgentMemory: `mem_mvtee8n7_19ec20ce2bae`.

## Exact macOS package at current Rust head — issue #9 / 2026-10-29

Built `87239f6a68e5b7e4ccc3efc8c0efffc5d11b1407` with `cargo run --release --locked --offline -- package-release`. The package manifest names that exact source commit and its binary SHA matches the extracted executable; `unzip -t` validates the complete archive, and the packaged executable's `--help` exits successfully. Archive SHA-256: `f2200d0db44d6e926505ef0ce806cc936d4bb65644863fee00194d688d03dcc4`; binary SHA-256: `081136f7ab226b91373ee196150b0923f86e11ad210db603eb7c51103566888b`. Evidence: `docs/evidence/issue9-macos-package-87239f6-20261029.json`. No GUI launch or screenshot was captured; Windows exact-package verification and all acceptance credit remain pending.
