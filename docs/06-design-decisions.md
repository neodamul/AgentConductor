# DD: Design Decisions

## DD-01 One Rust package

Rust is the user's chosen language. Start with clap, Tokio(process/io/time/sync),
serde/serde_json, and rusqlite. Invoke installed Git with argv instead of adding libgit2
or building shell strings. Optional reqwest supports HTTP memory integration; no async
trait framework. Verify resolved versions, features, and licenses against Cargo.lock.
Related: SF-01/03/06/09; QAS-05/08/09/11.

## DD-02 Coordinator-owned state

One writer updates SQLite WAL. Planned tables: tasks, edges, runs, events, artifacts,
test_runs, approvals, memory_outbox, lessons, knowledge_sources. Deduplicate by run ID/seq.
Task states: pending, running, waiting_approval, blocked, completed, failed.
Run lifecycle: prepared → ready → starting → running → stopping → stopped/completed/failed;
actual allowed transitions are explicit. start/stop are commands; starting/stopping are states.
Development phases: plan → task → verify → complete. Keep these three axes separate.
Process exit zero ends a run; it does not itself complete a task.
Related: SF-02/04/06; QAS-02/03/04.

## DD-03 Registered profiles

Target AgentProfile fields: id, transport(cli), executable, argv, resume_argv,
input_mode(text_stdin/file), output_mode(json/jsonl), parser, environment overrides,
permission capabilities, timeout policy, allowed scopes. Inject the selected workspace cwd per task.
Custom CLIs use the same contract; verify actual aipro flags before registration.
HTTP is a later optional transport, separate from LLM completion APIs.
Preserve parent authentication environment and HOME; do not register/copy API keys.
If API-key/endpoint overrides exist, report the authentication route without exposing values.
Related: SF-01; QAS-09.

## DD-04 Collect and normalize JSON

Use separate asynchronous tasks for stdin/stdout/stderr. Write text stdin and close it.
Spool stdout bytes to disk before UTF-8/JSON line parsing. Handle split lines, CRLF,
final lines without newline, unknown events, and maximum line size. Drain stderr separately;
a full bounded queue must not leave pipes undrained.
NormalizedEvent: schema_version, run/task/agent/session ID, seq, observed_at,
kind(session/phase/progress/tool/result/usage/error/exit), payload, raw_offset.
Result: summary, evidence[], changed_files[], test evidence refs, artifacts[], exit,
nullable usage. Keep agent claims separate from coordinator-measured Git/test evidence.
Mark internal tool-call audit completeness as partial when CLI output omits calls.
Related: SF-04; QAS-05.

## DD-05 Optional worktrees and integration candidates

Plan workspace_mode defaults to shared; worktree is an explicit option. In worktree mode,
create codex/ac-<goal>-<task> branches/worktrees from a fixed base SHA. Declare ownership
and expected new files; reject overlapping scopes. Worktrees reduce edit conflicts but
are not OS security sandboxes; they share Git metadata. Validate scope after execution;
strong enforcement requires sandbox/permission controls.
Collect status --porcelain -z, staged/unstaged diffs, and untracked files. Confirm agent
termination before candidate commits; raw diff alone omits new files.
Test agents work in parallel from agreed interface/test specifications. Run final tests
requiring implementation against the integration candidate. A shared base does not imply
successful integration.

Shared mode uses the existing checkout with a clean initial baseline and disjoint ownership.
Workers must not commit, reset, checkout, or modify the shared Git index; coordinator Git
mutations and final tests wait until all writers stop. Since final Git diffs cannot reliably
attribute edits to individual workers, verify the combined scope union and mark worker
attribution as unverified. Capture unexpected edits for review. Approval covers the frozen
combined candidate and evidence; changes invalidate it. Working-file edits precede approval;
protected-ref updates and external actions still require their own gate. Shared mode does not
provide branch isolation, rollback per agent, or OS security isolation.
Related: SF-03/04; QAS-01/06.

