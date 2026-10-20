# Native Rust UI migration

## Finalization progress parity — GitHub #50 / issue #9

The default JavaScript implementation was merged to `main` in PR #51. The Rust-native issue #9 run board now carries the same Verify → Merge → Workspace progression as an additional parity change: it derives active merge state from the existing job snapshot, keeps the apply explanation and busy action in the primary status banner, and disables duplicate merge requests. Regression coverage checks the rendered stages and Verify → Merge → Workspace transitions. Commit `b81caa5` plus the test-gate correction passed all-target serialized tests, 17 focused UI tests, the native feature-gate guard, Clippy, formatting, diff checks, and the changed-file 300-line check. The first full run caught a missing `native-ui-probe` gate on the new integration test; the corrected rerun passed.

This component-level Rust parity is not a packaged workflow acceptance. Exact packaged UI review and end-to-end cross-platform workflows remain open; verified acceptance is still `0/25`.

## Exact-source macOS package — issue #9 / `ca17284`

The optimized 17,627,872-byte native executable from source `ca17284b68e531bfb16476981d80844b53bbaab` has SHA-256 `0652485f9bbf9d2964e50b9b7585194889898f94acb2f21b7d235f8d31aed85a`. The exact package contains the bilingual Markdown/HTML manuals, README, design guide, MCP reference, assets, SBOM, and 214 reviewed dependency licenses. Package smoke exited 0 for shared and worktree modes on both Git repositories and plain folders; every case ran three parallel mock tasks and tests and verified both refusal before approval and successful merge after approval. Archive SHA-256: `786e83c54de8194bbda019f96aaac62d70a5583498638ad3a0b535254d5de933`. The evidence summary is tracked at `docs/evidence/ca17284-macos-package.json`.

This is package/CLI evidence on macOS, not visual acceptance of the desktop UI or Windows package. Cross-platform native UI acceptance remains `0/25`.

Status: active migration. Published v0.1.24 still uses WebView; the issue branch defaults to the Rust eframe UI and has removed project-owned JavaScript/Python source and test files. Exact macOS/Windows package acceptance and final native hardware checks remain open. The serial full Rust suite passes: 719 passed, 14 ignored, across 247 test targets (2026-10-20).

## Exact-source macOS package — issue #9 / `fd177ea` (2026-10-20)

The optimized Rust package built from exact source `fd177ea145326407d485d7f18ccd1d916f1fc00e`. `package-release` completed all four isolated Git/folder × shared/worktree smoke cases; each used three parallel mock tasks and tests and verified that unapproved merge is refused while approved merge succeeds. `validate-package` confirms 214 dependency license records, binary SHA-256 `6bd00e6f520e0172a0b1f912c15fe558695d4a3a6c2d80477395b9da7cae6c88`, and archive SHA-256 `b4d17e6b8ecb152e4056aa2e39d1cabe4e9f7880d8398435c3bb2d132578e6e5`. One isolated package window showed the native empty Workspace and was closed. This is local macOS packaging evidence, not a release or full UI acceptance; #9 remains 0/25 and Windows acceptance remains open. The serial all-target suite subsequently passed 719 tests with 14 ignored across 247 targets.

## Reviewing-phase heading — issue #9

In v0.1.30, a phase proposal changes the phase panel heading from “Next” to “Reviewing phase n/N”; completed stale proposals are ignored and the next phase remains available. Rust continued to label a phase as next even when its proposal was under review. A rendered regression failed on that contradiction. Native Rust now uses the localized reviewing heading and marks any non-completed proposed phase as under review. Two goal-list tests, nine phase-progress tests, the native approval/start journey, source-layout, formatting, and warning-denied native-feature Clippy pass. Exact packaged UI acceptance remains open; end-to-end acceptance is 0/25.

## Goal-list identity and stage — issue #9

The v0.1.30 goal list identifies each saved goal with its slug and localized stage, shows the goal description, and gives inactive goals a separate Select action. Native Rust previously displayed only the active goal's description and rendered inactive descriptions as buttons. A rendered regression failed on the omitted identity/stage; Rust now shows those fields and preserves the existing revision-bound selection event. The goal-list and nine phase-progress tests, native approval/start journey, review copy, source-layout, formatting, and warning-denied native-feature Clippy pass locally. Exact package layout/accessibility and full workflow acceptance remain open (0/25).

## Preserve warnings when progress has no phases — issue #9 / DD-63

`slug_progress::progress()` can return `phases: []` plus a mismatch when `plan/plan.json` is unreadable. The native Goals & Progress card previously returned before rendering, hiding the integrity warning. A rendered empty-phase regression failed; the card now remains visible when warnings exist and avoids falsely reporting that zero phases are complete. All nine phase-progress tests, the source-layout guard, formatting, and warning-denied native-feature Clippy pass. Exact package appearance and accessibility remain open; acceptance remains 0/25.

## Goal-progress mismatch warnings — issue #9 / DD-63

Legacy `renderGoals()` displays each mismatch derived from phase/task files below the goal. Native Rust omitted these integrity warnings. A rendered regression failed on the missing text; the native card now renders every mismatch in the warning color with a visible marker. All eight phase-progress tests, review-copy, source-layout, formatting, and warning-denied native-feature Clippy pass locally. Exact packaged macOS/Windows appearance and accessibility checks remain open; workflow acceptance stays 0/25.

## Coordinator analysis disclosure — issue #9

The shipped v0.1.30 phase panel shows the coordinator's whole-goal analysis in a collapsed disclosure before the phase list. Rust omitted it. A rendered regression failed on the missing localized disclosure; native Rust now reads the saved plan analysis and reveals it through the existing localized collapsible control. All seven phase-progress tests, the native approval/start journey, three parallel mock tasks through shared/worktree approval and merge, review-copy/source-layout checks, formatting, and warning-denied native-feature Clippy pass locally. Exact package and keyboard/accessibility checks remain open; end-to-end acceptance remains 0/25.

## Workspace phase briefs and task details — issue #9

The v0.1.30 Workspace expands the current or upcoming phase by default and exposes its brief plus planned task ID, first-line instruction summary, assigned agent, and task state. The Rust Goals & Progress card previously showed phase names and aggregate counts only. Rendered regressions failed for the missing brief/task details and verified that a phase under review keeps the existing editing notice rather than duplicating its editable task list. Six phase-progress tests, the native approval/start UI journey, and the three-task shared/worktree approval-and-merge workflow pass locally. The UI journey test now follows the rendered accessible task headings and summaries and scrolls to offscreen controls. Exact packaged macOS/Windows and keyboard/screen-reader checks remain open; this does not advance end-to-end acceptance from 0/25.

## Review Plan task-role badges — R-47 / UC-39 / UC-106

The released Review Plan task summary includes a role badge when a task has a role, lays out its two-digit task ID/role above the instruction summary, and labels its removal control `Remove task`. The native card omitted the role, combined the ID and summary into one line, and shortened the action to `Remove`. Rendered regressions caught each difference; Rust now restores the two-line hierarchy, localized role/remove labels (including the special `knowledge_assetization` key), raw-role fallback, and trailing ellipsis after the shipped 160-character limit. AccessKit confirms the role badge remains inside a 480px review viewport. Exact packaged macOS/Windows comparison and the 0/25 workflow acceptance remain open.

## Review Plan snapshot-base cue — R-21 / UC-15

The shipped v0.1.30 Review Plan displays the first 12 characters of the plan's captured repository base and prompts the operator to inspect executables, permission modes, scope, and tests before starting. Native Rust previously showed this only after opening exact execution consent. An AccessKit regression failed because the cue was missing from Review Plan; the native view now uses the existing localized warning and snapshot base before the task list. The consent dialog still presents its full repository-base evidence. Five Review Plan unit tests and ten focused task-card/edit/review-copy/source-layout tests pass, along with warning-denied Clippy and formatting. Packaged macOS/Windows confirmation remains open; end-to-end acceptance stays 0/25.

## Current acceptance checkpoint — 2026-10-20

The native UI source-coverage inventory remains **25/25 workflows implemented (100%)**, **0 partial**, and **0 not started**. End-to-end operator acceptance is now **1/25 (4%)**: the exact v0.1.30 macOS package completed a disposable shared-folder journey with three parallel mock sessions, passing tests, human review and merge approval, and a clean completed repository. Evidence: [`issue9-35a5c09-native-workflow-01.json`](../../docs/evidence/issue9-35a5c09-native-workflow-01.json). This does not verify broad visual parity, Windows UI, authenticated provider CLIs, accessibility/IME, or the remaining 24 journeys. The shipped Rust replacement remains 0/25 until a release ships. Keep implementation coverage, verified workflow acceptance, and shipped replacement as separate metrics.

## Historical acceptance checkpoint — 2026-10-19

The native UI source-coverage inventory marks **25/25 workflows implemented (100%)**, **0 partial**, and **0 not started**. This does not establish cross-platform end-to-end parity. Source/control presence does not prove functional parity: verified native end-to-end parity remains **0/25 (0%)**, and the shipped Rust replacement remains **0/25 (0%)**. Published v0.1.24 still uses WebView; exact packaged Windows, full accessibility/IME, and complete cross-platform workflow acceptance remain open. Component tests and package smoke checks do not count as accepted end-to-end workflows. Keep these metrics separate.

## Exact-head platform and package verification — `de95fc5` (2026-10-20)

Run `39271534953` attempt 2 passed every macOS and Windows platform-validation step. Windows release compilation, Rust-rendered bilingual manuals, exact package execution/smoke, and artifact upload also passed; the uploaded Windows package artifact has SHA-256 `0bc32e45402eb7a87aff17a8fa5f21018f42862158e65f7256d35b1a4a50696d`. The macOS native-package job was cancelled while queued without starting any step, so no exact-head macOS release-package result is claimed. The platform timeout correction is verified; this evidence does not advance native UI acceptance from 0/25. Exact current app screenshots, full native UI workflow comparison, and live IME/accessibility checks remain open.

## Clickable native release-notes link — issue #9 (2026-10-20)

The native Updates panel previously rendered “Release notes” as a button that copied the URL to the clipboard, even though it appeared to be a link. Regressions first failed because there was no AccessKit link node and an arbitrary external origin could be linked. The Rust view now uses egui's native hyperlink only for `https://github.com/neodamul/AgentConductor/releases/` and retains its localized accessible name plus the existing keyboard-focusable, read-only, bounded release-body view. Local verification passed `native_ui_updates` (7/7), `native_ui_update_links` (2/2), `source_layout` (1/1), formatting, diff checks, and focused Clippy-driver checking with warnings denied. The exact guarded Rust source was rebuilt (binary SHA-256 `9582c6cf9396c2b1baf20a4c8c7a794d861a4e78fbdc6e2326904fc95ac5cc4c`) and opened in one isolated macOS app window; Settings → Updates displayed the actual v0.1.30 release notes and trusted accessible link, and the check resolved to v0.1.30. The app and isolated HOME/bundle were closed and removed without opening the browser link or changing workspace data. Exact packaged browser activation, keyboard traversal, and scrolling on both platforms remain unverified; overall acceptance remains 0/25.

## Exact-head Windows validation attempt 2 — 2026-10-19

For run `39265015072`, macOS validation passed every step. Windows build, the source/test layout guard, and all applicable tests passed. The 20-minute job limit then cancelled the opt-in Rust UI compile shortly after it started; formatting and Clippy therefore did not run, and the native-package jobs were skipped. The platform-validation job timeout is raised to 45 minutes for the exact-head rerun. This is CI-duration remediation only and does not change the 0/25 end-to-end UI acceptance count. Local Rust compilation remains unavailable on this host because Homebrew LLVM requests the missing `libz3.5.1.dylib`.

## Rust-only source and layout checkpoint — 2026-10-19

On issue branch `codex/issue-9-native-ui-parity` at `a11848e`, `git ls-files` finds no project-owned Python, JavaScript, or TypeScript source/test files. The historical estimate of 19 Python contract modules is stale. The locked offline `source_layout` integration test passes 1/1 and verifies source/test separation plus the 300-line per-file limit. This is repository hygiene evidence only; it does not advance packaged UI acceptance from 0/25.

## Agent Room goal context — 2026-10-19

The active `report.plan.goal` now appears under the localized `CURRENT GOAL` label above the run-status banner, matching the released Agent Room's information order. AccessKit layout coverage failed before the header was added and passes with the full goal exposed before status. The optimized prototype was not involved: the exact locally built Rust app bundle was replaced in place, then one isolated fixture window showed the goal and its three completed tasks while the candidate remained unapproved and unmerged. No subscription CLI or user workspace was used. Graphify re-extracted the code graph to 6,720 nodes / 11,655 edges. CocoIndex Code remains unavailable because its installed environment lacks `mcp.server.fastmcp`; no dependency was added. This corrects one visible omission; packaged/native workflow acceptance remains 0/25.

## Agent Room integration verification summary — 2026-10-19

