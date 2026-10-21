# Current-main behavior carried into the Rust UI branch

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

## Visual evidence status

The checked-in Workspace image is a historical Rust capture from `392aa20`, not a current-main screenshot. The exact `cf92f39` v0.1.32 package was visually checked in one live Workspace window, but no screenshot artifact was retained. No current-main screenshot was retained or reviewed in this check. Do not claim current-main screenshot parity from the checked-in images or this component check. Packaged workflow acceptance remains 6/25, and shipped/default Rust UI replacement remains 0/25.

## Provenance

The behaviors were compared against origin/main v0.1.32 commit 3123ece. Cargo.toml and Cargo.lock changed only for the v0.1.32 package version; this port adds no external dependency or license obligation. Graphify's code-only extraction contains 7,699 nodes and 13,223 edges. CocoIndex could not run because its configured Python environment lacks mcp.server.fastmcp. The integration is tracked by GitHub issue #9 and its draft PR #22.

## Exact-head CI regression and package evidence — 2026-10-21

CI run `39488170157` exposed two cross-platform native UI test assumptions. The workspace action test used the Save button's left edge, which varies with layout/font metrics; it now checks the right edge of the complete right-aligned action group. The settings journey waited for a duplicate heading absent from the accessibility tree; it now waits for the stable MCP dialog title and verifies the Save control. Both focused regressions pass locally; the native-feature all-target suite passes 801 tests with 14 ignored across 271 suites. Exact pushed source `ae2540c496fde4a25be527a468d84114961265a6` has a verified macOS package and four approval-gated Git/folder × shared/worktree smoke passes. In an isolated temporary HOME, the package UI was inspected on Workspace (light/dark), Agent Roles, CLI registration, MCP, sharing, Messenger, and cache preview; theme was restored to light, no state was saved, and no screenshot files were retained. Run `39495359230` passed all four macOS/Windows validation and native-package jobs. Artifacts: macOS `12298464724`, SHA-256 `2d4897392e12721db809ad7b5678e1939089a6cbefe0006077032071886cb2fe`; Windows `12299926070`, SHA-256 `9b977f603024571c591b78f851d4695d6b3f6053760f94fbb0cb1fc397809bda`. Graphify re-extraction reports 7,708 nodes, 12,192 edges, and 587 communities. This evidence does not increase packaged UI acceptance (6/25) or shipped Rust UI replacement (0/25). Full visual parity and Windows GUI review remain unverified.

## Plan import and manual drafting actions — 2026-10-21

The native Agent Roles stage now exposes the existing “Read plan” and “Write plan manually” actions. Import uses the platform-native file picker on the request worker; both actions first save changed workspace settings, then call the existing `read_plan` or `draft` backend request. The manual-draft integration sends pointer input to the rendered button with an unsaved goal, asserts that the async Save happens first, then verifies a three-task review proposal uses the newly saved goal without starting an agent. Its Drop guard removes the temporary repository on both success and assertion failure. The AccessKit rendering regression failed before the buttons were restored; the focused workspace batch passes 20 tests, source-layout passes 2, and all-target Clippy passes with warnings denied. These controls do not constitute a complete packaged workflow, so acceptance remains 6/25 and shipped Rust UI replacement remains 0/25. Exact-package behavior and the complete cross-platform workflow remain unverified.
