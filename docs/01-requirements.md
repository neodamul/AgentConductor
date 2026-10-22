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

The planning rules taught so far (analysis first, phases, task → verification → completed, role assignment, TDD, source/test separation, 300-line files, English documents, recording and assetization) live in a global, editable rules file passed to the planning coordinator. A workspace may override them; the override is stored in the workspace settings JSON. When no workspace override is set, Markdown rules under `<repository>/.agentconductor/plan-rules/` are recursively discovered and appended in repository-relative lexical order. Existing global rules remain supported. Source: user request, 2026-10-11; GitHub issue #8.

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

R-43 activation (user decision, 2026-10-11): start messenger integration. **Phase 1 — outbound notifications** for Slack (Incoming Webhook) and company messengers that accept an incoming webhook (configurable JSON body). Configuration is per workspace; the webhook URL is a machine-local secret and never exported. Events: plan ready, candidate ready for review, tool request waiting, usage limit wait, run completed / failed / stopped / blocked, plus a test message. **Phase 2 — answering decisions from the messenger** stays planned until the company messenger and its API are chosen.

DD-68 UI amendment (2026-10-11): configure MCP connections through the top-bar Settings menu; keep reference insertion and preview beside the goal. The Settings gear uses a 26 px glyph in a 44 × 44 px pointer target.


### R-72 Windows CLI metadata and provider visibility
Windows CLI-only installations show recognizable provider badges, prefer executable/.cmd launchers over extensionless npm scripts and query existing Codex/Claude subscription metadata without generation. No desktop agent installation, new server or API key is required.


### R-73 Self-hosted CLI model catalogs without discovery
A working self-hosted CLI may expose no model-list command. Explain that capability limit and register company-supplied IDs without HTTP endpoints, API keys, guessed IDs or a generation prompt. Preserve existing models when reopening the editor. Source: user confirmed aipro has no /models command, 2026-10-11.

R-43 phase 2 activation (user decision, 2026-10-11): linked approvers answer **pending tool requests** from the messenger. Slack: a reply `allow <code>` or `deny <code>` in the configured channel by a listed approver. Company messenger: a company-hosted relay returns signed decisions. Plan consent and final merge stay desktop-only. Which company messenger will be used is still open; the relay contract keeps that choice independent.

### R-74 Slugs are named automatically

The operator no longer types a slug. Each new goal gets a readable, unique slug derived from the goal: ASCII words as before, Korean (Hangul) romanized by the Revised Romanization of Korean, other scripts fall back to `goal_<yyyymmdd>`; a taken slug gets `_2`, `_3`…; at most 48 characters, cut at a word boundary. The slug is shown read-only in Goals & progress. Source: user request, 2026-10-11 (amends R-55).

### R-75 Enterprise quota budgets
Enterprise capacity prioritizes provider-reported remaining allocation and total allocation, not invented five-hour/weekly windows. Support Codex individual spend limits and credit balances; keep missing denominators unknown. Windows Enterprise failures require actual sanitized query evidence; do not claim live account verification from fixtures.


### R-76 Foreground pickers and console-free desktop execution
Repository browsing must open a visible app-owned native picker. Windows desktop launch and background subprocesses must not create command windows; CLI stdout/stderr, containment and cancellation remain functional.


### R-77 Save new project folders before Git setup
Workspace settings must save existing project directories without invoking Git. Plan creation requires a Git root with an initial commit under the current diff/review architecture, checked before creating goal artifacts or launching agents. Quota collection is independent of project Git state. A future non-Git execution design or explicit Git setup flow requires an operator product decision.


### R-78 Nonblocking agent registration and metadata
Registration/edit forms open immediately while executable resolution proceeds asynchronously. Model and quota discovery must not disable unrelated agent controls, workspace Save or plan preparation. Serialize resource-intensive metadata collectors through a bounded durable queue; discard late model results for changed agent execution settings.


### R-79 Approved Git setup for new project planning
Generate plan and manual planning offer explicit approval when the selected folder lacks Git or an initial commit. Create an empty baseline only after approval, preserve existing files and staged changes, then recheck the normal file-commit consent. Refuse replay and existing/nested/inaccessible repository replacement; never silently initialize.


