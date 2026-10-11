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
