# Implementation plan and TDD

Current status: full product under development; verified CLI checkpoint with optional worktrees, parallel subprocess tasks, SQLite
evidence, review-gated integration, local memory delivery, and restriction/session controls.
107 tests passed at the current checkpoint; current verification is recorded below. Native Claude/Codex catalog queries are verified; OpenCode has no
configured model entries. Paid live agent execution, source revalidation, enforced dangerous-action controls and a working UI remain separate unverified scopes.

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
rtk cargo fmt --check; lockfile-based license/text checks and SBOM generation through
the packaging script. `cargo deny` is an optional additional check and is not installed
on the current host.
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
- [x] Task: cross-run native session leases and paused-workspace checkpoint validation (DD-14).
- [x] Verify: live-owner rejection, SIGKILL lease release, absent/ambiguous process ownership refusal, and resumed evidence preservation.
- [x] Verify: killed coordinator's live children also prevent a different run; dirty-repository rejection does not create an unknown ownership claim.
- [x] Verify: native session reuse refuses live/unknown previous ownership; workflow does not spawn while leased elsewhere; shared/worktree edits block resume.
- [x] Complete: compiled CLI recovery/control verification and documented limits. Native provider generation remains a separate full-product acceptance item.

A PID or leftover lock file alone is not proof of live ownership. Keep the lock inode stable;
use an OS advisory exclusive lease rather than deleting a possibly active owner's file.
Record owned process/group identity before starting I/O collection. Recovery cannot assume
that an absent process record means the subprocess did not start.

## T-05 follow-up — Review feedback and dependency rework

- [x] Plan: R-04–07/09 → BG-02/03 → SF-02–06 → UC-03/04 → QAS-02/06/10 → DD-15.
- [x] Task: Red tests for human feedback, transitive invalidation, preserved prior attempts and refreshed dependency contents.
- [x] Task: bounded review feedback, audited rework, updated dependency context and safe worktree refresh.
- [x] Verify: shared/worktree dependency chains, failure feedback, conflicts, stop/resume and exact fresh approval (82-test regression suite).
- [x] Complete: compiled CLI controls, evidence, documentation and agentmemory update.

Changed predecessor results must be visible before a dependent retries. Preserve old
worktrees and evidence rather than resetting them in place. A refresh conflict blocks
that task with retained evidence; it must not launch a worker against ambiguous contents.

Structured coordinator review is verified below with mock subprocesses. Native provider
review contracts and enforced pre-execution permissions remain separate acceptance items.

## T-06 follow-up — Structured coordinator review

- [x] Plan: R-04/06/07/09 → BG-02/03 → SF-02/04/05/06 → UC-03/04 → QAS-02/03/05/06 → DD-16.
- [x] Task: Red tests for typed final JSON, unknown task/evidence rejection, bounded context and coordinator read-only preservation.
- [x] Task: registered subprocess review, strict proposal validation, bounded revision rounds, audited scoped feedback and retained ownership.
- [x] Verify: failure → reviewer → revised tasks → new tests → exact human approval; malformed proposals, altered evidence, stale feedback, stopped reviews and write violations.
- [x] Complete: compiled `review`/`apply-review` controls, documentation, agentmemory and audit evidence for the mock CLI contract.

Verification: `rtk cargo test` passed 96 tests (one intentionally ignored subprocess helper),
27 suites in 37.42 seconds; Clippy and formatting checks passed. Graphify extracted
234 nodes/737 edges without an LLM; CocoIndex located saved-proposal application in
CLI and workflow modules. No new runtime dependency was added. Native execution,
pre-execution permission gates, runtime knowledge and functional UI remain open.

## T-03/06 follow-up — Pre-execution approval transport

- [x] Plan: DD-17, native callback failure semantics and retained process containment.
- [x] Task: Red tests for permission request/reply, stdin lifetime, malformed output and failed callback containment.
- [x] Task: bounded bidirectional JSONL transport integrated with the existing supervisor.
- [x] Verify: mock denial creates no side effects; exact human ledger approval permits one execution; protocol failure kills/reaps owned processes and preserves raw evidence.
- [x] Complete: transport/controller/ledger component evidence and documentation; this does not complete native permission enforcement.
- [x] Task/verify: SQLite exact-request/context approvals, maximum 900-second expiry, single consumption under concurrent callbacks, central audit/outbox.
- [x] Verify: installed Claude 2.1.286 accepts SDK PreToolUse registration in metadata-only initialization; no generation prompt sent.
- [x] Task/verify: native controller preserves a permission decision owed by active work after a result frame (Red → Green).
- [ ] Task/verify: durable typed human decisions, provider capability integration and actual native denial/approval tests.