## DD-06 Human approval

Bind approval to action, repository, base SHA, candidate SHA, diff hash, test hash, scope,
and expiry. Deny merge/push/deploy/non-permitted transmission/secret access before approval.
CLI permission flags have different semantics. If headless execution cannot relay synchronous
approval, deny the tool and stop/report; resume with an approved new attempt.
A stdout tool_call may describe an already executed action; parsing is not a pre-execution gate.
Only the coordinator should own protected-branch changes and deployment credentials.
Custom CLIs without enforceable controls require read-only/sandbox execution or an unsupported
classification. Worktrees alone cannot fully isolate arbitrary code from the host.
Distinguish necessary provider requests from arbitrary task-originated external transmission.
Related: SF-05; QAS-02/06.

## DD-07 Timeout, retry, resume

Proposed defaults: wall timeout 30min; silence 5min triggers warning only;
SIGINT 5s → SIGTERM 5s → SIGKILL/reap. Use POSIX process groups; Windows Job Objects later
include grandchildren. Allow one transient retry with exponential backoff from 2s plus jitter;
429/subscription exhaustion enters cooldown or blocked. Do not retry authentication,
permission, or invalid-flag failures automatically. Test failures create feedback attempts.
Inspect snapshots/diffs before retrying timed-out partial edits. Resume the saved exact
session ID in the same cwd with a compatible CLI version. Avoid --last/--continue in parallel
automation. Revalidate fixture contracts on CLI upgrades. Related: SF-04/06; QAS-03/04.

## DD-08 Memory and wiki

SQLite is authoritative for execution/approval; Markdown wiki for reviewable knowledge;
agentmemory for shared context, reports, and lesson recall. Keep provider sessions separate,
linked by (goal, task, attempt, agent, provider_session). Pass only briefs, scopes, relevant
wiki/lessons, verification evidence, and failure feedback to the next worker.
Reuse one existing REST/MCP daemon. Structured slots may expose progress but must not own
scheduler locks or approvals. Delivery: SQLite transaction → outbox → memory_save/lesson.
Use the actual canonical session project ID. Records without agentId are shared; scoped
records use agentId. Server whitelists may discard extra fields; use content envelopes and
local mappings. Delivery is at-least-once; external-ID mappings suppress duplicate display.
Begin keyless search with BM25. Explicitly enable local embeddings only after measuring
resources; upstream and installed-skill defaults differ, so verify pinned-version behavior.
For availability or license issues, implement the same memory port with SQLite FTS5/Markdown.
Related: SF-07; QAS-07/10/11.

## DD-09 Development tools

RTK compresses human-readable build/test/search output; never filter raw agent JSON.
Ponytail favors existing code and standard functions while preserving error handling,
security, and required verification. Graphify code-only AST/Cargo relations help assess
ownership/impact; do not auto-select API-key backends. Start CocoIndex Code with ccc grep;
use local embeddings selectively. Identify derived indexes by base/branch/hash; worktrees
must not overwrite one mutable index. Share coordinator search results rather than launching
three resident models/indexers. Python sidecars are optional development aids, not core
runtime dependencies. Related: SF-07/09; QAS-08/10.

## DD-10 Structure and UI

Target structure (some modules remain unimplemented):

```text
src/
  main.rs, lib.rs
  profile.rs, protocol.rs
  adapters/{mod.rs,claude.rs,codex.rs,opencode.rs,custom.rs}
  process.rs, planner.rs, scheduler.rs, state.rs
  git.rs, verify.rs, approval.rs, memory.rs
tests/
  fixtures/{claude,codex,opencode}/
  process_contract.rs, workflow.rs, approval.rs, recovery.rs
  support/mock_agent.rs
migrations/
docs/01..07-*.md
knowledge/{raw/,schema.md,wiki/}
.agentconductor/                 # ignored runtime state
  state.sqlite, worktrees/, runs/<run-id>/
    prompt.txt, stdout.jsonl, stderr.log, events.jsonl
    result.json, diff.patch, tests.json
```

