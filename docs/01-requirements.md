# Requirements

Baseline: 2026-10-09 KST. The user's requests define scope; no additional product features are assumed.

| ID | Requirement | Priority | Acceptance criterion |
|---|---|---|---|
| R-01 | Reuse existing CLI authentication and subscriptions | Required | No orchestrator API key entry for Claude or Codex |
| R-02 | Control subprocess I/O | Required | Collect executable/argv/cwd/env, stdin, stdout, stderr, and exit independently |
| R-03 | Register three native CLIs and arbitrary CLIs | Required | Claude/Codex/OpenCode profiles and custom profiles with the same contract |
| R-04 | Decompose a goal into scoped tasks (originally exactly three; superseded by R-53: one to twelve) | MVP | Validated plan with inputs, scope, dependencies, completion conditions |
| R-05 | Optional worktree separation and parallel execution | MVP | shared default or explicit worktree mode; fixed baseline, disjoint scopes, concurrency limit of three |
| R-06 | Collect results, diffs, and tests | MVP | Inventory including new files; coordinator-run test evidence |
| R-07 | Human review and approval before merge | MVP | Reject merge when approved base/candidate/diff/test hashes change |
| R-08 | Approve dangerous actions | Required | Deployment, push, external transmission, secret reads denied by default; allow only paths supporting approval before execution |
| R-09 | Stop, resume, feedback, auditing | Required | Preserve sessions, attempts, stopped artifacts, restart recovery |
| R-10 | Shared memory, lessons, progress reports | Required | agentmemory records and local outbox; work continues during outages |
| R-11 | LLM-wiki knowledge assets | Required | raw/schema/wiki, sources, hashes, index, append-only log |
| R-12 | Rust and lightweight local architecture | Required | Single native executable including SQLite; Docker/desktop optional |
| R-13 | Development workflow and TDD | Required | Trace requirements to tests; no completion without Red/Green evidence |
| R-14 | Use development tools | Required | Record roles/evidence for RTK, Ponytail, Graphify, CocoIndex |
| R-15 | Review internal distribution licenses | Required | Review pinned versions, transitive dependencies, models, NOTICE |
| R-16 | Documentation language and public README | Required | English documents; Korean user communication; no development plans or internal decisions in README |
| R-17 | Functional lightweight intervention UI and README preview | Required | Task/agent states, live logs, diff/tests, approvals, feedback, session/role/model controls; label mockups and replace with actual UI evidence |

The first MVP covers registration, three-task planning, parallel agents with optional worktrees, JSON collection,
diff/test aggregation, and human-approved merge. Docker, self-hosted HTTP transport,
desktop UI, cost dashboards, and automatic deployment are later scope.
Self-hosted **CLI** agents use the common profile contract from the start.

Interpretation: DD means Design Decision; QAS means Quality Attribute Scenario.
Memory recall did not identify a previous template for this sequence. These documents
follow the user's explicit sequence and standard QAS fields.

## Availability, sessions, and model requirements

- R-18: Persist subscription/quota/transient restrictions, source evidence, observation time, and explicit reset timestamps. Never infer a timezone from ambiguous prose. Support an operator schedule with an explicit UTC offset. Resume due work after restart while the coordinator is running; retain attempts and cap automatic retries.
- R-19: Support multiple independent logical/provider sessions per profile, disjoint task ownership, configurable concurrency, and one active process per provider session. Reassign an idle restricted task using a bounded handoff and a new target-provider session. Sessions share account quotas.
- R-20: Query model catalogs through existing authenticated CLI subprocesses during registration. Persist source/version/query time. Select or change a session model only from the observed catalog; reject unknown models and invalidate prior approvals. Unsupported discovery requires an explicit operator catalog import, never an API-key fallback.

### R-21 — Native startup workspace

