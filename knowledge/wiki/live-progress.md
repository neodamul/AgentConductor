# Live intermediate task reports

Verified: 2026-10-10 KST. Scope: local compiled/mock process contracts.

## Sources

| Local source | SHA-256 |
|---|---|
| `src/state.rs` | `ff9d4aed0bfd5e7a4c3e1d47930dd656f338e8b842c9e0b49b0f5bb4aae0f39d` |
| `src/progress.rs` | `161593f86ffccaded795317e3439a05627739209a1a0da871eff44bd02495c1e` |
| `src/workflow.rs` | `79946ccf65ba1b8694347885d3a9b6a08bce9862bef8f3ba1d2dd55aaccbfca2` |
| `tests/live_progress.rs` | `d012e178db53d1f32dacce9d401d763b8a6bdba2a8b74165eb154ab0b88cbe00` |
| `tests/state.rs` | `ef52fd455d7d26f2d1c2e43445b6159e86fdfd1c91d5899577bb42645e82f7a9` |

Changed source hashes make claims stale. See [DD-20](../../docs/06-design-decisions.md) and [full-product audit](../../docs/09-completion-audit.md).

## Architecture and evidence

An observer owns the existing worker future and incrementally reads its owned stdout spool in 64 KiB batches on a 100 ms timer. Per-attempt normalized events retain sequence/raw offset/native session information in SQLite without writing coordinator-owned report state. A failed decoder drops the worker future and its process-group guard. Normal completion drains remaining chunks and the final unterminated JSONL record.

Compiled tests release a worker only after querying its live session/progress events, then verify the final result. Malformed output cancels a sixty-second worker within a three-second test bound, with process absence verified. Replay is idempotent; changed evidence is rejected; attempts remain independent. Queries return recent records with approximately 1 MiB and 1,000-record bounds. Full raw spools remain available. Memory outbox summaries contain task/attempt/kind/sequence only, without provider message contents.

## Lesson: reserve the writer before idempotency reads

Concurrent deferred SQLite transactions can fail upgrading read snapshots to writers despite busy_timeout. A native-worker mock regression exposed database-is-locked during revision/retry. BEGIN IMMEDIATE reserves writer ownership before replay checks; targeted observer/state/native-worker regression passes. Revalidate when transactional append or batch/concurrency policies change. Evidence metadata is an intermediate report, never permission or successful task completion.

## Remaining proof

Native all-provider progress formats, sustained-resource bounds, normalized final summaries/failure inventory, runtime lessons/context handoff, and functional UI remain acceptance work. The observer does not establish vendor containment or native authenticated execution.

Regression: 131 passed, two ignored, 35 suites in 50.14 seconds. Clippy/format/whitespace pass. Graphify: 352 nodes/1,110 edges; CocoIndex verified both worker transports.