The interface concept is a terminal-style console: editable goal/plan, selectable tasks,
feedback input, pause/resume/revise commands, workspace-mode selection, and an approval
queue with inspect/approve/reject/revise controls. These are design controls, not implemented UI.

MVP commands: plan, run, status, logs, diff, approvals, approve, reject, revise, stop,
resume, merge. Use tail for logs and a pager for diffs. Ratatui/Tauri are later options.
Separate reported usage from estimated cost; neither necessarily equals subscription billing.
README contains product information and a clearly labeled UI mockup, without development
plans or internal decisions. All documents are English; user communication remains Korean.

## DD-UI-01 Interface visual direction

User prefers the Dispatch desk palette: warm off-white surfaces, muted sage selection,
dark forest-green navigation and primary actions, and restrained amber/red statuses.
Apply this palette to Agent room and Review cockpit while preserving their layouts and
intervention controls. The console overview follows the same palette. These remain mockups.

## DD-11 — Persistent restriction recovery (R-18 / BG-07 / SF-09 / UC-09 / QAS-09)

Store restriction evidence and explicit UTC epoch times in the authoritative SQLite workflow document and audit/outbox transaction. Recognize explicit reset fields and timestamped messages; ambiguous local prose requires manual scheduling. A foreground `watch` coordinator or `resume-due` command claims due work under the run lock. No cloud scheduler, OS cron, or background daemon is installed automatically. Apply a retry ceiling and minimum delay to repeated expired reset times.

## DD-12 — Session ownership and role transfer (R-19 / BG-08 / SF-10 / UC-10 / QAS-10)

A task owns a logical session; multiple tasks can use one profile. Provider IDs are captured per task, not per profile. Enforce per-profile process caps and exclusion of active native session IDs. Account quota groups default conservatively to the executable and can be explicitly configured. Reassignment is permitted only while the coordinator is idle and creates a fresh provider session with bounded scope/evidence handoff. Preserve partial changes and all attempts; revoke integration approvals and invalidate affected dependents.

## DD-13 — Observed model catalogs (R-20 / BG-09 / SF-11 / UC-11 / QAS-11)

Catalog discovery uses native subprocess I/O with existing authentication. Codex uses local stdio `app-server` initialization and `model/list`; execution remains `codex exec`. OpenCode uses `opencode models`. Claude discovery uses the CLI control initialization response when supported; otherwise import an operator-observed catalog. Record timestamp, version, and source. Catalog membership is not a guarantee of remaining quota. No provider model names are hard-coded into product defaults. Session changes happen only between attempts.

## DD-14 — Durable ownership and paused-workspace checkpoints

Trace: R-02/05/09/18/19 → BG-02/03/07/08 → SF-02/03/04/06/09/10 →
UC-02/03/09/10 → QAS-03/04/09/10 → this decision → ownership/checkpoint tests.

Use exclusive OS leases on stable lock inodes. Explicitly unlock on normal release;
retain diagnostic records, and never use file existence or a stale PID as ownership
proof. Record launching, running, terminal, and not-spawned process states durably.
Explicit recovery requires an idle coordinator lease and proof that every recorded
process/group is absent. Missing records and ambiguous spawn windows refuse recovery.

A durable repository claim outlives coordinator death and prevents another run from
ignoring orphaned agents. Begin the claim only after the running state is persisted and
before spawning; failed pre-execution validation must not leave an unknown run claim.
Scope native session leases to provider/account/session identity across profiles and
repositories. A free lease does not permit session reuse until prior process evidence
proves quiescence. Unknown evidence remains an intervention case.