Launch without a preexisting run. Register native/custom executable profiles, discover/import observed catalogs, enter a repository and goal, generate or edit three scoped tasks, review literal commands and permission capabilities, and explicitly approve the exact plan before execution. Shared workspace is the default; worktrees remain optional. Keep settings, plans, jobs and evidence local. Native messages only; no UI HTTP service.

### R-22 Remaining subscription quota

Paid-agent summaries show provider-reported remaining quota per window, reset time, source and observation age. Used percentage is converted to remaining percentage; token context occupancy and estimated session cost are separate metrics. Missing values are unavailable, not zero. Preserve existing CLI authentication and prefer bounded local stdio metadata queries.

### R-23 Cross-platform development continuity

A developer can clone the private repository on Windows and resume from committed English requirements, checked implementation plans, tests and source-hashed knowledge assets. Host-specific credentials, executable paths and transient workspaces are configured locally. Native Windows execution must replace Unix-only process containment, leases, secure filesystem checks and randomness with verified equivalents. WSL may support interim CLI development but does not satisfy native Windows delivery.

### R-24 Verified GitHub release artifacts

Publish tested macOS and Windows binaries to the existing private GitHub repository's Releases after platform execution acceptance. Include checksums, dependency licenses and accurate platform/UI support notes. Compilation alone does not prove runtime support; never publish a failing or unverified Windows build as a working release.

### R-25 Deterministic supervision and optional AI coordination

The Rust supervisor remains available without an LLM subscription. Process health, deadlines, dependencies and approved execution policies are deterministic. An optional designated AI session provides bounded planning/review opinions; it cannot grant human approval. Self-hosted CLIs may lack live steering: queue feedback for the next attempt or stop, verify quiescence and start a new attempt. Silence alone does not prove failure.

### R-26 Per-agent availability fallback

An operator can configure a registered fallback agent and optional observed target model for each profile. Company presets can nominate the self-hosted CLI for paid profiles. Confirmed account restrictions may trigger this explicitly reviewed policy after owned jobs drain. Preserve partial files, diffs, attempt logs and restriction/reset evidence; use a new target-provider session. Prevent cycles, same-account substitutes, unsupported permissions and retries through unavailable targets. Final verification and human approval remain mandatory.

### R-27 Cooperative steering through agentmemory

The company CLI polls agentmemory before work and at cooperative checkpoints, publishes progress, and may accept another authorized agent's scoped instruction without native CLI steering. Distinguish observations from addressed directives. Preserve directive identity, issuer, target session/task, revision, expiry and acknowledged/applied/rejected outcome. A memory observation alone is not an execution or human approval grant. Rust reconciles acknowledgments with actual task/process evidence and never starts a competing writer while a live session may still apply an instruction.

### R-28 On-demand orchestration briefing

When the operator asks for current status, Rust or the designated AI coordinator briefs all agent sessions: assigned work, observed activity, last intermediate report and its age, execution/restriction state, evidence, pending approvals and next action. Reconcile agentmemory reports with authoritative local process/task records. Missing or conflicting reports are explicitly uncertain. Optional LLM prose cannot manufacture progress.

### R-29 At-a-glance agent status lights

Agent/session lists show green, yellow or red status lights. Running sessions blink green; completed sessions are steady green. Pending, approval/availability waits and stops are yellow; failures and blocked tasks are red. Text/accessibility labels explain the state; unknown states are yellow rather than healthy. Respect reduced-motion settings.

### R-30 Live human tool decisions

When a running worker requests a human-gated tool, it pauses before execution and waits for the operator's exact decision instead of terminating. Approval lets the same attempt run the identical input once; rejection refuses only that call and the worker continues. Decisions are possible from the native UI and CLI while the run is active. Stops, expiry and unanswered requests fall back to the stopped/waiting state with retained evidence. A resumed attempt must not require re-approving a regenerated variant of an already decided request.

### R-31 Planning with unverifiable company CLIs

