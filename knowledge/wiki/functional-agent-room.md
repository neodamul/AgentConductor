## Activity, raw logs, and reviewed lessons — 2026-10-24

The exact v0.1.37 macOS arm64 package displayed normalized task activity separately from JSONL stdout/stderr, and showed the task-specific cause and corrective action in Lessons. The fixture remained at the human approval gate; no approval or merge occurred. Nine focused activity/lesson/layout tests passed. This brings packaged UI acceptance to 20/25 (80%); shipped/default Rust UI remains 0/25. Windows UI, Korean IME, and screen-reader behavior remain unverified. See [workflow 21 evidence](../../docs/evidence/issue9-workflow21-task-activity-20261024.json).

# Functional Agent Room evidence

## Scheduling a quota-waiting task while the coordinator remains active — 2026-10-24

When an agent task is `waiting_availability`, its schedule timestamp field and Schedule retry action must remain usable even if the top-level workflow job is still `running`. Global `editable` also locks unrelated task controls, so do not reuse it for this wait-state action. Gate scheduling on the room having no pending action, then keep the server-side task-state/timezone validation authoritative. An AccessKit regression reproduces the disabled-input bug; focused tests, source layout, warning-denied Clippy, and formatting pass. The Windows package test showed its PowerShell `SendKeys` path did not leave a valid timestamp, so the driver now uses UI Automation `ValuePattern.SetValue` and checks the field and button before invoking. Exact Windows package UI verification is still pending. Graphify refreshed to 9,404 nodes / 15,859 edges; CocoIndex CLI is unavailable. See [schedule regression evidence](../../docs/evidence/issue9-windows-schedule-regression-20261024.json).


## Live snapshot must not wait on workflow ownership — 2026-10-24

QAS-158's exact v0.1.37 UI run showed that an active and pending task could leave the Agent Room spinner active and disable permitted controls. The source-level red test reproduced a snapshot taking longer than two seconds while the task runner held the coordinator lock. The watcher now derives due entries from the loaded report; snapshot and tool-request reads are read-only. Package ea672184 exposed a disabled running-task feedback editor; an AccessKit test failed before and passed after enabling that editor only for running tasks. Exact package 6aa687a3 confirmed live/pending rows, enabled feedback for the running task, kept it disabled for the pending task, and enabled Assign after selecting an alternate agent. No feedback or reassignment was submitted. Packaged UI acceptance is 18/25 (72%). Graphify refreshed at 9,278 nodes / 15,677 edges. CocoIndex's installed CLI did not start because `mcp.server.fastmcp` is missing from its environment. See [QAS-158 evidence](../../docs/evidence/issue9-qas158-live-snapshot-20261024.json).

Historical HTTP-transport checkpoint, superseded by [Direct native transport](direct-native-transport.md). Verified: 2026-10-10 KST. This is an implementation checkpoint, not full-product completion.

## Current package acceptance status — 2026-10-20

The first complete workflow in the exact v0.1.30 Rust package is accepted: isolated shared-folder planning, three concurrent mock sessions on disjoint paths, passing tests, human review and merge approval, and a clean completed repository. No provider CLI or external service was called. This moves the overall end-to-end workflow audit to 1/25; the older runtime checkpoints below retain the counts that applied when they were verified. See [package evidence](../../docs/evidence/issue9-35a5c09-native-workflow-01.json).

## Observed runtime

The actual embedded interface was browser-rendered against a named disposable local
fixture in `.agentconductor/verification/ui-room-live-v2`. Three compiled mock subprocesses
completed scoped tasks with test evidence. Computer-use interaction submitted targeted
feedback, moved one task from `local-builder` to `local-reviewer`, selected the observed
`fixture-large` model, resumed the affected task, inspected frozen diff/tests, approved
the exact candidate and separately confirmed merge. The authoritative run then became
`completed`; Git integration is also asserted in the backend test. The original failed
fixture input run remains retained as `ui-room-live` instead of being erased.

The fixture implements its fixed contract; it does not demonstrate AI reasoning,
provider model switching or authenticated native permission enforcement. Its task mode
returns fixed content even when feedback requests different content. The screenshot is
real runtime evidence with fixture results, not a fabricated native-provider session.

