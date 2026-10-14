# System features

| ID | Feature | Goal | MVP |
|---|---|---|---|
| SF-01 | Registered subprocess profiles and parsers | BG-01 | Included |
| SF-02 | Scoped plan (1–12 tasks, R-53) and dependency scheduler | BG-02 | Included |
| SF-03 | Shared workspace or optional worktrees, ownership, integration candidate | BG-02/03 | Included |
| SF-04 | Logs, results, diffs, test evidence, resume | BG-03 | Included |
| SF-05 | Dangerous-action controls, human approval, merge | BG-03 | Required; enforce provider/sandbox capabilities before execution |
| SF-06 | SQLite state, audit, recovery | BG-03 | Included |
| SF-07 | agentmemory, local wiki, lessons | BG-04 | Minimal integration |
| SF-08 | CLI status, review, approval list | BG-03/05 | CLI foundation verified; functional interaction UI remains required |
| SF-12 | License/SBOM and TDD verification | BG-05 | Development/release procedures |

Automatic planning runs the coordinator CLI once to produce JSON, then validates it
in Rust. The coordinator reuses its login. Manual plans use the same validation path.

## Availability and session controls

- SF-09 (BG-07): Local persisted reset detection, manual scheduling, due-work execution, bounded retries, and low-idle-cost watch mode.
- SF-10 (BG-08): Multiple sessions per profile, active-session exclusion, account quota grouping, per-profile concurrency, and audited reassignment.
- SF-11 (BG-09): Native catalog discovery/import and per-task session model controls with approval invalidation.

### SF-18 — Native startup and exact-plan execution consent

Persist a versioned local agent registry and goal workspace, discover observed catalogs, generate read-only proposals, render editable task scopes/tests/models, and require the exact reviewed plan digest for execution. Retain coordinator ownership and logs across UI lifecycle. Trace: BG-09 / R-21.

### SF-19 Account quota summary

Normalize provider-reported quota windows with remaining percentage, reset timestamp and freshness; show independent windows and unavailable states in native agent summaries. Query metadata through the registered CLI without generation prompts or API-key setup.

### SF-20 Cross-platform build and release

Commit a Windows development handoff, execute platform build/test jobs, implement native Windows safety primitives and publish verified platform artifacts with checksums and notices.

### SF-21 Deterministic supervision and reviewed fallback policies

Separate process supervision from optional semantic AI review. Register per-agent fallback targets/models, validate their permission/account boundaries, retain durable restriction observations and transfer work only after active attempts drain. Non-steerable CLIs receive scoped next-attempt handoffs. Trace: BG-11.

### SF-22 Evidence-based team briefing

Generate a bounded local briefing across agents and sessions using task/attempt/process evidence and intermediate memory reports. Include observation timestamps, unresolved conflicts, approvals and next actions. An optional coordinator may verbalize the same factual bundle. Trace: BG-12.

### SF-23 Agent status indicators

Render deterministic status-to-light mapping directly from observed task/session state. Preserve selection and pulse continuity across polling. Trace: BG-12 / R-29.

### SF-24 Cooperative memory control channel

At declared checkpoints, capable CLI sessions publish intermediate reports and pull addressed directives from agentmemory. Local command identity, acknowledgment and task/process reconciliation make this channel auditable across agents. Trace: BG-11/BG-12 / R-27.

### SF-25 Live tool decision gate

Hold a guarded worker's pre-tool callback while an exact ledger request awaits a human decision; decide it transactionally from UI/CLI while the coordinator owns the run; consume an approval once in the same attempt; refuse only the call on rejection. Trace: BG-13 / R-30.

### SF-26 Isolated planning snapshot

Materialize the reviewed base's tracked files into a disposable directory outside the repository, run the coordinator there, discard it and verify the target workspace is unchanged. Trace: BG-13 / R-31.

### SF-27 Input-method-safe native forms

Preserve composition state and focused values during polling/re-rendering and keep UTF-8 Korean text intact end to end. Trace: BG-13 / R-32.

### SF-28 Workspace navigation and draft discard

A persistent titlebar action returns to the current run's Agent Room, and a discard action restores the last saved workspace and plan. Both are local view operations; discarding requires confirmation when edits exist. Trace: BG-09 / R-33.

### SF-29 Provider application icons

Resolve the installed desktop application for each built-in provider, render its icon as a bounded PNG data URL in UI snapshots and fall back to letter avatars. Trace: BG-14 / R-34.

