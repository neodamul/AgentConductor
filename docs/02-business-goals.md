# Business goals

| ID | Goal | Requirements | Success measure |
|---|---|---|---|
| BG-01 | Coordinate work using existing CLI subscriptions | R-01–03 | Zero additional LLM API key registrations |
| BG-02 | Reduce waiting across implementation, tests, documentation | R-04–06 | Three execution intervals overlap; compare latency to sequential execution |
| BG-03 | Preserve human control and trustworthy changes | R-07–09 | Zero unapproved protected-branch changes or completions without evidence |
| BG-04 | Reuse internal knowledge and lessons | R-10–11 | Decisions, lessons, sources retrievable on resume; unverified claims marked |
| BG-05 | Minimize distribution and maintenance cost | R-12–15 | One core executable; zero bundled components with unreviewed licenses |

Measure resource use and latency on the actual host and CLIs. Subscription usage and
estimated token costs are different; missing cost data must not appear as zero.

## Continuity goals

- BG-07 (R-18): Continue work across subscription reset windows without busy retry loops or lost evidence.
- BG-08 (R-19): Preserve delivery continuity by transferring a role to an available agent while keeping ownership explicit.
- BG-09 (R-20): Give people control over session models without inventing account capabilities or changing authentication.

### BG-09 — Start and review work in one local application

An operator can configure CLI teammates and start a goal without manually editing files or opening another application. Existing authentication remains authoritative. Trace: R-21.

### BG-10 Capacity visibility and portable delivery

Operators can plan paid-agent work using actual remaining subscription windows, and developers can continue the same product on macOS or Windows without losing reviewed decisions or requiring shared credentials. Verified binaries are distributed through the private repository. Each release explains what changed (R-41).

### BG-11 Subscription-independent continuity

Keep supervision and operator control functional when paid CLI accounts are exhausted. Transfer approved roles to the company self-hosted CLI without losing evidence or requiring live steering. Trace: R-25/R-26.

### BG-12 Operator situational awareness

An operator can request one concise team briefing and identify active work, delays, capacity restrictions and decisions without inspecting every agent log. Trace: R-28. Long-running steps such as planning always show that they are in progress (R-45).

### BG-13 Low-friction trustworthy control

Operators approve sensitive tool calls and plan with any registered CLI without losing work or relaxing safety, and can operate the product in Korean. Blocking conditions are explained with a safe way forward (R-42). Trace: R-30/R-31/R-32.

### BG-14 Recognizable team at a glance

Operators identify which provider runs each session without reading labels. Trace: R-34.

### BG-15 Understandable first contact

A new operator understands what AgentConductor is and starts the desktop workflow from the README alone; advanced users find complete operating details in the manual. Trace: R-36.

### BG-16 Orchestration without AI dependency

The team can be orchestrated when no agent is suitable or trusted for planning: the human decides the work split and the engine executes it with the same safety guarantees. Trace: R-37.

### BG-17 Self-explanatory setup

Operators configure and switch workspaces from the window itself, without knowing file paths or command-line options. Trace: R-38/R-39. Named workspaces let one operator keep separate teams per project. Trace: R-40.

### BG-18 Team awareness outside the desktop (planned)

Operators and teammates learn about decisions and outcomes in the messenger they already use and can respond without sitting at the workstation, without weakening approval guarantees. Trace: R-43.

### BG-19 Predictable team composition

The operator decides the division of labour between agents once per workspace, so every plan follows the intended responsibilities instead of an LLM's ad-hoc choice. Trace: R-47.

### BG-20 Team-shared configuration

A development team keeps one reviewed AgentConductor configuration in the repository so every member orchestrates agents the same way, without sharing secrets or machine state. Trace: R-52.

### BG-21 Managed multi-goal delivery

A workspace becomes a durable record of every goal pursued in it — analysis, phases, tasks, verification and completion — that teammates can share, resume and audit, governed by rules the team can edit. Trace: R-55 – R-59.

### BG-22 Visible Claude capacity without additional consumption

Operators can assess Claude subscription capacity from existing execution metadata without spending tokens solely to query usage. Trace: R-60.

### BG-22 extension — Operators assess capacity directly from the agent list without opening an inspector.

