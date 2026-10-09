# Reviewed runtime lesson assets

Verified: 2026-10-10 KST. Scope: local compiled/offline workflow; native all-provider behavior remains separate.

## Sources

| Local source | SHA-256 |
|---|---|
| `src/knowledge.rs` | `d5a7c032409bbf38f9619889d8a73d517319ca9741518e63c24c0247a46ac973` |
| `src/workflow.rs` | `b8e6f14580c106105de2b2492c07d83af4c00a18bab41e804922b48512b7e5ef` |
| `src/state.rs` | `367d781281505b8ee4c8950e39b9f43c3f8be89046e7880d79d8c5a40fcb177f` |
| `tests/knowledge.rs` | `e012403dc61809c3fd643f755ed416f75a664542b15c235460b035d9c58a8fd1` |

Changed hashes make claims stale. See [DD-21](../../docs/06-design-decisions.md), [knowledge schema](../schema.md) and [completion audit](../../docs/09-completion-audit.md).

## Decisions and evidence

A typed draft contains task/context/failure/cause/response/evidence/confidence/revalidation trigger and ordinary repository source paths with content hashes. Exact digest promotion requires an idle coordinator and valid paused workspace. Agent success alone cannot promote a claim. SQLite stores reviewed assets and acceptance timestamps; wiki pages, index and append-only log are replicas. Re-promotion repairs mirrors without duplicating acceptance log entries.

Before worker handoff, source hashes are revalidated and only fresh lessons scoped to that task are passed, bounded to 8 KiB and explicitly treated as evidence, never permission. Hashing uses a cached 8 MiB budget, at most eight sources per lesson and 1 MiB per regular file. Sensitive/configuration paths and aliases are refused. Changed/missing/unsupported sources are excluded as stale; bounded checks do not freeze files or establish OS containment.

Compiled tests cover exact command digest/acceptance, wrong consent, offline next-attempt prompt inclusion, exclusion from other tasks, changed-source staleness, unsupported paths and wiki-directory aliases. An aliased replica refuses writes while preserving the committed reviewed SQLite asset; its error explicitly distinguishes authoritative save from failed replica materialization. Existing explicit local agentmemory delivery carries reviewed summary content through the outbox; no daemon is required for local context.

## Remaining acceptance

Confidence-aware native lesson service integration, cross-run/project retrieval, revalidation UI and native session handoff require additional proof. The current task-scoped local lesson path and explicit memory replica do not claim these are complete.

Regression: 134 passed, two ignored, 36 suites in 51.90 seconds. Clippy/format/whitespace pass. Graphify: 377 nodes/1,199 edges; CocoIndex located worker context injection. No runtime dependency added.