### SF-30 Public product documentation

A concise README (introduction, screenshots, install, desktop walkthrough) and a complete user manual covering the desktop workflow, every CLI command, configuration and safety behavior, packaged with releases. Trace: BG-15 / R-36.

### SF-31 Manual plan authoring

Create an editable three-task plan draft in the workspace without launching a coordinator; reuse plan editing, validation, digest approval and repository-base binding. Trace: BG-16 / R-37.

### SF-32 Workspace settings and folder picker

A labeled settings section (goal, coordinator, repository with native folder picker, worktree option); saving does not require a repository. Trace: BG-17 / R-38.

### SF-33 Workspace menu

Current workspace shown in the title bar with a menu of recent workspaces, open folder and new workspace; switching happens in place under the one-window-per-workspace lease. Trace: BG-17 / R-39.

### SF-34 Workspace names and registry

Workspace name field above the goal; registry of named workspaces with create-by-name, open-saved (listing workspaces stored in `~/.agentconductor`), switch and remove-from-list. Trace: BG-17 / R-40.

### SF-35 Release change summaries

`CHANGELOG.md` holds one section per version tag; the release workflow publishes that section as the release notes and stops when it is missing. Trace: BG-10 / R-41.

### SF-36 Baseline commit prompt

Before planning, report repository cleanliness and changed files; on the operator's choice, commit all changes as the baseline and continue, or stop without changes. Trace: BG-13 / R-42. Untracked OS metadata files can be excluded locally instead (R-44).

### SF-37 Messenger connectors (planned)

A provider-neutral notification and decision channel with a Slack connector and a generic company-messenger connector (outgoing webhook plus signed inbound callbacks or polling), per-workspace channel mapping, redaction policy and linked approvers. Trace: BG-18 / R-43.

### SF-38 Planning progress indicator

Job state with start time and log location; review area shows coordinator, elapsed time, ready notice with scroll, and failure reason. Trace: BG-12 / R-45.

### SF-39 Readable plan instructions

Sectioned, line-broken instruction format required from the planning coordinator in the goal's language; auto-growing instruction editor (minimum six rows). Trace: BG-13 / R-46.

### SF-40 Agent role rules

Workspace role table (planning = coordinator; verification, development, documentation, knowledge, testing → ordered registered agents, each with an optional observed model) passed to the planning coordinator; task `role` field; rule enforcement on generated plans; role shown in plan review; numbered steps 1 Workspace settings → 2 Agent roles → 3 Review the plan. Trace: BG-19 / R-47.

### SF-41 Immediate agent registry persistence

A dedicated, revision-checked request stores only the agent list, repairing the coordinator and role rules when an agent is removed. Trace: BG-09 / R-48.

### SF-42 Automatic model catalog preparation

Page-driven sequential discovery for built-in agents without a catalog while no plan proposal exists and nothing is unsaved; catalog preservation on save. Trace: BG-10 / R-49.

### SF-43 Workspace settings export/import

Versioned `agentconductor.workspace.json` written to a chosen folder (default: the repository); import from a folder with preview, local-state preservation and capability re-verification. Trace: BG-20 / R-52.

### SF-44 Variable-size plans and plan-document import

Plan validation for 1–12 tasks, coordinator prompt for the smallest disjoint split, add/remove task controls in review; **Read plan…** converts a chosen Markdown document through the read-only planning path with role rules. Trace: BG-02/BG-19 / R-53, R-54.

### SF-45 Slug work trees

Slug creation from goals (accumulating), per-slug folder tree in the repository's `.agentconductor/` (locally excluded from Git status) or the workspace data folder. Trace: BG-21 / R-55.

### SF-46 Phased planning and execution

Analysis document plus ordered phases of tasks from the coordinator; one run per phase; task files move task → verification → completed with the run's lifecycle. Trace: BG-21 / R-56.

### SF-47 Progress model and tree verification

Progress derived from file locations; workspace and export record the derived state; a verifier reports any disagreement. Trace: BG-21 / R-57.

### SF-48 Slug archives in export/import

Store-only `<slug>.zip` archives beside the settings JSON, listed with progress; import checks presence, names and contents before restoring. Trace: BG-20/BG-21 / R-58.

### SF-49 Plan rules file

`~/.agentconductor/plan-rules.md` (created with defaults, editable in the window) and a workspace override in the settings; sent to the planning coordinator. Trace: BG-21 / R-59.