## Boundary and lessons

- Typed loopback actions run in owned CLI children, with immutable revision checked
  under the workflow lock. Candidate/tool digests retain their separate authorization.
- Cookie names include the port: host cookies are not port-scoped. Red regression
  confirmed independent windows would otherwise replace each other's capability.
- Report APIs omit profile environment values. Bounded Unix artifact reads walk
  directory descriptors and refuse aliases/nonregular files instead of reopening
  previously checked paths. This is an evidence-read boundary, not OS worker isolation.
- Review stays frozen until explicitly refreshed. Feedback makes prior approval stale;
  acceptance is separate from a CLI child's successful exit.
- A new stop marker cannot be discarded by an older resume request. Separate short
  stop-marker locking serializes comparison/removal and atomic stop replacement.
- Whole-suite testing encountered a retained PID-based directory collision. Test UUID
  directories retain previous failed artifacts and avoid PID reuse.

## Evidence and limits

Backend suite: seven tests across UI and stop/resume suites. Whole suite: 142 passed,
four ignored, 39 suites in 55.78 seconds. Clippy clean. The later targeted UI test also
checks wrong merge digest refusal and that closing the server does not cancel accepted
merge; five tests passed in 2.18 seconds. No runtime dependency was added. Graphify
extracted 482 nodes/1,508 edges without an LLM; CocoIndex located both Controllers.
Ponytail guidance was applied; no executable/plugin invocation is claimed.

## Native issue #9 task-row selection checkpoint (2026-10-18)

The source-only Rust Agent Room renders task rows as selectable buttons. Selecting a
row updates the task-control selection before Activity renders, so the selected
task's detail request and the Inspector follow the same task. The existing selection
transition retains feedback and schedule drafts per task. A pointer/AccessKit widget
test selects the second row and verifies the resulting `TaskDetails(1)` request.
This does not establish selected-state visual parity, full keyboard navigation, or
macOS/Windows packaged acceptance; the issue #9 workflow acceptance count remains
unchanged. Graphify's no-clustering source refresh completed with 6,503 nodes and
11,250 edges.

Parent verification then reported two `native_room_layout` width failures. Running
the same test against an extracted `56182ca` baseline reproduced both failures at
the exact dimensions (`307.8125px` review editor; approval button x1 `906px` against
792px). The initial selectable button raised the approval x1 to `923px`; replacing
its layout-affecting button with a full-row AccessKit click overlay restored x1 to
`906px`, matching baseline. The two legacy width failures remain pre-existing and
are not fixed by this task slice.

## Narrow-window review/approval layout checkpoint (2026-10-18)

The `native_room_layout` assertions exposed two source-probe layout defects: the
task-direction editor had only 307.8125px at a 1140px viewport, and decision actions
ended at x=906 in a 792px viewport. The Inspector now receives 38% of available width
for regular/large Room windows (capped at 480px), while each decision action is
bounded by the visible content edge and wraps its label when needed. All three
`native_room_layout` tests pass: the editor reaches 420px, approval/reject/merge stay
inside the 792px viewport, and the 1200px three-column/accessibility case remains
valid. Exact packaged macOS/Windows visual and keyboard acceptance remains open.

## Approval next-step checkpoint (2026-10-18)

The native status banner now guides the approval sequence. With no current evidence,
Review changes selects the Diff & tests pane and requests evidence. If the run revision
changes, the CTA returns to review instead of exposing approval. Once evidence matches,
Approve candidate opens the existing approval confirmation; after approval, Merge
candidate opens the existing merge confirmation under the same revision gate. The
rendered interaction regression covers this pointer path and a stale-evidence retry.
This is a widget/source checkpoint only; packaged macOS/Windows keyboard and visual
acceptance remain open, and it does not complete UC-101 or the 0/25 workflow audit.

## Completed-run Workspace return checkpoint (2026-10-18)

The completed-state banner now offers `Return to Workspace` when the Room was opened
from the Workspace window. Activating it closes the embedded Room, selects Workspace
stage 1, and refreshes the saved workspace snapshot. A standalone Agent Room has no
Workspace context, so it displays localized guidance to open that window instead of
showing an action that cannot work. Rendered tests cover the action and the standalone
guidance. Exact packaged macOS/Windows behavior and full phase-planning parity remain
open; this checkpoint does not change issue #9's accepted workflow count.