Passing transport mocks does not prove complete R-08 enforcement. CLI profile labels,
ordinary shell hooks and post-execution integrity checks cannot substitute for native
pre-execution negative tests.

Current component regression: 107 tests passed, two opt-in/helper tests ignored,
30 suites in 37.64 seconds; native metadata test also passed separately
with `rtk cargo test --test native_control -- --ignored`. The native test is opt-in;
ordinary test discovery skips it and the existing subprocess helper. Clippy and formatting
checks pass. Graphify: 284 nodes/869 edges; CocoIndex locates the atomic consumption path.

Native metadata evidence is retained locally under
`.agentconductor/verification/claude-control-metadata-3042/process/`; stdout SHA-256
`935b6d736d8562ab968ac4a6fa440f34f46f0a00f3d3753e09e488a84857f5f1`.
Initialization succeeded, no user frames were sent, and ownership is terminal/quiescent.
`claude auth status` currently reports no login; actual generation/denial tests await
existing-subscription login. API keys are not a fallback.

## T-03/06 follow-up — Bounded worker policy and callback lifecycle

- [x] Plan: DD-18, file ownership and explicit approval boundaries.
- [x] Task: Red → Green for duplicate tool-use identities before approval consumption and requests after input closure.
- [x] Task: Red → Green for bounded tool inputs, owned edits, sensitive paths and filesystem alias rejection.
- [x] Verify: policy connected to bidirectional callback component, with pre-execution denial and retained process evidence.
- [x] Complete: component verification, local knowledge asset and agentmemory decision record.
- [ ] Task/verify: production worker context binding, human CLI controls and live native denial/approval.

Regression: 116 tests passed, two opt-in/helper tests ignored, 32 suites in 38.76
seconds. Clippy with warnings denied, formatting and whitespace checks passed.
Graphify extracted 308 nodes/941 edges without an LLM; CocoIndex found the classifier
and its evaluator bridge. No runtime dependency was added. Ponytail remains guidance;
no callable Ponytail tool or installed CLI was available in this environment.

Policy tests include replacement of the workspace directory at the same path,
directory/file symlinks, hardlinks, FIFO, ambiguous paths, unsupported input fields,
bounded input, secret filenames and task ownership. The controlled subprocess test
verifies no cross-task write, no pending-command side effect, one approval consumption
and denial after consumption. It uses a mock native protocol and fixed test context;
it does not verify production workspace fingerprints or native CLI enforcement.

## T-03/06 follow-up — Worker-bound native decisions

- [x] Plan: DD-19, bounded asynchronous verification and immutable request binding.
- [x] Task: Red → Green for asynchronous control cancellation and verifier containment.
- [x] Task: authoritative context verifier derives session/workspace/plan/Git binding from stored state; ignored sensitive targets use metadata without reading contents.
- [x] Task: opt-in Claude native callback worker integration with preassigned owned sessions.
- [x] Task: operator list/refresh/approve/reject controls and distinct waiting state.
- [x] Verify: compiled mock workflow controls in shared/worktree modes, stale/expired/denied decisions, native-session retention and separate merge approval.
- [x] Complete: worker/operator implementation checkpoint evidence, wiki and agentmemory; full-product completion remains open.
- [ ] Verify: actual authenticated provider tool denial/approval and containment.

Verifier/transport checkpoint: 121 tests passed, two ignored, 34 suites in 43.06
seconds. Clippy, formatting and whitespace checks passed. Compiled verifier binding
tests cover both workspace modes, changed working files, unknown tasks and foreign
write scopes. Owned async tests cover stop, callback cancellation without a cooperative
stop listener, pending callback timeout and normal-exit final-frame draining.
Actual Claude metadata-only initialization passed separately in 0.90 seconds; no
generation prompt was sent. This does not prove native tool enforcement.
Graphify extracted 321 nodes/987 edges without an LLM; CocoIndex located the verifier.
No runtime dependency was added. Worker/operator mock-contract implementation is now
present; live provider enforcement remains open. The follow-up adds durable decision
epochs, exact-ID receipt lookup, exclusive approved retries, session identity checks,
stopped-state preservation and explicit rejection handling. Red → Green covers old
receipts beyond the listing limit and tampered payloads. Compiled CLI tests cover both
workspace modes, pending/consumed permissions, expiry/refresh, rejection/revision,
explicit stopped resume, changed paused workspaces and separate final merge approval.

