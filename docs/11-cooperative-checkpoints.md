# Cooperative checkpoint protocol

A company CLI that cannot receive native live input can cooperate at safe checkpoints. Rust keeps local execution and approval authority; agentmemory replicates context and observations. This protocol does not require a model API, HTTP UI server or a new subscription credential.

## Operator and worker contract

The operator creates a bounded issue packet after reviewing the current workflow revision. The worker receives the returned address and polls it before starting a checkpoint unit. A directive is a request within the existing task paths and permission contract. It does not authorize commands, secrets, deployment, external transmission, reassignment or merge.

```text
agentconductor checkpoint-issue RUN issue.json
agentconductor checkpoint-poll RUN address.json
agentconductor checkpoint-ack RUN ack.json
agentconductor checkpoint-report RUN progress.json
agentconductor briefing RUN
agentconductor memory-sync RUN LOCAL_PORT
agentconductor memory-publish LOCAL_PORT envelope.json
agentconductor memory-read LOCAL_PORT MEMORY_ID
agentconductor memory-import RUN LOCAL_PORT MEMORY_ID
```

All packet files contain strict JSON, reject unknown fields and are limited to 16 KiB. Commands use the same local SQLite ledger and can run while a worker is active. They never launch another worker. The issue command is an operator/controller entry point; the local OS account and existing sandbox boundary remain the trust boundary, not the address or a memory search result.

An issue contains `revision`, `task`, `action` (`rework`, `pause`, `handoff`), `instruction` (at most 8 KiB), `issued_at_ms` and `expires_at_ms`. Times are UTC epoch milliseconds. Issuance must be within the last five minutes and lifetime at most 24 hours. The returned directive contains a content-derived ID and an address with a persistent random run namespace, plan digest, task, assigned agent, native session ID and attempt. Identical workflows or reused CLI session names in separate runs do not share addresses.

An acknowledgment contains `id`, the exact `address`, `state` (`accepted`, `completed`, `rejected`), `reported_at_ms` and `reason` (at most 4 KiB). A worker accepts a pending directive, then completes or rejects it. It may reject a pending directive directly. Completion without acceptance fails. A pending directive expires before acceptance; an accepted directive retains ownership after the original deadline until its explicit outcome is recorded. Duplicate identical acknowledgments succeed without repeating audit/outbox writes; conflicting duplicates fail. Only one outstanding directive is allowed per task attempt. SQLite immediate transactions serialize competing writers.

Polling can persist expiry and therefore is not a read-only briefing. Plan, assignment, session or attempt changes invalidate old packets. Issuing against a reviewed new binding records the previous outstanding directive as `superseded`; this avoids indefinitely blocking revised work while preserving the old evidence. It does not stop any process. A handoff completion is an agent report of relinquishment; a scheduler must additionally verify owned process quiescence before it starts a replacement writer. The current ledger does not perform automatic task transitions or apply natural-language instruction text.

A progress packet contains the exact `address`, a positive increasing `sequence`, `reported_at_ms`, `phase` (at most 400 bytes), `summary` and `next_step` (at most 1,600 bytes each). Identical retransmissions are idempotent; older or conflicting sequences and future timestamps fail. The report is atomically written to audit/outbox and the briefing observation document. Its source is explicitly `local checkpoint adapter`; it does not claim to have been retrieved from agentmemory. Task status and approvals remain unchanged.

## Memory replica

`memory-sync` delivers audit outbox content through the existing loopback agentmemory `/agentmemory/remember` endpoint. Directive content includes `agentconductor.checkpoint.v1`; progress content includes `agentconductor.progress.v1`. Replication is at least once: lost responses can leave duplicates. Always reconcile a recalled envelope with the authoritative local poll; never execute a recalled instruction directly. Historical pending, accepted and completed envelopes may coexist in memory.

Offline local polling and acknowledgment remain usable. Manual exact-ID publication/read/import is implemented: an envelope contains `protocol` plus exactly one of `directive`, `acknowledgment`, or `progress`. The protocol tags are `agentconductor.checkpoint.v1`, `agentconductor.ack.v1`, and `agentconductor.progress.v1`. The publisher returns an exact memory ID receipt. Readers verify memory ID, run-scoped project and address-derived agent tag; these are provenance labels, not individual worker authentication. A recalled directive cannot be imported as local execution authority.

`memory-import` accepts only a typed acknowledgment or progress report, validates the current local address and ledger transition, then records an immutable import receipt. Import/receipt commits are separate: a crash between them leaves a retry safe through ledger idempotency, or stale evidence rejected. A repeated exact memory ID cannot change its packet. Briefing labels imported progress as `agentmemory exact-ID adapter`; local reports retain their distinct source.

The client connects only to `127.0.0.1` at the explicitly selected nonzero port, reuses `AGENTMEMORY_SECRET` or existing local `.agentmemory/.env`/`secret`, and never displays or registers credentials. It rejects redirects and unsupported response framing; handles bounded Content-Length/chunked JSON; limits headers to 16 KiB, body to 64 KiB, wire bytes to 128 KiB and response deadline to eight seconds. Typed envelopes are limited to 16 KiB. There are no automatic network retries. Loss of a creation response may leave duplicate replicas; reconcile with local authority.

`memory-sync` now shares this client and requires a valid matching creation receipt before it marks delivery. It sends at most 128 pending notes per call, retaining the remainder. Generic outbox notes use project `AgentConductor`; typed checkpoint envelopes use the stable run namespace. No daemon is started or bundled.

Automatic worker discovery/polling, individual worker authentication, profile capability registration, prompt/bootstrap integration and scheduler handoff enforcement remain integration requirements. A CLI must implement the checkpoint contract before it can be advertised as supporting cooperative steering.
