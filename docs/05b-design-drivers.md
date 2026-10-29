# Design drivers and architecture

Method steps 7-8: QAS, constraints and architecturally significant functions are
prioritized into design drivers (DDR). Each driver is satisfied by design decisions in
`06-design-decisions.md`; the architecture view below shows where they land.

## Design drivers

Priority: business importance / technical risk (H, M, L).

| ID | Driver | Type | Inputs | Priority | Design decisions |
|---|---|---|---|---|---|
| DDR-01 | Human authority over every irreversible or sensitive action | Quality (security) | NFR-01, QAS-02, QAS-23, QAS-34, QAS-37, FR-07, FR-09, FR-17, FR-27, FR-29 | H/H | DD-06, DD-17, DD-18, DD-19, DD-32, DD-50, DD-52 |
| DDR-02 | Untrusted agents cannot change the repository outside reviewed scopes | Quality (integrity) | NFR-02, QAS-06, QAS-24, C-05, FR-16 | H/H | DD-05, DD-14, DD-18, DD-33 |
| DDR-03 | Crash-safe, single-writer orchestration state | Quality (recoverability) | NFR-03, NFR-08, QAS-03/04/09/10 | H/H | DD-02, DD-07, DD-11, DD-12, DD-14, DD-24 |
| DDR-04 | Subscription-only, process-level integration with heterogeneous CLIs | Constraint + modifiability | C-01, NFR-06, NFR-07, QAS-05/12 | H/M | DD-03, DD-04, DD-16, DD-17 |
| DDR-05 | Continuity across account exhaustion and memory outages | Quality (availability) | NFR-04, QAS-07/19/22 | H/M | DD-08, DD-28, DD-29 |
| DDR-06 | Lightweight single native executable | Constraint + performance | C-02, NFR-05, QAS-01/08 | M/M | DD-01, DD-10, DD-22, DD-23 |
| DDR-07 | Truthful, accessible operator awareness in the operator's language | Quality (usability) | NFR-09, NFR-10, QAS-17/20/21/25/26/38/39, FR-18, FR-19, FR-30, FR-31 | M/M | DD-20, DD-30, DD-31, DD-34, DD-39, DD-53, DD-54 |
| DDR-08 | Portable, compliant macOS/Windows delivery | Quality (portability) + constraint | NFR-11, NFR-12, QAS-14/18/33, FR-26 | H/H | DD-09, DD-27, DD-49 |
| DDR-09 | Low-friction decisions without regenerated duplicates | Quality (operability) | NFR-13, QAS-23 | H/M | DD-32 |
| DDR-11 | Desktop-first, learnable public documentation | Quality (usability) | FR-21, NFR-14, QAS-28 | M/L | DD-42 |
| DDR-12 | Orchestration does not depend on an AI planner | Quality (availability) + usability | FR-22, FR-32, NFR-15, QAS-29, QAS-40 | M/M | DD-45, DD-55 |
| DDR-13 | Self-explanatory workspace setup | Quality (usability) | FR-23, FR-24, FR-25, FR-33, FR-34, NFR-16, QAS-30, QAS-31, QAS-32, QAS-41, QAS-42 | M/M | DD-46, DD-47, DD-48, DD-56, DD-57 |
| DDR-15 | Team-shared configuration without shared secrets or trust | Quality (security) + collaboration | FR-37, NFR-18, QAS-44 | H/M | DD-59 |
| DDR-16 | Durable, consistent multi-goal work records | Quality (integrity) + usability | FR-40–FR-44, NFR-19, QAS-46–QAS-50 | H/H | DD-61–DD-65 |
| DDR-17 | Extend custom CLI event and quota behavior through reviewable data | Modifiability + availability | FR-139, QAS-150, NFR-06/07 | H/M | DD-03, DD-04, DD-164 |
| DDR-14 (planned) | Remote awareness and decisions without weakening local authority | Quality (security) + integration | FR-28, NFR-17, QAS-35, QAS-36 | H/H | DD-51 (proposed) |
| DDR-10 | Recognizable providers without redistributing their artwork | Quality (usability) + constraint | FR-20, NFR-10, NFR-12, QAS-27 | M/L | DD-40 |

## Architecture

