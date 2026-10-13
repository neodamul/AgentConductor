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

A machine-wide preference (`~/.agentconductor/preferences.json`, `language`: `en` | `ko`) is embedded in the page at load; a Settings dialog changes it at runtime. A translation table maps exact English interface strings (text, placeholder, title, aria-label) and known prefixes; untranslated strings stay English. Trace: R-70 / BG-27.

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

### SF-87 Goal folders under the workspace name
When a workspace has a project folder, each goal (slug) folder is created as <project>/<workspace name>/<slug>/ with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Existing goal folders move to the new place when first used, and the folder follows a workspace rename.
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