Worker/operator checkpoint: 127 tests passed, two ignored, 34 suites in 49.61 seconds;
Clippy, formatting and whitespace checks passed. Actual Claude metadata-only control
initialization passed separately in 0.90 seconds. Graphify extracted 341 nodes/1,083
edges without an LLM; CocoIndex located exact-ID receipt reads. No runtime dependency
was added. Native tools, provider containment, runtime knowledge and functional UI
remain open in the full-product audit.

## T-03/05/07 follow-up — Live intermediate reports

- [x] Plan: DD-20, owned spool observation, independent attempt evidence and bounded local summaries.
- [x] Task: Red → Green tests observe normalized evidence before worker completion, reject malformed streams and verify final draining.
- [x] Task: bounded observer and SQLite attempt event ledger connected to both worker transports.
- [x] Verify: regression, cancellation, final records, retained raw evidence and bounded queries.
- [x] Complete: evidence, wiki and agentmemory checkpoint; native runtime proof remains separate.

Live-progress checkpoint: 131 tests passed, two ignored, 35 suites in 50.14 seconds.
Clippy, formatting and whitespace checks pass. Graphify extracted 352 nodes/1,110
edges without an LLM; CocoIndex confirmed both worker transports use the observer.
No runtime dependency was added. A deferred SQLite read-to-write upgrade conflict
was reproduced in native-worker mock rework and fixed with an immediate transaction.

## T-07 follow-up — Runtime reviewed lessons

- [x] Plan: DD-21, bounded typed promotion, repository source hashes and task-scoped handoff.
- [x] Task: Red → Green for exact promotion consent, stale source exclusion and scope/alias refusal.
- [x] Task: authoritative local assets, derived wiki/index/log, operator controls and worker context.
- [x] Verify: compiled digest/accept controls, offline next-attempt task-scoped context, changed sources, mirror alias refusal and recovery.
- [x] Complete: checkpoint evidence and wiki/agentmemory; full-product acceptance remains open.

Runtime lesson checkpoint: 134 passed, two ignored, 36 suites in 51.90 seconds.
Clippy, formatting and whitespace checks pass. Graphify: 377 nodes/1,199 edges, no LLM;
CocoIndex located worker context injection. No runtime dependency added. Staleness
uses source hashes; bounded metadata checks do not freeze filesystem paths. Native
lesson service/confidence-aware retrieval, cross-run context and UI remain open.

## T-04/09 follow-up — Native Codex subscription/session contracts

- [x] Plan: existing ChatGPT authentication, explicit session IDs, no API-key fallback and retained process evidence.
- [x] Verify: installed Codex 0.162.0 stdin/JSONL initial turn and exact session resume preserving previous context.
- [x] Verify: two actual concurrent sessions retain distinct IDs and recall their own context on resume.
- [x] Task: Red → Green for nonterminal Codex error items classified as visible warnings; terminal outcomes remain separate.
- [x] Complete: regression evidence, wiki and agentmemory checkpoint; full-product goal remains active.
- [ ] Verify: native scoped implementation, dangerous-tool containment/approval and all-provider model/session behavior.

Native Codex checkpoint: 135 passed, four ignored, 37 suites in 52.91 seconds.
The two generation tests were also run explicitly: exact context resume passed in
7.74 seconds; concurrent independent contexts passed in 7.13 seconds. Installed
version: 0.162.0. Clippy/format/whitespace checks pass. Graphify: 377 nodes/1,199
edges, no LLM; CocoIndex located the nonterminal item normalization. No runtime
dependency added. Actual risky tool/native implementation tests and functional UI
remain open; these text-turn proofs do not close R-08 or the full product.

## T-08/09 follow-up — Functional intervention room