## Active-goal phase progress checkpoint (2026-10-18)

The native Workspace now reads the active goal from the existing `slugs` snapshot and
shows its ordered phases, current stage, and task/verification/completion counts. If an
unfinished phase is not already open in Review Plan, the accessible `Plan next phase`
action sends the current workspace revision with the backend's existing `next_phase`
request; the UI moves to Review Plan only after the async request succeeds. The action
is absent when all phases are complete or that phase is already under review. Rendered
tests cover phase visibility, pointer activation, request shape, completed state, and
duplicate suppression. Selecting other goals, phase-spec disclosure, and exact packaged
macOS/Windows acceptance remain open; the accepted workflow count is unchanged.

Cocoa/WebKit launcher compiled and started. Embedded Mach-O Info.plist follows Apple's
[single-file tool metadata guidance](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Procedures/Procedures.html)
and [build-setting section contract](https://developer.apple.com/documentation/xcode/build-settings-reference).
The computer-use app inventory initially did not recognize the unbundled executable.
A local app metadata wrapper around the same executable enabled actual Cocoa/WebKit
review rendering and a native screenshot. Cmd-W initially left the window open; adding
standard application/edit/window menus fixed it. The updated wrapped process exited 0
after Cmd-W, and its local endpoint stopped. Bare-executable inventory identity remains
open. Native lifecycle with active workers is covered separately by the backend child
continuation test; it is not yet a native active-worker UI interaction proof.
Browser rendering at native 1280×860 and narrow 715px exposed and corrected clipped
controls. System Cocoa/WebKit is linked, not bundled for redistribution; final release
closure/resource/license checks remain open. Initial goal/profile setup and UI schedule/
tool-gate end-to-end interactions are also open in the completion audit.

## Exact packaged Rust plan-to-merge run (2026-10-19)

The validated `aarch64-apple-darwin` package from issue #9 source `eeb90a9`
completed a disposable plan-to-merge run. An isolated Rust planner returned three
role-scoped tasks; three Rust mock workers wrote distinct files and all three
`git diff --check` commands passed. The package displayed the exact-plan consent,
the `Starting agent execution…` indicator, 3/3 task completion, 3/3 verification,
and the candidate diff with exit codes. The operator approval and merge were
separate confirmed actions. The app reached `Completed`, and the temporary Git repo
recorded candidate commit `0e3a59d` with all three expected files.

The intentionally invalid missing-role plan was rejected before execution. The
second run used explicit role mappings. Both used an isolated HOME and no provider
credentials; the package process, extracted bundle, mock executable, and test repo
were removed after verification. The checked-in package ZIP remains available.
This confirms one macOS runtime path only. It does not establish cross-platform
visual parity, live CLI behavior, or complete UC-101 acceptance; issue #9 remains
at 0/25 under the documented workflow audit.

## Approved v0.1.30 rerun (2026-10-19)

After cleanup removed the staged fixture, the user approved a fresh disposable run
that retained the same three file scopes and `git diff --check` gate. The exact
v0.1.30 package binary (`d2b66628…`) launched once with an isolated HOME. A local
Rust mock planner created three role-mapped tasks; the registered mock worker ran
them as separate sessions. The regenerated plan digest (`9e764764…`) and fixture
base (`ed1841f`) differed from the deleted staged plan (`a44381e…`, base `0c17ea0`),
so the regenerated goal, assignments, writable paths, and commands were reviewed
in the app before execution. The candidate contained only `one.txt`, `two.txt`, and
`three.txt`; each recorded test exited 0, and an independent `git diff --check`
also exited 0. After inspecting the exact diff and test evidence, the candidate was
approved and separately merged. The disposable Git repository ended clean at
`8e152db` with exactly those three added files. No provider CLI or external service
was called. After verifying the test process exited, the temporary app bundle, HOME,
workspace, repo, mock binary, and compatibility link were removed; the user's existing
workspace was not touched. This is mock macOS runtime evidence only; Windows and the
full 25-flow UI audit remain open.

## Fixture hashes

- Integrated candidate: `00c139f33a718e84697eea470ac782cf56649db9`
- `.agentconductor/verification/ui-room-live-v2/run/report.json`: `3a6a94cf39e280272bacd739079eb0397ab245b72cbfb3d8b4991ce7d1a9cb1b`
- `.agentconductor/verification/ui-room-live-v2/run/diff.patch`: `99b82e014aaef706a0cde81f93071d218c83085f73eeb8bd1673ea659f3a77ba`

## Source fingerprints

- `src/ui.rs`: `2d0ab9957bf89473f4f1af1c8581abd2d899df48b62c8c90c6a850c818ec405a`
- `src/ui/native.rs`: `00500880ed225ba8a702ffb675308fcc7fbc78e888a295dcc4622773121a9763`
- `src/ui/Info.plist`: `1f63e96938d60adc940e3bf09571934a7484ca59ddbd8341b6cbfa0c55c38bd9`
- `src/ui/room.html`: `1b861537466a3c8d96b8b69b07dcb769cdd82e3b5886550f5d8d57cbbd6ee5f5`
- `src/ui/room.css`: `ebd1c7802e8b5d3dd7676e8209752347320d4f67bc7bf8ad681ec6b2789ba825`
- `src/ui/room.js`: `b31c507b45d5795694ca76602d825bb62ff73b5b3beb62426dfaf5c6e9da1ac8`
- `src/workflow.rs`: `d37be52e2921d511b4aaf57530dc849ca69715e10f4a2d033c06f4121894678f`
- `tests/ui.rs`: `abccea94bc0843d7620edf82221ccd7af36ccc77d9ce3067fbb133e337b7980c`
- `tests/ui_resume.rs`: `05eae4d05383b5ea32cd901b88483650ff4859d7988b89b08229a581b3b49435`
- `docs/assets/ui-agent-room-live.jpg`: `6fd7c9ba6a622ff2b2fa448b3353e43faf7451bb1e173a8badf8505723fdeb1a`

## Verification process cleanup

Direct user correction: close previous owned verification windows/processes and verify
exit before opening a replacement. Store lesson `lsn_5169ce1ac0c2a86d` with confidence 0.7.
The native interface now holds a per-run kernel lease; another CLI process refuses
opening the same run while it is held, and release permits reopening. This does not
terminate coordinators or allow unsafe takeover. All known native/server handles were
terminated or observed exited before final cleanup. macOS pgrep inventory is unavailable
(`sysmond service not found`), so it must not be cited as proof of absence. Native app
inventory and owned handles/endpoints provide the observed cleanup evidence.

A separate availability fixture (`ui-availability-live`) retained three children ending
without an exit code or stdout; a direct single fixture invocation later emitted a
valid usage-limit event. The cause of this subprocess outcome remains unresolved, and
availability UI acceptance stays open. Do not hide it by replacing the retained run.

- `docs/assets/ui-agent-room-native.jpg`: `d979c61985047936e26464020946e4d16bd20bffcc772dd88f0d86af6a51aa0e`

## Generated role mapping gate — 2026-10-23

The exact v0.1.36 macOS package rejected a generated task when the planner assigned the `documentation` role without a configured agent (`task task-2 uses role documentation, which has no rule`). Assigning that role to the deterministic mock agent allowed the three disjoint tasks to run. All three `git diff --check` commands passed. The candidate was inspected and left waiting for human approval; it was not merged. This fixture checks role mapping and the packaged review path only, so it adds no workflow acceptance credit. No provider CLI, MCP server, network service, or user repository was used. See [the run evidence](../../docs/evidence/issue9-mock-approval-ui-20261023.json).

## Session and model controls — 2026-10-24

Workflow 19 is verified in the exact v0.1.37 macOS package. The Agent Room shows the selected task's provider session ID as read-only and lets an operator choose only from that agent's observed catalog. Saving a different model goes through the Rust dispatcher and invalidates prior task evidence so the work is re-verified. The deterministic package fixture confirmed the UI and persisted task model, but it does not stand in for an authenticated provider's session or live catalog. See [workflow evidence](../../docs/evidence/issue9-workflow19-session-model-20261024.json).