Restored the released Agent Room's integration-verification cue below the native session list. The Rust view derives passing count and failed/running/completed status from the existing report test records, and renders only the summary; stdout/stderr paths remain private. An egui render regression failed before the panel existed and passes with the localized summary. Full offline all-target tests and Clippy pass, and the exact rebuilt isolated macOS app's AccessKit tree shows the summary for three passing task tests. Graphify now records 6,736 nodes / 11,677 edges. This is one visible-parity slice, not end-to-end acceptance; issue #9 remains 0/25 and open.

## Agent Room Human Review rail — 2026-10-19

Restore the compact left-rail Human Review section with a Review candidate link, pending tool-request count, and current report status. The link selects the Review tab; it requests evidence only if the cached revision is missing or stale, and never approves or merges. The red-first egui pointer/AccessKit regression passes, together with 22 focused tests, the source-layout guard, all-target Clippy, rustfmt, and a local binary build. The full offline all-target suite also passes after commit `2ec925b`. An isolated instance of that build's actual Agent Room reports the task-test summary and exposes `HUMAN REVIEW`, `Review candidate`, `Tool requests · 0`, and `Waiting approval` in AccessKit. Graphify has 6,747 nodes / 11,693 edges. A screenshot could not be captured from this display, so layout pixels are not visually evidenced. This is a parity slice, not end-to-end acceptance; issue #9 remains open at 0/25 until cross-platform workflows are accepted.

## Native sidebar provider icons — 2026-10-18

The Rust sidebar now renders bounded provider artwork from the existing native icon
snapshot contract. A rendered-widget regression failed before the icon texture path
was added and passes with it. An optimized macOS build showed Claude and Codex icons
with isolated fake CLI profiles; the custom profile retained its monogram. This does
not verify Windows discovery/rendering or alter the 0/25 workflow acceptance count.
The committed executable SHA-256 matches the visually inspected build; package smoke
passed shared and worktree modes against Git and ordinary folders, including approved
merge and rejected unapproved merge. See [runtime icon evidence](runtime-icons.md).

## Startup screen parity checkpoint — 2026-10-18

Compared the pre-migration startup layout at `ce5b58c` with the exact Rust app. The Rust startup flow now retains the same shared goal headline and three-stage tabs, the Workspace settings card, the five-column role table, Generate Plan on Agent roles, and a Review Plan card with an explicit empty-plan state. The header and role-table assertions were added red-first, then passed after the layout was restored. The three stages were inspected at a 1140×772 macOS viewport without changing the saved workspace. The reviewed-plan state, live agent operations, full accessibility/platform acceptance, and remaining operator workflows are still pending; this checkpoint does not change verified workflow progress from 0/25.

## Equal-width Workspace stage tabs — 2026-10-19

Legacy `v0.1.21` uses three equal-width stage tabs across the Workspace content column. A rendered-bounds regression reproduced unequal Rust tab widths at 480px (`142/178/205px`). The Rust header now allocates the content width evenly, wraps long labels inside their cells, and draws a full-width selected underline; localized button names and click handling remain unchanged. The native-header and 300-line source-layout suites pass (4 tests), along with rustfmt and diff checks. Graphify re-extracted the issue branch at 7,021 nodes / 12,186 edges. CocoIndex Code is installed but could not start because its environment lacks `mcp.server.fastmcp`; no dependency was added. Windows/macOS package visuals have not been re-inspected, so this component correction does not advance workflow acceptance from 0/25.

## CI-safe mock start journey wait — 2026-10-19

Platform run `39256513641` showed the mock plan-to-Agent-Room test can take 6.13 seconds on a hosted macOS runner, exceeding its five-second final UI-state wait. Only the post-start transition now has a 15-second deadline; immediate UI-state checks remain at five seconds. The source/test line guard caught the test file growing to 307 lines, so the reusable wait loop was moved to `tests/native_ui_start_journey/ui_wait.rs`. Local start-journey, header, and source-layout suites pass (5 tests). This only stabilizes CI timing; it does not change the app's runtime timeout or advance packaged workflow acceptance from 0/25.

## Shared Workspace page eyebrow — 2026-10-19

Legacy `startup.html` presents `Prepare a goal` above the stage tabs on all three screens, with the selected section's stage eyebrow appearing inside its card. The Rust page header instead repeated `STEP 1/2/3` above the heading while the active card repeated the same label. A bilingual rendered regression failed on English step 1 because the shared page eyebrow was absent. The header now uses the existing localized `startup.prepare_a_goal` entry; four focused header/source-layout tests, focused Clippy, rustfmt, diff checks and the 300-line limit pass. Graphify re-extracted the updated branch at 7,022 nodes / 12,187 edges. The checked-in screenshot has not yet been refreshed from a visually reviewed exact Rust build; workflow acceptance remains 0/25.

## Accessibility probe after restoring the shared Workspace eyebrow — 2026-10-19

Exact-head Platform run `39259634447` confirmed the new rendered header shows the step label once, but the older `native_probe_controls_have_accessible_names` test still expected the duplicated page/card label count of two. Its failure is a stale assertion, not a lost accessible name. The test now expects one rendered step label. macOS build and source-layout checks passed before the stale assertion failed; Windows build was still in progress at diagnosis. Rerun the exact-head platform checks after this test-only correction. The screenshot/manual refresh and end-to-end acceptance remain open at 0/25.

## Exact linked-branch preview — 2026-10-18

Issue #9's development branch was created from the issue's Development section.
After its PR was opened, GitHub removed the branch from the linked-branch list and
shows the PR there instead; an empty `gh issue develop --list 9` is expected at this
stage. The current draft PR is #22 with head `codex/issue-9-native-ui-parity` and
base `main` ([GitHub behavior](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-a-branch-for-an-issue)).

Built source HEAD `0c5b7d9` as a macOS release binary (SHA-256
`78f835c8b6766c3f6367ce7838951aff88c5c8e806131d68a70bfee63db31ef9`). Stopped and
verified exit of the previous preview process before replacing the bundled executable;
the copied executable hash matched, and only one app window was relaunched. At 1140×772,
inspected Workspace, Agent roles, and the empty Review Plan states. The saved workspace
values remained untouched and no generation request was sent. These three screen
observations do not establish all 25 workflows, the generated-plan state, Windows parity,
live IME, or release readiness. Public README/manual screenshots were not replaced.
The same exact build's Settings menu, MCP JSONC editor, messenger form, and cache preview
were also inspected through AccessKit and screenshots. No settings were saved and no
cache was cleared; the preview listed 8 removable candidates (0.0 MiB) and 4 protected or
review-required items. Update details, other agent-profile/quota actions, generated-plan/live
Agent Room states, and platform-specific controls remain unverified.

### Runtime quota refresh — 2026-10-18

On the same packaged binary, opened and cancelled the Claude profile editor without
saving. Codex quota refresh displayed progress immediately and then reported 65%/79%
(5-hour/weekly remaining). Claude refresh also displayed progress immediately, then
reported the CLI-observed weekly 0% and budget-wait status. No generation request was
sent. This verifies asynchronous refresh for these two configured profiles only; it
does not establish quota parity for other providers or release acceptance.

## Transactional registration checkpoint — 2026-10-18

R-48/UC-40/DD-56: sidebar **+ Add** now opens the profile modal directly,
starting with the same custom `aipro` default as shipped v0.1.24.
Keep Add/Edit drafts in the editor and send a separate candidate registry to the
existing revision-bound `save_profiles` worker. Only confirmed persistence updates
the live list and coordinator/role renames. Rejection keeps the editor and refreshes
the authoritative registry/revision for retry; it does not save the draft goal.
The redundant Team registration menu is removed.

Five UI-to-Room tests cover Add, Cancel, save/reopen, stale-revision retry and
coordinator renaming. Three editor boundary tests cover capacity/duplicate IDs,
provider switching and concurrent removal. Actual macOS interaction verified
Cancel, custom-profile saving, the saved notice, first-agent coordinator selection,
and retained unsaved name/goal. Native inspection also found an untranslated
None value and an invisible modal heading; a two-language/two-theme rendered
regression now covers the corrected title and None value. Final native follow-up
and full-suite evidence are recorded in the raw registration audit.
This fixes part of workflow 3; it does not establish full visual or cross-platform
parity, and verified replacement progress remains 0/25.

### QAS-124 — agent-card quota contrast

The compact quota percentage must use the active theme's readable text color.
The sidebar's strong monospace label defaulted to white on the light surface; a
rendered-widget test caught the 1:1 contrast failure. The sidebar now explicitly
uses `ui.visuals().text_color()`. The focused light/dark widget regression passes;
full native workflow parity remains 0/25.

### Exact-source macOS package — 2026-10-18

Commit `2996dc6af777dfe2b511d207e02e83ffdabf81c6` builds a 17,411,584-byte
`aarch64-apple-darwin` binary. Package validation confirms its source commit and
SHA-256; the archive includes the README, both HTML manuals, dependency license
texts, and CycloneDX SBOM for 214 dependencies. The archive SHA-256 is
`c97f3e174336abce9de48e4607800b16c4205c318ae1972998312942915be91d`. Packaged
smoke passes Git and non-Git folder projects in shared and worktree modes, with
three tasks/tests and approval required for merge. This is a local package only;
Windows, final UI acceptance, and release publication remain open.

### Workspace keyboard acceptance — 2026-10-18

On the exact packaged Rust binary (SHA-256 `c4618e60f777de0f96e21c69864fbe34267686a70f771eebf435eb921cc36d7f`), a clean isolated Workspace screen was traversed with Tab through every enabled control. Disabled coordinator and import-preview controls were skipped. Space changed the isolation radio selection; the temporary Git worktree selection was returned to Shared without saving. This covers Workspace keyboard traversal only. A public screenshot could not be saved: macOS `screencapture` returned `could not create image from display`; the CUA screenshot was visually inspected but its bytes were not written to the repository. Public screenshots and the bilingual manuals therefore remain pending.

### Native Workspace visual review — 2026-10-19

The source-build Rust executable at `2ec925b` was opened in one temporary Workspace window. CUA screenshots of the actual window were inspected in Korean light and dark themes at 1140×772, and the AccessKit tree exposed the corresponding fields and controls. The empty Workspace form, step navigation, Jira/Confluence reference buttons, isolation radios, save action, and settings import/export stayed inside the viewport and readable in both themes. This verifies only the empty Workspace surface; populated agent cards, profile/MCP dialogs, Agent Room pixels, keyboard/IME, Windows, and end-to-end workflows remain unverified. The CUA screenshots were not persisted as repository assets, so README/manual images remain unchanged and issue #9 remains 0/25. Graphify refreshed to 6,749 nodes / 10,780 edges / 527 communities. CocoIndex CLI is unavailable in this host.

## Historical source checkpoint — 2026-10-17

- The 25 operator workflows have native source-build implementations. This is prototype parity, not release conversion; v0.1.24 remains unchanged.
- The 300-line source-layout guard passes across Rust sources and tests. Legacy browser assets and obsolete JavaScript/Python verification were removed; subprocess fixtures are Rust.
- Local verification: 596 tests passed, 14 ignored, 0 failed; all-target check and Clippy with warnings denied pass; rustfmt and diff checks pass.
- The optimized macOS executable builds and its `--help` CLI smoke exits successfully without opening a desktop window.
- Not verified: exact macOS/Windows release packages, a Windows Rust target on this host, final screenshot review, Korean IME, accessibility on real windows, and idle resource measurements. Keep issue #9 open until those checks pass.

## Architecture boundary

Keep Room, Startup, Controller, validation, approval decisions, and persisted state in Rust. A native desktop view calls the existing Room::dispatch boundary through a bounded background request worker; process and filesystem waits stay off the UI frame thread. Preserve separate Workspace and Agent Room windows and the existing user-visible workflows.

Cargo enables `native-ui-probe` by default, so the actual `ui-new` and `ui` commands open the shared eframe launcher in a source build. The WebView shell and project-owned browser assets have been removed from the issue branch. The published v0.1.24 package is unchanged. Do not publish the new UI until exact cross-platform package acceptance, accessibility, keyboard/IME behavior, visual layout, idle resources, window lifecycle, and dependency-license checks pass.

For GitHub #50, the default WebView behavior was merged to `main` in PR #51. PR #55 carries the same finalization feedback into the optional Rust UI as supplemental parity; do not report the issue as Rust-only or treat the Rust port as the shipped default.

## Production launch-path pilot

The `ui-new` and `ui` commands still identify their `Workspace` or `AgentRoom` surface and pass the existing `Room` into one shared eframe launcher. Cargo's default feature now selects this path without creating another controller. A local macOS build launched successfully. Exact macOS/Windows package UI acceptance, screenshot review, Korean IME, accessibility, idle resource comparison, and license/resource review are still pending.

## UI conversion denominator

Measure native UI development against the 25 end-to-end operator workflows below, drawn from the published Workspace, Agent Room, settings, and help surfaces. Each workflow counts once and is accepted only after its native controls and complete result are verified against the released behavior, design guide, and platform requirements. Partial and unverified workflows receive no credit. Current accepted parity is 0/25. The table is a historical source implementation inventory; its entries do not establish parity. Published release conversion also remains 0/25.

