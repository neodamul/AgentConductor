# Current-main behavior carried into the Rust UI branch

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
