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
