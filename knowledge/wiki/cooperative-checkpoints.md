# Cooperative checkpoints

Trace: R-27 → BG-11/BG-12 → SF-24 → UC-21 → QAS-22 → DD-29.

## What is implemented

Strict local SQLite directives bind persistent run namespace, plan digest, task, assigned agent, native session and attempt. Operator issuance compares the reviewed workflow revision in the same immediate transaction as audit/outbox writes. Worker poll/ack/report CLI commands use bounded strict JSON files; no UI server or model provider request is required.

Only one outstanding directive is allowed per task attempt. Exact duplicate acknowledgments are idempotent; conflicting duplicates fail. Expired pending requests cannot be accepted. Accepted ownership remains outstanding past the original deadline until completion/rejection, or supersession by an explicitly reviewed new binding. Supersession invalidates instruction evidence; it neither stops a process nor grants a replacement worker permission.

Monotonic checkpoint progress is an observation consumed by factual briefing, with explicit local adapter source. It cannot change task status, approve tools or merge. The existing agentmemory outbox records bounded protocol envelopes for explicit replication. Recalled replica text must be reconciled with the local ledger because historical/duplicate envelope versions can coexist.

## Verification and lessons

TDD first failed for the missing cooperative module and CLI commands. A behavioral regression then exposed rollback of expiry when returning a late-acknowledgment error; expiry now commits before returning rejection. Another regression reproduced an accepted old-plan directive permanently blocking new reviewed requests; the old record is now superseded without reusing its acknowledgment.

Eight cooperative tests cover addressed lifecycle and restart, conflicting duplicates, expiry, unknown packet fields, stale revision/plan, monotonic and future progress, concurrent issuance, compiled CLI commands/packet bounds, separate namespaces for identical runs, and reviewed plan supersession. Six briefing and two memory tests passed together with them. These are local contract/fixture checks, not authenticated company-provider integration proof.

## Remaining integration

Manual exact-ID publication/read/import is now verified against an existing daemon; see [memory transport](memory-transport.md). Automatic worker polling, individual worker authentication, profile capability declaration, prompt/bootstrap instructions, voluntary company CLI polling during a live work unit, and scheduler handoff/quiescence enforcement remain open. Local OS account/sandbox is the current trust boundary; an address is not an authentication secret. No new dependency or API-key integration was added. Native Windows acceptance and release remain separate unfinished requirements.

## Sources at the local-ledger checkpoint

| Local source | SHA-256 |
| --- | --- |
| `src/cooperative.rs` | `7bf5767fdcd70e303e5c4a120ccfe29255dbf1086121f6252c2ee5e84464cdc6` |
| `src/state.rs` | `3d535dbba647705f3e057f87b649f3a8fadb01097a8d62a60477a83f44b7c652` |
| `src/main.rs` | `307564e448251ba705d7932922931985b881fcbe12db8ac4db9f06d58b9f8764` |
| `tests/cooperative.rs` | `4c5ee1eb7dbc67730847f27f293e3d9dd2348cd18e6bb48238dd81ab2df93b27` |
| `docs/11-cooperative-checkpoints.md` | `250778948cad3890746884f7022be78ea18fb149af16ccd9b51c72eec8f2fae9` |

Final full regression: 177 passed, 4 ignored, 0 failed; 46 suites in 107.05 seconds. Logs retained under `.agentconductor/verification/cooperative-ledger/`. Clippy/format/diff checks passed; Graphify 1,216 nodes / 2,847 edges, CocoIndex located the report ingress. No native window was opened for this backend checkpoint.
