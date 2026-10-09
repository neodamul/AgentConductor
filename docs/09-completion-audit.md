# Full-product completion audit

Updated: 2026-10-10 KST. The full user-requested product remains in development.
A prior CLI MVP release is a checkpoint, not product completion. Mock tests prove
local contracts only; native end-to-end behavior requires separate evidence.

| Requirement | Current authoritative evidence | Remaining acceptance work |
|---|---|---|
| R-01 Native authentication | Native Claude/Codex metadata queries | Real headless task execution with existing login; OpenCode/native provider configuration |
| R-02 Subprocess control | Process tests, raw spools, exit/timeout | Streaming normalized progress and crash ownership records |
| R-03 Registered providers | Templates, custom contract, parser/argv tests | Real provider contract fixtures and end-to-end registration/execution |
| R-04 Planning | Planner mock tests and plan validation | Goal -> native planner -> registered agents -> verified result end-to-end |
| R-05 Isolation/parallelism | Shared/worktree mock E2E; leases/checkpoints and transitive dependency refresh | Native concurrency evidence |
| R-06 Evidence | Git/test integration, snapshots, reviewed rework, retained workspaces, bounded typed coordinator proposals | Normalized summaries/failure inventory and native review contract evidence |
| R-07 Approval | Exact candidate binding and mock E2E | Full negative tests for target/index/diff/test changes; native execution review |
| R-08 Dangerous actions | Typed SQLite approval ledger, bidirectional control/denial mocks, bounded file-policy/controller integration, alias/replay/closed-channel negative tests and metadata-only native initialization | Production worker bindings, authoritative context revalidation, actual native denial/approval/failure containment for every provider; profile labels alone do not prove containment |
| R-09 Lifecycle | Quota, CLI stop/resume/revise/review, leases, checkpoints, crash recovery and rework tests | Native contract evidence and interrupted native review recovery |
| R-10 Memory | Outbox/offline recall tests and development saves | Runtime session context, lessons, progress, bounded context handoff, revalidation |
| R-11 Wiki | English raw/schema/wiki files | Runtime source hashing, stale-source checks, lesson asset generation |
| R-12 Rust/lightweight | Single macOS binary, bundled SQLite | Updated final binary after remaining work, runtime resource measurements |
| R-13 Workflow/TDD | Trace documents and Red/Green tests | Requirement-by-requirement final audit and evidence, no premature completion |
| R-14 Development tools | RTK, Graphify extraction, CocoIndex structural grep, Ponytail guidance | Retain concrete evidence as subsequent changes are reviewed |
| R-15 Distribution licenses | 42 dependency declarations/texts, SQLite/Unicode notices | Recheck final dependency closure and all bundled/optional distribution boundaries |
| R-16 English/public docs | Current English documents and public README | Keep status accurate and exclude internal decisions from README |
| R-17 UI | Clearly labeled mockups | Functional lightweight interaction UI, diff/tests, approval and session controls |
| R-18 Availability | Parsing, schedule, bounded retries, watch tests | Native restriction fixtures, crash-safe schedule ownership and runtime UI |
| R-19 Multi-session transfer | Mock concurrency/reassignment, native session exclusion and refreshed dependency context | Native execution evidence |
| R-20 Session models | Native Claude/Codex catalogs; model edit tests | OpenCode/self-hosted catalogs, native model changes and UI controls |

## Execution order

- [x] Restore full completion criteria and record the user's correction in agentmemory.
- [x] Finish T-03 explicit stop/resume, including integration-test interruption and CLI verification.
- [x] Finish T-02 crash/ownership recovery and cross-run session/working-file ownership.
- [x] Complete human feedback/rework and dependency propagation.
- [x] Implement and verify coordinator-generated review feedback from bounded diff/test evidence using mock subprocess contracts; native contracts remain open above.
- [ ] Enforce dangerous-action capabilities and human gates before native execution.
- [ ] Complete runtime memory/wiki/lessons and source revalidation.
- [ ] Implement the lightweight human-intervention UI using the approved visual direction.
- [ ] Perform real native CLI end-to-end execution/model/session verification.
- [ ] Verify final licenses, resource usage, artifact checksum and complete-product acceptance.

Proceed autonomously under existing authorization while resources permit. Preserve the
user's original scope. Keep the goal active until this audit is proven, rather than
presenting the checkpoint as a finished product.