```text
Native UI (WKWebView / Windows WebView)  ──direct native IPC──┐
CLI commands (agentconductor <command>) ─────────────────────┤
                                                              ▼
                 Coordinator (Rust, owns run lock + workspace lease)        [DDR-03]
   ┌──────────────┬───────────────┬───────────────┬────────────────┐
   │ Planner       │ Scheduler      │ Review/Merge   │ Briefing/Quota │
   │ snapshot      │ attempts,      │ candidate,     │ factual bundle │
   │ isolation     │ restriction,   │ tests, exact   │ status lights  │
   │ [DDR-02]      │ fallback       │ approval       │ [DDR-07]       │
   │               │ [DDR-05]       │ [DDR-01]       │                │
   └──────┬────────┴──────┬────────┴───────┬────────┴────────────────┘
          ▼               ▼                ▼
   Process supervisor (owned process groups, spools, signals)            [DDR-04/06]
          │  stdin/stdout JSONL, PreToolUse/can_use_tool control
          ▼
   Native guard: file policy -> automatic | per-call deny | human decision
          │                                   │
          │                     Tool ledger (SQLite, exact request digests) [DDR-01/09]
          ▼
   Provider CLIs (claude, codex, opencode, company CLIs)
          │
   SQLite state/audit/outbox ──► agentmemory (optional)                  [DDR-05]
```

Decision points introduced in this slice:

- **Live tool decisions (DDR-01, DDR-09 → DD-32).** The guard holds a human-gated
  callback open; UI/CLI decide through the ledger while the coordinator keeps the run
  lock; the same attempt consumes the approval once.
- **Planning snapshot (DDR-02 → DD-33).** Every coordinator, including unverifiable
  company CLIs, plans in a disposable tracked-files copy; the target checkpoint must be
  unchanged afterwards.
- **IME-safe forms (DDR-07 → DD-34).** Polling never overwrites focused or composing
  controls.

### Claude quota extension

R-60 / QAS-51 require passive subprocess metadata collection, conservative unknown states and no additional cloud or credential integration.

### Goal import drivers (planned)

R-62 / QAS-53–55 prioritize OpenCode configuration compatibility, company-server portability, read-only context acquisition, source provenance and bounded context. A Jira/Confluence server is operator-supplied; its license and deployment terms must be checked before installation.


### Compact quota presentation driver

R-64 / BG-24 / QAS-57 prioritize complete window visibility and an adjacent refresh target without a detached toolbar button. Preserve provider evidence instead of guessing subscription identity.

### Agent identity and capacity clarity drivers

R-68 / R-69 / QAS-61–62 prioritize a profile that never mixes two CLIs, reference integrity on rename, and reset information already present in quota observations (no additional queries).

### Interface language driver

R-70 / QAS-63 prioritize operator comprehension without touching user content or adding runtime cost, and a single English source of truth for interface strings.


### Driver DD-78 Windows metadata parity
Cross-platform usability must preserve minimum resource use and existing CLI authentication. Avoid a resident terminal server or vendor artwork redistribution. Trace: QAS-65.


### Driver DD-80 Metadata capability boundaries
A compatible serving protocol does not imply its CLI exposes a model-list command. Prefer explicit configuration over inventing a discovery request or changing transport. Trace: QAS-67.


### Driver DD-83 Enterprise quantity semantics
Subscription rolling windows and enterprise allocation are different metrics. Reuse bounded CLI metadata collection and normalize explicit budget fields without inferring organization scope or an unknown total. Trace: QAS-70.


DD-84 responds to R-76 / QAS-71: foreground native selection and console-free Windows desktop/background operation without extra dependencies or services.


### Driver DD-100 Optional-service resilience and transport choice
Communication must remain usable locally when optional memory infrastructure fails. Durable identity, bounded resource use and provider capability boundaries matter more than a transport implementation. Separate message receipt, acceptance and actual execution; preserve remote delivery and approval trust boundaries. Trace: QAS-88.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.

### Driver DD-162 Plan retry timeout and recovery
Planning preparation may include filesystem work before a background job descriptor exists. Keep that startup window bounded consistently across the initiating request and status poll; when it expires, preserve automatic recovery and provide only actionable guidance. Trace: QAS-148.

### Driver DD-165 Local company version label
Company identity belongs in optional local build metadata, not Cargo's semantic version or a published release. Keep the release failure guard at the Rust packaging boundary. Trace: QAS-149.

### Driver DDR-18 Bounded database contention
Concurrent task progress must survive brief writer contention without hiding sustained failures or blocking indefinitely. Use SQLite's bounded busy handling at the shared connection boundary. Trace: QAS-153.
### Driver DD-163 Recoverable timeout with human control
Subprocess timeouts must not erase partial work or make the run look like an unexplained terminal failure. Retry without an LLM, maintain bounded resource use, make progress visible through the existing report channel, and honor a human Stop before relaunch. Trace: QAS-149.