### SF-50 Passive Claude quota collection

Normalize owned `rate_limit_event` messages into quota snapshots during execution; display source, freshness, independent reset windows and unknown utilization. Claude Refresh checks retained observations without launching a worker. Trace: BG-22 / R-60.

### SF-51 Inline quota badges — Compact capacity badge beside paid agent names in workspace and Agent Room lists. Trace: R-61 / BG-22.

### SF-52 Configured MCP goal imports

OpenCode-compatible local/remote MCP registration in workspace settings, source bindings, goal-reference chips, previewed read-only import and source-linked planning. Trace: BG-23 / R-62.

### SF-53 Per-agent quota refresh and compact actions

Independent per-agent refresh deadlines, manual refresh icons and accessible agent action icons. Trace: R-63 / BG-22.


### SF-54 Compact plan-aware quota control

A single visual group contains slash-separated percentages on one line and the per-agent refresh button. Period labels stay in the tooltip. Provider plan metadata and window durations survive cache persistence. Trace: R-64 / BG-24.

### SF-55 Native agent preferences

Persist per-agent cadence, resolve executable paths through filesystem lookup, offer an OS file picker, and embed shared AC artwork in the native executable. Trace: R-67 / BG-25.

### SF-56 Provider-driven agent profile

Selecting a provider fills ID, resolved executable, arguments, input, permission, sessions and cadence from that provider's template; renaming an agent rewrites references (coordinator, roles, fallbacks). Trace: R-68 / BG-26.

### SF-57 Exhausted-quota reset time

The quota control appends the latest reset time among exhausted windows; the tooltip lists every window's reset. Trace: R-69 / BG-26.

### SF-58 Language setting and UI translation

A machine-wide preference (`~/.agentconductor/preferences.json`, `language`: `en` | `ko`) is embedded in the page at load; a Settings dialog changes it at runtime. Stable message IDs address language-specific JSON catalogs for interface text, placeholders, titles, aria-labels and native app diagnostics. Explicit bindings rerender on language changes; missing translations fall back to English. Trace: R-70 / BG-27.

### SF-59 On-demand fallback catalog

The fallback model select reflects the chosen fallback agent's catalog, queues one model fetch when missing and refreshes in place. Trace: R-71 / BG-26.


### SF-60 Windows provider metadata parity
Resolve runnable Windows launchers, collect Claude /usage through a short-lived ConPTY session, retain existing Codex app-server collection and show original built-in provider badges when installed artwork is unavailable. Trace: BG-28.


### SF-61 Explicit operator model catalogs
Custom Models opens a compact ID editor with optional observed JSON import, explicit provenance and no subprocess/network query. Trace: BG-29.

SF-37 phase 2: inbound decision channel by bounded polling (no public URL, no resident process): Slack `conversations.history` with a bot token, or a company relay returning HMAC-SHA256-signed decisions; linked approvers per workspace; decisions applied through the existing exact tool-decision path.

SF-46 amendment (R-74): automatic slug naming with Hangul romanization and uniqueness; no slug input.

### SF-62 Budget-aware quota normalization
Collect CLI-reported budgets independently of rolling usage windows, preserve scope, allocation, reset and provenance, and render remaining/allocated compactly. Trace: BG-30.


### SF-63 Native window ownership
Use an in-process AppKit picker on the main thread and Windows GUI subsystem with console-free subprocess creation. Trace: BG-31.


### SF-64 Separate directory settings from execution baseline validation
Validate selected directories at save time, and validate Git plus committed baseline only at planning entry. Quota does not depend on either. Trace: BG-32.


### SF-65 Asynchronous metadata lane
Separate foreground planning from a single metadata worker with a bounded queue, per-agent busy controls and nonblocking executable lookup. Trace: BG-33.


### SF-66 Empty baseline setup with explicit consent
Classify missing Git/HEAD, ask consent, create a truly empty commit with compare-and-set HEAD, preserve the index, audit locally and recheck existing file changes. Trace: BG-34.


### SF-67 Private folder snapshots and approved application
Copy a bounded source snapshot into the workspace, retain original-file fingerprints, reuse existing plan/execution/diff/test/approval machinery and apply only the approved candidate to the unchanged original. Trace: BG-35.


### SF-68 Static agent tools guide
A Settings dialog lists each tool with its purpose, prerequisites and exact install commands for Claude Code and Codex; the user manual mirrors it. Trace: BG-36.


