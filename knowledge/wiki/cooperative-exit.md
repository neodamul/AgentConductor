# Cooperative exit reconciliation

Trace: R-27 → BG-11/BG-12 → SF-24 → UC-21 → QAS-22 → DD-29.

## Failure and correction

The Red fixture completed a pause acknowledgment and exited cleanly with a provider
result. Rust incorrectly marked partial work completed and produced a merge candidate.
Directive completion must be distinguished from task completion.

Rust now verifies owned process/group quiescence and the exact local addressed ledger
after clean exit. Completed pause/handoff produces a stopped attempt/run with preserved
partial work and explicit resume required. No tested integration candidate or merge
permission is created. Pending or accepted directives at exit block completion; expiry
still applies only before acceptance. Completed relinquishment cannot be replaced by a
new directive in the same attempt. Rust never selects a new agent from free text.

## Verification scope

Compiled CLI fixtures keep the worker alive after completed relinquishment acknowledgment:
kernel ownership remains present and reassignment is refused. After its actual exit,
partial files are retained, the run is stopped, due scheduling leaves its revision
unchanged and merge is refused. Explicit resume creates attempt 2, preserves the native
session and invalidates the old address, then requires fresh verification/approval.
Pending and accepted directives at clean exit separately produce blocked outcomes.
The broader worker suite continues to cover shared/worktree rework and foreign identities.

These are controlled non-steerable processes, not authenticated company LLM execution.
No daemon/UI or Cargo dependency is added. Individual worker authentication, automatic
memory collection, reviewed-target automatic handoff and interrupted cooperative recovery
remain unfinished. Native Windows/release acceptance remains open.

Full regression: **193 passed, 5 ignored, 0 failed**, 48 suites, 67.02 seconds summed suite time (not wall time). Clippy all-targets with warnings denied, formatting and whitespace passed. Evidence: `.agentconductor/verification/cooperative-exit/`. Graphify: 1,319 nodes / 3,099 edges without LLM; CocoIndex located exit_directive. Lesson `lsn_d9c15a15246342f9` records the separate completion/exit boundaries. Preceding CI 38027108692 on c4f6291 completed: macOS success, Windows build failure (E0425 and four platform warnings). Ponytail has no exposed callable capability; no execution claimed.

## Source fingerprints

Historical fingerprints at exit checkpoint `7a9e022`. The later
[recovery fixture](cooperative-recovery.md) extends test coverage without a runtime change.

| Source | SHA-256 |
| --- | --- |
| `src/cooperative.rs` | `f0affcf5c6ffc9ea057a14b882fd10bf9242073ba2320684d7ee1e6e6b193b76` |
| `src/workflow.rs` | `55823ffd3c64714deb5d08f294d5986848956177f0ae72a87ef677b7bb92cea8` |
| `tests/cooperative_worker.rs` | `a890270c926a6a0fec7831c947d4c34c54f057bd621a50dbd71f689f65d07fb5` |
| `tests/support/cooperative_worker.py` | `2710c6d81443cbee454134df47545f28c5d0969b2b641e0699ed10adac939c59` |
| `docs/11-cooperative-checkpoints.md` | `297b84c1aacea3d700986422dc3c60852b8b849e5ab8c38d8e98d6644d572c3e` |
