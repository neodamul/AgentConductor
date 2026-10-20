# Full-product completion audit

## Current checkpoint — 2026-10-20

The full user-requested product remains in development. Issue #9 is open and PR #22 remains draft. The latest product-code batch is `c995e00754766a84ff0dc7f11c12d2e82d8094b2`; the exact macOS package evidence below remains historical at source `7289562e5de394555b196da75b0ecf3904f3c53c`. The Rust native UI has source implementations for 25/25 workflows, but only **2/25 (8%)** have packaged end-to-end UI acceptance; the shipped Rust replacement remains **0/25** until an accepted release ships.

The grouped UI batch (`cb49fa1`) fixed four audited defects: return from a running Agent Room without stopping it, preserve the selected Review step, safely stop-and-delete a workspace with an active plan only after plan/metadata work settles, and fit the agent editor at 640px. The subsequent Rust scheduler batch (`fdc3528`) restores timeout accounting, bounded automatic retry, and Stop-aware backoff. Its targeted verification passes 24 tests with 1 ignored across eight suites; the follow-up bilingual-docs regression passes 7 tests across `public_docs` and `source_layout`. The last complete 759-pass native-feature suite and exact macOS package evidence are from code source `7419557`, before `fdc3528`, and are historical evidence rather than validation of the current head.

The exact macOS package from code source `7419557` was opened in one isolated app window for a grouped screen/control audit: Workspace Settings, Settings/MCP registration, profile registration, and Agent Roles. Two temporary workspaces were created and one inactive workspace archived; a delayed Rust mock plan showed Running/Stop planning and then returned the coordinator to Ready after stop-and-delete. Evidence: `docs/evidence/issue9-7419557-packaged-ui-audit.json`. No provider CLI or external service ran. This spot-check does not constitute a complete packaged workflow, so acceptance remains 2/25. On 2026-10-20, exact source `7289562e5de394555b196da75b0ecf3904f3c53c` built a 17,925,856-byte macOS binary and passed `package-release`, `validate-package`, packaged `--help`, ZIP CRC/checksum, and four mock parallel-workflow smoke combinations. Evidence: `docs/evidence/issue9-7289562-macos-package.json` and `docs/evidence/issue9-7289562-macos-smoke.json`. This CLI/package evidence does not count as UI acceptance. The latest full native-feature run at product-code batch `c995e00` passes 768 tests / 14 ignored across 260 suites. Its grouped Settings dialog regressions pass 10 tests across four suites, covering MCP server and reference editors, cache, sharing, and Messenger dismissal. All-target Clippy, formatting, diff, documentation, layout, and 300-line checks pass. These source-level checks do not count as packaged UI acceptance. The active v0.1.31 user app remains open and untouched.

This checkpoint is not completion evidence for Windows, the other workflows, authenticated providers, accessibility/IME, idle resources, or final license closure. Do not merge PR #22 or publish a release until the remaining criteria below are verified. See the current issue #9 checklist in [the implementation plan](07-implementation-plan.md).

Historical audit baseline: 2026-10-10 KST. The detailed table below began as an audit of requirements through R-59/DD-65; rows are updated where current evidence is now authoritative, while remaining work is explicit. See [requirements reconciliation](../knowledge/wiki/requirements-reconciliation-2026-10-11.md) for the reviewed historical baseline.