### BG-23 Plan from company work records

Turn an existing Jira Story and Confluence evidence into a reviewable local plan without re-entering requirements. Trace: R-62.


### BG-24 Readable account capacity

Operators compare both remaining percentages in one line and inspect the tooltip for periods, while refreshing without wasted list space. Trace: R-64.

### BG-25 Quiet, lightweight agent configuration

Operators configure installed tools and refresh cadence with consistent controls, recognize the AC app in the launcher, and receive quota results without distracting loading messages. Trace: R-67.

### BG-26 Fewer agent setup mistakes and predictable capacity

Operators switch an agent to another CLI without leftover settings from the previous one, and know when an exhausted agent becomes usable again without opening an inspector. Trace: R-68, R-69.

### BG-27 Operate in the operator's language

Korean-speaking operators read and act on the workspace and Agent Room in Korean while English remains the shared default for documentation and collaboration. Trace: R-70.

BG-26 extension (R-71): fallback configuration is understandable and selectable without leaving the dialog.


### BG-28 Windows operators use the same lightweight workflow
Operators recognize their agents and inspect subscription capacity on Windows without launching additional desktop apps. Trace: R-72.


### BG-29 Company CLIs remain first-class without metadata APIs
Operators configure known company model IDs while execution and authentication stay on the registered CLI. Trace: R-73.

BG-18 phase 2: an operator away from the desk unblocks a waiting agent from the messenger without weakening the exact-request approval guarantees.

BG-12 extension (R-74): fewer fields to fill before planning; work folders stay recognizable in the operator's language.

### BG-30 Accurate enterprise capacity
Show company users the remaining budget relative to their actual allocation while retaining existing CLI login and subscriptions. Trace: R-75.


### BG-31 Uninterrupted desktop operation
Keep project selection visible and remove distracting Windows console windows without another daemon or dependency. Trace: R-76.


### BG-32 Configure a new project without premature execution constraints
Let operators save their workspace before repository initialization and keep account metadata usable independently. Trace: R-77.


### BG-33 Responsive team configuration
Operators keep configuring and reviewing work while agent metadata arrives without multiple desktop apps or unbounded subprocesses. Trace: R-78.


### BG-34 Start new projects through an understandable approval flow
Replace a raw Git prerequisite failure with an actionable, optional baseline setup before planning. Trace: R-79.


### BG-35 Develop without repository setup
Operators can start an ordinary folder immediately and review changes without being forced to initialize or commit their project. Trace: R-80.


### BG-36 Faster agent setup without leaving the app
Operators find the commands that improve Claude Code and Codex collaboration where they configure the app, instead of searching several project pages. Trace: R-81.


### BG-37 Windows project selection works first time
Operators on Windows select their project with the native picker without retyping the path. Trace: R-82.


### BG-38 Operators can trust long-running planning
Operators can tell a working coordinator from a stalled one without opening log folders. Trace: R-83.


### BG-39 Recognizable goal folders
Operators recognise goal folders in English regardless of the goal's language. Trace: R-84.


### BG-40 Comparable capacity at a glance
Operators compare Enterprise and subscription capacity in the same unit. Trace: R-85.


### BG-41 Actionable capacity errors
Operators learn what to do when Claude capacity cannot be read, instead of waiting for a timeout. Trace: R-86.


### BG-42 Readable Windows project paths
Trace: R-87 -> BG-42 -> SF-74 -> UC-77 -> FR-72 -> QAS-83 -> DD-95.
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity.


### BG-43 Reliable named workspace creation
Trace: R-88 -> BG-43 -> SF-75 -> UC-78 -> FR-73 -> QAS-84 -> DD-96.
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable.


### BG-44 Delete an unwanted workspace
Trace: R-89 -> BG-44 -> SF-76 -> UC-79 -> FR-74 -> QAS-85 -> DD-97.
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first.


### BG-45 Register MCP servers in Settings
Trace: R-90 -> BG-45 -> SF-77 -> UC-80 -> FR-75 -> QAS-86 -> DD-98.
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration.


### BG-46 Trust profile changes
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