Capture HEAD, the complete non-ignored Git working tree, and the real index fingerprint
when execution becomes idle. Validate all retained checkouts before automatic or explicit
resume. Session edits retain those checkpoints; they cannot silently accept external
file edits. Explicit crash recovery records the observed partial state after process
absence is proven. Ignored files and OS permission containment require separate controls.

## DD-15 — Audited review feedback and dependency refresh

Human feedback targets one task and invalidates its transitive dependents, integration
tests and approval. Keep feedback bounded, linked to prior artifacts and separately
audited; permit revisions only with an idle coordinator. Retain original task intent.
Workers receive only declared dependency results and evidence references, not an
unbounded shared conversation.

Record the dependency snapshot versions used by each worktree. When versions change,
create a fresh checkout from the approved baseline and current predecessor snapshots,
then carry forward the task's own scoped partial changes. Preserve the old checkout,
logs, diff and native session identity as evidence. Use a fresh native session for the
changed checkout with a structured handoff. A merge/apply conflict blocks execution
and requires review; never reset an old checkout to force a retry.

## DD-16 — Bounded coordinator review through native CLI I/O

Run the registered coordinator as a read-only subprocess against a frozen evidence bundle.
Accept only a typed final JSON document containing verdict, summary, known evidence IDs
and at most three unique existing task IDs with bounded feedback. Do not accept generated
commands, paths, profiles, model names or permission changes. Accept is a quality opinion,
never a human merge approval. Revised tasks retain their original scopes and test commands.

Limit automated rounds and retained prompt excerpts; keep complete raw evidence on disk.
Persist running ownership before launching the reviewer, use the same stop marker/process
supervisor, and include review processes in crash recovery. Read-only preservation checks
support evidence integrity; full pre-execution permission enforcement remains a separate
capability requirement and cannot be inferred from profile labels.

`review` saves a proposal without revising tasks. `apply-review` requires the exact
canonical proposal SHA-256, unchanged workflow binding and unchanged evidence hashes;
it uses the saved proposal without another model call. `review-apply` is an explicit
operator choice for immediate scoped revisions. Three rounds per run bound retry cost.
Stopped runs refuse a new review before process launch or round consumption. A live
review can be stopped through the ordinary owned process-group lifecycle.

## DD-17 — Bidirectional pre-execution control and failure containment

Trace: R-02/08/09 → BG-03 → SF-01/05/06 → UC-04 → QAS-02/03/06 → T-03/06.
Native callback protocols require keeping stdin open while receiving tool requests.
Add bounded JSONL control messages to the existing subprocess supervisor, preserving
raw I/O and ownership. A malformed message, callback failure or invalid response must
terminate and reap the owned process group before returning an error. Callback logic
cannot spawn tools or grant approval implicitly. Final result alone does not close the
channel when a native provider still reports active work; its idle marker ends input.

Installed Claude 2.1.286 does not support the documented `onFailure: block` command
hook option (requires 2.1.295). Ordinary hook failure can allow a tool through, so use
the bidirectional SDK callback wire protocol instead of treating shell-hook installation
as proof of enforcement. Preserve print/stream-json subprocess execution and existing
authentication. Durable human decisions and native runtime negative tests remain
separate acceptance work after the transport foundation.

