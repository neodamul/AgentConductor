# Requirements

Baseline: 2026-10-09 KST. The user's requests define scope; no additional product features are assumed.

| ID | Requirement | Priority | Acceptance criterion |
|---|---|---|---|
| R-01 | Reuse existing CLI authentication and subscriptions | Required | No orchestrator API key entry for Claude or Codex |
| R-02 | Control subprocess I/O | Required | Collect executable/argv/cwd/env, stdin, stdout, stderr, and exit independently |
| R-03 | Register three native CLIs and arbitrary CLIs | Required | Claude/Codex/OpenCode profiles and custom profiles with the same contract |
| R-04 | Decompose a goal into exactly three tasks | MVP | Validated plan with inputs, scope, dependencies, completion conditions |
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