- [x] Plan: DD-22, approved visual direction, embedded native surface and exact-state control broker.
- [x] Task: Red → Green for newer-stop preservation, stale mutation refusal, local capability enforcement, cookie collisions and artifact/env exposure.
- [x] Task: embedded Agent Room with real task/evidence/approval/session controls.
- [x] Verify: backend integration and real rendered fixture feedback/reassignment/model/resume/diff/tests/approval/merge.
- [x] Verify: native Cocoa/WebKit review rendering and Cmd-W close with process exit 0 and endpoint shutdown, using a local metadata wrapper.
- [x] Task: Red → Green for same-run native-window lease, child CLI duplicate refusal and reopen after release.
- [ ] Verify: unbundled identity and remaining availability/tool-gate interactions.
- [x] Complete: actual browser UI screenshot, public usage, wiki and agentmemory checkpoint.
- [ ] Verify: UI runtime resources and final distribution/license evidence.
- [ ] Verify: goal/profile setup and all native provider/model/permission acceptance.

Intervention room checkpoint: 142 tests passed, four ignored, 39 suites in 55.78
seconds. Clippy passes. Graphify extracted 482 nodes/1,508 edges without an LLM;
CocoIndex located the local and native control implementations. No Cargo dependency
was added. The first whole-suite attempt exposed a retained PID-based temporary-run
collision; UUID fixture paths preserve failed evidence and prevent reuse. Real browser
verification uses a named local mock fixture, not authenticated provider execution.
Native Cocoa/WebKit review rendering and Cmd-W close were verified using a local app
metadata wrapper around the same binary. The first close shortcut failed; standard
application/edit/window menus fixed it, and the updated process exited 0 with its endpoint
stopped. Bare-executable computer-use identity, quota/tool-gate UI and resource acceptance
remain open. User correction requires closing prior owned verification windows/processes
and checking their exit before replacement; stored lesson lsn_5169ce1ac0c2a86d.

### Direct native transport (DD-23)

- [x] Plan: replace loopback HTTP with an embedded native message bridge.
- [x] Task / Red: missing Room compilation failure; additional fields exposed unit-variant deserialization weakness.
- [x] Task / Green: Rust Room controller, bounded typed requests, WebKit reply handler and embedded document.
- [x] Verify: full suite, targeted final-source tests, Clippy/rustfmt; actual native feedback/resume/review/approve/merge, zero TCP listeners and exit-0 cleanup.
- [x] Complete: README and actual native screenshot refreshed; direct-native wiki evidence and agentmemory records saved; checkpoint prepared.

### Unix termination evidence (DD-24)

- [x] Plan: retain numeric exit status and termination signal as separate outcomes.
- [x] Task / Red: missing signal fields failed compilation; coordinator incorrectly reported synthetic exit -1.
- [x] Task / Green: separate optional signal in process/ownership/test/coordinator evidence; completed coordinator flag prevents signal failure appearing running.
- [x] Verify: 33 targeted tests; complete suite 144 passed / 4 ignored / 0 failed; Clippy/rustfmt. Fresh older-fixture run succeeded, so historical failure remains unexplained.
- [x] Complete: source-hashed process-outcomes wiki, completion audit and agentmemory updated; checkpoint prepared.

### Native startup workspace (DD-25)

- [x] Plan: trace R-21 through business goal, feature, use case, QAS and DD-25.
- [x] Task / Red: registry persistence/redaction/stale revisions; modified profile consent bypass, late repository baseline change and oversized settings temporary-file leak reproduced.
- [x] Task / Green: local startup controller, owned catalog/planning jobs and exact consent; repository baseline rechecked under execution lease; settings bounds checked before temporary-file creation.
- [x] Task / UI: profile cards, goal setup, editable plan review and same-window run opening; macOS JSON smart quote substitution disabled and verified.
- [x] Verify: full regression 148 passed / 4 ignored / 0 failed; Clippy/rustfmt. Actual native empty startup → custom profile registration → planning → task edits → exact plan consent → same-window run → diff/tests → candidate approval → merge → home; exit 0 and kernel absence confirmed. Final text-input correction separately verified in the rebuilt native app.
- [x] Complete: public documentation and actual screenshot, source-hashed wiki and agentmemory checkpoint (`mem_mv1ukwwx_de6b3718e639`). Graphify: 548 nodes / 1,752 edges; CocoIndex located reviewed execution. Provider-authenticated planning, dialog accessibility and OS containment remain part of the full-product audit.