| # | Operator workflow | Historical source inventory (acceptance pending) |
|---:|---|---|
| 1 | Save workspace name, goal, repository, and isolation mode | Complete |
| 2 | Browse for a repository folder | Complete |
| 3 | Register, edit, and remove agent profiles | Partial on audit — transactional Add/Edit save corrected; complete registry/UI/platform acceptance pending |
| 4 | Assign roles and choose a coordinator | Complete |
| 5 | Discover models and select role models | Complete |
| 6 | Generate a plan and inspect live progress/failure logs | Complete |
| 7 | Edit a plan or write task instructions manually | Complete |
| 8 | Review consent and start execution | Complete |
| 9 | Create, open, switch, and delete workspaces | Complete |
| 10 | Import and export workspace settings | Complete |
| 11 | Change language and theme | Complete — language and theme persist; Workspace, Agent Room, status, quota roster and accessibility labels use English/Korean catalogs |
| 12 | Configure MCP servers and import referenced sources | Complete |
| 13 | Configure and test messenger delivery | Complete |
| 14 | Preview and clear local cache | Complete |
| 15 | Open manuals, installation guidance, and release updates | Complete |
| 16 | Inspect run/task status and agent activity lights | Complete |
| 17 | Inspect and refresh per-agent quota | Complete |
| 18 | Send task feedback and reassign work | Complete |
| 19 | Select a task session and model | Complete |
| 20 | Schedule work after an availability restriction | Complete |
| 21 | Inspect task activity, logs, and lessons | Complete |
| 22 | Read a factual team briefing | Complete |
| 23 | Allow, deny, or refresh a tool request | Complete |
| 24 | Inspect diff/tests and approve, reject, or merge a candidate | Complete |
| 25 | Stop, resume, or recover a run | Complete |

Update this table after each completed parity slice; do not report the prototype percentage as release conversion.

## Agent card ordering correction — R-102 / UC-92 / DD-110

Evidence: [native UI ordering audit](../raw/2026-10-18-native-ui-ordering-audit.md).

The old UI exposed drag handles and accessible move arrows, but the native rail omitted both. The native implementation now emits the same bounded move action from either arrow clicks or a typed egui drag payload. It moves the existing profile without reconstructing it and uses the existing revision-bound `save_profiles` request on the background worker. Coordinator, roles, fallback references, private environment and observed catalogs remain attached to their existing profiles. The first Up and last Down controls, and all reorder controls during profile edits/pending work, are disabled.

`tests/native_ui_agent_order.rs` reproduces the missing action in Red and now covers English/Korean boundary labels, actual arrow/drag pointer events, disabled controls, saved order after reopening, reference/secret retention, invalid moves, and stale-revision refusal. Local widget and dispatcher evidence is not native Windows or complete operator acceptance. Graphify refreshed the local AST graph to 6,247 nodes/10,839 edges; CocoIndex could not start because its installed environment lacks `mcp.server.fastmcp`.

Native-window inspection then caught defects that synthetic checks had missed: overflowing framed tabs, 0px input strokes with the expected color, a missing font icon, and an ineffective label-based drag handle. Dedicated egui drag sources, vector icons, real stroke widths and bounded/wrapping tabs now pass 20 focused regressions. In an isolated macOS window, actual dragging and process restart retained gamma/alpha/beta and coordinator beta. This evidence is limited to that surface; the full replacement remains unaccepted.

The audit also exposed a shared persistence defect: comparing serialized profile arrays treated presentation order as changed execution settings and discarded the reviewed proposal. The execution view now sorts its clone by agent ID, leaving the stored display order intact. `tests/model_catalogs.rs` first reproduced `ready` becoming `null`; after the fix it verifies the same proposal digest after reorder/reopen and invalidation after a real argv change. The six affected persistence/consent test targets pass 24 tests. Registration's Add → editor → immediate-save path remains a separate outstanding correction under R-48/UC-40.

## Verified prototype progress

- The native Review screen now edits bounded task plans (1–12): task ID, registered agent, observed session model, instructions, owned paths, dependencies, and JSON argv test command. Assignment/model choices come from registered agent profiles. The editor refuses generation/start while drafts are dirty, validates the complete typed plan before saving, binds edits to the visible revision and plan digest, and requires review of the newly digested plan before execution consent. The Rust dispatcher rechecks the submitted digest against the persisted `review.json` before replacing it. Local verification: 54 tests across six native UI suites, all-target feature Clippy with warnings denied, source layout, rustfmt, and diff checks pass; the largest changed file is 496 lines. Prototype parity is 13/25 (52%), one partial and 11 not started; release conversion remains 0/25.
- After exact-plan consent, the native Review surface now shows “Starting agent execution…” while the start request and report are pending. It polls asynchronously and opens Agent Room only when `run_available` refers to the exact run path returned by the approved start; an older run's existing report cannot trigger the transition. Agent Room receives its snapshot through the existing `Room::dispatch` boundary. A real dispatcher integration test checks the approved run path, open-run route, and resulting report snapshot. Local validation passed 57 native UI tests, 11 compatibility tests, feature Clippy (`-D warnings`), source layout, formatting, and diff checks. This remains an opt-in prototype; GitHub bug #16 stays open until the production Rust UI replaces the shipped WebView.
- The native plan-review panel now uses the saved English/Korean catalogs for its heading, connection state, coordinator activity, elapsed/output details, fallback error, logs, generation action, and save guidance. Agent-provided error text, coordinator IDs, and log paths remain untouched. TDD catalog/interpolation tests pass, as do 202 tests across the native-feature all-target run (4 ignored) with the separate source-layout guard excluded, all-target Clippy, formatting, and diff checks. The guard now reports 35 files above the 300-line limit. Workflow 11 remains partial, prototype parity remains 24/25, and shipped/default conversion remains 0/25.
- Workspace navigation now localizes its saved-workspace list, create/open/archive actions, confirmation prompts, and dirty-workspace loss warnings. The new workspace name field has a localized accessible name; archive wording continues to state that project source files are preserved. The former 498-line navigation module is split into the request/state module and bounded view, confirmation, context, and helper modules. Fourteen focused navigation/i18n tests and native-feature all-target Clippy pass; the strict guard now reports 35 remaining oversized files. Workflow 11 and parity remain unchanged at 24/25 prototype and 0/25 release/default conversion.
- When the task-plan draft differs from the reviewed plan, the native Review surface now shows “Plan changes need review” and an accent-filled “Save changed plan and review again” button. The next step is clear, and the save action remains disabled when the draft is unchanged or a save is pending. Its AccessKit regression verifies both dirty-state labels. This improves prototype feedback only; GitHub bug #15 remains open until this behavior is in the shipped UI. The 25-workflow parity count is unchanged.
- During native plan generation, Review now displays an accessible coordinator activity light using the same pulsing running-state color as Agent Room. It only appears while the plan job is running; the existing 500 ms repaint cadence animates it. A regression verifies the indicator color changes over time and disappears for completed jobs. This remains prototype work; GitHub bug #11 stays open until shipped UI behavior is verified.
- A newly pending tool request now opens an accessible native modal with the tool, task, reason, and exact input. Allow/deny remain two-step, revision-bound decisions through the existing Rust dispatcher. “Decide later” suppresses only that request; any other pending request can then surface, while all receipts stay in the inline list. Regressions cover the popup and per-request deferral; five live Claude callback integration tests pass. GitHub bug #14 remains open until the default shipped UI has the Rust implementation. This refines workflow 23 without changing the 25-workflow completion count.

- ec1339c adds asynchronous repository-folder selection to the Workspace probe. Run 38799761260 passed macOS and Windows validation and exact release-package smoke.
- 8683862 adds a Shared-folder default with optional Git worktree isolation. Run 38804235226 passed macOS and Windows validation and exact release-package smoke.
- d39e173 adds profile registration from the existing Rust templates, collision-free profile IDs (up to the existing 16-profile limit), and coordinator selection. The focused nine-test suite, feature Clippy, and formatting pass locally. Exact-source CI run 38808098099 passed after a failed-job-only retry: macOS and Windows validation, native UI probe, formatting/Clippy, bilingual manual rendering, and both exact release-package smoke jobs passed. The initial Windows UI Automation `NO_HEADING` was transient and did not reproduce.
- A follow-up resource slice removes a full workspace snapshot clone from every egui frame by borrowing the snapshot alongside the editable Team fields. The existing nine tests, feature Clippy, and formatting pass locally. Exact-source run 38812385807 passed macOS and Windows validation, optional probe, formatting/Clippy, bilingual manual rendering, and exact release-package smoke.
- The profile and coordinator save uses the existing Rust dispatcher, which retains profile validation and carries private environment values from stored profiles.

### Workspace planning and role assignment

The opt-in Rust Workspace probe now presents the numbered Workspace settings, Agent roles, and Review the plan steps. The Team step edits coordinator and registered profiles, assigns up to five allowed agents to each existing role, and pins an optional model only from that agent's observed catalog. Saving uses the existing workspace dispatcher and RoleRules validation. The Review step sends a revision-bound plan request asynchronously, refreshes running coordinator progress and output size, and renders the resulting proposal or failure log. Local verification passed 26 focused native UI tests, the 500-line source-layout test, feature Clippy with warnings denied, formatting, and diff checks; the largest changed Rust test file is 492 lines.

This remains an opt-in eframe probe. The production application still uses its WebView UI; this work does not establish production workflow parity, platform UI/IME acceptance, or resource measurements.

### Per-profile runtime preference editor

The opt-in native Team screen can edit a saved agent's executable, JSON argument vector, stdin/file input mode, session cap (1–3), and quota-refresh cadence (1–60 minutes). The executable picker uses the existing asynchronous operating-system picker worker; selecting a path does not launch a CLI. Save sends only the profile registry through the existing revision-bound `save_profiles` operation, preserving unrelated unsaved Workspace drafts and retaining backend validation, derived permission behavior, private environment carry-over, and reviewed proposals when only quota cadence changes. A changed executable clears its observed model catalog so stale models cannot be selected; the existing execution-settings comparison invalidates a reviewed proposal and requires review. The accessible editor is covered by a UI tree regression, a real Room persistence regression, and a regression for executable-change catalog invalidation. Local verification passed 33 focused native UI tests, 2 quota-preference tests, source-layout, feature Clippy with warnings denied, formatting, diff checks, and the 500-line source bound (largest touched file: `workspace.rs`, 377 lines). This remains opt-in and does not complete the shipped UI migration.

### Native provider switching

The native profile editor now offers the observed built-in templates plus the existing custom `aipro` profile shape. Switching providers replaces the profile with the selected template, assigns a collision-free provider-based ID, clears the prior model catalog, and asynchronously resolves the suggested executable through the existing Rust Room dispatch on the bounded worker (no CLI process is launched). The profile-only save sends the old→new ID map; backend `save_profiles` rewrites stored coordinator, role and fallback references, while the native draft rewrites its in-memory references so unsaved role edits survive. Tests cover template replacement, suffix generation, exact rename payloads, local coordinator/role/fallback migration, stale asynchronous path results and the existing backend provider-switch persistence suite. Local verification passed 36 native UI tests, 5 provider-switch tests, source-layout, feature Clippy with warnings denied, formatting, diff checks, and the 500-line source bound (largest touched file: `profile_tools.rs`, 388 lines). The flow remains in the opt-in probe.

### Native agent profile removal

The opt-in Team screen now exposes an accessible trash action with an explicit confirmation step. Removal updates the in-memory profile draft, clears a deleted coordinator (selecting the first remaining profile), removes deleted IDs from role and fallback assignments, cancels an editor targeting that profile, and submits the remaining registry through the existing asynchronous `save_profiles` dispatcher. Removal is disabled while profile editing or that agent's quota/catalog work is active. Local verification passed 12 native profile tests, 5 existing provider-switch persistence/reference tests, default all-target check, feature all-target Clippy with warnings denied, changed-file formatting, diff checks and the 500-line count. CocoIndex indexed 525 files / 5,549 chunks; scoped Graphify extraction contains 714 nodes / 1,458 edges. This completes prototype workflow #3: prototype parity is 15/25 (60%), with 0 partial and 10 not started; the shipped app/release remains 0/25.

### Native workspace lifecycle