Primary references: [Claude hook failure semantics](https://code.claude.com/docs/en/hooks),
[SDK control protocol](https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/_internal/query.py),
[Codex hook boundaries](https://learn.chatgpt.com/docs/hooks).

Typed tool requests bind the exact input/reason, task, native session, workspace and
plan/workspace fingerprints. The local SQLite ledger starts every request as pending.
Human decisions require the exact request hash and unchanged context; approvals expire
within 900 seconds and are consumed atomically once. Decision, consumption, audit and
memory-outbox updates share one SQLite transaction. Full tool inputs stay local; replica
progress summaries contain request IDs/status only. Native permission phases may reuse
the already evaluated pre-tool decision only for the same tool-use ID and identical
input; a final result cannot discard it while the native session is still active.

The control handler is trusted, bounded engine logic. It must not run arbitrary tools
or perform unbounded blocking work inline. Provider binding must derive and revalidate
context from authoritative workspace/session state. These components are not yet
connected to production worker profiles; R-08 stays open until actual native tool
denial/approval, callback-failure behavior and provider containment are proven.

## DD-18 — Bounded file policy before native worker binding

Trace: R-08/09 → BG-03 → SF-05/06 → UC-04 → QAS-02/03/06 → T-03/06.

Use a small explicit policy for Claude built-in tool inputs. Ordinary reads and edits
are evaluated against a canonical workspace; writes additionally require an owned
task scope. Reject traversal, filesystem aliases, repository metadata, agent settings
and unsupported tool inputs. Never classify shell commands or content searches as
safe through command-string heuristics. They require a typed human decision, as do
recognized sensitive file paths. The classifier does not grant that decision itself.

Limit input size and filesystem inspection. Do not read file contents, traverse entire
directories, start subprocesses or access the network in a callback classifier.
Filesystem checks are evidence at evaluation time, not protection against a concurrent
path replacement; production integration still requires native containment and live
negative tests. Filename recognition cannot discover credentials embedded in arbitrary
source text. No profile becomes verified merely because this component passes tests.

## DD-19 — Asynchronous authoritative tool-context verification

Trace: R-02/08/09 → BG-03 → SF-01/05/06 → UC-04 → QAS-02/03/06 → T-03/06.

Native control waits asynchronously for an owned, timeout-bounded verifier process.
The verifier derives task, native session, canonical workspace and plan/workspace
fingerprints from authoritative run state. It cannot grant permission. Keep verifier
process ownership in the run's recovery inventory. Never execute unbounded Git
commands inline in a synchronous callback. Cancellation must terminate the verifier
and the native worker; retained output is evidence rather than authorization.

Risky tools are stopped while pending, preserving the native session. The operator
must inspect the exact typed request and unchanged context before approving. A stale
request requires explicit refresh to a new pending request, not implicit approval.
Approved retries run alone, revalidate context immediately before consuming one
decision, and never confer merge approval. Native provider enforcement remains an
acceptance item requiring live tool-denial and callback-failure evidence.

The opt-in Claude worker reserves a random native session UUID before process launch;
failed initialization cannot pretend that a provider session already exists. Callback
and recovery session identities must match the reserved identity. Request capture
waits for parallel jobs to drain. The scheduler drains live jobs before executing an
approved retry and considers dependency readiness to avoid starving prerequisite work.

Explicit refresh persists an incremented decision epoch before issuing a fresh pending
receipt and revoking the prior receipt. A crash cannot make old consent valid again.
Revisions and reassignment invalidate affected epochs. Operator controls and workers
retrieve current receipts by exact ID independently of the bounded history listing.
Rejection stays blocked across automatic scheduling; approval while stopped preserves
the stopped state. These local contracts have compiled mock CLI evidence.

Workspace evidence includes Git-visible changes and metadata for recognized sensitive
file-tool targets. It does not fingerprint arbitrary ignored/external Bash dependencies
or freeze files between callbacks. Callback mediation is not an OS sandbox; actual
filesystem/network/ref containment and native failure tests remain required. Existing
pre-epoch receipt payloads cannot be consumed under the new canonical binding without
an explicit refresh and fresh human review.

## DD-20 — Live normalized task evidence from owned spools

Trace: R-02/06/09/10 → BG-02/03/04 → SF-01/04/06/07 → UC-03/04/06 → QAS-03/04/06 → T-03/05/07.

Observe the existing owned stdout spool while a worker runs. One bounded incremental
decoder per attempt retains sequence/raw-offset/native-session evidence in SQLite;
the report document remains coordinator-owned. Never interpret provider messages as
instructions. Preserve raw payloads locally, without transmitting them to memory.
Record compact task/attempt/kind summaries for intermediate recall and audit.
Use a bounded batch and timer, not a daemon or framework. Dropping observation must
drop the worker future and retain its existing process-group termination guarantee.
Final draining handles split UTF-8 and an unterminated JSONL record. Invalid streams
fail observation instead of manufacturing progress or successful completion.

## DD-21 — Reviewed runtime lessons with source revalidation

Trace: R-06/10/11 → BG-04 → SF-04/07 → UC-06 → QAS-04/06/07 → T-07.

Keep draft observations separate from canonical lessons. Promotion requires an idle
coordinator and an exact digest of a bounded typed lesson. Store context, failure,
cause, response, evidence, confidence and revalidation triggers with task scope and
source hashes. Sources must be ordinary repository files and must not be recognized
sensitive/configuration paths or aliases. SQLite is authoritative; Markdown wiki/index
and append-only log are derived local assets. A missing/changed source makes a lesson
stale at retrieval, never implicit permission to rewrite it. Session handoff includes
only fresh reviewed lessons for its task, bounded to 8 KiB and explicitly framed as
evidence requiring inspection, never a grant of tool/merge permission. Local recall
works without a running agentmemory server; the existing outbox carries reviewed
lesson summaries for explicit delivery to the local memory replica.

## DD-22 — Embedded local intervention room

Trace: R-07/09/12/17/18/19/20 → BG-03/05/07/08/09 → SF-06/08/09/10/11 → UC-04/08/09/10/11 → QAS-05/08/09/10/11 → T-06/08/09.

Embed the approved sage/forest Agent Room interface in the Rust executable. On macOS
use the system Cocoa/WebKit window; an optional browser surface shares the same local
backend. Do not require Node, a UI build service or another model connection. Bind
the server to loopback, authenticate with a random HttpOnly same-site capability,
reject foreign Host/Origin and unsupported requests, and render provider text as text.

Use existing workflow actions behind a bounded typed broker. Mutations bind to the
currently inspected run revision, with exact candidate/tool SHA requirements retained.
Show actual state, normalized progress, retained log tails, diffs, tests, sessions,
model catalogs, availability schedules and reviewed lessons. Never present previews as
runtime evidence or pretend an asynchronous action finished when merely accepted.

Coordinator launch is a separate owned CLI process so closing the window does not
abandon native workers. Serialize launches, retain coordinator stdout/stderr/ownership,
and include their process groups in crash recovery. Explicit resume binds the previously
reviewed stop marker; a newer stop request during launch must not be cleared.

## DD-23 — Direct native Room transport

The native desktop is the default review surface. Its embedded HTML/CSS/JavaScript talks to the Rust controller through WKScriptMessageHandlerWithReply, not a loopback HTTP server. There is no browser runtime fallback, listener, cookie, bootstrap URL or `ui-serve` command. The current native shell supports macOS; CLI use remains independent.

Only the owned WebView main frame may submit messages. The native boundary accepts bounded UTF-8 JSON and an explicit request enum. Replies run on the Cocoa main queue; controller work runs on bounded background workers. External navigation is refused. An embedded CSP permits only the bundled scripts/styles and disables network connections. Mutation review revisions, exact candidate approval and coordinator ownership remain authoritative in Rust. A per-run kernel lease prevents duplicate windows. Closing the window does not cancel already accepted coordinator work.

Trace: local-first deployment, lightweight native UI, human review, QAS security and process lifecycle. Verify typed rejection, stale revisions, exact merges, retained work after close, artifact safety and actual native interaction.

## DD-24 — Preserve Unix termination signals

A child terminated by a signal has no numeric exit code. Preserve `termination_signal` separately in process results, terminal ownership records, test evidence and UI coordinator results; never fabricate 128+signal as an exit code. Old records deserialize with an absent signal. Errors identify recorded signal outcomes without inferring a cause or treating them as quota. Trace: standardized process outcomes, R-06 failure evidence and R-09 lifecycle.

## DD-25 — Local startup workspace

Add a native startup surface sharing the existing embedded message bridge and owned CLI job broker. Persist a private versioned registry, selected goal/repository and exact plan artifacts outside the repository. Reuse native CLI catalog/planner/executor commands; never parse shell command strings or accept a browser endpoint/API key. Registry snapshots omit environment values; editing a redacted registry preserves retained private environment values. Settings and plan changes invalidate earlier execution consent. Read-only proposal generation cannot grant worker or merge approval.

Trace: R-21 -> BG-09 -> SF-18 -> UC-15 -> QAS-16. Registration validation and exact-plan tests precede UI implementation.

### DD-26 Provider-reported quota windows

Use existing CLI stdio metadata and structured provider observations. Codex documents account/rateLimits/read; Claude documents status-line rate_limits windows, whose headless availability must be verified. Preserve original used percentage and derive remaining percentage with explicit unknown/freshness states. Do not infer paid-plan balance from token counters, context occupancy or USD cost.

### DD-27 Windows continuity and verified releases

Keep Rust core/state/protocol portable, isolate OS process/lease/filesystem/desktop primitives, and prove Windows behavior with native runners. A committed handoff and WSL CLI development are interim continuity measures; the final Windows executable requires native containment and UI acceptance. Publish only verified binaries to the existing private GitHub Release with checksums/licenses.

### DD-28 Subscription-independent supervision and fallback

Trace: R-25/R-26 -> BG-11 -> SF-21 -> UC-18 -> QAS-19.

Keep Rust as the durable execution authority. Optional LLM planning/review is bounded and replaceable; subscription failure cannot disable process/status/approval controls. A per-profile fallback specifies a registered target and optional observed model, covered by exact plan consent. Validate the entire reachable permission chain and reject cycles or same-account substitutions. Transfer only confirmed restricted/pending work after owned jobs drain; retain account restrictions independently of the task's new assignment. Preserve source attempts and bounded file-scoped handoffs, clear native session identity and reverify final output. No live steering assumption is made. Fine-grained idle supervision remains a separate unfinished acceptance item.

### DD-29 Cooperative memory directives

Trace: R-27 -> BG-11/BG-12 -> SF-24 -> UC-21 -> QAS-22.

Native live-input steering and cooperative agentmemory polling are separate capabilities. Non-steerable company agents can voluntarily read addressed directives at safe checkpoints and report acceptance/completion. Use a durable typed command ledger with local authority and explicit agentmemory delivery/acknowledgment; do not treat arbitrary recall text as a command. Operations are scoped rework, pause-at-checkpoint and handoff requests under existing plan permissions. A task reassignment requires an acknowledged relinquishment and proven process quiescence before a new writer starts. Missing acknowledgment is pending, not completed or ignored. Duplicate and expired directives cannot reexecute; rejected instructions retain reason/evidence. Runtime memory polling, host adapter and capability verification remain unfinished.

The first ledger increment binds each directive to the reviewed workflow revision at issuance and to the plan digest, task, assigned agent, native session and attempt throughout acknowledgment. Only one outstanding directive per addressed attempt is allowed. States are pending, accepted, completed, rejected, expired and superseded; exact duplicate acknowledgments are idempotent, conflicting duplicates fail. Expiry applies before acceptance; an accepted instruction retains ownership until explicit completion or rejection. A handoff completion is evidence of voluntary relinquishment, never permission to start another writer. The existing local outbox replicates a structured envelope to agentmemory, while SQLite remains authoritative. Local polling and acknowledgment commands give company CLIs an explicit adapter contract; remote memory retrieval and scheduler handoff enforcement require separate integration verification.

The REST transport increment uses loopback-only HTTP to an operator-configured existing agentmemory daemon. Explicit publish returns a validated memory ID receipt; exact-ID reads verify response identity, run-scoped project and addressed agent/session provenance. Only typed acknowledgments/progress may be imported into the local ledger; recalled directives cannot create execution authority. Lost responses permit at-least-once replicas, reconciled by local idempotency. HTTP response framing, size/deadline limits, no redirects, and secret redaction are tested before live daemon verification. Existing daemon authentication is distinct from model API keys. No daemon, engine or credential is bundled or automatically started.

The worker bootstrap increment makes `cooperative: checkpoint_v1` an explicit profile capability,
disabled by default. Its initial supported contract is a custom CLI accepting exactly one
`{session_id}` argv binding. Rust allocates and persists that identity before launch, holds its
session lease, and rejects conflicting returned identities. Provider-generated initial session
IDs require separately verified live binding and are not implicitly supported. Reserved
`AC_CHECKPOINT_*` environment names belong to Rust. The worker receives the local run directory,
an addressed JSON file and the running executable path, plus checkpoint instructions in its
task prompt. These are routing context, not worker authentication or added permission.
The CLI itself must poll before work and between safe units, acknowledge addressed directives
and publish monotonic reports. Rust does not inject live stdin or infer compliance from the
capability declaration. Local polling remains available offline; automatic memory discovery,
individual worker authentication and handoff-driven scheduler transitions remain separate work.

Exit reconciliation distinguishes directive completion from task completion. A custom
checkpoint worker may finish a pause/handoff request while deliberately leaving partial
work. Rust must prove owned process/group absence, reconcile the exact addressed ledger,
then retain partial evidence and mark that attempt stopped with explicit resume required.
Pending or accepted directives at clean exit block task completion. A completed
pause/handoff cannot be replaced by a later directive in the same attempt. Handoff does
not select a replacement from free text or confer execution/merge permission; a reviewed
target policy or explicit human reassignment remains required. Automatic target scheduling
and independent worker authentication remain separate acceptance work.

Interrupted cooperative recovery uses the same ownership inventory as ordinary recovery.
A completed pause/handoff acknowledgment cannot substitute for the missing coordinator's
child-process absence. Recovery must refuse while any owned worker/group remains, preserve
the original custom session binding and partial work after quiescence, and leave the run
stopped until explicit resume. Completed directive evidence remains in the local ledger;
recovery does not create a replacement worker or merge approval. Continuation of pending
or accepted instructions after interruption requires separate context/reconciliation tests.

Explicit resume of a stopped checkpoint worker preserves unfinished local rework as
bounded continuation context. Only the exact original plan/task/agent/session/attempt
binding is eligible. Unexpired pending and accepted rework retain their instruction and
source; completed/rejected/expired requests and pause/handoff instructions are not replayed.
The original directive is superseded and an immutable continuation receipt is stored in
the same transaction, so a crash before saving the resumed workflow can recover it.
The next worker receives this as uncertain prior work to reconcile against partial files,
not as acknowledgment, tool permission or a recalled executable command. Human feedback
and current scopes remain controlling. New-attempt reporting starts with a new sequence.

### DD-30 Read-only factual team briefings

Trace: R-28 -> BG-12 -> SF-22 -> UC-19 -> QAS-20.

Rust builds a bounded factual briefing from persisted task/session/attempt/progress evidence, account restrictions, pending approvals and acknowledged cooperative reports. Native UI and CLI consume the same typed bundle. Local state remains usable without memory server or LLM availability; external report failures/staleness are explicit. A coordinator LLM may summarize the bundle but cannot override its facts, timestamps, authority or human gates. Querying is read-only and launches no worker.

### DD-31 Status lights with stable polling

Trace: R-29 -> BG-12 -> SF-23 -> UC-20 -> QAS-21. Render status lights in session rows, retain textual state and accessibility labels, animate running only and honor prefers-reduced-motion. Rebuild the list only when its displayed data/selection changes, retaining pulse continuity during unchanged polling. Colors denote execution state, not an inferred guarantee of semantic quality.