A company or self-hosted CLI whose read-only behavior cannot be proven may act as planning coordinator. The product, not an operator label, guarantees that planning cannot change the target repository: planning runs in a disposable copy of the reviewed base containing tracked files only, which is discarded afterwards, and the target workspace must be unchanged before a proposal is accepted. Untracked secrets are not exposed through that copy.

### R-32 Korean text entry

All native text fields (goal, feedback, plan instructions, profile fields) accept Korean input-method composition. Background polling and re-rendering must not interrupt composition, drop committed syllables or overwrite a focused field. Korean goals, prompts and feedback round-trip unchanged into plans, worker prompts and evidence.

### R-33 Leave the workspace without saving

From the workspace (startup) screen the operator can always return to the parent screen — the Agent Room of the current run when one exists — without saving. Unsaved workspace or plan edits are discarded only after an explicit confirmation that names what will be lost; nothing is written and no planner or run is started. Without a run, the operator can still discard unsaved edits and return to the last saved workspace.

### R-34 Recognizable agent icons

Built-in agent kinds (Claude Code, Codex, OpenCode) are shown with the same icon as their installed desktop application instead of a letter avatar, in the workspace agent list and the Agent Room session list. Custom agents keep the letter avatar. Icons are read from the operator's installed applications at runtime; trademarked artwork is not bundled or redistributed. When the application is not installed or its icon cannot be read, the letter avatar is shown.

### R-35 Maintainable source layout

Production code lives under `src/` and tests under `tests/`; no inline `#[cfg(test)]` modules remain in production files. Every individual source, test, script, style and markup file stays at or below 500 lines. Splits follow module responsibilities without changing behavior.

### R-36 Product README and separate user manual

The public README introduces the product for someone who has never used it: what it does, who it is for, real screenshots, how to install and a short desktop-first walkthrough. It contains no command-line reference. Detailed operation, every CLI command, configuration fields and safety rules move to a separate English user manual linked from the README and shipped in release packages. The product is a desktop application; the CLI is an advanced/automation interface. The README must also answer first-contact questions: the main capabilities; how orchestration works (what the deterministic engine decides versus what an optional LLM coordinator proposes); how roles are assigned to agents; how work is redistributed when an account's hourly or weekly usage limit is reached; whether agentmemory is required; and what the user must prepare.

### R-37 Plan without an LLM coordinator

The operator can write the plan in the desktop workspace without any planning agent: the product provides a three-task form for the current goal and registered agents, and the operator fills in each task's agent, instructions, owned paths, dependencies and test. An LLM coordinator remains optional for proposing the same form. A hand-written plan is validated, reviewed and approved exactly like a generated one, and is bound to the repository base at the time the form was created. Source: user decision, 2026-10-11.

### R-38 Discoverable workspace settings

The workspace screen groups its settings under a visible **Workspace settings** heading. The goal comes first; the planning coordinator follows; the repository sits under the coordinator together with the Git-worktree option. The repository is chosen with a native folder picker (typing a path remains possible) and is optional for saving a workspace; it is required only to create or run a plan. Source: user request, 2026-10-11.

### R-39 Multiple workspaces

The window shows which workspace directory is open and offers a workspace menu to open a recent workspace, open an existing folder or create a new one, without restarting the application or using the command line. One window per workspace is preserved. Source: user request, 2026-10-11.

### R-40 Named, registered workspaces