| Requirement | Current authoritative evidence | Remaining acceptance work |
|---|---|---|
| R-01 Native authentication | Native Claude/Codex metadata; installed Codex 0.162.0 ChatGPT-authenticated stdin/JSONL generation and exact-ID context resume with API-key environment variables removed | Claude login and OpenCode configuration; native implementation and all-provider task execution |
| R-02 Subprocess control | Process groups/raw spools, exit/timeout/recovery tests, live bounded normalized SQLite attempt events and fail-fast malformed-stream tests | Native all-provider progress fixtures and sustained resource bounds |
| R-03 Registered providers | Templates, custom contract, parser/argv tests | Real provider contract fixtures and end-to-end registration/execution |
| R-04 Planning | Planner mock tests and plan validation | Goal -> native planner -> registered agents -> verified result end-to-end |
| R-05 Isolation/parallelism | Shared/worktree mock E2E; leases/checkpoints and transitive dependency refresh | Native concurrency evidence |
| R-06 Evidence | Git/test integration, snapshots, reviewed rework, retained workspaces, bounded typed coordinator proposals; distinct persisted Unix signal/exit outcomes | Normalized summaries/failure inventory and native review contract evidence |
| R-07 Approval | Exact candidate binding and mock E2E | Full negative tests for target/index/diff/test changes; native execution review |
| R-08 Dangerous actions | Typed SQLite approval ledger, opt-in Claude worker binding and human CLI controls, compiled shared/worktree approval/expiry/rejection/stop/staleness tests, owned verifier and async cancellation/alias/replay/closed-channel negatives, native metadata initialization | Actual native denial/approval/failure containment for every provider; ignored/external command dependencies and OS containment; profile labels alone do not prove containment |
| R-09 Lifecycle | Quota, CLI stop/resume/revise/review, leases, checkpoints, crash recovery and rework tests | Native contract evidence and interrupted native review recovery |
| R-10 Memory | Outbox/offline recall tests, live intermediate summaries, reviewed task-scoped runtime lesson handoff with current source hashes, development saves | Native lesson service integration, confidence-aware retrieval, cross-run/project context and native session proof |
| R-11 Wiki | English raw/schema/wiki files, reviewed runtime assets with source hashes/staleness and derived pages/index/log; mirror alias/failure tests | Native ingestion/review flow, source revalidation UI and all asset integration evidence |
| R-12 Rust/lightweight | Exact source package `7289562`: one 17,925,856-byte macOS executable, bundled SQLite, 214 dependency records; no project-owned JS/Python source files remain in the current tree | Exact final binary after remaining work; Windows package and cross-platform idle resource measurements |
| R-13 Workflow/TDD | Architecture trace documents and Red/Green tests; full native-feature suite: 768 passed, 14 ignored across 260 suites; latest grouped dialog regressions: 10 passed across 4 suites; Clippy, fmt, diff, documentation/layout and 300-line checks pass | Requirement-by-requirement final acceptance audit and evidence; do not close early |
| R-14 Development tools | RTK and Ponytail used; Graphify refreshed to 7,456 nodes / 11,822 edges / 563 communities on 2026-10-20 | CocoIndex Code is installed but currently fails to start because `mcp.server.fastmcp` is missing from its Python environment; use it when repaired and retain evidence |
| R-15 Distribution licenses | Exact macOS package manifest records 214 dependencies; package validation and license bundle checks pass | Recheck final dependency closure and every bundled/optional boundary for both platforms |
| R-16 English/public docs | Current English documents and public README | Keep status accurate and exclude internal decisions from README |
| R-17 UI | Eframe-native source surfaces for 25/25 workflows; 2/25 packaged end-to-end workflows accepted; source-level interaction tests do not add to packaged acceptance; direct native transport with no UI HTTP listener | 23 packaged workflows; exact current GUI parity/screenshots; Windows rendering; full accessibility and Korean IME; resource measurements |
| R-18 Availability | Parsing, schedule, bounded retries, watch tests | Native restriction fixtures, crash-safe schedule ownership and runtime UI |
| R-19 Multi-session transfer | Mock concurrency/reassignment, native session exclusion and refreshed dependency context; installed Codex two concurrent independent contexts with exact-ID resumes | Claude/OpenCode/custom native concurrency, reassignment and interrupted task evidence |
| R-20 Session models | Native Claude/Codex catalogs; model edit tests | OpenCode/self-hosted catalogs, native model changes and UI controls |

| R-21 Native startup | Private revision-bound registry, read-only planner, three-task review, exact consent/baseline lease tests; actual native fixture setup through merge | Provider-authenticated planning, dialog accessibility and final packaging |

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

## New requirement acceptance (R-22–29)

| Requirement | Current evidence | Acceptance |
| --- | --- | --- |
| R-22 quota | Codex authenticated stdio/native refresh; normalized window tests | Partial: Claude capture/resources remain |
| R-23 Windows continuity | CI 38039813985 on 47eb2d7 passed native Windows build, applicable tests, Job containment, WebView2 smoke, layout and Clippy | Partial: packaged runtime and full native workflow acceptance |
| R-24 platform releases | Exact macOS issue-head package from `7419557`: manifest, 214 dependencies, SBOM, checksum, packaged `--help`, archive CRC and four approval-gated smoke combinations pass | Windows exact-head package/UI, complete packaged UI acceptance, final license audit; do not release before all acceptance gates pass |
| R-25 subscription-independent supervision | Deterministic execution loop; optional structured review | Partial: idle policies/designated AI lifecycle |
| R-26 per-agent fallback | Eight tests; registered policy/model UI implemented | Partial: native policy interaction/chain restart acceptance |
| R-27 cooperative memory steering | UC-21/DD-29; ledger/REST and daemon fixture; six worker tests cover bootstrap, polling, quiescent exits, unresolved directives, killed-coordinator pause/handoff recovery and explicit resume | Partial: automatic memory discovery/import, worker authentication, company LLM acceptance, interrupted pending/accepted instruction continuation and automatic reviewed-target handoff |
| R-28 status briefing | Six tests; CLI/native factual query; actual three-session read-only UI; exact-ID agentmemory progress import verified against the configured daemon | Partial: automatic live worker report collection, active-provider and expired approval acceptance |
| R-29 status lights | Actual three-state native pulse and Stop/cleanup; mapping tests | Partial: live reduced-motion/resource acceptance |
