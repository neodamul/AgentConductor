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