### SF-69 Encoding-independent picker result
The Windows picker process writes the chosen path as UTF-8 without a byte-order mark, and the app strips any leading BOM and line ending before using it. Trace: BG-37.


### SF-70 Planning activity feedback
Planning-entry requests allow for the bounded folder copy; the snapshot carries the coordinator output's size and last write; the review area reports first output, recent output or two minutes of silence. Trace: BG-38.


### SF-71 Coordinator-proposed slug
Non-English goals start with a date placeholder; the planning prompt asks for an English snake_case slug, which is validated and replaces the placeholder when the plan is ready. Trace: BG-39.


### SF-72 Percentage allocation badge
The capacity badge shows reported remaining_percent, or remaining divided by a positive allocation, as N%/100%; otherwise —. Trace: BG-40.


### SF-73 Windows Claude collector working folder and blockers
The ConPTY collector passes an ordinary drive path as working directory and stops at trust or first-run screens with a specific message. Trace: BG-41.


### SF-74 Readable Windows project paths
Trace: R-87 -> BG-42 -> SF-74 -> UC-77 -> FR-72 -> QAS-83 -> DD-95.
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity.


### SF-75 Reliable named workspace creation
Trace: R-88 -> BG-43 -> SF-75 -> UC-78 -> FR-73 -> QAS-84 -> DD-96.
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable.


### SF-76 Delete an unwanted workspace
Trace: R-89 -> BG-44 -> SF-76 -> UC-79 -> FR-74 -> QAS-85 -> DD-97.
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first.


### SF-77 Register MCP servers in Settings
Trace: R-90 -> BG-45 -> SF-77 -> UC-80 -> FR-75 -> QAS-86 -> DD-98.
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration.


### SF-78 Awaited profile saving with inline feedback
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


### SF-79 Configurable signal transport and automatic local fallback
Workspace Settings selects Local Rust or agentmemory preferred independently of service availability. A Rust-owned durable inbox/outbox routes exact-session messages and retains delivery/acceptance evidence. A failed preferred memory transport can use the local adapter without duplicating execution. Expose configured preference, effective transport and a bounded failure reason. Remote recipients without a working authenticated route remain pending rather than being reported as locally delivered.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### SF-80 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.


### SF-81 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.


### SF-82 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.

### SF-83 Agents work inside the workspace sandbox; the coordinator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Work confined to the task workspace runs at once; a request reaching outside it (network, installs, history-changing Git, privilege, other processes, outside paths, secrets) is decided by the workspace coordinator , and a person is asked only when no coordinator can decide. The permission level is not chosen in the agent profile; it follows from the CLI. Time spent waiting for a decision does not count toward the task limit, and a task that times out can be resumed instead of failing.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.

### SF-84 Usage query accepts the Claude folder-trust prompt
Claude usage must load even when Claude first asks whether to trust the project folder. The usage query runs in a separate session with no tools, hooks or project settings, so it accepts the prompt automatically in both forms (Enter for the default choice, y for the screen-reader "Enter y/n:" form) at most twice, and stops with guidance instead of waiting for a timeout if Claude keeps asking.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.

### SF-85 Planning start and workspace naming feedback
Generate plan shows progress with elapsed time from the moment it is pressed, including while the project folder is copied before the coordinator starts. Saving workspace settings updates the titlebar name at once. Deleting the current workspace asks for the name of the fresh workspace that replaces it instead of always naming it "New workspace".
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.

### SF-86 Run and task status at a glance
The Agent Room shows the run's current final state immediately: a run status banner with the state in bold colour, task progress and the last error, and a prominent Resume button whenever the run can continue. Task states (failed, completed, pending, waiting, running) are bold and colour-coded in the session heading and in the left session list. In the left list a failed task is red, a task waiting for a decision or tool request has a blinking border, pending tasks use a strong colour, and the Tool requests count turns red and blinks while requests wait.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.

### SF-87 Goal folders directly in the project folder
When a workspace has a project folder, each goal (slug) folder is created directly in it as <project>/<slug>/ (for example C:Templadderladder_and_priority_games) with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Goal folders in a former place, including the interim <project>/<workspace name>/<slug>/ of an earlier build that misread the request, move there when first used.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.

