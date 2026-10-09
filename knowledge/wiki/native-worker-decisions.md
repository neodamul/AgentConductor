# Worker-bound human decisions

Verified: 2026-10-10 KST. Status: compiled mock CLI contracts; native tool enforcement remains open.

## Sources

| Local source | SHA-256 |
|---|---|
| `src/native_guard.rs` | `6920c7aa93ff232e15d2424483999ec7c537e4ae663b99059472d360457d66b1` |
| `src/workflow.rs` | `b8e6f14580c106105de2b2492c07d83af4c00a18bab41e804922b48512b7e5ef` |
| `src/claude_control.rs` | `9730ebf82e2435ec538835230d470ec145348db77057907f53318a3ab38646a7` |
| `src/gates.rs` | `4d2053ed70693626aaf73079361e92b636c75c6a83e0f0a0c7d0e18594675ed2` |
| `tests/native_worker.rs` | `a87f271034ce174843f43767bee3440c40d9da19c90b854bc25a0bd7f0ca378a` |
| `tests/claude_control.rs` | `4377d0c809c77df6796aabdd442ecc785aaf27326fe16a7b9daea3f8f650aa1c` |
| `tests/gates.rs` | `18fc551dd601aa724119a9e06bb3cd62f7c709d4dc58ac265da8df46f94a01a7` |

Changed source hashes make verification claims stale. See [DD-19](../../docs/06-design-decisions.md), [context verification](tool-context-verification.md), and [completion audit](../../docs/09-completion-audit.md).

## Decisions and evidence

Reserve and persist a random native UUID before launch; resume only after matching session evidence. Validate foreign/duplicate callbacks before evaluating or consuming consent. Capture human requests after parallel workers drain, then retry an approved task alone after ready dependencies. Exact-ID receipts remain addressable beyond the bounded history listing and reject payload tampering.

Explicit refresh increments the persisted epoch before a new pending receipt is issued. It never renews approval. Feedback/reassignment invalidate epochs; explicit rejection remains blocked across automatic resume. Consent while stopped preserves the stopped run, requiring explicit resume. Final merge approval is independent. Compiled CLI tests exercise shared and optional worktree modes, expiry, refreshed pending consent, rejection/revision, stop/resume, paused workspace changes and single consumption.

## Limits and revalidation

These tests use a mock provider. Metadata-only native initialization establishes wire compatibility without an authenticated user turn. Ignored/external Bash dependencies are not fully fingerprinted, files are not frozen between callbacks, and the callback policy does not provide OS filesystem/network isolation. Live provider denial/approval, callback failure, containment, concurrency and resume must be verified before the corresponding full-product requirement can close. Older pre-epoch receipts require explicit refresh and fresh consent. Revalidate when native protocol, tool set, session semantics, snapshot policy or scheduler changes.

Regression: 127 passed, two ignored, 34 suites in 49.61 seconds. Clippy/format/whitespace checks pass. Native metadata-only initialization passed separately in 0.90 seconds. Graphify: 341 nodes/1,083 edges; CocoIndex located receipt lookups. No new runtime dependency.
