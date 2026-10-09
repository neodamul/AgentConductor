# Implementation plan and TDD

Current status: full product under development; verified CLI checkpoint with optional worktrees, parallel subprocess tasks, SQLite
evidence, review-gated integration, local memory delivery, and restriction/session controls.
59 tests passed at the prior checkpoint; current verification is recorded below. Native Claude/Codex catalog queries are verified; OpenCode has no
configured model entries. Paid live agent execution, crash ownership recovery, source revalidation, and a working UI remain separate unverified scopes.

Each task: **plan** (linked R/BG/SF/UC/QAS/DD and acceptance criteria) → **task**
(Red → Green → Refactor) → **verify** (counterexamples, integration, review) → **complete**
(evidence, report, lessons). QAS defines quality goals; tests verify them.

| Task | Dependency | DD | Tests to fail first | Completion evidence |
|---|---|---|---|---|
| T-01 Profile/plan schema | Design documents | 01/03 | Wrong task count, cycles, overlapping scopes, argv injection | Schema/boundary tests |
| T-02 SQLite state/audit | T-01 | 02 | Illegal transitions, duplicate events, stale lease recovery | Restart tests |
| T-03 Subprocess supervisor | T-01 | 04/07 | stdin EOF, stderr flood, split UTF-8, stop grandchildren | Mock process contract |
| T-04 Three parsers/resume | T-03 | 03/04/07 | Fixtures to normalized events, errors, exact sessions | Version-pinned fixtures; separate live smoke |
| T-05 Workspace/scheduler | T-02/03 | 05 | Both workspace modes, three parallel executions, fixed base, dependency blocks, scope violations | Temporary Git integration |
| T-06 Diff/test/approval/merge | T-04/05 | 05/06 | Untracked files, failing tests, unapproved/changed candidate rejection | Protected-branch before/after SHA |
| T-07 Memory/wiki | T-02 | 08/09 | Outage outbox, duplicate delivery, stale sources | Offline fallback/source checks |
| T-08 CLI E2E/release | T-01–07 | 10 | Register → three tasks → verify → approve → merge; rejection/revision | Measured QAS, license/SBOM |

Verification: rtk cargo test; rtk cargo clippy --all-targets -- -D warnings;
rtk cargo fmt --check; lockfile-based cargo deny check licenses and SBOM generation.
Unit tests use mocks/captured fixtures, not paid model calls. Record live CLI smoke separately
using existing authentication. Completion requires passing tests, tested integration candidates,
and human approval matching exact SHAs.

Trace example: R-07 → BG-03 → SF-05 → UC-04 → QAS-02 → DD-06 → T-06 →
approval_changed_head_rejected. Add actual test names and evidence during implementation.

## Progress checklist

Checked items have current evidence. A foundation is not completion of the entire task.
Update this checklist after verification, not merely after writing code.

### T-01 — Profile and plan validation

- [x] Plan: requirements and acceptance criteria linked to DD-01/03.
- [x] Task: three-task schema, explicit workspace mode, DAG and scope validation.
- [x] Verify: invalid plans, path escapes, overlapping scopes, and custom profiles tested.
- [x] Complete: foundation committed; TDD evidence recorded.

### T-02 — Persistent state and audit

- [x] Plan: lifecycle, audit, and recovery requirements defined.
- [x] Task: SQLite transitions and event persistence implemented.
- [x] Verify: illegal transitions, duplicate events, and database reopen tested.
- [x] Task: workflow document, audit, and memory-outbox transactions integrated.
- [ ] Verify: interrupted coordinator recovery and stale process ownership.
- [ ] Complete: full recovery scope verified and documented.

### T-03 — Subprocess supervisor

- [x] Plan: process contract and timeout policy defined.
- [x] Task: direct argv, cwd, stdin EOF, concurrent stdout/stderr spool, Unix process groups.
- [x] Verify: six cases cover large I/O, nonzero exit, spawn failure, timeout descendants,
  unread stdin, and forced-kill escalation.
- [ ] Task/verify: explicit user stop and restart recovery.
- [ ] Complete: full lifecycle controls verified. Windows containment is a separate target.

### T-04 — CLI protocols and resume

- [x] Plan: native headless and resume commands researched.
- [x] Task: Claude/Codex/OpenCode/custom JSONL normalization and native argv builders.
- [x] Verify: synthetic contracts, malformed output, exact session selection, resume flag differences.
- [x] Task: environment/input-mode/permission profiles connected to workflow execution.
- [ ] Verify: installed-version fixtures and live CLI smoke; resumed permission capability checks.
- [ ] Complete: supported CLI/version contracts documented with evidence.

### T-05 — Workspace and scheduler

- [x] Plan: shared default, optional worktrees, disjoint ownership, dependency semantics.
- [x] Task: Git candidate snapshots with a temporary index and optional worktree creation.
- [x] Verify: three Git tests cover new files, unchanged HEAD/index, scope violations, exact base.
- [x] Task: three-agent parallel scheduler and dependency propagation.
- [x] Verify: shared/worktree E2E, overlapping mock attempt intervals, failed-dependency blocking.
- [ ] Complete: both execution modes verified and documented.

### T-06 — Evidence, approval, and merge

- [x] Plan: bind approval to candidate, base, diff, and test evidence.
- [x] Task: integration candidate, coordinator tests, review artifacts, approve/reject/merge commands.
- [ ] Verify: failing tests, modified files/evidence, changed target branch, missing approval.
- [x] Complete: reviewed candidate merged in mock E2E test with exact before/after SHAs.