### Driver DDR-20 Observable activity without inference
A quiet task must be explainable from recorded facts only. Reuse the existing task-activity document and owned-process probe, add the current tool to that document, and compute elapsed time in the page. Trace: QAS-155.

### Driver DDR-22 Reuse the read-only coordinator path
Chat must not add a second way to run an agent: reuse the coordinator's read-only invocation and the worker-job slot. Trace: QAS-157.
### Driver DDR-21 Follow progress without overriding the operator
Selection may move only when the operator is watching the session that just stopped. Trace: QAS-156.

### Driver DDR-23 Explicit approval of the exact change
A plan change must be reviewed as data and approved by digest, so what is approved is what is applied. Trace: QAS-158.

### Driver DDR-24 One repair loop for every routine failure
Routine failures must not add a second automation path: reuse the frozen-evidence review and bounded repair loop, and widen only the condition that starts it. Trace: QAS-159.

### Driver DDR-25 Share evidence through the channels that already exist
Do not add a new store or protocol: keep lessons in the run document store, deliver through the memory outbox and the existing prompt context, and show them through the existing task detail. Trace: QAS-160.

### Driver DDR-26 A control that hides information must be called back by urgency
Anything the operator can fold away must reopen when the run needs the operator, and the call must be visible without sound or layout shifts. Trace: QAS-161.

### Driver DDR-27 Decide with checks that cannot hallucinate
A model proposes, a deterministic validator disposes. Trace: QAS-162.

### Driver DDR-28 Reuse the audit trail as the single source
Do not create a second log of judgments: derive the list from the audit records the coordinator already writes. Trace: QAS-163.

### Driver DDR-29 Amend by recorded diff, never by silent edit
Every plan change by the coordinator must carry what changed, why and what it replaced, so it can be shown and reversed. Trace: QAS-165.

### Driver DDR-30 Anchor change to a recorded baseline
Every limit on automatic change must be measured against something fixed, recorded when the run started. Trace: QAS-166.

### Driver DDR-31 Synchronize tests with evidence, not elapsed time
Tests must observe the event or state under test before proceeding; timeouts bound failure detection but do not stand in for readiness. Trace: QAS-167.

### Driver DDR-32 Reuse the review round for operator requests
One coordinator path, one evidence bundle, one validator: an operator request adds text to that round instead of creating a second coordinator workflow. Trace: QAS-168.

### Driver DDR-34 Review guidance instead of new gates
Most mechanical rules are already default Clippy lints denied in CI, and the rest would fail on many existing sites. A short review policy gives the value without churn. Trace: QAS-171.

### Driver DDR-35 Gate only after the count is zero
Enabling the lint first would fail on every existing site and block unrelated work; documenting first and gating last keeps each step small and verifiable. Trace: QAS-172.

### Driver DDR-37 Reuse the chat job instead of a new runtime
The coordinator chat (DD-171) already runs a read-only coordinator as a bounded worker job and returns its artifact in the next snapshot. Reusing that path adds no process type, no new permission and no polling of its own; the bundled skill is only prompt text. Trace: QAS-174.

### Driver DDR-38 JSON Lines over JSON-RPC for v1
Bidirectional control is needed, but plain control lines with acks need no request/response framing, so a shell-level CLI can implement them; JSON-RPC would add id correlation and a transport for no extra capability here. A schema per message keeps both directions checkable. Not evaluated against every agent-client protocol in depth. Trace: QAS-181.

### Driver DDR-39 Distinguish observation from operator action
An automatic snapshot and an explicit user action share the request bridge but have different feedback requirements. Mark only scheduled and visibility-resume polls as background, and pause the same request deadline while the document is hidden instead of adding a second transport or retry policy. Trace: QAS-182.

### Driver DDR-41 Reuse the process supervisor and approval ledger for ACIO
The existing worker owns child lifetime, output capture, persistence, and workspace scope; the existing tool ledger owns exact human decisions. Route opted-in ACIO input/events through those paths instead of adding a second runner, event store, or authorization mechanism. Trace: R-187, QAS-185.

### Driver DDR-42 Keep prompt correction bounded and CLI-independent
Build the requested answer shape in the orchestrator and allow at most one correction attempt when the process succeeds but its output lacks the requested JSON document. Preserve the first attempt's logs, include only the bounded parse reason in the retry prompt, and leave timeout, stop, capacity and provider-error behavior on their existing paths. Do not require CLI changes or add a protocol for prompt correction. Trace: QAS-184.