Each workspace has an editable name entered above the goal and saved with the workspace. The operator can register several workspaces: create a new one by name only (its folder is created under the application's data directory), open any workspace already saved in that data directory (`~/.agentconductor`) from a list by name — including ones removed from the menu — see all registered workspaces by name, switch between them and remove one from the list without deleting its files. Source: user request, 2026-10-11.

### R-41 Release notes summarize changes

Every GitHub Release states what changed since the previous release — new features, fixes and known limitations — instead of a generic packaging statement. The summary is kept in the repository with the code and a release without one is not published. Source: user request, 2026-10-11.

### R-42 Explain and resolve an unclean repository

Planning still requires a clean repository. When the operator starts planning with uncommitted or untracked changes, the window explains why a clean baseline is required, lists the changed files and asks whether to commit them as the baseline and continue, or stop. Nothing is committed without that explicit choice. Source: user request, 2026-10-11.

### R-43 Messenger integration (planned)

**Status: planned, not implemented.** AgentConductor will connect to Slack and to company-provided messengers. The operator and their team receive concise notifications when a decision is needed (plan approval, tool request, candidate review, usage limit with no fallback), when a run completes or fails, and on request a team briefing. Later phases allow answering a pending decision from the messenger. The desktop application stays the source of truth: no source code, diffs or secrets leave the machine unless the operator enables it per workspace, and a messenger decision is accepted only from a linked, authorized user and only for the exact pending request. Source: user request, 2026-10-11.

### R-44 Ignore operating-system files and continue

When the unclean changes include untracked operating-system metadata files (for example `.DS_Store`, `Thumbs.db`, `desktop.ini`, `._*`), the baseline dialog explains that these are generated by the OS, guides how to ignore them permanently, and offers **Ignore and continue**, which excludes them locally for this repository without a commit and continues when nothing else remains. Source: user request, 2026-10-11.

### R-45 Visible planning progress

While a coordinator generates the plan, the **Review the plan** area shows that planning is in progress, which coordinator is working and the elapsed time. When the plan is ready the window says so and brings the plan into view; when planning fails the same area shows the reason and where the logs are. Source: user request, 2026-10-11.

### R-46 Human-readable task instructions

Task instructions and completion conditions are easy to read and edit: the coordinator writes them as short sections on separate lines (goal, inputs, steps, expected output, done-when) with bullet points, in the same language as the operator's goal, and the plan editor shows them in a box at least six lines tall that grows with the content. Source: user request, 2026-10-11.

### R-47 Agent role rules before planning

Before generating a plan the operator defines which registered agents perform which roles — for example Claude for planning and verification, Codex for development, documentation and knowledge assetization, OpenCode or a company agent for testing. The planning coordinator receives these rules, gives every task a role and assigns an agent allowed for that role; a plan that breaks the rules is rejected with the reason. Each role entry may also name the model the agent must use, chosen from that agent's observed model catalog. The operator may still change an assignment while reviewing. The workspace screen presents the flow as numbered steps: **1. Workspace settings**, **2. Agent roles**, **3. Review the plan**. **Save workspace** sits in step 1. Final candidate approval and merge remain human decisions; the verification role means agent quality review. Source: user request, 2026-10-11.

### R-48 Agent registrations are saved immediately

Adding, editing or removing an agent with **+ Add**, **Edit** or **Remove** saves the agent list at once, without pressing **Save workspace** and without saving other unsaved edits such as the goal. Removing an agent that is the planning coordinator or appears in role rules updates those settings consistently. Source: user request, 2026-10-11, after added agents were lost when the window was reopened.

### R-49 Model lists prepared per agent

Each built-in agent's available models are fetched once automatically with its existing login (no generation prompt) when the agent is added or a workspace is created or opened in the window (lists already observed on this machine for the same CLI are reused immediately); the agent's **Models** button refreshes the list, so the role table and the plan's model selectors offer real model lists without pressing **Models**. Fetching never invalidates a generated plan, never moves focus or redraws what the operator is editing, and saving a workspace keeps previously observed model lists. Source: user request, 2026-10-11.

### R-50 Workspace changes keep a reviewed plan

Saving the workspace, loading models or changing role rules keeps a generated plan unless the change affects what the plan was reviewed against: the goal, repository, workspace mode, or an agent's executable, arguments, input mode, permission capability, sessions, quota group or fallback. Source: user report 2026-10-11 (a plan disappeared after pressing **Models**).

### R-51 Compact agent list

Agent cards in the team column use space efficiently and show more at a glance: icon, name, provider, session limit, number of models (or *not queried*), permission capability and fallback, with the executable on one truncated line and small actions. Source: user request, 2026-10-11.

### R-52 Shareable workspace settings (export/import)

The operator can export a workspace's settings — name, planning coordinator, workspace mode, agent profiles and role rules — to a versioned file intended to be committed to the project repository, and import such a file into a workspace, so several developers share one configuration through Git while each keeps their own machine state. The file never contains private environment values, observed model catalogs, absolute local paths of the repository, goals, plans or run records. Importing shows what will run before applying, keeps local environment values and catalogs for agents with the same ID, and does not trust another machine's operator-verified sandbox claims unless the importing operator confirms them. Source: user request, 2026-10-11.

### R-53 Plans with one to twelve tasks

A plan may contain between one and twelve tasks instead of exactly three. The planning coordinator chooses the smallest number of tasks that keeps owned paths disjoint, the operator can add or remove tasks while reviewing, and execution still runs independent tasks in parallel within each agent's session limit. Source: user decision, 2026-10-11 (needed to import existing plans with more work items).

### R-54 Read an existing plan document

For a project that already has a plan, the operator chooses **Read plan…**, selects a Markdown file of any format (for example a plan written by Claude or Codex, or a project checklist) and the planning coordinator converts its remaining work into a reviewable plan that follows the role rules. The coordinator reads the document and a read-only snapshot of the repository; it does not edit anything. Items already marked done are not planned again. Source: user request, 2026-10-11.

### R-55 Goals become slugs with a work tree

Every goal set in a workspace becomes a slug — a short human-readable phrase such as `healthcheck_interval_adjust` — and goals accumulate: setting a new goal adds a slug instead of replacing the previous one. Each slug owns a folder tree `plan/`, `phase/`, `task/`, `verification/`, `completed/` under `<repository>/.agentconductor/<slug>/`, or under `~/.agentconductor/<workspace>/<slug>/` when the workspace has no repository. A slug may exist without a plan. Source: user request, 2026-10-11.

### R-56 Analysis first, then sequential phases

Planning starts with an analysis of the goal and codebase, then divides the work into phases. Each phase runs its tasks through task → verification → completed, and the next phase starts only after the previous one is completed. Source: user request, 2026-10-11.

### R-57 Progress tracking without tree mismatches

The workspace shows, for each slug, how far it has progressed (no plan, analysis, planned, phase n in progress, completed) and, for each phase, which tasks are in task, verification or completed. The recorded state can never disagree with the folder tree: the tree is the single source of truth and inconsistencies are detected and reported. Source: user request, 2026-10-11.

### R-58 Export/import includes slug archives

Exporting a workspace writes the workspace settings JSON and one `<slug>.zip` per slug into the same folder; the JSON lists each slug's archive name and progress. Import requires the JSON and every listed archive in the same folder, verifies them, and restores the slug trees. Source: user request, 2026-10-11 (extends R-52).

### R-59 Editable plan rules

The planning rules taught so far (analysis first, phases, task → verification → completed, role assignment, TDD, source/test separation, 500-line files, English documents, recording and assetization) live in a global, editable rules file passed to the planning coordinator. A workspace may override them; the override is stored in the workspace settings JSON. Source: user request, 2026-10-11.

### R-60 Collect Claude subscription quota without extra generation

Collect provider-reported utilization and reset windows from owned Claude CLI streams, show remaining percentages in Agent Room, and retain unavailable/stale states when metadata is absent. Refresh must not generate a prompt, read authentication secrets, or pretend to obtain a new account observation. Source: user request, 2026-10-11.

### R-61 Inline agent quota — Show paid CLI remaining capacity beside the agent name in the left workspace and session lists; unknown and stale values remain explicit. Source: user request, 2026-10-11.

### R-62 Jira and Confluence goal imports

Workspace settings register Jira/Confluence MCP connections using the same `mcp` server-object format as OpenCode. Import a Jira Story and referenced Confluence pages as goal context, with explicit goal keywords and a reference preview. Source: user requests, 2026-10-11.

R-61 correction: fresh workspaces must independently collect supported paid-agent quota; a retained execution run must not be required for displaying account capacity.

### R-63 Agent-specific refresh and action icons

Each agent has its own quota refresh icon and periodic refresh schedule. Remove uses a trash icon, Models keeps its text with a reload icon, and Edit has a pencil icon. Source: user requests, 2026-10-11.

### R-65 Idle footprint within the resource budget

AgentConductor exists because three agent desktop applications at once exhaust the host, so an idle window must stay within the NFR-05 budget (core RSS < 100 MiB, idle CPU < 1 %). The window refreshes quickly only while work is happening (a job runs, a plan is generated, edits are unsaved), slows down when idle, stops while hidden, and never resends unchanged heavy data such as application icons. Source: user directive and measurement, 2026-10-11 (idle 105–106 MiB, CPU spikes 3–4.5 %).

### R-64 Compact quota windows

Keep quota values and their refresh icon in one compact control beside each agent. Show one compact numeric line such as `64%/100% ↻`. Explain Enterprise Current / Total or provider periods (5h, Daily, Weekly or others) in the tooltip. Show independent remaining percentages rather than hiding windows behind a minimum. Never infer Enterprise from a company name or numeric limits.


R-64 / UC-54 interaction refinement: keep a 5px gap between quota values and refresh. All enabled buttons, including their icon/text children, show a pointer cursor; disabled controls keep the default cursor.

### R-67 Agent preferences and consistent native identity

Default quota refresh is five minutes, independently configurable per agent in Edit (1–60 whole minutes). Locate installed executable paths automatically and provide a native Browse fallback without launching the CLI. Single-line form controls share height and paired columns share width. Use the existing AC brand mark as the native application icon. Quota background refresh shows no loading message.
### R-66 New workspaces start without agents

A new workspace has an empty agent list; the operator adds the agents they actually use with **+ Add**, where the Claude Code, Codex and OpenCode templates remain available. Nothing is pre-registered, no model lookup or CLI process starts for agents the operator did not add, and planning stays disabled until at least one agent and a coordinator exist. Tests add their agents explicitly before starting. Source: user request, 2026-10-11.

### R-68 Provider choice defines the whole agent profile

In the agent dialog, Provider comes first and Agent ID second. Changing the provider replaces every provider-derived value at once: the agent ID (the provider's template ID, suffixed `-2`, `-3`… when taken), the located executable path, arguments, input mode, permission mode, session limit and quota cadence; models observed for the previous CLI are discarded. When an existing agent's ID changes, the coordinator, role rules and other agents' fallbacks follow the new ID instead of being dropped. Source: user requests, 2026-10-11.

### R-69 Show when exhausted quota returns

When a paid agent's quota window reaches 0 %, the compact quota control shows the reset time beside the percentages (time of day today, otherwise date and time), and the tooltip lists the reset time of every window. Source: user request, 2026-10-11.

### R-70 Interface language

All text is UTF-8 end to end (pages, settings, release notes, CLI output read by the app). A **Settings** menu lets the operator choose the interface language; English is the default and Korean is available. The choice applies immediately, survives restarts and applies to every workspace on the machine. User content (goals, task instructions, logs, agent output) is never translated. Source: user request, 2026-10-11.

R-70 amendment (2026-10-11): the Settings button shows only the gear icon; its name stays available as tooltip and accessible label.

### R-71 Fallback model choices are always offered

In the agent dialog the fallback model is the model the *fallback agent* runs with when this agent's account is limited — not this agent's own model. Choosing an alternate agent shows its model list; when that agent's models were never fetched, they are fetched once and the list fills in while the dialog stays open. Source: user question, 2026-10-11.


DD-68 UI amendment (2026-10-11): configure MCP connections through the top-bar Settings menu; keep reference insertion and preview beside the goal. The Settings gear uses a 26 px glyph in a 44 × 44 px pointer target.