### T-07 — Memory and knowledge

- [x] Plan: SQLite authority, memory outbox, local fallback, wiki rules defined.
- [x] Task: English source/wiki/schema assets and development progress in agentmemory.
- [x] Task/verify: outbox delivery, offline continuity, local FTS recall, HTTP mock delivery.
- [ ] Task/verify: runtime lessons and source revalidation.
- [ ] Verify: offline continuity, delivery retries/deduplication, changed sources.
- [ ] Complete: runtime memory and knowledge workflow verified.

### T-08 — End-to-end verification and binary

- [x] Plan: MVP and release evidence defined.
- [x] Verify: initial native validate/normalize binary built and smoke-tested; 35 baseline tests passed.
- [x] Task: host dependency license declaration and source-text inventory (42 packages).
- [x] Verify: registered-profile planner tests and mock CLI run → parallel tasks → evidence → approval → merge E2E.
- [x] Verify: 42 dependency license texts, notices, CycloneDX inventory, and startup resource measurement; live agent resources remain unmeasured.
- [x] Task: macOS ARM64 binary, checksum, usage examples, and release manifest.
- [ ] Complete: full user-requested product acceptance audit and final binary. Prior mock CLI release is a verified checkpoint only.

## Distribution requirements

Distribute one native executable with rusqlite bundled SQLite, without required Python/Node
sidecars. Apply release optimizations. Track OS/architecture artifacts, checksums, dependency
and license manifests as release evidence.

## Commit and push cadence

Keep commits small and reviewable. Push accumulated commits at meaningful checkpoints
or when remote review is needed; pushing after every change is not required.
This updates the earlier request to commit and push each small unit.

## T-09 — Availability, multi-session roles, and models

- [x] Plan: R-18–20 → BG-07–09 → SF-09–11 → UC-09–11 → QAS-09–11 → DD-11–13.
- [x] Task: Red tests for restriction parsing, persisted scheduling, reassignment, and model approval invalidation.
- [x] Verify: three restriction parser tests pass (explicit offsets, relative delays, ambiguous messages, multiple windows).
- [x] Task: persisted due-work execution and bounded automatic retries.
- [x] Task: multiple sessions, per-profile concurrency, account groups, and audited reassignment.
- [x] Task: native catalog query/import and session model selection/change.
- [x] Verify: five library recovery tests, binary session-control E2E, two catalog tests; native Claude/Codex metadata smoke.
- [x] Complete: CLI controls documented, 59 tests and native metadata evidence retained, agentmemory updated, release binary produced.

## Current verification evidence

- `tests/availability.rs`: explicit/relative reset, multiple windows, offset parsing, ambiguous messages.
- `tests/recovery.rs`: restart scheduling, role transfer, model approval invalidation, concurrency cap, retry ceiling.
- `tests/models.rs`: metadata-only CLI protocol mocks; malformed catalogs rejected.
- `tests/cli_workflow.rs`: exact plan consent and merge approval; CLI schedule/reassign/model/resume commands.
- `tests/workflow.rs`: shared/worktree integration, retained failures and changed evidence.
- `tests/planner.rs`, `tests/git.rs`, `tests/memory.rs`: three-task planning, temporary-index snapshots, outbox and offline recall.

The full product backlog is larger than the deliverable CLI MVP. Unchecked scope must not
be described as verified or shipped. Native metadata smoke is not live paid execution.

## T-03 follow-up — Explicit stop and resume

- [x] Plan: preserve process containment (DD-04/07), ownership, artifacts, and human review gates.
- [x] Task: Red tests for process-tree stop, stop-before-launch, and workflow stop/resume.
- [x] Task: durable stop request, interrupted attempts, and explicit resume commands.
- [x] Verify: two supervisor stop tests and two workflow stop/resume tests, including interruption during integration verification.
- [x] Complete: documented CLI controls, binary-command verification, and passing regression suite.

A human stop must terminate the owned process group, retain raw I/O and partial files,
prevent automatic restart, and revoke any candidate approval. Resume must verify the target
Git baseline and permission configuration, append a new attempt, and repeat integration
verification. Unknown/crashed ownership is a separate recovery operation.

Full completion is governed by [the requirement-by-requirement audit](09-completion-audit.md).

## T-02 follow-up — Crash ownership and leases

- [x] Plan: recover only after kernel ownership is idle and all recorded processes/groups are terminal; preserve unknown ownership for intervention.
- [x] Task: Red tests for persistent exclusive leases and symlink rejection.
- [x] Task: kernel-backed coordinator lease and durable process ownership records.
- [x] Task: explicit crash recovery and cross-run repository ownership claims.
- [ ] Task: cross-run native session leases and paused-workspace checkpoint validation.
- [x] Verify: live-owner rejection, SIGKILL lease release, absent/ambiguous process ownership refusal, and resumed evidence preservation.
- [x] Verify: killed coordinator's live children also prevent a different run; dirty-repository rejection does not create an unknown ownership claim.
- [ ] Complete: real CLI recovery/control verification and documented limits.

A PID or leftover lock file alone is not proof of live ownership. Keep the lock inode stable;
use an OS advisory exclusive lease rather than deleting a possibly active owner's file.
Record owned process/group identity before starting I/O collection. Recovery cannot assume
that an absent process record means the subprocess did not start.