### R-80 Work in ordinary project folders
A project need not contain a Git repository or initial commit. Generate plan, manual planning, phased execution and tests use an app-managed working copy; the original is changed only after exact diff/test review and approval. Refuse concurrent original edits. Git worktrees and branch merges remain optional. Git is still used internally for private change history, with no .git created in the original folder. Source: user clarification, 2026-10-12. This supersedes the mandatory initialization prerequisite in R-77/R-79; the unreleased initialization flow is superseded.


### R-81 Install guide for companion agent tools
Settings or help shows how to install agentmemory, ponytail, rtk, graphify and cocoindex-code for Claude Code and Codex on macOS and Windows. Guidance only: the app never installs, runs or requires these tools. Source: user request, 2026-10-12.


### R-82 Picked project paths are used exactly on Windows
A folder chosen with Browse must save and plan exactly like the same path typed by hand, including non-ASCII (for example Korean) folder names and systems whose console code page is not UTF-8. Source: user report, Windows, 2026-10-12.


### R-83 Planning start and progress are truthful
Generate plan must not report a timeout while planning is still being prepared, and the review area must show whether the coordinator is actually producing output, not only elapsed time. The progress light must visibly blink even when the operating system asks for reduced motion. Source: user report, Windows, 2026-10-12.


### R-84 English slug names
A goal written in another language gets a real English slug, not a romanization. Source: user report, 2026-10-12 ("sadari_geimeul_mandeulja_gerondae" is not English). Supersedes the romanization part of R-74.


### R-85 Enterprise usage as a percentage
Enterprise allocation shows as a remaining percentage like the subscription windows; raw amounts stay in the tooltip. Source: user report, 2026-10-12.


### R-86 Claude capacity on Windows reports why it is unavailable
Claude capacity collection on Windows must run the CLI in the project folder even when the path is in extended-length form, and must report a first-run setup screen immediately instead of timing out. Source: user report, Windows Claude Enterprise, 2026-10-12 ("claude quota query timed out no generation request"; terminal showed "UNC 경로는 지원되지 않습니다" and the theme selection screen).


### R-87 Readable Windows project paths
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity. Source: user correction, 2026-10-12.


### R-88 Reliable named workspace creation
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable. Source: user correction, 2026-10-12.


### R-89 Delete an unwanted workspace
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first. Source: user correction, 2026-10-12.


### R-90 Register MCP servers in Settings
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration. Source: user correction, 2026-10-12.


### R-91 Save an edited agent profile visibly
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