### SF-88 Agent CLI list is a master list
The registered agent CLI list is kept as a master list for the machine, not per workspace: deleting a workspace never removes agents, and every workspace opens with the master list. When a workspace refers to an agent the master list no longer has (planning coordinator or role assignments), the assignment is shown in red and can be reassigned; Generate plan stays disabled until every assignment refers to a listed agent.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.

### SF-89 Reorder agent cards
Agent CLI cards can be moved up and down with the mouse (drag and drop), with keyboard-accessible move controls as an alternative. The order is saved, and every agent selection list (planning coordinator, roles, fallback) follows the same order.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.

### SF-90 Readable file paths everywhere
Screens, error messages and notifications never show Windows extended-length path prefixes such as \\?\C:\Work\Temp or //?/C:/Work/Temp; paths read as C:\Work\Temp.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.

### SF-91 Activity tab reads like the CLI
The Activity tab shows agent output the way the CLI shows it: assistant messages as text, tool calls as a one-line action with its key input, tool results as short summaries and errors highlighted. Raw JSON is never shown there; it remains in the Raw logs tab.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.

### SF-92 Coordinator final step in the task list
The Agent Room task list shows the step the coordinator runs after the tasks, integration verification of the combined result, as its own entry with its status (pending, running, completed, failed) and which task tests failed, so a run that fails after every task completed shows why without opening other views.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.

### SF-93 Decide a live tool request from its window
When a tool request window opens while the run is working, Allow once and Deny are available at once, without closing the window with Decide later and deciding again from the Tool requests list.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.

### SF-94 Coordinator decides routine requests and records each decision
Simple decisions are made by the coordinator so the user is not asked often: it allows routine development requests, denies only publishing, outside-project changes, secret exposure or system changes, also for runs started before orchestration existed, and every decision is recorded with the request, the result, the reason and the time, visible in the task's Activity. Commands inside a workspace that itself lives under an app folder (such as ~/.agentconductor) are workspace commands.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.

### SF-95 Task list without account usage
The Agent Room task list does not show account usage badges or refresh buttons; account limits are shown in the inspector's Account capacity section.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.

### SF-96 The coordinator is selectable and shows its work; one name
The interface and documents use one name, coordinator, for the agent that plans, verifies the combined result and decides tool requests (previously also called orchestrator). In the Agent Room task list the coordinator entry is selected like a task: the center shows its heading, status and an updating Activity of its run steps, tool decisions and integration test results, and Raw logs show the integration test output, following the Agent room design (select on the left, progress in the center, evidence on the right).
Trace: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117. Source: user request, 2026-10-13.

### SF-97 Failure cause and the next action on a failed run
When a run fails, the status banner shows why (for an integration verification failure, the failed task and the first lines of its test output) and offers Fix with feedback, which selects the failed task and drafts a direction with the failure output for the user to review and send; sending it reruns the task and repeats verification.
Trace: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118. Source: user request, 2026-10-13.

### SF-98 The next step is always shown
The Agent Room status banner always says what to do next and offers that action: review the changes when the run waits for approval (Approve stays unavailable until the latest evidence is reviewed), approve after review, apply or merge after approval, and return to the workspace to plan the next phase when the run is complete.
Trace: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119. Source: user request, 2026-10-13.

### SF-99 Plan review and run completion are organised by phase
A phased goal shows its phases in Review the plan: which phase is under review (phase n of m), which are completed, which comes next, and a Plan phase n action when the next phase is ready to load. When a run finishes, the Agent Room says which phase completed and which phase is next, and that it is planned from the workspace; a completed phase is recorded even if the goal folder moved after the run started.
Trace: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121. Source: user request, 2026-10-13.

### SF-100 A blocking verification finding fails integration verification
When a verification report changed in the run records a blocking issue (a Blocking/차단 section with items, or a positive blocking-issue count), integration verification fails like a failing test, naming the report and the task responsible for the files the issue names, so the coordinator does not accept a candidate its verifier rejected and Fix with feedback drafts the correction for that task.
Trace: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120. Source: user request, 2026-10-13.

### SF-101 The run's phase beside the workspace name
The Agent Room titlebar shows, beside the workspace name, the phase the run executes (phase n of m and its name) and what it covers (the phase description, or its task list when the phase file has no description); the full text is in its tooltip.
Trace: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122. Source: user request, 2026-10-13.

### SF-102 Readable Agent Room text and agent icons sized to their names
The Agent Room's text is at least 11 px and scaled up in proportion (body text 14 px instead of 12 px) for readability, workspace screen text is at least 11 px, and the agent icons in the task list are the height of the name beside them instead of a large tile.
Trace: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123. Source: user request, 2026-10-13.