The opt-in Workspace screen now lists registered workspaces and supports named creation, opening/switching, removing an entry from the list, and archival deletion after an exact-name confirmation. Operations use the existing asynchronous Room dispatcher and refresh persisted workspace state after create/switch/archive. Backend ownership and busy-state checks remain authoritative; archival preserves the project source folder. Local verification passed five navigation contract/integration tests, four workspace registry tests, eight release-workflow tests, default all-target check, native-feature all-target Clippy with warnings denied, source-layout, formatting, and the 500-line file bound (largest changed navigation file: 439 lines). Exact-source macOS/Windows validation and native-package checks passed for `7bfad6e` in [Actions run 38844468171](https://github.com/neodamul/AgentConductor/actions/runs/38844468171). This completes prototype workflow #9: prototype parity is 16/25 (64%), with 0 partial and 9 not started; the shipped app/release remains 0/25.

### Per-profile model and quota refresh

The native Team step shows observed model counts, existing per-profile quota badges/tooltips, and accessible model/quota refresh actions. A profile with no observed catalog displays the localized “Models not queried” / “모델 미조회” state instead of suggesting that zero models are available. Claude/Codex profiles with no quota observation show an accessible localized unavailable marker, matching the main agent list rather than leaving a blank gap. For paid profiles, the quota value and refresh icon share a compact one-line group with a five-pixel gap; the icon uses the small button style and the model action no longer separates the pair. Requests carry the current startup revision and use the existing Rust metadata queue, which runs the installed authenticated CLI rather than an API-key client. The view refreshes every second only while metadata work is queued/running and returns to five-second idle refresh; it does not poll or redraw continuously while idle. Metadata replies update quota badges and merge a catalog only into the matching saved profile (same ID, parser, and executable), retaining unsaved/new profile drafts. The issue #9 branch also includes the manual custom-CLI catalog editor described below. This remains branch-level native-probe work and does not count as accepted shipped workflow parity or a release.

### Manual custom-CLI model catalog editor (issue #9)

Custom CLI profiles can enter model IDs manually, one per line, or paste a previously observed `ModelCatalog` JSON object. The flow deliberately does not call `/models`, discover endpoints, or require an API key. Saved entries retain operator provenance and use the existing revision-bound local Rust metadata queue; a successful save refreshes the profile snapshot before closing the editor. Input is bounded to 256 KiB, model IDs to 256 characters, and the catalog to 1,024 unique IDs. Focused catalog, sidebar, persistence, profile, localization, layout, and plan tests pass; native-probe Clippy with warnings denied and formatting pass. Physical native-window visual review, Windows package validation, and exact packaged-app persistence remain open acceptance gates.

### Bilingual profile controls and bounded modules

The native Team and profile editor now localize role/profile labels, model and quota refresh actions, removal confirmation, profile fields, and accessible names in English and Korean. The existing `profile_tools::show` entry point remains an English wrapper, while the production Team view passes the selected language through the new view entry point. Profile removal, metadata queueing, validation, save requests, and the existing Rust dispatcher retain their behavior. Focused verification passes 20 tests across five suites; native-feature all-target Clippy passes with warnings denied. The broader all-target run reported 533 passed and 14 ignored across 158 suites before its RTK runner was interrupted after the summary (exit 130). Changed Rust files stay below 300 lines after extracting profile metadata and pending-status helpers. The repository-wide 300-line guard still fails on 34 other files; prototype parity remains 24/25 and shipped/default conversion remains 0/25. Graphify was refreshed with 6,284 nodes, 10,530 edges, and 478 communities.

### Verification migration: localization contracts

The next Rust verification slice preserves duplicate-ID rejection, English/Korean catalog-key and placeholder parity, explicit source references, localized static markup, and removal of the legacy text-matching helper. Keep the live language-switch and user-content preservation regression until the production WebView UI is migrated; moving its Python wrapper alone does not prove the JavaScript UI has been removed.

At the original DD-160 checkpoint, all 5 Rust i18n contract tests passed after removal of the Python catalog module; its 307-line test file met the then-current 500-line cap. R-158 later changed the ceiling to 300 lines, and the current source-layout regression caught the oversized test. Its independent legacy-content assertion now lives in `tests/i18n_legacy_compatibility.rs`: the contract file is 294 lines and the new test file is 14. All five tests pass, while the repository-wide 300-line gate still reports 51 other files. The earlier 515-test and 38 Python-test results belong to that historical checkpoint, not current verification.

Exact-source CI run 38817585058 passed macOS and exposed a Windows timeout in the independent `mcp_remote` SSE fixture: its 3-second per-request limit expired before `discover` completed. The local focused regression passes. Increase only that fixture's deadline while keeping the server stream open until DELETE, so a client waiting for EOF still fails the test.

The prototype covers Workspace identity, repository path, goal, isolation mode, Team registration/coordinator selection, role-agent/model assignment, plan generation/progress/review, plus the separate Agent Room probe slices below. It does not replace the shipped Workspace setup, settings, MCP, approvals, logs, diffs, or production workflow controls.

### Agent Room task-status rendering

`src/native_ui_probe/render.rs` reads the real `Controller::snapshot` envelope (`report.state`, `report.plan`, and `report.tasks`) as well as the startup snapshot (`run_state`, `proposal.plan`) and top-level fixture form. A regression using the real envelope first failed because the renderer only read top-level fields. The view renders the goal/state and each planned task's assigned agent and current status. Status colors are Rust-side: running alternates between two green levels on 500 ms repaint boundaries, waiting approval and blocked are amber, failed is red, completed is green, and pending/unknown are gray. Repaint is scheduled only while a task is running, so an idle view does not continuously redraw. The earlier pulse test caught that a sine wave could repeat at repaint boundaries; half-second color alternation now passes. The native UI suites pass 12 tests; all-target compile, source-layout, focused feature Clippy, rustfmt and diff checks pass. Graphify extraction of the touched renderer/test produced 10 nodes and 13 edges. CocoIndex was unavailable as a CLI or enabled tool, and no project dependency was added.

This slice fixes the snapshot projection only. The probe is still opt-in and currently launches the Workspace path; it has not replaced or been connected to the shipped Agent Room window. Production integration, workflow/accessibility parity and cross-platform UI acceptance remain open.

### Real Agent Room probe path

The existing opt-in binary also accepts `--room <run-dir>`. It opens the exact run with `Room::open`, then `Probe::connect_room` consumes `Controller::snapshot` through the existing bounded worker. The Room surface hides Workspace edit controls and exposes an accessible refresh action. Snapshot refresh is asynchronous: one second for a running job/task and five seconds while idle, matching the shipped Room poll cadence; completed worker replies trigger repaint. This probe path does not call the WebView launcher or construct a second Room/controller; `Room::open` retains its standard recovery/auto-resume policy. Locally, an integration regression opens a real SQLite-backed completed run, checks the actual Room snapshot envelope and verifies the Room-only accessibility controls. The 15 native UI tests, optional probe binary check, default all-target compile, source-layout, feature Clippy and rustfmt pass. This does not yet create the app's production native window or establish macOS/Windows UI/resource acceptance.

Run the opt-in probe against an existing run with `cargo run --features native-ui-probe --bin agentconductor-native-ui-probe -- --room <run-dir>`; the default invocation still opens the Workspace probe.

### Confirmed stop action

The Room probe exposes Stop only while the run is running, ready, waiting for availability/tool approval, or already stopped. The operator must confirm and provide a nonempty reason no longer than 16 KiB. The worker sends the current snapshot revision and stop action through `Room::dispatch`; the Rust coordinator records/processes the request. The UI refreshes the snapshot after acceptance and shows the coordinator message, so acceptance is not represented as an instantaneous process kill. If a refresh shows the run has become terminal, the pending confirmation is cleared. A real SQLite-backed run regression verifies `waiting_availability` transitions to `stopped`; UI contracts cover accessible confirmation controls, reason input, terminal-state hiding/clearing, and empty/oversize reason refusal. Seventeen focused native UI tests, default all-target compile, source-layout, focused Clippy, formatting and diff checks pass locally. Graphify's isolated probe graph now contains 85 nodes and 179 edges. The change remains opt-in; full production UI parity and native platform acceptance remain open.

### Read-only review evidence

The native Room probe can load review evidence on demand through the existing Rust `Controller::evidence` dispatch. It uses the bounded background worker, shows the candidate commit, per-task exit/timeout status, captured test output, truncation notice and exact diff, and rejects evidence whose revision differs from the current snapshot. A later refreshed snapshot clears cached evidence when the revision changes. UI rendering and stale-revision contract tests pass; the existing real `Room::dispatch` diff-review test now also verifies candidate and test output. Local validation passed the default all-target suite (333 passed, 14 ignored), 21 focused native Room/UI/diff tests, feature Clippy, formatting and source-layout. The action is still part of the opt-in probe; production UI migration and native platform acceptance are not complete.

### Candidate approval, rejection, and merge

The native Agent Room now exposes explicit candidate decisions. Approve and merge are enabled only after current-revision evidence has been loaded; each action requires a second confirmation, and rejection asks for a nonempty reason capped at 16 KiB. Requests include the exact candidate commit and current run revision. A state/revision change clears an approval or merge confirmation, and the Rust controller remains authoritative for the approved candidate, diff/test evidence, revision and merge checks. TDD observed both a missing decision request and a stale confirmation that remained actionable after the run revision changed; both regressions now pass. Across five native suites, 41 tests pass with native-feature Clippy, source-layout, formatting and diff checks. Changed source/test files stay within 500 lines (`room.rs` 448; `native_room_render.rs` 392; `native_ui_probe.rs` integration tests 497). Scoped Graphify extraction of the launch/decision slice reports 197 nodes and 383 edges. CocoIndex remains unavailable in this environment. These controls are currently available only in the opt-in native build; the shipped WebView still has additional Room controls, including task reassignment, model/session selection, availability scheduling, tool approvals, activity/log/lesson panels, and team briefing.

### Resume and recover

The native Room now mirrors the shipped Resume state gate and revision-binds the action through the bounded worker. Recover appears only for a running run, requires an explicit confirmation, and is cleared if a fresh snapshot changes the run out of `running`. The Rust recovery implementation still refuses live or unknown recorded processes; the native confirmation does not weaken that guard. UI tests cover the available controls, confirmation state, and state-change cancellation; request tests cover the allowed run states and exact revision. The five native suites now pass 43 tests, feature Clippy, source-layout, formatting, and diff checks. The scoped Graphify graph contains 214 nodes and 408 edges. These remain opt-in pilot screens; full workflow parity and release cutover are still outstanding.

### Human-gated tool requests

The native Room renders the current tool receipt's tool, reason, task, status, and complete bounded input from the Rust snapshot. Pending receipts expose one-time allow, deny, and refresh actions; each requires a second confirmation and sends the exact receipt ID plus current revision through the Rust controller. Approved receipts expose refresh only, matching the shipped WebView. The request builder rejects malformed IDs, missing receipts, ineligible receipts, or incomplete request/context evidence. Active runs can receive a live decision while the worker is paused; idle approvals resume only after the `tool_approve` job succeeds and the refreshed run is ready/stopped. Idle decisions remain disabled while another controller job is active. Stale confirmations are cleared when request status or revision changes. Verification now passes 45 tests across five native suites, all-target feature Clippy, source-layout, formatting and diff checks; largest changed files remain below 500 lines (`room.rs` 461, `room_actions.rs` 334, `native_ui_launch.rs` 235). Scoped Graphify is 220 nodes / 432 edges; CocoIndex is unavailable in the environment. This does not port the Agent Room's other panes, assignment/model/scheduling controls, team briefing, or settings navigation, so the shipped WebView remains the release UI.

### Read-only team briefing

The native Agent Room now requests the existing Rust `briefing` response through its background worker and renders a collapsible, read-only summary. It displays observed session/task/work state, session/model/attempt, process and activity observations, availability restrictions, attention/next action, and recorded approvals. Timestamps are converted to local time when the platform offset is available, with a UTC fallback. The view does not send commands to agents or modify run state. The TDD regression covers rendered session, work, activity, attention, and approval details; a real completed-run fixture verifies the background worker receives the read-only briefing response through the Agent Room dispatcher. Five native suites pass 47 tests; feature all-target Clippy, source-layout, formatting, and diff checks pass. The scoped Graphify graph contains 232 nodes and 446 edges. CocoIndex is unavailable. The feature remains opt-in and does not complete Agent Room or Workspace parity.

### Reviewed execution consent

Workspace Review now opens explicit consent only when the saved proposal is ready. It shows the goal, repository/base commit, workspace mode, plan digest, and every task's agent/model/executable/arguments/permission/fallback, writable paths, and test command. The operator must separately confirm or cancel; a revision, digest, or phase change clears confirmation. Starting sends the current revision and digest through the existing Rust startup dispatcher, which retains its source-context, permission, repository-cleanliness/base, reviewed-plan and digest checks. Tests cover stale/invalid request rejection, exact consent evidence, and starting a generated plan through the native worker against a real Room. Five native suites pass 50 tests; all-target feature Clippy, source-layout, formatting and diff checks pass. Scoped Graphify: 247 nodes / 471 edges. CocoIndex is unavailable, and the feature remains opt-in.

### Strict workflow parity score

Use the 25-workflow denominator above, not the total system use-case count: many system use cases have no operator UI. The source inventory marks 25/25 workflows implemented, 0 partial, and 0 not started. One isolated macOS source-build interaction verifies Workflow 3 profile registration, edit/save, and removal. This does not prove released-app visual parity, Windows behavior, or the full operator journey; verified end-to-end native parity remains 0/25. Workflow 11 language and theme controls and Workflow 12 MCP discovery/source import have source implementations. Workflow 15 includes the native manual and screenshot viewer, installation guidance, and automatic release updates. This inventory does not prove end-to-end native parity, which remains 0/25; the shipped-app/release score also remains 0/25 because the default launcher still selects WebView. The parity table is the auditable denominator and must be updated as each workflow is verified against released behavior.

## Native local-cache maintenance

The native Workspace probe exposes disposable-cache maintenance without a WebView or local service. It requests the existing Rust Startup preview on the background `RequestWorker`, displays the bounded inventory using a virtualized list, and reports the retained protected/uncertain count. Clear is enabled only when the preview contains entries and sends the exact preview digest back to the existing Rust cache handler, which revalidates the inventory before deletion. Reviewed run evidence remains protected. Tests cover the payload contract, empty previews, accessible Workspace entry, preview rendering, and the asynchronous Room-to-Startup preview/clear path against a disposable cache fixture.

## Workspace settings import and export

The native Workspace settings step now exposes import and export through the existing asynchronous request worker. Export stays disabled while workspace fields, role/profile editing, or task instructions have unsaved changes. Import first previews the selected folder, shows the settings summary and backend warnings, defaults local capability claims to unverified, and requires a separate confirmation before applying; the backend retains machine-local secrets and model catalogs. Import refreshes the workspace snapshot after success. Native request-contract and accessible-preview tests pass alongside the existing real Startup import/export tests. This completes workflow 10 for the opt-in prototype; the shipped app and release conversion remain 0/25. Exact-source CI initially caught the older Workspace accessibility contract's static button list; it now includes both accessible sharing actions. Run 38852463148 passed all macOS/Windows validation and exact native-package jobs for `a69132b`, including bilingual manual rendering and execution of both exact release packages. The corrected test file stays under the 500-line limit. Prototype parity remains 17/25 (68%); shipped/default conversion remains 0/25.

### Bilingual import/export and result handling

This UI-only slice extends UC-42 and R-52/DD-59 under DD-160 without changing the preview/apply request contract. TDD first failed on untranslated sharing labels and English-only asynchronous results. The opt-in Workspace now localizes the unsaved-change warning, folder preview, agent/coordinator summary, permission-claim checkbox, import confirmation, and import/export results. Result state stores the outcome and path rather than pre-rendered text, so changing the selected language updates a result already on screen. The short English preview wording is preserved while its accessible name stays descriptive. Verification passes 13 tests across three sharing/workspace/i18n suites plus 13 native-probe accessibility checks; native-feature all-target Clippy, formatting, and diff checks pass. Changed Rust files are below 300 lines; the global source-layout gate reported 34 other oversized files at that checkpoint, reduced to 33 by the following messenger-view split. Workflow 10 remains complete; workflow 11 is still partial because other controls remain untranslated. Prototype parity is 24/25; shipped/default conversion is 0/25.

### 300-line refactor — native Workspace messenger renderer

The failing source-layout gate identified 34 violations before this refactor, including the 339-line native messenger module. The existing messenger regression passed before the move. `State::show` and its rendering helpers now live in `workspace/messenger/view.rs`; `messenger::State::show`, request builders, events, and dispatcher entry points remain unchanged. The messenger suite still passes (2 tests), native-feature all-target Clippy is clean, formatting and diff checks pass, and the two files are 135 and 213 lines. The repository-wide gate still fails on 33 other oversized files. Graphify now contains 6,307 nodes, 10,539 edges, and 478 communities. CocoIndex could not refresh because its installed CLI fails to import `mcp.server.fastmcp`; no new CocoIndex index is claimed. This organization slice does not change prototype parity (24/25) or shipped/default UI conversion (0/25).

### 300-line refactor — native Room task controls

The strict source-layout gate identified `room/task_controls.rs` at 417 lines. Existing feedback, reassignment, model, and scheduling tests passed before extraction. Rendering now lives in `room/task_controls/view.rs`, revision-bound request validation lives in `request.rs`, and the existing `task_controls::show` and `task_controls::request` API paths remain available. All 6 task-control/scheduling tests pass after extraction; native-feature all-target Clippy, formatting, and diff checks pass. The three files are 41, 281, and 104 lines. The global 300-line gate still fails on 31 other files. Graphify now contains 6,318 nodes, 10,542 edges, and 483 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. Prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### 300-line refactor — Workspace settings sharing handlers

The source-layout gate identified the 326-line workspace sharing handler. Existing workspace-sharing, archive, messenger-settings, and MCP integration tests passed before the move. Export, preview, and import/archive staging now live in separate Rust child modules while retaining the `Startup` dispatch methods and their setup-level visibility. All 6 focused tests pass after extraction; native-feature all-target Clippy, formatting, and diff checks pass. The files are 41, 70, 37, and 192 lines. The global 300-line gate still reports 30 other violations. Graphify now contains 6,336 nodes, 10,543 edges, and 480 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. This is a source-organization change only; prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### 300-line refactor — native Agent Room view

The source-layout gate identified the 497-line Agent Room module. Existing render, language, tool-dialog, and candidate-decision tests passed before extraction. The `show` and decision-control renderers now live in `room/view.rs`; callers keep the original `room::show` path and state/event contracts. All 19 focused tests pass after extraction; native-feature all-target Clippy, formatting, and diff checks pass. The facade is 237 lines and the view module is 268 lines. The repository-wide gate still reports 29 other oversized files. Graphify now contains 6,342 nodes, 10,543 edges, and 489 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. This source-only refactor does not change prototype parity (24/25) or shipped/default conversion (0/25).

### 300-line refactor — native Room tool controls

The strict source-layout gate identified the 491-line Room actions module. Existing request-validation, tool-dialog, and render tests passed before extraction. The tool-request view now lives in `room_actions/tool_view.rs`; the `room_actions::tool_controls` path, request validation, action enums, and dispatch remain intact. All 27 focused tests pass after extraction, and native-feature all-target Clippy, formatting, and diff checks pass. The facade is 211 lines and the view is 286 lines. The repository-wide 300-line gate still reports 28 other files. Graphify now contains 6,349 nodes, 10,545 edges, and 484 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. Prototype parity remains 24/25 and shipped/default conversion remains 0/25.

## Native MCP registration and source imports — workflow 12 complete in prototype

The shared Settings menu opens a native editor initialized from the workspace's `mcp` and `goal_sources` maps. It accepts the existing OpenCode-compatible JSON/JSONC format, saves revision-bound settings through the Rust worker, and discovers tools only on explicit request. The Workspace goal editor inserts explicit `jira:KEY-123` and `confluence:PAGE_ID` references. Preview requires the saved goal, a configured MCP server, and a coordinator; it uses the existing revision-bound Rust dispatcher and server-generated preview. The bounded review shows each record's source, tool, title, URL, version, update time, retrieval time, hash, summary, and collapsible full content. Jira records remain required and unresolved Jira references block apply; Confluence records can be deselected. Apply accepts only the exact displayed preview ID and selected records, then refreshes the saved snapshot.

Verification passed 32 tests across six native UI and MCP suites, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks. Every changed Rust source/test file remains at or below 300 lines. The full native-feature all-target run passes all targets except the strict `source_layout` guard, which reports 36 files above the 300-line requirement. This completes workflow 12 in the opt-in prototype and raises prototype parity to 24/25; the shipped WebView UI and release packages remain 0/25. Live GUI acceptance was not performed. CocoIndex still cannot start in the installed environment because `mcp.server.fastmcp` is missing; no new CocoIndex index is claimed.

## Native messenger settings and delivery — workflow 13

The native Settings menu opens an asynchronous messenger editor using the existing Rust settings dispatcher. It supports Slack Incoming Webhook and company webhook settings, event selection, optional goal inclusion, JSON body templates, approvers, and masked secret fields. Save requests are bound to the visible workspace revision; saved secrets are never read back into the UI, and blank secret fields preserve machine-local values. The explicit test action works with a newly entered or already-saved webhook URL and displays a localized result. No message is sent until the operator selects **Send test message**.

Verification: 28 tests across seven targeted suites, native-feature all-target Clippy (`-D warnings`), `source_layout`, bilingual message-catalog checks, formatting, diff checks, and the 500-line source limit pass locally. This workflow is in the opt-in prototype only; production GUI visual acceptance, Korean IME/accessibility walkthrough, and live company-webhook delivery remain unverified. Shipped/default UI conversion remains 0/25.

## Native Workspace roster and goal header — workflows 11/16 partial

The opt-in native Workspace now keeps registered agents visible beside the workspace form. Each roster card shows status, session count, queried model count, quota details with a full tooltip/accessibility name, coordinator marking, and the existing edit/register actions. Quota refresh remains asynchronous; the only scheduled repaint is while planning is active. The form now restores the goal-first headline and stage context and uses the guide's surface and secondary-text colors. Bilingual accessibility tests preserve the original numbered stage names and cover the goal headline.

Verification: 43 tests across nine native/UI/catalog/source-layout suites, feature all-target Clippy (`-D warnings`), native probe build, formatting, diff checks, and the 500-line bound pass locally. The rebuilt single macOS preview visibly shows all three agents and the goal-first Workspace hierarchy with the existing draft intact. Prototype parity remains 18/25 (72%), 2 partial, and 5 not started; shipped/default conversion remains 0/25. The native screen is still a prototype: comparison against the complete prior screenshots in both themes, Korean IME/keyboard/screen-reader acceptance, and Windows verification remain open.

## Native task feedback and reassignment — workflow 18

The Agent Room task inspector now selects a planned task, edits per-task direction, and reassigns it to a registered agent. Requests are bounded, tied to the displayed run revision, and validated against current task/profile IDs; assignments stay disabled during an active run. Feedback goes through the existing Rust dispatcher and resumes an eligible stopped run after the revision is accepted. Prior evidence remains available.

Verification: three focused task-control tests plus the native Room, rendering, language, launch, status, tool-dialog, probe, and source-layout suites pass (50 tests across nine suites). Native-feature all-target Clippy (`-D warnings`), `cargo fmt --check`, offline native binary build, and the 500-line limit pass. Controller integration verifies reassignment and feedback persist in the run report. Prototype parity is 19/25 (76%), 2 partial, and 4 not started; shipped/default conversion remains 0/25. Live-window keyboard/IME and Windows acceptance remain open.

## Native task session and model selection — workflow 19

Selecting a task in the native inspector identifies its existing provider session and displays the captured session ID. The model selector offers only that task agent's observed catalog; changing the model requires an idle, editable run and sends the revision-bound action through the existing Rust controller. The Rust workflow validates the catalog, clears stale approval/evidence and requires a fresh attempt before approval. No provider session IDs can be typed or substituted.

Verification: native task-control, Room, rendering, language, launch, status, tool-dialog, probe, and source-layout suites pass (51 tests across nine suites). The Room-dispatch integration verifies reassignment, model selection, and feedback persist. Existing native-session/recovery tests (8) verify session ownership, invalid-catalog rejection, model-change approval invalidation, and a new attempt. Native-feature all-target Clippy (`-D warnings`), formatting, offline native build, diff checks, and the 500-line limit pass. Prototype parity is 20/25 (80%), 2 partial, and 3 not started; shipped/default conversion remains 0/25. Live GUI, keyboard/IME, and Windows acceptance remain open.

## Native manual scheduling after an availability restriction — workflow 20

When an Agent Room task is waiting on a temporary provider restriction, the task inspector shows its recorded reason, an explicit ISO 8601 timestamp field with timezone, and a schedule action. It prefills a known reset time; when none is known, the operator enters the time. The revision-bound action is accepted only for a waiting task with a valid timestamp, and the existing Rust scheduler persists it across the affected quota group without starting work before the scheduled time.

Verification: the bilingual task-control and real Room-dispatch integration tests cover the visible controls, malformed timestamp/state rejection, and persisted `manual_schedule` time. The relevant 11 suites pass (61 tests), along with native-feature all-target Clippy (`-D warnings`), formatting, offline build, source layout, diff checks, and the 500-line bound. Prototype parity is 21/25 (84%), with 2 partial and 2 not started; release conversion remains 0/25. Live-window, Korean IME/accessibility walkthrough, and Windows acceptance remain open.

## Native task activity, logs, and lessons — workflow 21

The Agent Room task inspector now has localized Activity, Raw logs, and Lessons tabs. Selected-task details load asynchronously through the existing bounded Room request worker and refresh with the Room snapshot cadence. Claude, Codex, and OpenCode events are shown as concise transcript entries; tools, results, warnings, and errors are distinguished, unknown event types retain their type label, and provider JSON is reserved for Raw logs. That tab shows bounded recent stdout/stderr. Lessons remain task-scoped and show only the reviewed lesson status, response, source hashes, and revalidation note returned by the existing Rust knowledge service.

Verification: 66 tests across 12 native UI/session/recovery suites cover provider normalization, unknown events, both language catalogs, raw-log/lesson visibility, and a real Room-dispatch details response. Native-feature all-target Clippy (`-D warnings`), formatting, offline build, source layout, diff checks, and the 500-line bound pass. Scoped Graphify extraction reports 57 nodes / 125 edges across 4 Room-control files; CocoIndex CLI is unavailable in this environment. Prototype parity is 22/25 (88%), with 2 partial and 1 not started; release conversion remains 0/25. The feature remains opt-in; live GUI, Korean IME/accessibility walkthrough, and Windows acceptance remain open.

## Native appearance selector — workflow 11 partial

The shared Workspace/Agent Room navigation now exposes Settings, English/Korean, and Light/Dark choices. Language and theme use the existing machine-wide preference file; cached state localizes navigation labels and applies egui's active theme without per-frame disk reads. Workspace cards use the active panel fill and border. The macOS preview verifies Korean menu/step labels and persisted selections after restarting the probe. Language persistence/accessibility, appearance, workspace, native probe, catalog-integrity, and existing interface-language tests pass; feature all-target Clippy, changed-file rustfmt, diff checks, and the changed-source 500-line bound pass. Workflow 11 remains partial while the rest of Workspace/Agent Room content is not ported; the shipped UI remains WebView.

The native egui theme now maps the design guide's light E and dark F palette to panel/window surfaces, secondary text, selection, borders, and active controls. TDD verified the exact palette after first exposing egui's stock dark panel color. Eleven tests across native appearance, theme/language contracts, and source layout pass, along with feature all-target Clippy (`-D warnings`), formatting, diff checks, and the offline probe build. Workflow 11 remains partial and release conversion remains 0/25. Visual review of the rebuilt preview in both themes is still required before this styling slice is complete.

The native Workspace form now renders localized labels and accessible names for workspace name, project folder, goal, execution isolation, shared-folder/worktree choices, save, and import/export settings actions. It reuses the existing English/Korean catalog and preserves draft values. Regression coverage now checks these labels under the persisted language and the native control contract derives expected labels from the active catalog. Focused Workspace, language, appearance, native-probe, i18n, interface-language suites and native-feature all-target Clippy pass. This remains part of partial workflow 11; other Agent Room content and shipped/default UI conversion remain outstanding.

The native plan and Agent Room task list now display compact localized status tags with accessible task identity. Running adds a pulsing green dot and schedules repaint only while active. Waiting states use amber, failures/blocked states red, and completed states green; semantic foreground/background pairs meet WCAG AA in both themes. Status colors and text use the existing theme and bilingual catalogs without disk reads. Two status tests cover translated AX labels, unknown-state fallback, active pulse and contrast across seven states in both themes. Room/render, launch, tool-dialog, probe, i18n tests and native-feature all-target Clippy pass. The rebuilt Korean preview was checked in light and dark themes. Scoped Graphify reports 456 nodes / 718 edges and CocoIndex indexed 532 files / 5,639 chunks, reprocessing 11 with no errors. Workflow 11 is still partial because other Workspace/Agent Room controls remain; shipped/default conversion is 0/25.

The native Agent Room refresh, team-briefing, stop-confirmation, and review-evidence controls now use the selected English/Korean catalog for visible text and accessibility names. Stop confirmation retains the existing bounded-reason validation and Rust dispatcher; no new polling or background work was added. Focused verification passed 35 tests across Agent Room language/render, status, launch, tool-dialog, and i18n suites; feature all-target Clippy, formatting, and the 500-line bound also pass. A previous English-only test assumption was updated to follow the saved language, so Korean users can run the same regression suite. The single macOS preview build is being refreshed for inspection. Workflow 11 remains partial while other native screens and shipped/default conversion remain outstanding.

Native candidate decisions now use bilingual visible labels and accessible names for approve, reject, and merge, plus their confirmation prompts, rejection reason, cancel, and confirm controls. The exact current-candidate/evidence gate, separate approval and merge steps, and bounded nonempty rejection reason remain unchanged. A Korean AccessKit test covers all three decisions and the rejection field. Focused Agent Room language/render, status, launch, tool-dialog, and i18n verification passed 33 tests; focused Clippy with warnings denied passed. `room.rs` remains below 500 lines at 492 and the new test is 86 lines. Workflow 11 remains partial; other Agent Room content and shipped/default UI conversion are still outstanding.

Native tool-request cards and their decision modal now localize the request count, task/status context, allow/deny/refresh actions, confirmation prompts, and accessible action names. A Korean AccessKit regression first failed on English labels and now covers the localized decisions while preserving current revision and pending-request checks. Nine focused tool-dialog, Agent Room language, candidate, and i18n tests pass; one unrelated broad source-reference test was excluded after it stalled reading the source tree. Rustfmt and diff checks pass, and changed Rust files remain below 500 lines. The refreshed opt-in macOS preview opens as one window with the existing Workspace data intact. Scoped Graphify extraction reports 258 nodes / 325 edges; CocoIndex indexed 534 files / 5,662 chunks, reprocessing 7 with zero errors. Native-feature Clippy and full source-layout verification remain pending; the shipped WebView path is unchanged.

Native Agent Room Resume and Recover actions, their accessibility names, and the recovery safety explanation now follow the selected language. Existing Rust state gates remain authoritative: Resume stays unavailable in terminal/approval states, Recover is limited to a running run and still refuses live or unknown processes. The Korean AccessKit regression first failed on the old English labels; after localization, the UI regression and bilingual catalog key/placeholder contract pass. A disk-full linker failure exposed 50.9 GiB of stale Rust build output; clearing only Cargo's generated `target` cache restored build space. The rebuilt native probe reopened as one macOS window. Scoped Graphify reports 256 nodes / 323 edges; CocoIndex indexed 534 files / 5,667 chunks, reprocessing 7 with no errors. Native-feature Clippy remains pending.

The read-only Agent Room team briefing now receives the selected language and localizes its heading, captured state, session metadata, process status, availability message, approval summary, and missing-value fallbacks. Known run states use the shared status catalog, while agent-provided work and notes remain unchanged. A Korean AccessKit rendering regression first failed on the English-only briefing; both Agent Room language tests and the bilingual catalog ID/placeholder contract now pass. Rustfmt, diff checks, and a debug-info-disabled native probe build pass; the changed Rust files remain below 500 lines (`room.rs` 493, `room_briefing.rs` 179, test 135). The refreshed opt-in macOS preview is a single window and retains the current Workspace values. Scoped Graphify reports 54 nodes / 111 edges; CocoIndex indexed 534 files / 5,674 chunks, reprocessing 7 with zero errors. Native-feature Clippy and full source-layout verification remain pending. Prototype parity remains 17/25 (68%); shipped/default UI conversion remains 0/25.

The native Agent Room heading and read-only review-evidence panel now use the selected language. Revision, candidate, test exit/timeout, stdout/stderr, and truncation labels are localized; captured test output and diff content remain byte-for-byte user data. The existing English `render_evidence` API remains available, while the Room passes its current language through the new localized renderer. Korean regressions first failed for the missing heading and language-aware renderer. Verification passes 3 Room-language tests, 12 Room-render tests, the bilingual catalog contract, native-feature all-target Clippy (`-D warnings`), the complete source-layout test, rustfmt, diff checks, and a native probe build. Scoped Graphify reports 129 nodes / 283 edges; CocoIndex indexed 534 files / 5,686 chunks, reprocessing 9 with zero errors. The rebuilt macOS preview remains one window with existing Korean Workspace values intact. Prototype parity remains 17/25 (68%); shipped/default UI conversion remains 0/25.

## Structural map

The latest Graphify update rebuilt the repository graph with 6,163 nodes, 10,349 edges, and 461 communities. CocoIndex was invoked but could not start because the installed `cocoindex-code` entry point cannot import `mcp.server.fastmcp`; earlier successful indexing numbers refer to an older source snapshot. Both are local analysis tools, not application runtime dependencies; the graph is code-structure evidence, not proof of production UI coverage.

### Safe OpenCode MCP examples — GitHub #17 / DD-163

When no MCP server is registered, the native Settings editor displays commented local and remote examples using OpenCode's `mcp` field shape; registered servers suppress both examples and their explanatory note. The local command is deliberately not installed, the remote host is `mcp.example.invalid`, and credentials use `{env:COMPANY_MCP_TOKEN}`. Existing `goal_sources` values remain in the editor when examples are shown. The untouched JSONC parses and saves without creating a job or starting a process. Saved server definitions still render from the workspace snapshot. The UI is an opt-in native preview; the shipped application continues to use WebView and workflow 12 still lacks native discovery/import review/apply.

### Native agent tools guide — workflow 15 complete in prototype

The native Workspace menu opens an inline, scrollable installation guide using the existing English/Korean message catalog. Each static command is selectable and has an explicit copy button with an accessible name. Copying only writes to the system clipboard; the guide does not install tools, start processes, or make network requests. It keeps optional setup out of Settings and uses no additional runtime or crate.

Verification: 11 tests across native appearance, workspace navigation, and language suites cover the menu item, localized close/copy controls, and all 12 commands. Native-feature all-target Clippy (`-D warnings`), formatting, offline probe build, source layout, diff checks, and the changed-file line limit passed for this slice. Scoped Graphify extraction reported 30 nodes / 61 edges. CocoIndex was unavailable at that earlier checkpoint. Together with the native manual/screenshots and release-update slices below, workflow 15 is complete in the opt-in prototype; shipped/default conversion remains outstanding.

### Native offline manual and screenshots — workflow 15 complete in prototype

`src/manual.rs` parses its supported Markdown subset into shared Rust blocks used by both committed HTML generation and the opt-in egui viewer. The Workspace menu opens the selected language's bundled manual; language changes reload the cached blocks. Headings, paragraphs, lists, tables, code, inline command copying, and six bundled screenshots render natively without WebView, JavaScript, subprocesses or network access. Screenshot decoding is lazy, restricted to bundled assets, and cached only while the manual is open.

Verification: manual parsing/rendering, native manual/workspace views, appearance and language suites cover exact generated HTML parity for both languages, escaped table pipes, safe local HTML images, native command tables, screenshot rendering, code blocks, menu exposure, and copy/close controls. The screenshot slice passed 3 native manual and 4 HTML renderer tests; native-feature Clippy, offline build, formatting, diff checks, and changed-file 300-line limits pass. Repository-wide source layout remains red on existing files over the new 300-line limit.

### Native release updates — workflow 15 complete in prototype

The native Settings menu opens a localized release-update panel through the existing Rust dispatcher. It shows current/latest versions, status and errors, release notes in a read-only, keyboard-focusable multiline control inside a bounded scroll view, opens a release-page link only for GitHub release URLs, and offers the existing check and install/restart actions. Release bodies remain literal plain text; the panel does not interpret remote Markdown or HTML. After the first workspace snapshot, the same background worker checks immediately and every 600 seconds; it polls status every 1.5 seconds only while an update operation is active, even when the panel is closed. A localized, accessible update button appears beside Settings only when a newer version is available. Repaint deadlines reuse egui's event loop, and the existing checksum-verified installer and all-workspaces idle gate remain authoritative. The production WebView remains unchanged.

Local verification: 16 focused update, appearance, and workspace tests; native-feature all-target Clippy (`-D warnings`); default and native-probe builds; formatting; diff checks; and changed-file 300-line limits pass. A first-frame response-order race is prevented by waiting for the initial snapshot before requesting update status. The repository-wide source-layout check is still pending the existing 300-line refactor. Graphify reports 6,102 nodes / 11,077 edges; CocoIndex indexed 566 files / 5,974 chunks with no errors. Native prototype workflow 15 is complete; overall native prototype parity is 24/25 (96%), while shipped/default conversion remains 0/25.

Issue #9 follow-up (2026-10-18): replaced the selectable release-note Label with a read-only multiline text control to restore the legacy `<pre tabindex="0">` keyboard-focus contract while retaining the same 260-point scroll viewport. Seven `native_ui_updates` tests pass, including AccessKit role/read-only/content and clipped geometry. Native-window keyboard traversal and exact-package acceptance remain open.

### Native task editor localization — workflow 11 partial

The native task editor receives the selected English or Korean language and uses the existing message catalogs for its heading, task fields, assignment and session-model controls, and plan-review/save actions. The English `task_controls` entry point remains available; the Workspace Review flow uses the language-aware entry point. Task IDs, instructions, paths, dependencies, commands, and agent/model IDs remain user data.

Verification passed 11 tests across the task editor, language, and message-catalog suites, plus native-feature all-target Clippy (`-D warnings`), formatting, and diff checks. Every changed Rust source/test file remains below 300 lines. Workflow 11 remains partial because other native Workspace and Agent Room controls still need bilingual coverage; prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### Native role assignments localization — workflow 11 partial

The native Team role editor now uses the selected language for its heading, predefined role names, add/remove actions, default-model text, and accessible names. Registered agent IDs and observed model IDs stay unchanged. The existing English `show_role_rules` entry point remains available; the Workspace uses the language-aware entry point.

Verification passed five role and catalog tests, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks. Changed Rust files are 296, 86, 132, and 84 lines. Workflow 11 remains partial; prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### Native execution consent localization — workflow 11 partial

The execution-consent panel now uses the Workspace's selected English or Korean language for its heading, warning, plan/repository labels, task details, and confirmation actions. User-provided goal, repository path, agent/model identifiers, arguments, permissions, writable paths, test commands, and plan digest are preserved as values. `execution::controls` remains an English compatibility entry point; the Plan Review screen calls the language-aware function.

Verification passed 15 tests across execution consent, launch compatibility, and message catalog contracts. Native-feature all-target Clippy (`-D warnings`), default `cargo check --all-targets`, formatting, and diff checks pass. Graphify refreshed the canonical graph to 6,363 nodes / 10,574 edges / 492 communities. CocoIndex still fails to import `mcp.server.fastmcp`, so no fresh index is claimed. The repository-wide 300-line guard still reports 29 existing files. The cache-cleanup panel still has hardcoded English copy; workflow 11 remains partial, native prototype parity 24/25, and shipped/default conversion 0/25.

Implementation: commit `a242d2a`, pushed on the issue #9 branch; [progress record](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218320478). AgentMemory records: `mem_mvb35m0t_ac1b8524bc43` (localization decision) and `mem_mvb3hgge_406665e57871` (default-test feature gate).

### Native cache-cleanup localization — workflow 11 partial

The native Workspace cache-cleanup panel now receives the selected interface language. Its title, preview and removal summaries, skipped-entry notice, loading and empty states, unavailable-path fallback, action labels, and clear-action accessible name use the existing English/Korean catalogs. The English `show` entry point remains available, and cache paths/counts remain data.

Verification passed 8 tests across native cache and language suites; default `cargo check --all-targets`, native-feature all-target Clippy (`-D warnings`), formatting, and the native probe build pass. The touched Rust files are 202 and 233 lines. Graphify refreshed to 6,373 nodes / 10,593 edges / 487 communities. CocoIndex was retried, but its CLI still fails to import `mcp.server.fastmcp`, so no fresh index is claimed. AgentMemory record `mem_mvb3rvo0_9541202c30e2` and Claude handoff `sig_mvb3sfe1_a8dff65a4949` track the slice; [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218462009) is recorded. The cache-panel localization subunit is complete; the broader native bilingual audit remains open. The prototype remains 24/25 and shipped/default conversion remains 0/25.

### Default-build gates for native UI integration tests

Auto-discovered integration tests that import optional `eframe` or `native_ui_probe` are now gated by `native-ui-probe`, unless Cargo already declares that requirement for the target. The `native_ui_targets` regression scans root integration test files so new native-only tests cannot silently break the default all-target build.

The regression passes (2 tests); default `cargo check --all-targets` and native-feature all-target Clippy (`-D warnings`) pass. This is test-build wiring only; it does not convert the production WebView launcher. The source-layout gate remains red on 29 files above 300 lines.

### Native bilingual activity and default launch — issue #9 / workflow 11

The native Workspace header no longer displays its probe-only marker. Fixed CLI-generated activity text (session start, task-list update, token counts, changed-file counts, command exit codes, and clipped line counts) now uses the selected English/Korean message catalog; agent-produced text and identifiers remain unchanged. `ui-new` and `ui` use the shared Rust eframe launcher in the default Cargo build.

TDD added bilingual rendering and default-feature contract tests. The former failed on `NATIVE UI PROBE` and on English generated activity in Korean mode. Focused native localization suites passed 14 tests; `cargo build --bin agentconductor --locked` and the default-launch contract passed. The locked all-target run reached every target before the source-layout guard; all targets after that guard were then run separately and passed. The sole all-target failure is `source_layout`, which still reports 29 files above 300 lines. Native all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. Workflow 11 remains partial pending the bilingual copy audit. The WebView resources, fallback code, exact cross-platform package tests, and final release remain incomplete. Committed and pushed as `1c3413e`; [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219183508), AgentMemory `mem_mvb55br8_bd0c3051a682`, and Claude signal `sig_mvb55edm_6600327d067f` record this subunit.

## Remaining acceptance

- Port every shipped screen and interaction to the native Rust application while preserving existing workflow behavior.
- Audit all remaining visible Workspace and Agent Room text in both languages before closing workflow 11.
- Verify Korean IME composition, keyboard navigation, screen-reader accessibility, responsive layout, and separate-window lifecycle on macOS and Windows.
- Measure idle CPU and memory for the packaged native application against the existing resource budget.
- Remove project-owned JavaScript and Python runtime/build/verification paths only after equivalent Rust behavior and checks pass.
- Audit shipped dependencies and license notices, then build and smoke-test exact macOS and Windows release archives.

### Release package writer split — issue #9 / 300-line source bound

The package API and output contracts stay in `release_package::assemble`; the private `PackagePlan` and archive-writing implementation live in `src/release_package/archive.rs`. This separates package validation/license selection from archive output without changing exported functions or error handling.

The source-layout guard identified `src/release_package.rs` at 350 lines before extraction. The seven release-package and license tests passed before and after; native all-target Clippy (`-D warnings`), the locked application build, rustfmt, and diff checks pass. The source file is now 219 lines and its new archive module is 140 lines. The repository guard still reports 28 other files above 300 lines. Commit `6d763fa` is pushed; progress is in [GitHub #9](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219267552), AgentMemory `mem_mvb5aiww_40844f57405f`, and Claude signal `sig_mvb5b1pu_c9527e792667`. The full #9 migration remains open.

### Release smoke validation and workflow split — issue #9 / 300-line source bound

The public `release_smoke::validate_package` and `release_smoke::run_smoke` entry points now re-export code from focused modules: package manifest/checksum validation, subprocess setup/collection, and the four git/folder workflow scenarios. Command timeouts, error details, and evidence-file output retain their existing behavior.

The source-layout guard identified `src/release_smoke.rs` at 453 lines before extraction. All six release-smoke tests pass before and after; native all-target Clippy (`-D warnings`), the locked application build, rustfmt, and diff checks pass. The root is now a 6-line module; validation, runner, and workflow files are 74, 146, and 242 lines. The guard reports 27 other files above 300 lines. Commit `ecf2414` is pushed; progress is in [GitHub #9](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219375990), AgentMemory `mem_mvb5ie1k_0362c7154e2b`, and Claude signal `sig_mvb5ig35_7102ed248986`. The full #9 migration remains open.


## Quota display module split — issue #9 / DD-41

The 404-line quota display module is split into a small facade, label/time helpers, and badge rendering. The `quota::{Badge, badge, badges, budget_rows, window_labels, window_reset_labels}` API remains stable, so the controller and integration tests need no caller changes. The existing quota display suite passed 8/8 before and after the move. The locked app build, native-feature all-target Clippy with warnings denied, rustfmt, and diff checks pass. The repository-wide 300-line guard now reports 26 remaining files; #9 is still open. Graphify: 6,413 nodes / 10,623 edges / 488 communities. CocoIndex CLI was not available. AgentMemory `mem_mvb5qjfg_74253af00fa0`; Claude signal `sig_mvb5qlrj_00a813ca9aa4`.

GitHub issue #9 progress: https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219503212.


## Bilingual native workflow 11 — completed in the source build

The final audit found two remaining English strings in the eframe UI: the visible `Run` prefix on status tags and the profile-avatar accessibility name. English/Korean catalog messages now render both while preserving profile IDs and status values as data. New render/accessibility regressions first failed against the original hardcoded text, then passed after localization. A source scan found no remaining direct English labels in these controls; the invariant `AgentConductor` product heading remains branded.

Verification passes 33 tests across 14 UI/i18n suites, covering language persistence, light/dark appearance, Workspace and Agent Room controls, navigation, profiles, task activity, and accesskit labels. The default application build, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. Native source-build parity is 25/25 (100%); published v0.1.24 remains WebView, so release conversion is still 0/25 pending exact macOS/Windows package, accessibility/IME, visual, resource, and license acceptance. Graphify: 6,416 nodes / 10,626 edges / 478 communities. AgentMemory `mem_mvb62w4q_5fedb050b69f`; Claude signal `sig_mvb62yin_0028abb68615`.

GitHub issue #9 workflow 11 verification: https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219714532.

### Default native shell and runtime requirements

The default Rust eframe launcher now owns the Workspace and Agent Room windows. The unused Cocoa/WebView launchers and direct `wry`/`tao` dependencies are removed; Windows no longer lists WebView2 as an application requirement. The native viewport loads the embedded AC icon. Focused release, i18n, branding, and Windows tests pass (9 total); default all-target compilation and native-feature Clippy pass. Full all-target tests have one known failure: the 300-line source-layout guard reports 25 files, including one remaining JavaScript file, above the limit. Exact macOS/Windows package acceptance, visual/IME/accessibility/resource checks, and the remaining file splits are still open. Published v0.1.24 is unchanged until a later release.

The unused `Room::document` and setup HTML assembly are removed from the compiled application. The Rust native UI remains responsible for Workspace/Agent Room rendering and the manual viewer; both native headers show the package version. A Rust source guard prevents the removed document modules from embedding browser assets. The old JS/HTML/CSS source files and tests inspecting them are not all deleted yet. The release binary scan has no WebView runtime or browser-script markers. Focused verification passes 39 tests across 10 UI suites, all-target check, native-feature Clippy, optimized release build, rustfmt, and diff checks. The strict layout test still reports 25 files over 300 lines; exact package/UI/platform acceptance remains open.

The 367-line `src/ui/room.js` and its markup-only tests are deleted. Rust quota-formatting and snapshot coverage remains; native eframe tests own task rows, quota presentation, routing, and the UI. The prior static JS assertion for Korean IME handling was removed because it only tested the deleted WebView page; native IME behavior still needs real macOS/Windows validation. Focused verification passes 21 tests with 1 environment-dependent icon test ignored across 8 suites; all-target check, native-feature Clippy, release build, rustfmt, and diff checks pass. The rebuilt app contains no legacy JavaScript asset names and does not link WebView. The strict source-layout test is now reduced from 25 to 24 remaining files over 300 lines. Other unused browser assets/tests and exact package acceptance remain open.

### Agent Room run-status banner — issue #9 / R-99 / DD-107

The native Agent Room now renders a state-colored banner above the run controls. It summarizes the localized run state and completed-task count, and displays the run error or first failed task error when available. Stop and Resume controls remain in their existing action row. A focused rendered-UI regression first failed without the banner, then passed after integration; the new banner test and all nine Agent Room render tests pass. Changed Rust files are below 300 lines. Actual packaged macOS/Windows placement and visual comparison are still required, so this does not change the 0/25 accepted-workflow count. Graphify refreshed to 6,427 nodes / 11,127 edges. AgentMemory records `mem_mvdqgjox_5d95f8ca3e7a` and `mem_mvdqhogq_3e475cb866fa`; CocoIndex was not needed for this Rust UI slice.

### Agent Room task-row status attention — issue #9 / R-99 / DD-107

Failed task rows now have a persistent red outline. Rows waiting for approval or a tool decision pulse their outline; setting egui's animation duration to zero leaves a static outline without animation repaint requests. The rendered regression first failed when no row outline existed, then passed for failure color, two pulse phases, and the static reduced-motion style. The related focused suites pass. The app does not currently verify or expose an OS-level reduced-motion preference, and exact packaged macOS/Windows appearance remains untested. Overall accepted workflow count remains 0/25. Graphify refreshed to 6,437 nodes / 11,144 edges. AgentMemory record: `mem_mvdqn2f4_256393044629`.

### Failed-run feedback action — issue #9 / UC-100 / DD-118

The native failure banner now exposes the localized “Fix with feedback” action when a failed integration test or worker maps to a planned task. Clicking it selects that task and drafts its bounded captured output into the existing editable feedback field, retaining any prior task-specific draft for the operator to review. The new rendered interaction regression failed before the action existed, then passed after wiring the banner to task-control state; it verifies button accessibility, task selection, and visible draft contents. The test and touched Rust modules remain below 300 lines. This widget-level behavior does not prove the packaged app or end-to-end retry path; macOS/Windows visual and send/reverify acceptance remain open, and #9 remains at 0/25 accepted workflows.

### Review Plan task disclosure — issue #9 / UC-106 / DD-124

Review Plan task editors now start as compact disclosures showing task ordinal, ID, and the first non-label instruction line; Remove remains available in the header. Expanding a card exposes the same editable agent, model, instruction, scope, dependency and test-command fields. The Red test reproduced all fields being expanded with no summary; Green verifies the one-line summary, hidden body by default, and body visibility after an actual disclosure click. The existing task-edit and width tests pass with their editor body explicitly exposed for those checks. Changed Rust files remain under 300 lines. Packaged macOS/Windows, keyboard, screen-reader and multi-task visual review remain open; overall issue #9 stays at 0/25 accepted workflows.

### Agent Room review evidence pane — issue #9 / UC-04 / DD-06

The legacy Agent Room keeps Activity, Diff & tests, Logs, and Lessons in separate tabs. The
native Room previously rendered diff/test evidence inline beneath Activity, making the central
human-approval evidence harder to find. The Rust Room now provides a dedicated Diff & tests tab;
it only renders evidence bound to the current run revision. If the run has changed, the pane
shows the localized stale-evidence warning and accessible Review latest action, which reuses the
existing revision-bound evidence request. Candidate approval/merge still depends on current
evidence. TDD covers selecting the tab, keeping diff/test output out of Activity, warning for
stale evidence, and emitting the evidence-refresh event. This is native widget evidence only:
packaged macOS/Windows, keyboard navigation, and live approval gating remain unaccepted, so the
verified operator workflow count stays 0/25.

The Workspace phase summary also lists multiple goals. It marks the active goal as current and
sends inactive goal selection through the existing revision-bound `select_goal` command; the
control is disabled while a request or job is active. On success, the Workspace reloads its
snapshot so the goal form and phase summary agree. The controller archives the previous goal's
last run and clears the old proposal when changing goals. A rendered interaction test covers
the accessible selector and serialized request; it does not prove packaged parity.

### Start feedback stays visible at short window heights — issue #16

During plan approval, pending/error feedback now renders in a bottom panel outside the scrollable
Workspace content. The approval-to-Agent-Room test reproduces the old failure at 700px height: the
scroll-contained status label was not rendered inside the viewport. With the panel, the test checks
that the pending status bounds remain inside the short viewport while the approved start is still
pending, then verifies the real Room dispatch opens the started run.
running, then verifies the real Room dispatch opens the started run. The panel reuses existing
status text and asynchronous updates. This branch-level regression does not establish packaged
macOS/Windows acceptance; issue #16 remains open until that runtime check follows #9 integration.

### Generated-plan review journey coverage — issue #9

The existing native approval journey now checks that all three tasks generated by the mock planner
appear in the Review Plan accessibility tree before exact-plan consent. It then continues through
pending-start feedback and the Agent Room transition. The focused offline journey and 300-line
source-layout checks pass. This verifies native-probe plan visibility, not exact packaged UI or any
of the 25 accepted operator workflows; issue #9 remains open at 0/25.

### Resume preserves bounded integration repair — R-123 / UC-113 / DD-132

The upstream failed-run guard blocked every `failed` report, including the repairable integration
failures that R-123 explicitly routes back through the coordinator. Native Resume now uses the
existing `workflow::repairable` predicate: only failures whose error begins with `integration ` and
whose tasks are all complete enter `resume-due` and the existing three-round repair loop. Other
failed runs remain blocked until the operator revises or reassigns work. TDD confirmed the new
repairable-integration regression fails under the blanket guard and passes with the predicate;
the non-repairable guard test remains green. Nine tests across `ui_failed_resume`, `repair`, and
`ui` pass; native-feature all-target Clippy, rustfmt, diff checks, and source-layout test pass.
This corrects the Resume entry point but does not change verified workflow acceptance (0/25).

### Latest-main full regression — 2026-10-19

After merging `main` commit `5993970`, the complete offline locked native-feature all-target
suite passed: 659 passed, 14 ignored, 0 failed across 230 suites (193.74 seconds). The first
sandboxed attempt reported four AgentMemory contract tests failing only because macOS denied
their `127.0.0.1` test-server binds; rerunning the same suite with local loopback permission
passed all four. Native-feature all-target Clippy (`-D warnings`), rustfmt, diff checks and
the 300-line source-layout test also pass. This is regression evidence, not end-to-end parity:
Windows package/UI, real IME/accessibility, and all 25 operator workflows remain unaccepted.
Workspace content. The approval-to-Agent-Room test reproduces the old failure at 700px height: the
scroll-contained status label was not rendered inside the viewport. With the panel, the test checks
that the pending status bounds remain inside the short viewport while the approved start is still

### Agent Room four-stage workflow context — 2026-10-19

The Rust Agent Room now shows the existing four-step workflow row (`01 Plan`, `02 Tasks`, `03 Verify`, `04 Complete`) after its run-status banner. The active step comes from the persisted report: prepared plans map to Plan, active work to Tasks, waiting approval/approved reports to Verify, and completed reports to Complete. Failed runs remain on Plan when planning produced no tasks, on Tasks when planned work is incomplete, or on Verify when all planned work completed and verification failed. The current step has a localized AccessKit name; the stage labels fit the 792×532 minimum test viewport. Red-first rendered tests cover state mapping and English/Korean labels. The final offline all-target suite passes 667 tests, 14 ignored and 0 failed across 235 test targets; all-target Clippy with `-D warnings` also passes. Exact isolated macOS review shows the waiting-approval candidate highlighting Verify. Graphify was refreshed to 6,728 nodes / 11,666 edges. AgentMemory record `mem_mvelvv92_2b278ee1838f`; Claude signal `sig_mvem4axx_855da1382df5`. CocoIndex is unavailable in this host environment. This does not advance accepted end-to-end workflows: issue #9 remains 0/25.

Temporary-space audit on 2026-10-19 found `/private/tmp` at 535 MiB, not the reported 41 GiB. The Rust `target` directory had grown to 9.0 GiB during regression; after removing its 2.8 GiB regenerable incremental cache it is 7.9 GiB. Temporary compiler wrappers and test logs were removed; the current isolated review app and its test evidence remain. Host Rust tools require temporary Z3 soname and invocation wrappers because Homebrew LLVM references a missing `libz3.5.1.dylib`; these are not product dependencies.

### Agent Room candidate review visual check — 2026-10-19

Using the Rust executable built from source commit `2ec925b`, a fresh isolated mock run reached
`waiting_approval`: all three tasks completed and all three recorded test commands exited with code
0. At 1140×772, the actual Rust Agent Room was inspected in light and dark themes. Its status,
four-stage progress, task/session list, candidate review rail, test evidence, and review controls
were visible. The read-only “Load change and test evidence” action displayed the current revision,
candidate commit, and passing test results; approval and merge were not invoked. Temporary run
data and the 77 MB app bundle were deleted after closing the app. The already-built source-layout
test binary also passes (1 passed). The regular Cargo test command was not rerun because this
host's rustc cannot load Homebrew LLVM's configured `libz3.5.1.dylib` soname. No screenshot files
were retained. This mock-run visual check does not count as a real-agent workflow or advance issue
#9's verified end-to-end acceptance from 0/25; Windows, real approval/merge, IME, accessibility,
and the remaining workflow gates stay open.

### Inline quota refresh spacing — 2026-10-19

R-64/UC-54 requires the compact per-agent quota value and refresh action to remain together
with a 5px gap. The rendered egui regression measured an 8px gap at the 248px rail width before
the change. The Rust sidebar now groups the value and refresh/spinner in a local horizontal row
with 5px spacing; provider-window details remain in the tooltip and accessible label. The new
geometry regression and related sidebar roster/density/icon, quota display, and source-layout
suites pass (16 tests total); focused Clippy, rustfmt, and diff checks pass. The source/test files
are 282/176 lines. Exact packaged macOS/Windows screenshots and full workflow acceptance remain
open; issue #9 stays at 0/25. Graphify refreshed to 6,754 nodes / 10,785 edges / 518 communities.
CocoIndex structural grep located the sidebar's row construction; semantic indexing is unavailable
because this repository has not been initialized for CocoIndex.

### Exact-source macOS package — 2026-10-19

The optimized Rust release build for `aarch64-apple-darwin` and the Rust `package-release`
command succeeded at source commit `7df4a93d5d347e7a80e523cd2826a170ba646c12`. Explicit
`validate-package` passed: 17,560,864-byte single executable, 214 dependency license records,
binary SHA-256 `9d9772bc2e27e23deb588e9e1a46bf2a88cced35c0182ae9a144494fc4500e83`; archive
SHA-256 `28ce1d33f465af229a955470cd56f4234673a4a58cfec80347ba87f42e20dff3`. The package command
also completed all four shared/worktree × Git/folder smoke cases, including approval-gated merge
checks. A GUI inspection attempt was not valid evidence: a separate CUA launch displayed the
existing default Workspace rather than the requested isolated workspace. No controls or Save were
activated; the window and CLI process were closed and task-owned temp files removed. Reuse a single
verified package-app process with explicit isolated arguments for future UI inspection. The local
macOS artifact is not a release or Windows validation and does not advance issue #9 from 0/25.

Correction (2026-10-19): the exact package UI was then launched once with explicit `ui-new` and an
isolated workspace argument. The process command line confirmed the packaged `.app`; CUA attached
to that same process and inspected its 1140×772 empty Workspace screenshot and accessibility tree.
The expected setup fields were present and Coordinator was disabled; no save or user data access
occurred. This verifies only the packaged empty Workspace, not quota cards or end-to-end parity.
The task-owned isolated HOME/workspace was removed afterward. `/private/tmp` measured 456 MiB;
unrelated work directories and the reusable Rust build cache were retained.

### Exact packaged availability-restriction spot-check — 2026-10-19

The exact macOS package was run with an isolated HOME, workspace, repository, planner fixture,
and Rust mock agent. A simulated `usage_limit_reached` JSON event put all three tasks into
`Waiting availability`; the task inspector showed the restriction reason, timezone-labelled
schedule field, and retry button. The focused `native_ui_task_scheduling` suite passes all three
tests, including a real Room-dispatch persistence check. The CUA keyboard inherited the active
Korean IME and could not enter the ISO timestamp, so the packaged click and due-time resume were
not manually exercised. This does not indicate a product defect and does not advance #9's strict
0/25 acceptance count. The isolated 41 MiB package/HOME/workspace fixture and compatibility link
were deleted after the package and watcher processes exited; unrelated temp data was preserved.

### Test fixture temp-storage correction — 2026-10-19

Investigation found 1,145 stale Rust mock CLI executables (774 MiB) in the macOS user temp
directory. The shared process and workflow test fixtures compiled those executables there and
created their run roots there as well. Removed only the known test-generated executable names
after confirming no corresponding test process remained. Added a failing-then-passing regression
and changed both shared fixtures to use Cargo's `target/debug/tmp`, keeping repeat test output in
the disposable build tree. The full offline locked all-target suite completed with all executed
tests passing; environment-dependent provider/login tests remained ignored. After the issue #36
worker finished, removed its disposable 3.4 GiB Cargo target cache while retaining its 22 MiB
source worktree and draft PR. `/private/tmp` returned to 466 MiB. The current user temp directory
measures 804 MiB; unrelated macOS temp entries and active lease files were left untouched.

### Exact packaged agent-card spot-check — 2026-10-19

Opened the exact package once in an isolated HOME/workspace and registered a non-running fixture
(`parser=claude`, executable `/usr/bin/true`). The 1140×772 package screenshot and accessibility
tree showed the provider icon, status, session count, model refresh, compact quota value/refresh,
edit/remove and reorder actions. The quota refresh displayed the trusted-project-folder requirement;
it did not invoke Claude or query account usage. Closed the one package process and removed its
isolated files. This is visual/component evidence only. Real provider quota/model contents, Windows,
saved public screenshots, and workflow parity remain unverified; #9 stays at 0/25.

### Exact current-HEAD package attempt — 2026-10-19

Built the optimized macOS binary offline from `a8e023b9e4f26a63cc6480d2db9374bd02193cf8` in a
temporary Cargo target directory. The exact package executable launched `--help`, and
`validate-package` accepted its manifest: SHA-256 `ab023bf255a7e0fb87c81dc0b3aac96c27403b0a41481f029be093180a55fb74`,
214 dependency license records, `aarch64-apple-darwin`. The package smoke proceeded to its first
run command, then process creation returned OS `Operation not permitted`, including after setting
`TMPDIR` to an allowed `/private/tmp` directory; direct invocation of the same packaged `run`
command hit the same boundary. This is not a passing workflow or evidence of a product defect.
No package archive or UI acceptance is claimed. The temporary package source checkout, package,
isolated HOME/workspace, 712 MiB build target and compatibility alias were removed; `/private/tmp`
returned to 466 MiB. The strict native UI acceptance count remains 0/25.

### Workspace sharing menu route — 2026-10-19

UC-42 settings import/export now starts from **Settings → Workspace settings sharing** rather than
the bottom of the Workspace form. The existing asynchronous folder picker, import preview,
permission re-verification and result state remain unchanged; the sharing flow opens in a native
window. A Probe-level pointer/accessibility regression checks that export/import controls are absent
from the closed Workspace and appear after selecting the Settings item. Focused native sharing,
appearance, workspace, style and source-layout tests pass (20/20), as do all-target native-feature
Clippy, formatting and diff checks. This fixes menu placement only; exact-package macOS/Windows
inspection is pending and issue #9 remains at 0/25 verified workflows.

### Issue #16 default-path compatibility and Rust counterpart — 2026-10-19

PR #38 merged the localized start-pending feedback into the shipped WebView compatibility path
(`65c77b7`); issue #16 closed after macOS and Windows validation and exact-package smoke passed.
The Rust branch already implements the corresponding post-approval progress state and exact-run
transition. Its `native_ui_start_journey` regression passes 1/1, so copying the JavaScript patch
would add a second implementation instead of improving the Rust app. This closes the compatibility
bug, not native UI parity: issue #9 remains 0/25 pending packaged workflow acceptance.

### Exact 0.1.30 package launch — 2026-10-19

Built and validated the exact `dbad31d` macOS package (214 dependency license records; binary SHA-256
`d2b66628f69d248e5dc92b1244c035afb783ab4280057831fa20174ae0e38e85`). In a single isolated app
window, the native Workspace rendered at 1140×772 and saved its goal and project folder to the
temporary workspace state. No authenticated CLI or external service was used. This proves launch
and one save action only; no legacy screen comparison or end-to-end parity is claimed. Issue #9
remains 0/25, with Windows and full workflow acceptance open.