R-86 clarification (merged PR #1): an empty project path is a prerequisite error, not a process timeout. Preserve all-model and named-model Claude windows separately. Codex API-key authentication must report unsupported subscription quota without presenting invented percentages.


### R-92 Selectable signal transport with Rust fallback
The operator can choose the signal backend in workspace Settings even while agentmemory is available. Provide Local Rust and agentmemory preferred. Local Rust uses the durable local signal ledger without requiring a memory daemon or MCP connection. agentmemory preferred falls back automatically for eligible local recipients when the agentmemory server or its memory/signal MCP connection is unavailable, times out or fails validation. This requirement covers coordination signals; it cannot replace unrelated MCP document/tool operations. Failure must not discard a signal or change its intended session. Source: user direction, 2026-10-12. Status: implemented for local app-owned recipients (DD-100); remote recipients have no route yet and stay out of scope.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### R-93 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.


### R-94 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.


### R-95 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.

### R-96 Agents work inside the workspace sandbox; the coordinator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Work confined to the task workspace runs at once; a request reaching outside it (network, installs, history-changing Git, privilege, other processes, outside paths, secrets) is decided by the workspace coordinator , and a person is asked only when no coordinator can decide. The permission level is not chosen in the agent profile; it follows from the CLI. Time spent waiting for a decision does not count toward the task limit, and a task that times out can be resumed instead of failing.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.

### R-97 Usage query accepts the Claude folder-trust prompt
Claude usage must load even when Claude first asks whether to trust the project folder. The usage query runs in a separate session with no tools, hooks or project settings, so it accepts the prompt automatically in both forms (Enter for the default choice, y for the screen-reader "Enter y/n:" form) at most twice, and stops with guidance instead of waiting for a timeout if Claude keeps asking.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.

### R-98 Planning start and workspace naming feedback
Generate plan shows progress with elapsed time from the moment it is pressed, including while the project folder is copied before the coordinator starts. Saving workspace settings updates the titlebar name at once. Deleting the current workspace asks for the name of the fresh workspace that replaces it instead of always naming it "New workspace".
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.

### R-99 Run and task status at a glance
The Agent Room shows the run's current final state immediately: a run status banner with the state in bold colour, task progress and the last error, and a prominent Resume button whenever the run can continue. Task states (failed, completed, pending, waiting, running) are bold and colour-coded in the session heading and in the left session list. In the left list a failed task is red, a task waiting for a decision or tool request has a blinking border, pending tasks use a strong colour, and the Tool requests count turns red and blinks while requests wait.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.

### R-100 Goal folders directly in the project folder
When a workspace has a project folder, each goal (slug) folder is created directly in it as <project>/<slug>/ (for example C:Templadderladder_and_priority_games) with its plan, phase, task, verification and completed folders, instead of <project>/.agentconductor/<slug>/. Goal folders in a former place, including the interim <project>/<workspace name>/<slug>/ of an earlier build that misread the request, move there when first used.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.

### R-101 Agent CLI list is a master list
The registered agent CLI list is kept as a master list for the machine, not per workspace: deleting a workspace never removes agents, and every workspace opens with the master list. When a workspace refers to an agent the master list no longer has (planning coordinator or role assignments), the assignment is shown in red and can be reassigned; Generate plan stays disabled until every assignment refers to a listed agent.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.

### R-102 Reorder agent cards
Agent CLI cards can be moved up and down with the mouse (drag and drop), with keyboard-accessible move controls as an alternative. The order is saved, and every agent selection list (planning coordinator, roles, fallback) follows the same order.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.

### R-103 Readable file paths everywhere
Screens, error messages and notifications never show Windows extended-length path prefixes such as \\?\C:\Work\Temp or //?/C:/Work/Temp; paths read as C:\Work\Temp.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.

### R-104 Activity tab reads like the CLI
The Activity tab shows agent output the way the CLI shows it: assistant messages as text, tool calls as a one-line action with its key input, tool results as short summaries and errors highlighted. Raw JSON is never shown there; it remains in the Raw logs tab.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.

### R-105 Coordinator final step in the task list
The Agent Room task list shows the step the coordinator runs after the tasks, integration verification of the combined result, as its own entry with its status (pending, running, completed, failed) and which task tests failed, so a run that fails after every task completed shows why without opening other views.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.

### R-106 Decide a live tool request from its window
When a tool request window opens while the run is working, Allow once and Deny are available at once, without closing the window with Decide later and deciding again from the Tool requests list.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.

### R-107 Coordinator decides routine requests and records each decision
Simple decisions are made by the coordinator so the user is not asked often: it allows routine development requests, denies only publishing, outside-project changes, secret exposure or system changes, also for runs started before orchestration existed, and every decision is recorded with the request, the result, the reason and the time, visible in the task's Activity. Commands inside a workspace that itself lives under an app folder (such as ~/.agentconductor) are workspace commands.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.

### R-108 Task list without account usage
The Agent Room task list does not show account usage badges or refresh buttons; account limits are shown in the inspector's Account capacity section.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.

### R-109 The coordinator is selectable and shows its work; one name
The interface and documents use one name, coordinator, for the agent that plans, verifies the combined result and decides tool requests (previously also called orchestrator). In the Agent Room task list the coordinator entry is selected like a task: the center shows its heading, status and an updating Activity of its run steps, tool decisions and integration test results, and Raw logs show the integration test output, following the Agent room design (select on the left, progress in the center, evidence on the right).
Trace: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117. Source: user request, 2026-10-13.

### R-110 Failure cause and the next action on a failed run
When a run fails, the status banner shows why (for an integration verification failure, the failed task and the first lines of its test output) and offers Fix with feedback, which selects the failed task and drafts a direction with the failure output for the user to review and send; sending it reruns the task and repeats verification.
Trace: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118. Source: user request, 2026-10-13.

### R-111 The next step is always shown
The Agent Room status banner always says what to do next and offers that action: review the changes when the run waits for approval (Approve stays unavailable until the latest evidence is reviewed), approve after review, apply or merge after approval, and return to the workspace to plan the next phase when the run is complete.
Trace: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119. Source: user request, 2026-10-13.

### R-112 Plan review and run completion are organised by phase
A phased goal shows its phases in Review the plan: which phase is under review (phase n of m), which are completed, which comes next, and a Plan phase n action when the next phase is ready to load. When a run finishes, the Agent Room says which phase completed and which phase is next, and that it is planned from the workspace; a completed phase is recorded even if the goal folder moved after the run started.
Trace: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121. Source: user request, 2026-10-13.

### R-113 A blocking verification finding fails integration verification
When a verification report changed in the run records a blocking issue (a Blocking/차단 section with items, or a positive blocking-issue count), integration verification fails like a failing test, naming the report and the task responsible for the files the issue names, so the coordinator does not accept a candidate its verifier rejected and Fix with feedback drafts the correction for that task.
Trace: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120. Source: user request, 2026-10-13.

### R-114 The run's phase beside the workspace name
The Agent Room titlebar shows, beside the workspace name, the phase the run executes (phase n of m and its name) and what it covers (the phase description, or its task list when the phase file has no description); the full text is in its tooltip.
Trace: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122. Source: user request, 2026-10-13.

### R-115 Readable Agent Room text and agent icons sized to their names
The Agent Room's text is at least 11 px and scaled up in proportion (body text 14 px instead of 12 px) for readability, workspace screen text is at least 11 px, and the agent icons in the task list are the height of the name beside them instead of a large tile.
Trace: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123. Source: user request, 2026-10-13.

### R-116 Task cards in Review the plan fold to one line
In Review the plan each task is folded by default, showing only its number, name and a one-line description (the first line of its instructions that is not a label); an arrow unfolds it to show and edit every field.
Trace: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124. Source: user request, 2026-10-13.

### R-117 Review the whole phased plan up front
The coordinator plans every phase of a goal when the plan is generated, so Review the plan briefs each phase (what it will do, from the coordinator's analysis) and lists each phase's planned tasks, with the coordinator's analysis of the whole goal available; the phase about to run is open and its tasks are edited below, completed phases are marked, and later phases can be unfolded to review their tasks before any of them runs.
Trace: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125. Source: user request, 2026-10-13.

### R-118 A run survives closing or losing the desktop app
Closing the desktop app (or the app process ending) must not end a run: the run coordinator and its agent workers keep working and the app reattaches when reopened. Closing the window while a run is active asks: Keep running in the background (default), Stop the run and close, or Cancel.
Trace: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127. Source: user request, 2026-10-13.

### R-119 Interrupted runs recover and resume automatically
When a run is recorded as running but its coordinator and workers are gone (app killed, crash, reboot), opening the workspace or the Agent Room recovers it automatically (verifying every recorded process is absent, as Recover does) and, with automatic resume on by default, resumes it: completed tasks are kept, the interrupted task continues in its own native session (or a new one if the session cannot resume) with a note that its previous attempt was interrupted and its owned files must be inspected. After two consecutive automatic resumes that are interrupted again, it stops and asks the operator.
Trace: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128. Source: user request, 2026-10-13.

### R-120 Interruptions and silent workers are visible
The Agent Room shows when a run was interrupted and recovered (banner history and coordinator Activity), collapses a worker's consecutive thinking_tokens events into one line (Thinking… n updates, last at hh:mm) instead of empty Thinking tokens rows, and warns when a running task has produced no output for several minutes or its process is gone, suggesting Recover.
Trace: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129. Source: user request, 2026-10-13.

### R-121 Reassign tasks that have not started while the run is working
While a run is working, the operator can change the agent (and model) of a task that has not started yet, for example to move work from a busy Claude account to Codex; the change applies before that task starts. Running and finished tasks still need the run to be idle.
Trace: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130. Source: user request, 2026-10-13.

### R-122 Agents run with the highest permission by default
Agent workers run with the highest permission their CLI offers by default (Claude without permission prompts and with the operator's MCP servers such as Playwright, so it can open and inspect a browser UI; Codex without approvals or sandbox), while the planning and review coordinator stays read-only. The operator can lower a profile's permission.
Trace: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131. Source: user request, 2026-10-13.

### R-123 The coordinator resolves verification failures
When every task of a run has completed but integration verification fails (a failing test command or a verification report that records a blocking issue), the run does not stop at Failed waiting for the operator: the coordinator reviews the evidence, revises the tasks whose files must change and the verifying task, and the run resumes. A finding the run cannot resolve (outside every task scope, or needing an environment the agents lack) is recorded by the verifying task as a deferred non-blocking follow-up with its reason and owner. After three repair rounds without passing, the run stays Failed with the reason, for the operator to revise or reassign. Resume on such a failed run starts this repair.
Trace: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132. Source: user request, 2026-10-13.

### R-124 Execute in the selected project without adding Git metadata
Ordinary-folder shared executions create task files and results directly in the selected project. No `.git` directory or link is created there. Record an immutable baseline before tasks start; Accept finalizes the verified result and Reject restores owned files to that baseline. Warn about manual edits to owned files while tasks run and refuse to overwrite later edits. Existing Git repositories retain their current behavior; legacy runs retain their original bindings.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user workspace-path requirement and Claude review, 2026-10-13.

### R-124 Deleting workspaces needs no replacement name
Deleting a workspace asks for no new workspace name. When the current workspace is deleted, the most recently used other workspace opens, so several workspaces can be deleted in a row; only when none is left does one empty workspace start.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user request, 2026-10-13.

### R-125 Release-matched English and Korean manuals
Publish English and Korean operator manuals, a concise public README and genuine native screenshots matched to the shipped version. Keep implementation plans and internal decisions out of the README.
Trace: R-125 -> BG-80 -> SF-112 -> UC-115 -> FR-110 -> QAS-121 -> DD-134. Source: user release/manual requests, 2026-10-14.

### R-126 The app updates itself from GitHub releases
Every 10 minutes the app checks the latest GitHub release in the background. When a newer version exists, an update icon, a downward arrow, appears left of Settings; the icon and the workspace menu item Check for updates open the same dialog with the installed and latest versions. Update and restart downloads the archive for this platform, verifies it against the release SHA256SUMS, replaces the installed executable (or macOS app bundle) and restarts the app. It is refused while any run or app job is active, because restarting ends their agents. The update repository defaults to the project's GitHub repository. Settings can change it to a GitHub `owner/name` or an HTTPS repository URL on any GitHub-compatible host, such as an in-house GitHub Enterprise server; release notes, the archive and `SHA256SUMS` come from that repository's latest release, and only the repository is configured, not the download path (#130).
Trace: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135. Source: user request, 2026-10-14.


### R-127 Durable quota resume and five-hour fallback
Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

### R-128 Selectable interface themes

Settings offers two themes with identical information and layout: **Light** (default; native, system-settings-like surfaces with a blue accent) and **Dark** (the sage brand on dark surfaces). The choice applies immediately to the workspace screen and Agent Room, survives restarts and is machine-wide. In Agent Room each session row leads with the task name; a small agent icon, the agent name and a compact status tag follow on one line; the tag always stays inside the list, whatever the language or status length. Source: user decision on design proposals E and F, 2026-10-14.

### R-129 Plans know whether the project uses Git, and verification commands that cannot work are replaced
The planning coordinator is told whether the project folder is a Git repository and does not plan git test commands when it is not. If a planned test command cannot work in the project (for example git in a folder without Git) or an agent reports that its test command must change, the coordinator review replaces it with a command that verifies the same work instead of deferring the failure, and the replacement is recorded. Test commands of a folder without Git run against the history AgentConductor keeps outside it.
Trace: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138. Source: user request, 2026-10-14.

### R-130 Compact quota cards and readable step explanations

Keep account percentages and refresh together beside the agent name. Show quota reset times in the tooltip instead of extending an exhausted badge. Use slightly smaller, matching badge/icon sizes and reduce excess space around the card controls. Place a one-line explanation below every workspace step title; explanations use available width and wrap only when needed.
Trace: R-130 -> BG-85 -> SF-117 -> UC-120 -> FR-115 -> QAS-126 -> DD-139. Source: user, 2026-10-14.

### R-131 Readable and focused Settings

Separate section labels from selection controls with readable spacing. Hide agentmemory server configuration for Local Rust, retaining its saved value. Move Agent tools guide from Settings to the workspace menu.
Trace: R-131 -> BG-86 -> SF-118 -> UC-121 -> FR-116 -> QAS-127 -> DD-140. Source: user, 2026-10-14.

### R-132 Custom CLIs that approve their own tools can run
A custom agent CLI that approves its own tool use (for example one set to auto-approve tools) can be marked so in its agent settings; it then runs after plan approval instead of being refused as unverified. The mark is the operator's claim for this machine and is not trusted when settings are imported from another machine.
Trace: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141. Source: user request, 2026-10-14.

### R-133 Read actual release notes inside the update dialog

Show the actual GitHub release body below its existing release link. Long notes are scrollable; preserve the update and close controls.
Trace: R-133 -> BG-88 -> SF-120 -> UC-123 -> FR-118 -> QAS-129 -> DD-142. Source: user, 2026-10-14; next release after v0.1.18.

### R-134 Coordinator-selected MCP sources and large documents

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.


### R-135 Clear retry feedback and show coordinator activity

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.


### R-136 Approval readiness and Claude API usage limits

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.


### R-137 Codex transport recovery / R-138 in-app manual

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

### R-139 Workspace and Agent Room feedback refinements
Step 1 shows the planning coordinator right of the workspace name and the project folder above the goal, without a divider. MCP lookups show that they are running; Preview opens its window at once with a progress pulse and fills it when the lookup ends or shows the failure. jira:KEY and confluence:ID references in the goal are shown as tags that stay plain, deletable text, and hovering one shows its title and summary. The goal heading status follows the started run instead of staying Started. In the Agent Room, account usage shows Enterprise budgets as a remaining percentage like the agent cards, and the task shown on the right has a strong border in the list.
Trace: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148. Source: user requests, 2026-10-14.

### R-140 Runs keep going and stay visible
A model reply such as "Selected model is at capacity" is retried after a short wait, at most three times, for tasks and for the coordinator. Files an agent creates outside its owned paths in a project without its own Git (for example a CLI saving .aipro/plans) are moved out of the project into the run folder and recorded. Agent Room task rows lead with the reviewed task summary. A pressed button shows that its work is running, distinct from a disabled button, until the request and any background job it started end. The workspace page and the Agent Room stay open side by side: the operator switches freely, the room keeps its live progress, and the workspace shows the run state on its back-to-run button.
Trace: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149. Source: user requests, 2026-10-14.


### R-141 Selected signal instructions / R-142 responsive native path selection

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

### R-144 Readable offline HTML manuals

Update README and English/Korean manuals for0.1.21. App-menu help displays actual HTML tables, code, links and screenshots; standalone HTML copies are saved and shipped.
Trace: R-144 -> BG-99 -> SF-131 -> UC-134 -> FR-129 -> QAS-140 -> DD-153. Source: user, 2026-10-14.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

### R-147 Retained completed rooms do not block workspace creation

Allow creation/switching after a completed room remains open; preserve old evidence. Active run or UI job still blocks departure.
Trace: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156. GitHub #4, user2026-10-14.

### R-148 Claude prompt-compatible metadata collection

Recognize known interactive prompt variants before sending /usage; keep existing auth, no generation, bounded trust and timeout. Do not fabricate unavailable Enterprise allocations.
Trace: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157. GitHub #5, user2026-10-14.

DD-157 implementation note: the operator confirmed Windows Claude2.1.296 uses a > prompt and a Yes-I-trust-folder choice, with an explicit spent/allocated Enterprise line. Parse that declared allocation using existing exact-decimal subtraction; the display percentage is derived from remaining/allocated amounts. Date-only resets remain unscheduled. Committed fixtures use synthetic amounts; no operator account values are published.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

### R-150 One project workspace with multiple goal records

A project folder has one active owning workspace. New goals are added within it; registered slugs, completed evidence and prior run access remain visible. Reject duplicate project assignment without discarding draft input.
Trace: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159. GitHub #6, user2026-10-15.

### R-151 Rust-only application and release toolchain
All application behavior, user-interface interaction, release packaging, manual generation, and project-owned verification tooling shall be implemented in Rust. The distributed application shall not execute JavaScript or Python. Replace project-authored JavaScript and Python in these paths with Rust; do not vendor non-Rust implementation source from third-party projects. Preserve user-selected external agent CLIs and operating-system services as process/runtime boundaries.
Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160. Source: user, 2026-10-15.

### R-152 Recover clearly from a slow plan retry
When Generate plan is retried after an earlier failed or partially completed planning attempt, preparation, save, baseline checks, and progress snapshots use the bounded planning timeout rather than the short interactive-request timeout. Timeout guidance must describe the existing automatic status refresh and a usable workspace/run-reopen recovery path; it must not direct the operator to a nonexistent refresh control.
Trace: R-152 → BG-107 → SF-139 → UC-142 → FR-137 → QAS-148 → DD-162. GitHub #36, user, 2026-10-19.

### R-155 Maintain a local company build label
Keep Cargo's package version numeric and let developers set a local suffix in the tracked, empty-by-default `company-suffix.txt`. Workspace and Agent Room titlebars show `0.1.25.company_name` when populated; ordinary builds and public releases remain unsuffixed. Release packaging must refuse a non-empty suffix.
Trace: R-155 → BG-110 → SF-142 → UC-145 → FR-140/NFR-52 → QAS-151 → DD-165. GitHub #13.
### R-154 Custom CLI behavior belongs to its agent profile
An operator can define a custom CLI's event-name mapping and session-ID JSON Pointer in the per-agent JSON profile, and optionally configure a quota argv, timeout, and output pointer. Existing built-in providers keep their current defaults. Custom quota output uses the versioned application Snapshot schema; commands execute directly as argv without a shell. Shared workspace export/import preserves these settings without credentials.
Trace: R-154 → BG-109 → SF-141 → UC-144 → FR-139 → QAS-150 → DDR-17 → DD-164. GitHub #12, 2026-10-19.
### R-153 Explain task timeouts and resume automatically
When an agent subprocess reaches its wall-time limit, retain its partial work and logs, record the timeout attempt, and automatically retry after a bounded backoff unless the operator has stopped the run. Show timeout evidence while retrying and expose the cause when retries finish blocked so operators can choose a recovery action.
Trace: R-153 → BG-108 → SF-140 → UC-143 → FR-138 → QAS-149 → DD-163. GitHub #44, user, 2026-10-19.

### R-156 Make final review, approval, merge, and return to workspace visibly continuous
The Agent Room shall show a distinct merge stage, keep merge-in-progress feedback in the primary status banner and action area, and explain that final application completes the run and returns the operator to the workspace. Verified review, human approval, and final apply remain separate gates.
Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166. GitHub #50, user, 2026-10-20.

### R-157 Preserve task-state writes during transient SQLite contention
Parallel tasks sharing a run database must wait through short-lived writer locks and persist their state instead of failing immediately. Keep the wait bounded and return a visible error if contention persists; do not drop or silently retry non-idempotent data.
Trace: R-157 → BG-112 → SF-144 → UC-147 → FR-142 → QAS-153 → DDR-18 → DD-167. GitHub #56, 2026-10-20.

### R-158 Configure custom CLI models, execution options, and result decoding
Custom agent profiles shall use a manually managed model catalog unless an explicit discovery command and JSON response mapping are configured. Profiles may declare argument groups with templates, allowed values, defaults, and model-specific restrictions; tasks may override those groups. Omit an unset optional group as a whole, preserve selected values across resume, and revalidate them after reassignment. Configure a JSON Pointer for result extraction and an optional pointer/value condition that maps an otherwise normal result event to an error. Preserve these settings through profile editing and workspace export/import; keep built-in CLI behavior unchanged.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62, 2026-10-20.

### R-159 Show live agent activity, current tool, and time since last output
While a task runs, the Agent Room shall show what is observable without reading logs: the run state, the tool currently executing with its elapsed time, the time of the last output and the time since, and whether the owned process is present, absent or unverifiable. The display must not label silence as thinking or as failure; a CLI-provided reasoning signal is shown as such. Running tasks in the session list animate their agent icon, except under reduced motion.
Trace: R-159 → BG-114 → SF-146 → UC-149 → FR-144/NFR-54 → QAS-155 → DDR-20 → DD-169. GitHub #100, user, 2026-10-21.

### R-161 Chat with the coordinator from the workspace screen
The workspace screen shall offer a slide-over chat with the registered coordinator, closed by default. The operator can ask how to run or use a finished result, where outputs are, or how to use the current screen; the coordinator answers from the project's files, plans and documents and from the manual. The chat is read-only: it gives exact commands for the operator to run and never changes files or runs commands itself.
Trace: R-161 → BG-116 → SF-148 → UC-151 → FR-146/NFR-55 → QAS-157 → DDR-22 → DD-171. GitHub #115, user, 2026-10-21.
### R-160 Keep the Agent Room focus on the work that is in progress
When the session an operator is watching finishes and the run moves on, the Agent Room shall move its selection to the next running session, or to the integration entry when the run is waiting for approval, so the activity shown matches the run state. A session the operator selected for review after it finished stays selected.
Trace: R-160 → BG-115 → SF-147 → UC-150 → FR-145 → QAS-156 → DDR-21 → DD-170. GitHub #109, user, 2026-10-21.

### R-162 Recover from an out-of-scope integration failure through the app
When a worker finishes but files outside the approved task paths block integration, the operator shall be able to see every offending file, each task's approved paths and the files quarantined in stray-files; review and explicitly approve an exact plan change (owned paths, test command); restore quarantined files without overwriting; and continue from the finished work with the earlier evidence preserved. Feedback alone never changes owned paths.
Trace: R-162 → BG-117 → SF-149 → UC-152 → FR-147/NFR-56 → QAS-158 → DDR-23 → DD-172. GitHub #122, user, 2026-10-22.

### R-163 The coordinator decides when a task stops without finishing
When a task fails, exits without a result, or stalls until it can no longer continue, the workspace coordinator shall review the retained evidence and decide how to proceed (revise the task's instructions and resume, or stop for a person), so the run keeps going without asking the operator for routine judgement. A person is asked only when a decision needs more authority or intervention: a scope change, a tool or privilege approval, a coordinator verdict of blocked, or exhausted review rounds. The first slice covers failed and silent tasks; deciding what to do while an account is out of quota (for example moving a waiting task to an available agent) is a follow-up under the same issue.
Trace: R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173. GitHub #135, user, 2026-10-22.

### R-164 Lessons learned at each step are shown and shared with the agents
When a step teaches the run something worth sharing (the coordinator revises a task after a failure, replaces a test command, or an approved scope change), the app shall record it as a step lesson with its cause and response, show it in the Agent Room, give it to every later task as evidence in the prompt (not only the task it came from), and queue it for agentmemory so agents that use agentmemory or the shared signal can recall it. Lessons are bounded evidence: they never grant tool or merge permission and are never trusted as instructions over the task, scope and approvals. The earlier hash-pinned reviewed lessons stay as they are.
Trace: R-164 → BG-119 → SF-151 → UC-154 → FR-149/NFR-58 → QAS-160 → DDR-25 → DD-174. GitHub #138, user, 2026-10-22.

### R-165 The task detail column can be folded and still never hides a decision
The operator shall be able to fold the Agent Room's task detail column to a narrow strip and unfold it, and the choice shall be remembered across runs and restarts. When the run needs a person (approval, tool approval, blocked, or a failure that needs guided recovery) a folded column shall open by itself once per decision, show the candidate review first, and, if the operator folds it again while the decision is open, show a blinking warning dot on the strip until the decision is made. After the decision the column returns to the operator's own choice. Folding removes no information, and the dot blinks only for a decision and not under reduced-motion settings.
Trace: R-165 → BG-120 → SF-152 → UC-155 → FR-150/NFR-59 → QAS-161 → DDR-26 → DD-175. GitHub #146, user, 2026-10-22.

### R-166 The coordinator guards the goal, the plan rules and the scope as one charter
The coordinator shall hold the project goal, the planning rules, the owned scopes and the test contract of a run as one charter and use it at three gates: before approval (plan), before launch (a dry run of every test command, tools and runtime present, repository and file limits) and while running (failures, time-outs, scope escapes, tests that cannot run, a plan that cannot pass). A block at a gate is decided by the coordinator through a typed amendment (owned path, test command, dependency, task split, limit) that a deterministic validator checks against the charter; low-risk amendments are applied and recorded with undo, and only widening outside the project or sensitive paths, deleting files, changing the goal or exhausting the amendment budget asks a person.
Trace: R-166 → BG-121 → SF-153 → UC-156 → FR-151/NFR-60 → QAS-162 → DDR-27 → DD-176. GitHub #149, user, 2026-10-22.

### R-167 Every coordinator judgment is shown, whatever the level of trust
Each judgment the coordinator makes (tool decisions, reviews and their verdicts, replaced test commands, time-out resumes, recoveries, hand-offs, lessons, launch-gate results and amendments) shall be listed for the operator with the situation, the judgment and the result, in the coordinator view, in the activity of the task it concerns, and as a count on the coordinator row. Visibility does not depend on the amount of automation or trust and cannot be turned off.
Trace: R-167 → BG-122 → SF-154 → UC-157 → FR-152/NFR-61 → QAS-163 → DDR-28 → DD-177. GitHub #149, user, 2026-10-22.