### BG-47 Continue coordination independently of memory services
Preserve task direction, intermediate reports and handoff communication during optional memory-service outages while giving the operator a persistent transport choice. Keep resource use bounded, existing CLI authentication and human execution authority intact.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### BG-48 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.


### BG-49 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.


### BG-50 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.

### BG-51 Agents work inside the workspace sandbox; the orchestrator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Work confined to the task workspace runs at once; a request reaching outside it (network, installs, history-changing Git, privilege, other processes, outside paths, secrets) is decided by the workspace coordinator acting as orchestrator, and a person is asked only when no orchestrator can decide. The permission level is not chosen in the agent profile; it follows from the CLI. Time spent waiting for a decision does not count toward the task limit, and a task that times out can be resumed instead of failing.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.

### BG-52 Usage query accepts the Claude folder-trust prompt
Claude usage must load even when Claude first asks whether to trust the project folder. The usage query runs in a separate session with no tools, hooks or project settings, so it accepts the prompt automatically in both forms (Enter for the default choice, y for the screen-reader "Enter y/n:" form) at most twice, and stops with guidance instead of waiting for a timeout if Claude keeps asking.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.

### BG-53 Planning start and workspace naming feedback
Generate plan shows progress with elapsed time from the moment it is pressed, including while the project folder is copied before the coordinator starts. Saving workspace settings updates the titlebar name at once. Deleting the current workspace asks for the name of the fresh workspace that replaces it instead of always naming it "New workspace".
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.

### BG-54 Run and task status at a glance
The Agent Room shows the run's current final state immediately: a run status banner with the state in bold colour, task progress and the last error, and a prominent Resume button whenever the run can continue. Task states (failed, completed, pending, waiting, running) are bold and colour-coded in the session heading and in the left session list. In the left list a failed task is red, a task waiting for a decision or tool request has a blinking border, pending tasks use a strong colour, and the Tool requests count turns red and blinks while requests wait.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.

### BG-55 Goal folders under the workspace name
When a workspace has a project folder, each goal (slug) folder is created as <project>/<workspace name>/<slug>/ with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Existing goal folders move to the new place when first used, and the folder follows a workspace rename.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.

### BG-56 Agent CLI list is a master list
The registered agent CLI list is kept as a master list for the machine, not per workspace: deleting a workspace never removes agents, and every workspace opens with the master list. When a workspace refers to an agent the master list no longer has (planning coordinator or role assignments), the assignment is shown in red and can be reassigned; Generate plan stays disabled until every assignment refers to a listed agent.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.

### BG-57 Reorder agent cards
Agent CLI cards can be moved up and down with the mouse (drag and drop), with keyboard-accessible move controls as an alternative. The order is saved, and every agent selection list (planning coordinator, roles, fallback) follows the same order.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.

### BG-58 Readable file paths everywhere
Screens, error messages and notifications never show Windows extended-length path prefixes such as \\?\C:\Work\Temp or //?/C:/Work/Temp; paths read as C:\Work\Temp.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.

### BG-59 Activity tab reads like the CLI
The Activity tab shows agent output the way the CLI shows it: assistant messages as text, tool calls as a one-line action with its key input, tool results as short summaries and errors highlighted. Raw JSON is never shown there; it remains in the Raw logs tab.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.

### BG-60 Orchestrator final step in the task list
The Agent Room task list shows the step the orchestrator runs after the tasks, integration verification of the combined result, as its own entry with its status (pending, running, completed, failed) and which task tests failed, so a run that fails after every task completed shows why without opening other views.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.

### BG-61 Decide a live tool request from its window
When a tool request window opens while the run is working, Allow once and Deny are available at once, without closing the window with Decide later and deciding again from the Tool requests list.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.

### BG-62 Orchestrator decides routine requests and records each decision
Simple decisions are made by the orchestrator so the user is not asked often: it allows routine development requests, denies only publishing, outside-project changes, secret exposure or system changes, also for runs started before orchestration existed, and every decision is recorded with the request, the result, the reason and the time, visible in the task's Activity. Commands inside a workspace that itself lives under an app folder (such as ~/.agentconductor) are workspace commands.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.

### BG-63 Task list without account usage
The Agent Room task list does not show account usage badges or refresh buttons; account limits are shown in the inspector's Account capacity section.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.
