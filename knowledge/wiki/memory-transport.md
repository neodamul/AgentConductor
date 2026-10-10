# agentmemory checkpoint transport

Trace: R-27 → BG-11/BG-12 → SF-24 → UC-21 → QAS-22 → DD-29; R-28 briefing observations.

## Implemented contract

The Rust client connects to an explicitly selected loopback port on an existing daemon. It creates typed checkpoint envelopes and returns verified memory ID receipts, reads exact IDs with matching run project/address tag, and imports only acknowledgments or progress into local authority. It cannot import a recalled directive to create execution permission. Namespace/tag checks are provenance checks, not individual worker authentication.

The client reuses existing daemon authentication, never registers a model key and never starts/bundles a daemon. It sends no provider-generation request. Response framing supports bounded Content-Length/chunked JSON; redirects, malformed/ambiguous framing, oversize, truncation and wrong identities fail. Header/body/wire bounds are 16/64/128 KiB; response deadline is eight seconds. No automatic retry is introduced. Imported observations retain their actual source in factual briefings.

Legacy outbox sync shares the same transport. It requires a matching creation ID/content/project receipt before marking delivery, reads at most 128 pending notes, and retains the remainder or a failed delivery. Import ledger and immutable receipt commits are separate; retries reconcile with ledger idempotency or reject stale evidence after interruption.

## TDD and verification

Red stages: missing transport/import APIs and compiled CLI commands; closed-before-headers connection spun for eight seconds; bodyless 201 falsely marked delivery; pending query returned all 140 rows instead of a 128-row batch. Green stages reject EOF promptly, validate receipts, preserve failed outbox records and retain remaining batches.

Five transport tests cover typed publish/read, authorization header handling, foreign project/identity, redirects, oversized/truncated/ambiguous replies, fragmented chunked JSON, bodyless creation failure, prompt EOF and compiled CLI commands. Cooperative tests verify exact ack/progress imports, repeated imports, truthful briefing source, remote directive rejection and compiled import without workflow changes. Authentication file precedence/bounds are covered with private fixtures.

Final regression: **188 passed, 5 ignored, 0 failed**, 47 suites, 105.23 seconds. Clippy all-targets with warnings denied and format/diff checks passed. A separately invoked ignored test used the existing port-3111 daemon with a controlled workflow fixture; typed directive/ack/progress publication and exact reads/imports, duplicate rejection, unchanged workflow authority and confirmed outbox sync passed. This is actual daemon interoperation, not actual company CLI execution.

Retained fixture run namespace: `06b1465da47287ed055deed247dbd3fa`; exact IDs `mem_mv1xgwyf_b1cae0b653de`, `mem_mv1xgwz3_54afe4f5da47`, `mem_mv1xgwzq_6441f9c37d67`. Logs/metadata under `.agentconductor/verification/memory-transport/`. No new window or daemon was started. Graphify: 1,280 nodes / 3,021 edges without LLM; CocoIndex located remember_note.

## Upstream and licenses

REST handlers were inspected at [agentmemory v0.9.30](https://github.com/rohitg00/agentmemory/blob/v0.9.30/src/triggers/api.ts). They create memories with a receipt and expose exact-ID reads. The implementation uses these API contracts; no server code or engine is bundled.

[Version-specific LICENSE](https://github.com/rohitg00/agentmemory/blob/v0.9.30/LICENSE) is Apache-2.0; Git blob `cb13c72881997c8ad1662ec42c1484cc15859147`, SHA-256 `76c8d49ab42216a2533f603fbafa20a1bf71b56de136a401be52da034dcc012c`. The existing separate iii/Elastic review and release dependency notice gate remain open. No Cargo dependency was added.

## Remaining integration

Explicit custom capability and worker prompt/bootstrap were subsequently verified in the
[worker bootstrap slice](cooperative-worker-bootstrap.md). Automatic memory discovery/import,
individual worker authentication, company CLI acceptance and scheduler handoff with proven
quiescence remain unfinished. Native Windows and release acceptance remain open. CI
38025340939 ended with macOS success and Windows build failure. This checkpoint is not full-product completion.

## Sources

Historical fingerprints at transport checkpoint `1925d39`; later worker-bootstrap changes
are recorded separately in the linked asset.

| Source | SHA-256 |
| --- | --- |
| `src/memory.rs` | `7a4385a67a3354a30bd1b2136dd5cc6a07aec6a1e0c66fba5de7fbe5baf07a5d` |
| `src/memory/client.rs` | `04ae76abf32b4295f8e70a33a35456d08e2cb6a1722bef54b261cb9186de94f5` |
| `src/memory/checkpoints.rs` | `a36e20f7ca8395bf971d8f931c5ad03189065557e5fc72bfb42f2fd607b3bd39` |
| `src/cooperative.rs` | `2ce5fdfa805a67fe1d4750228cdf2519a0eca364fe3839cb0bd911058fcea244` |
| `src/state.rs` | `74dacc412b7decf2dd490173619942e2bf8f22bbc83f64d9b0008a2c02acef9b` |
| `src/main.rs` | `e66e6f325aad64d5308e64458a98dc3e0ba035ee0b99ab6ebf6ad7900820e9fa` |
| `tests/memory_transport.rs` | `6525fe0ee334f29247f5fec14b73188bdb8ef899cb0dc994fafcd6b2574b396d` |
| `tests/memory.rs` | `33e50d71843ed217fcba07acba55fc61a4b068a62c913dc9e1754af289958c9a` |
| `tests/cooperative.rs` | `13b45d2c4fdb219e9edd59645fc1f579809b37cf56bd9f9e7be0338d3d42651c` |
| `docs/11-cooperative-checkpoints.md` | `bf52c33c05dc647e0615215c2146c367e652847d7fbcdfe48b7e82ea51b4f753` |