### SF-103 Task cards in Review the plan fold to one line
In Review the plan each task is folded by default, showing only its number, name and a one-line description (the first line of its instructions that is not a label); an arrow unfolds it to show and edit every field.
Trace: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124. Source: user request, 2026-10-13.

### SF-104 Review the whole phased plan up front
The coordinator plans every phase of a goal when the plan is generated, so Review the plan briefs each phase (what it will do, from the coordinator's analysis) and lists each phase's planned tasks, with the coordinator's analysis of the whole goal available; the phase about to run is open and its tasks are edited below, completed phases are marked, and later phases can be unfolded to review their tasks before any of them runs.
Trace: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125. Source: user request, 2026-10-13.

### SF-105 A run survives closing or losing the desktop app
Closing the desktop app (or the app process ending) must not end a run: the run coordinator and its agent workers keep working and the app reattaches when reopened. Closing the window while a run is active asks: Keep running in the background (default), Stop the run and close, or Cancel.
Trace: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127. Source: user request, 2026-10-13.

### SF-106 Interrupted runs recover and resume automatically
When a run is recorded as running but its coordinator and workers are gone (app killed, crash, reboot), opening the workspace or the Agent Room recovers it automatically (verifying every recorded process is absent, as Recover does) and, with automatic resume on by default, resumes it: completed tasks are kept, the interrupted task continues in its own native session (or a new one if the session cannot resume) with a note that its previous attempt was interrupted and its owned files must be inspected. After two consecutive automatic resumes that are interrupted again, it stops and asks the operator.
Trace: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128. Source: user request, 2026-10-13.

### SF-107 Interruptions and silent workers are visible
The Agent Room shows when a run was interrupted and recovered (banner history and coordinator Activity), collapses a worker's consecutive thinking_tokens events into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows, and warns when a running task has produced no output for several minutes or its process is gone, suggesting Recover.
Trace: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129. Source: user request, 2026-10-13.

### SF-108 Reassign tasks that have not started while the run is working
While a run is working, the operator can change the agent (and model) of a task that has not started yet, for example to move work from a busy Claude account to Codex; the change applies before that task starts. Running and finished tasks still need the run to be idle.
Trace: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130. Source: user request, 2026-10-13.

### SF-109 Agents run with the highest permission by default
Agent workers run with the highest permission their CLI offers by default (Claude without permission prompts and with the operator's MCP servers such as Playwright, so it can open and inspect a browser UI; Codex without approvals or sandbox), while the planning and review coordinator stays read-only. The operator can lower a profile's permission.
Trace: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131. Source: user request, 2026-10-13.

### SF-110 The coordinator resolves verification failures
When every task of a run has completed but integration verification fails (a failing test command or a verification report that records a blocking issue), the run does not stop at Failed waiting for the operator: the coordinator reviews the evidence, revises the tasks whose files must change and the verifying task, and the run resumes. A finding the run cannot resolve (outside every task scope, or needing an environment the agents lack) is recorded by the verifying task as a deferred non-blocking follow-up with its reason and owner. After three repair rounds without passing, the run stays Failed with the reason, for the operator to revise or reassign. Resume on such a failed run starts this repair.
Trace: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132. Source: user request, 2026-10-13.

### SF-111 Direct ordinary-folder execution and external baseline history
Keep ordinary-project Git objects and indexes in app-managed external history. Bind every run to its selected source root and recorded baseline. Expose Accept, baseline-restoring Reject, and owned-file edit warnings.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user workspace-path requirement and Claude review, 2026-10-13.

### SF-111 Deleting workspaces needs no replacement name
Deleting a workspace asks for no new workspace name. When the current workspace is deleted, the most recently used other workspace opens, so several workspaces can be deleted in a row; only when none is left does one empty workspace start.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user request, 2026-10-13.

### SF-112 Bilingual release documentation
Expose both manuals from the README and include their linked actual screenshots in release documentation assets.
Trace: R-125 -> BG-80 -> SF-112 -> UC-115 -> FR-110 -> QAS-121 -> DD-134. Source: user release/manual requests, 2026-10-14.

### SF-113 The app updates itself from GitHub releases
Every 10 minutes the app checks the latest GitHub release in the background. When a newer version exists, an update icon appears left of Settings; the icon and the workspace menu item Check for updates open the same dialog with the installed and latest versions. Update and restart downloads the archive for this platform, verifies it against the release SHA256SUMS, replaces the installed executable (or macOS app bundle) and restarts the app. It is refused while any run or app job is active, because restarting ends their agents.
Trace: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135. Source: user request, 2026-10-14.


### SF-114 Persistent availability coordinator
Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

### SF-115 Theme tokens and theme preference

All interface colors come from semantic tokens (surface, ink, line, accent, ok, warn, err, info, titlebar); `data-theme` on the page root selects the light or dark set; the preference is stored with the language. Trace: R-128 / BG-83.

### SF-116 Plans know whether the project uses Git, and verification commands that cannot work are replaced
The planning coordinator is told whether the project folder is a Git repository and does not plan git test commands when it is not. If a planned test command cannot work in the project (for example git in a folder without Git) or an agent reports that its test command must change, the coordinator review replaces it with a command that verifies the same work instead of deferring the failure, and the replacement is recorded. Test commands of a folder without Git run against the history AgentConductor keeps outside it.
Trace: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138. Source: user request, 2026-10-14.

### SF-117 Compact capacity and consistent step introductions

Shared quota formatting preserves reset information in the accessible tooltip. Workspace cards use compact controls; three persistent step panels share a title/introduction hierarchy.
Trace: R-130 -> BG-85 -> SF-117 -> UC-120 -> FR-115 -> QAS-126 -> DD-139. Source: user, 2026-10-14.

### SF-118 Focused Settings and menu-based tool guide

Settings fields use explicit label/control spacing. Signal transport controls server-field visibility; the workspace menu opens the existing static installation guide.
Trace: R-131 -> BG-86 -> SF-118 -> UC-121 -> FR-116 -> QAS-127 -> DD-140. Source: user, 2026-10-14.

### SF-119 Custom CLIs that approve their own tools can run
A custom agent CLI that approves its own tool use (for example one set to auto-approve tools) can be marked so in its agent settings; it then runs after plan approval instead of being refused as unverified. The mark is the operator's claim for this machine and is not trusted when settings are imported from another machine.
Trace: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141. Source: user request, 2026-10-14.

### SF-120 Embedded release body

Expose the fetched GitHub body through update status and display it as bounded-height scrollable text below the release link.
Trace: R-133 -> BG-88 -> SF-120 -> UC-123 -> FR-118 -> QAS-129 -> DD-142. Source: user, 2026-10-14; next release after v0.1.18.

### SF-121 Server-linked automatic document tool selection

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.


### SF-122 Current planning job and CLI icon activity

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.


### SF-123 Actual review changes and structured API restrictions

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.


### SF-124 / SF-125 Transport warnings and embedded manual

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

### SF-126 Workspace and Agent Room feedback refinements
Step 1 shows the planning coordinator right of the workspace name and the project folder above the goal, without a divider. MCP lookups show that they are running; Preview opens its window at once with a progress pulse and fills it when the lookup ends or shows the failure. jira:KEY and confluence:ID references in the goal are shown as tags that stay plain, deletable text, and hovering one shows its title and summary. The goal heading status follows the started run instead of staying Started. In the Agent Room, account usage shows Enterprise budgets as a remaining percentage like the agent cards, and the task shown on the right has a strong border in the list.
Trace: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148. Source: user requests, 2026-10-14.

### SF-127 Runs keep going and stay visible
A model reply such as "Selected model is at capacity" is retried after a short wait, at most three times, for tasks and for the coordinator. Files an agent creates outside its owned paths in a project without its own Git (for example a CLI saving .aipro/plans) are moved out of the project into the run folder and recorded. Agent Room task rows lead with the reviewed task summary. A pressed button shows that its work is running, distinct from a disabled button, until the request and any background job it started end. The workspace page and the Agent Room stay open side by side: the operator switches freely, the room keeps its live progress, and the workspace shows the run state on its back-to-run button.
Trace: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149. Source: user requests, 2026-10-14.


### SF-128 Backend-aware prompts / SF-129 direct Windows Common Item Dialog

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

### SF-131 Generated offline HTML manual

Markdown remains the editable source. A dependency-free converter produces standalone HTML for both languages; the native app loads the chosen language only when help opens.
Trace: R-144 -> BG-99 -> SF-131 -> UC-134 -> FR-129 -> QAS-140 -> DD-153. Source: user, 2026-10-14.
