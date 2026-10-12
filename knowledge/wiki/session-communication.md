# Session communication, durable state and knowledge

Verified: 2026-10-12. Local evidence: [source observation](../raw/2026-10-12-session-communication.json).
Trace: [UC-21](../../docs/04-usecases.md), [DD-08 / DD-29](../../docs/06-design-decisions.md), [checkpoint contract](../../docs/11-cooperative-checkpoints.md).

## Terminology and external evidence

A CLI is an execution/control interface to an agent runtime. Its process output is not automatically a peer-message endpoint. AgentConductor adapts individual CLI sessions and mediates task delegation, context and results.

[A2A](https://a2a-protocol.org/v1.0.1/specification/) specifies task, message, artifact and status exchange. It does not provide the project's reviewed lessons, wiki or execution approval ledger. A session context identifier is a routing/grouping identifier, not shared durable knowledge.

[Deep Agents subagents](https://docs.langchain.com/oss/python/deepagents/subagents) use delegation tools; that alone does not establish A2A interoperability. Its [persistent memory](https://docs.langchain.com/oss/python/deepagents/memory) is a separate filesystem-backed facility. Their separation informs this design; neither Python framework nor SDK is needed in the Rust executable.

## Current implementation

| Responsibility | Current source of truth | agentmemory role |
|---|---|---|
| Task/session state, permissions, approvals and recovery | Local SQLite and owned process evidence | A replica cannot authorize execution |
| Addressed cooperative instructions and acknowledgment | Local checkpoint ledger | Optional typed publication/import |
| Intermediate task reports | Local monotonic progress records | Optional observation sharing |
| Reviewed runtime lessons and maintained knowledge | Local lesson records and Markdown wiki | Optional recall/sharing |

The checkpoint protocol already supports local poll/ack/report without a running agentmemory server. Bindings include run namespace, plan digest, task, assigned agent, session and attempt. Duplicate acknowledgment is idempotent; conflicting or stale bindings are refused. Reported completion is reconciled with process quiescence before a replacement writer can start.

The current memory client uses an explicitly configured loopback endpoint. That is not automatic cross-machine replication. Knowledge indexing is rebuildable; it does not own runtime state or approvals.

## Recommendation: optional memory, durable local messaging

Keep the Rust engine as the router and SQLite as the durable authority. Pass only the needed context and artifact references to each CLI session; store large output in bounded files. Keep reviewed, source-linked knowledge in the LLM-wiki. agentmemory can remain an optional adapter for context/lesson sharing and discovery.

Local stdio, checkpoint commands or an explicitly designed local IPC binding can carry messages. Copying A2A concepts into a JSON envelope is not sufficient to claim A2A compliance. A standards adapter would need its own capability, transport and interoperability verification.

## Direct instructions from the orchestrator

The orchestrator can originate addressed instructions itself; agentmemory hooks are not required to create them. An adapter must deliver each instruction through a channel that the target runtime actually accepts. Persist the instruction before attempting delivery and record the runtime acknowledgment separately.

Current behavior: ordinary process execution writes the initial task to stdin and then closes it. Provider resume selectors can deliver subsequent work through another invocation where supported. Claude control callbacks handle registered tool permission exchanges; that does not establish generic live prompt steering. The local cooperative checkpoint protocol can carry addressed directions to a worker that implements polling.

Proposed live direction support is capability-specific. A supported runtime control channel could accept a new turn or an interrupt-and-follow-up operation. A cooperative wrapper may deliver an instruction at a tool/hook boundary. Neither mechanism guarantees mid-operation preemption. A runtime exposing neither requires a safe, quiescent transition to a resumed/new attempt with bounded handoff context. A session ID addresses context; it does not by itself create an input channel to an already running process.

The Rust router and durable inbox can therefore replace signal storage/routing while a CLI adapter or cooperative boundary replaces hook-based delivery. No universal steering capability is claimed for Claude, Codex, OpenCode or aipro; each concrete invocation and installed version needs a verified capability contract. This is an extension proposal, not a change to the current runtime.

## Can signals be replaced?

Yes. A self-managed durable inbox can provide the equivalent coordination function of agentmemory signal send/read. The existing checkpoint ledger covers scoped task directions; it is not yet a general-purpose peer signal service.

A proposed signal record contains message ID, sender and recipient session bindings, type, correlation/reply ID, bounded content or artifact references, sequence, creation time and optional expiry. Delivery states distinguish pending, delivered, acknowledged, rejected, expired and superseded. Read progress is separate from acceptance and from execution success.

Persist the outgoing record before transmission. Use an outbox and recipient deduplication for retry-safe delivery; a lost response can cause retransmission, so do not promise exactly-once transport. Acknowledgments must bind to the exact message and current session. Stale retries must not retarget a replacement session or resurrect old instructions. Message identity/routing is not authentication; keep the existing OS/account boundary for local use and authenticate remote peers separately.

Delivery adapters must honor actual CLI capabilities:

- Native live control: use only the CLI's supported input/control channel.
- Cooperative, non-steerable CLI: fetch addressed instructions at safe checkpoints, acknowledge them, and report the resulting action.
- CLI without either capability: retain the message for an explicitly resumed/new attempt with reviewed handoff context; arbitrary text written to stdin is not reliable steering.

A remote machine needs a configured authenticated transport or explicit import/export. Shared SQLite files across machines are not the recommended message bus. No peer may grant tool, secret, deployment or merge approval by sending a signal. Rust retains the authority and human approval gates.

## Required selectable fallback (R-92 / DD-100)

The operator now explicitly requires Settings choices for Local Rust and agentmemory preferred, even when agentmemory is healthy. Local Rust is the default for unconfigured workspaces. A preferred agentmemory server or memory/signal MCP failure triggers Rust delivery for eligible local recipients, with stable message identity, durable pending records, bounded timeout/backoff and late-response deduplication. Remote recipients without a valid route remain pending; unrelated MCP operations are outside this fallback.

This is a recorded requirement, not a shipped setting or transport. Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100. See [implementation checklist](../../docs/07-implementation-plan.md).

## Proposed verification before general signal support

- Restart/crash recovery before send, after send and before acknowledgment.
- Duplicate delivery, conflicting payloads, expiry, ordering and wrong-session rejection.
- Safe checkpoint delivery to a real non-steerable CLI; refusal to claim receipt merely from process output.
- Quiescent handoff and preservation of approval boundaries.
- Bounded payload, queue growth, polling and resource use; authenticated remote delivery if enabled.

These are design criteria, not completed implementation claims. This documentation change does not add a generic mailbox, A2A server, remote transport or new runtime dependency.

## Lesson and revalidation

Context: CLI cooperation was conflated with shared memory. Response: separate message delivery, durable execution state and reviewed knowledge; make external memory optional. Confidence: high for the architectural distinction and the observed local checkpoint contract. Revalidate when CLI control capabilities, checkpoint bindings, memory transport or remote trust boundaries change. Code evidence is pinned by revision and hashes in the source observation; it must be refreshed after relevant implementation changes.
