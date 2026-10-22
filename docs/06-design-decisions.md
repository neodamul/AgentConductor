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
final lines without newline, unknown events, and maximum line size. Ignore one optional UTF-8
BOM only at absolute stream offset zero; malformed records and non-object JSON remain errors.
Drain stderr separately;
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

Keep Rust core/state/protocol portable, isolate OS process/lease/filesystem/desktop primitives, and prove Windows behavior with native runners. A committed handoff and WSL CLI development are interim continuity measures; the final Windows executable requires native containment and UI acceptance. Publish only verified binaries to the existing private GitHub Release with checksums/licenses. Package assembly selects the host-native executable name (`agentconductor.exe` on Windows), refuses replacement of an existing output, requires an exact source commit, and checks the target-specific locked dependency closure before writing. License texts, Apache/Unicode/SQLite notices, SBOM and binary checksum accompany the executable. The public support notes and manifest state native platform prerequisites: installed Microsoft Edge WebView2 Runtime on Windows, system WebKit on macOS, Git and authenticated selected CLIs. When a published crate omits its upstream license text, use only a locally retained, reviewed upstream copy bound to exact package/version/SPDX, crate VCS commit and content SHA-256; missing, altered or mismatched provenance still refuses packaging. No network fetch or generic license substitution occurs during packaging. Retained upstream license bytes are exempt from Git text conversion so Windows autocrlf checkout preserves the reviewed digest. Copy only README-referenced public documents/screenshots; do not bundle internal plans or provider artwork. Packaging does not establish execution acceptance, and must not silently rewrite knowledge assets. Unit contracts run on both CI platforms before artifact publication. Native release CI builds the exact checkout after platform validation, assembles a package, and invokes its copied executable for shared/worktree three-agent fixture runs, diff and passing test collection, refused unapproved merge and exact approved merge. Record manifest checksum, process exit codes and parallel timing. Integration and verification worktrees are ordinary Git checkouts and therefore follow the operator's `core.autocrlf`; the packaged fixture compares content with normalized line endings instead of assuming LF bytes. Preserve failed smoke workspaces for diagnosis; upload archives as CI evidence only, without publishing a Release before the completion gates pass. Publication is tag-driven: pushing a `v*` tag runs `release.yml`, which calls the full platform validation (tests, packaging contracts, fmt, Clippy) and native package smoke as a reusable workflow, and only then a separate job with `contents: write` downloads both platform archives, combines their checksums into `SHA256SUMS` and creates a GitHub pre-release. Generated package text (checksums, manifest, SBOM, notices) is written as UTF-8 bytes with LF line endings on every platform, so the combined `SHA256SUMS` verifies with `shasum -c` anywhere. All other jobs keep read-only repository access; third-party actions stay pinned to reviewed commits.

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

### DD-32 Live tool decisions on the native callback

Trace: R-30 -> BG-13 -> SF-25 -> UC-22 -> QAS-23. The guarded Claude worker holds a human-gated PreToolUse callback open (hook timeout and control-evaluation limit raised to the decision window, default 600 s, `AGENTCONDUCTOR_TOOL_DECISION_WAIT_SECS`, 0 restores stop-and-resume). It verifies the tool context, records a pending ledger request and polls the ledger. While the run is `running`, `tool-approve`/`tool-reject` decide only through the transactional ledger: the coordinator keeps the run lock and report. If shared-workspace writers changed only the workspace fingerprint, the decision re-keys the identical request to the decision-time state. The paused worker consumes the approval once, re-verifying task, session, plan, epoch and workspace path; the fingerprint is bound at decision time because the requesting worker cannot act while paused. Rejection returns a per-call deny so the worker continues. The Agent Room snapshot lists pending ledger requests of running tasks with the matching session, so the paused request is decidable from the UI; requests created before the current attempt started are not shown as live. Each live request carries the CLI tool-call identifier in its reason, so an identical command in a later call or a resumed attempt needs its own decision instead of colliding with an earlier consumed or approved ledger entry. Stop, expiry or no decision falls back to the existing stopped/waiting flow.

### DD-33 Disposable tracked-files planning snapshot

Trace: R-31 -> BG-13 -> SF-26 -> UC-23 -> QAS-24. Operator labels cannot prove that a company CLI is read-only. Planning therefore runs in a temporary directory outside the repository populated from the reviewed base commit's tracked tree (`git archive`-equivalent), never from the working tree, so untracked and ignored files are absent. After the coordinator exits, the directory is removed and the target repository checkpoint is compared with the pre-planning checkpoint; any difference refuses the proposal. Verified native templates (Codex read-only sandbox, guarded Claude read-only tools) keep their restrictions in addition to the snapshot. Network access is not prevented and is documented as residual risk.

### DD-34 Input-method-safe forms

Trace: R-32 -> BG-13 -> SF-27 -> UC-24 -> QAS-25. Polling never assigns to a focused or composing control (`compositionstart`/`compositionend`), and change capture happens on committed input. Text is stored and passed as UTF-8 without normalization. Native acceptance uses the real system input method.

### DD-35 Cross-platform kernel leases

Trace: R-23 -> BG-10 -> SF-20 -> UC-17 -> NFR-03/NFR-11 -> QAS-18 -> DDR-03/DDR-08. Replace Unix `flock` with the standard library's exclusive file lock (`File::try_lock`, released by the kernel on process exit on both platforms). The private lease directory is the per-user temporary directory; on Unix it must also be mode 0700 and owned by the effective user, on Windows the per-user `%TEMP%` profile ACL provides the equivalent boundary. Lock files stay in place after release.

### DD-36 Windows process-tree containment

Trace: R-23/R-24 -> BG-10 -> SF-20 -> UC-17 -> NFR-03/NFR-11 -> QAS-04/QAS-18 -> DDR-03/DDR-08. Workers start suspended (`CREATE_SUSPENDED | CREATE_NEW_PROCESS_GROUP`), are assigned to a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, and only then resumed, so no descendant can escape before containment. Stop and timeout close stdin, wait the grace period, then terminate the job; a crashed coordinator's job handle closes and the kernel kills the tree. Ownership records the root PID; quiescence is proven when that PID cannot be opened or has exited. There are no POSIX signals on Windows, so `termination_signal` stays empty and exit codes are reported.

### DD-37 Windows file identity and randomness

Trace: R-08/R-23 -> BG-03/BG-10 -> SF-05/SF-20 -> NFR-01/NFR-11 -> QAS-06/QAS-18 -> DDR-01/DDR-08. File-policy identity uses `GetFileInformationByHandle` (volume serial number, file index, link count) on handles opened without following reparse points; reparse points (symlinks, junctions, mount points) and multi-link files are refused like Unix symlinks and hard links. Session identifiers use `BCryptGenRandom` with the system-preferred RNG instead of `/dev/urandom`. Windows APIs come from `windows-sys` (MIT OR Apache-2.0), reviewed under the release license policy.

### DD-38 Windows desktop shell

Trace: R-17/R-21/R-23 -> BG-09/BG-10 -> SF-18/SF-20 -> UC-15/UC-17 -> NFR-10/NFR-11 -> QAS-16/QAS-18 -> DDR-06/DDR-07/DDR-08. The shared HTML/JS Room and startup pages are hosted in the system WebView2 runtime through a native window, using the same direct message transport contract as WKWebView (no local web server). Absence of the WebView2 runtime is reported with the CLI fallback. Acceptance requires native Windows evidence; compilation alone is not support. Default desktop workspace selection keeps HOME precedence and falls back to non-empty USERPROFILE on Windows only; missing both reports the explicit ui-new alternative. Resolve environment values through a pure helper to test precedence and absent/empty values without mutating global environment; native Windows child-process acceptance removes HOME and sets USERPROFILE locally.

### DD-39 Workspace back navigation and discard

Trace: R-33 -> BG-09 -> SF-28 -> UC-25 -> FR-19/NFR-10 -> QAS-26 -> DDR-07. The workspace titlebar gains "Back to run" (shown when a run is available) and "Discard changes" (enabled while workspace or plan edits are unsaved). Both are view-only: the draft is rebuilt from the last saved configuration and the plan editor from the stored proposal; no save, plan, edit or start request is sent. When edits exist a confirmation dialog lists the unsaved areas (workspace settings, agent profiles, plan tasks) before discarding; cancel keeps them. Back navigation then reuses the existing `open_run` path. The workspace keeps the most recently started run as `last_run`, separate from the editable proposal: saving settings, importing catalogs or replacing the proposal no longer orphans a running or reviewable run. `last_run` is offered only while its directory lies inside the workspace and contains a run report; the run's own plan, approvals and leases remain authoritative.

### DD-40 Runtime application icons

Trace: R-34 -> BG-14 -> SF-29 -> UC-26 -> FR-20/NFR-10/NFR-12 -> QAS-27 -> DDR-10. A native `icons` module maps built-in providers to their desktop applications (macOS bundle identifiers `com.anthropic.claudefordesktop`, `com.openai.codex`, `ai.opencode.desktop`; Windows per-user install paths of the same applications). macOS resolves the bundle through NSWorkspace and renders its icon at 64 × 64; Windows looks for the application at known per-user install paths and in user and machine Start Menu shortcuts (`Claude.lnk`, `ChatGPT.lnk`/`Codex.lnk`, `OpenCode.lnk`, also one folder deep) and obtains the icon through `SHGetFileInfoW` (COM-initialized), which resolves shortcut targets. Microsoft Store packaged apps without a classic shortcut (the ChatGPT desktop app) keep the letter avatar; this is a documented limitation rather than a failure. Pixels are encoded by a small dependency-free PNG encoder into a `data:image/png;base64` URL capped at 64 KiB, cached per process and included in UI snapshots keyed by provider. Custom providers never receive icons. The pages render an `<img>` with an accessible label when an icon exists and the letter avatar otherwise.

### DD-41 Responsibility-based module split

Trace: R-35 -> BG-05 -> SF-12 -> C-04 -> NFR-07 -> QAS-12 -> DDR-04/DDR-06. Oversized modules become directories of focused submodules re-exported through the existing public paths so callers and tests stay unchanged. Workflow responsibilities are execution, scheduling, permissions, approval, review, editing, assignment and recovery. Process supervision separates control I/O, containment and the Windows Job API. Native shells separate Cocoa and WebView2. CLI dispatch separates memory/UI, sessions/review and execution commands.

Unit-test bodies reside in `tests/unit/`, linked from their original private modules with `#[cfg(test)]` and `#[path]`; this preserves private invariant coverage without widening the production API. Integration tests stay under `tests/`, with the memory round-trip cases in a nested module. CI explicitly runs `tests/source_layout.rs`, recursively checking source, tests, scripts, examples, styles and markup plus an optional root build script for the 500-line limit and rejecting test bodies under `src/`. Source modularity does not change single-executable distribution or add a runtime framework.

### DD-42 README as product introduction, manual as reference

Trace: R-36 -> BG-15 -> SF-30 -> UC-27 -> FR-21/NFR-14 -> QAS-28 -> DDR-11. `README.md` keeps only: purpose and audience, key capabilities, how orchestration works (Rust engine versus optional LLM coordinator), role assignment, usage-limit redistribution, prerequisites (agentmemory optional), real native screenshots, install and launch (running the app opens the default workspace), a five-step desktop walkthrough, safety summary and a link to `docs/user-manual.md`. The manual holds the desktop guide, CLI reference generated from the same command set as `--help`, configuration (profiles, permissions, fallback, quota groups, checkpoint workers), approvals and recovery, knowledge features, platform requirements and troubleshooting. Because packaging copies README-linked documents, the manual ships with releases; it links only assets the README already references. Contract tests keep the README free of shell command blocks and require every `--help` command in the manual.

### DD-43 Automatic reviewed-target handoff

Trace: R-27 / R-25 -> BG-11 -> SF-21/SF-24 -> UC-18/UC-21 -> QAS-19/QAS-22 -> DDR-05.

A completed cooperative `handoff` is relinquishment by the worker; the only target that may
take over automatically is one the human already reviewed: the task agent's `fallback`
chain, shown literally in plan consent. The directive text never names or authorizes a
target. After every scheduled attempt has drained, the scheduler verifies owned process and
group absence for all attempts (no competing writer), then follows the fallback chain past
restricted accounts and transfers the role through the same bounded handoff document,
fresh target session and dependent invalidation used for quota fallback. Without an
available reviewed target the task stays stopped and requires explicit resume or human
reassignment, as before. `pause` never transfers. Tests, candidate approval and merge
approval are always required again. Alternative rejected: letting the directive carry a
target agent, because issuance would then grant execution authority that the plan review
did not.

### DD-44 Scoped agentmemory discovery

Trace: R-27 -> BG-11/BG-12 -> SF-24 -> UC-21 -> QAS-22 -> DDR-05.

Exact-ID import required an operator to copy memory IDs. Discovery removes that step without
widening trust: for each cooperative task's current checkpoint address the engine derives
the same run project and address-digest agent tag used for publication, lists only that
agent's latest memories through the agentmemory 0.9.29 contract
(`GET /agentmemory/memories?agentId=…&latest=true&limit=50`), keeps entries whose project
and agent tag match exactly, and passes each new ID to the existing exact-ID import. The
exact read, envelope validation, ledger transition, idempotent receipt and the refusal to
import directives are therefore unchanged. Already-imported IDs are skipped; rejected
entries are reported, not retried silently. `memory-discover RUN PORT` runs one pass on
demand; it launches no worker and changes no task, approval or merge state. Rejected
alternative: full-text search, because similarity results are not provenance.

### DD-45 Manual plan drafts share the generated-plan path

Trace: R-37 -> BG-16 -> SF-31 -> UC-28 -> FR-22/NFR-15 -> QAS-29 -> DDR-12.

The startup controller gains a `draft` request. It checks the goal, registered profiles,
a clean repository and an artifact directory outside it, records the repository base and
writes a three-task draft (`task-1` … `task-3`, assigned to the first registered agent,
empty instructions, paths and tests) as the proposal's `review.json` with phase `ready`.
No job, process or coordinator is involved, so no read-only capability is required. From
there the existing `edit` request validates (nonempty fields, literal disjoint paths,
known agents and dependencies) and the existing `start` request enforces the exact
digest, unchanged goal/profiles, clean repository and unchanged base. An unfilled draft
can be shown and edited but never approved. Alternative rejected: a separate manual
planning pipeline, which would duplicate the consent and base-binding rules.

### DD-46 Workspace settings layout and native folder picker

Trace: R-38 -> BG-17 -> SF-32 -> UC-29 -> FR-23/NFR-16 -> QAS-30 -> DDR-13.

The goal card keeps the goal first, then a **Workspace settings** group with the planning
coordinator and, beneath it, the repository field with **Browse…** next to the worktree
option. Browse sends a `pick_folder` request; the native bridge already runs each request
off the UI thread, so a blocking platform dialog is acceptable. The picker uses tools the
OS ships (`osascript` `choose folder` on macOS, PowerShell `FolderBrowserDialog` in an STA
process on Windows), so no dependency is added; cancel returns no path. The returned path
only fills the field — saving still canonicalizes it through Git, so the dialog grants no
authority. `AGENTCONDUCTOR_PICK_FOLDER` substitutes the selection for automated tests.
Saving already accepts an empty repository; plan buttons require one.

### DD-47 In-place workspace switching with a recent-workspace registry

Trace: R-39 -> BG-17 -> SF-33 -> UC-30 -> FR-24/NFR-16 -> QAS-31 -> DDR-13.

The window owns its startup surface and workspace lease behind a mutex instead of fixing
them at launch. `workspaces` returns the current directory and the recent list from
`~/.agentconductor/workspaces.json` (at most 20 existing absolute directories, current
first). `switch_workspace {path}` refuses while a run view is open or a planning/start job
runs, creates the directory if needed, acquires the new workspace's window lease first,
opens its startup state, then swaps and releases the old lease, and records it as recent.
A failure at any step leaves the current workspace untouched. The page reloads its snapshot
after a successful switch. Rejected alternative: launching a second process and closing
the first, which loses the single-window lease handover and needs platform window control.

### DD-48 Workspace names and an explicit registry

Trace: R-40 -> BG-17 -> SF-34 -> UC-31 -> FR-25/NFR-16 -> QAS-32 -> DDR-13.

The workspace `Config` gains `name` (default empty, at most 80 characters, no control
characters), edited above the goal and saved with the existing revision-checked `save`.
The DD-47 registry becomes the authoritative list (up to 50 entries): `workspaces` returns
`{path,name}` for each entry, reading the name from that workspace's `startup.json` and
falling back to the folder name. `create_workspace {name}` validates the name, creates a
unique folder `~/.agentconductor/workspaces/<slug>[-n]`, switches to it through the same
lease-first path and stores the name. `forget_workspace {path}` rewrites the registry
without that path; it refuses the current workspace and never deletes files. Rejected:
deleting folders from the menu, because a workspace holds plans, runs and evidence.

### DD-49 Changelog-driven release notes

Trace: R-41 -> BG-10 -> SF-35 -> UC-32 -> FR-26 -> QAS-33 -> DDR-08.

`CHANGELOG.md` keeps a `## vX.Y.Z[-pre]` section per release. The Rust command
`agentconductor release notes <tag>` prints exactly that section and exits non-zero
when it is missing or empty. The publish job writes it to a file and passes
`--notes-file`, so notes are reviewed in the same commit as the code they describe and
nothing is generated from commit messages at release time. Earlier pre-releases receive
their sections retroactively and their notes are updated with `gh release edit`.

### DD-50 Explained clean-baseline gate with an explicit commit choice

Trace: R-42 -> BG-13 -> SF-36 -> UC-33 -> FR-27 -> QAS-34 -> DDR-01.

The clean-repository rule stays: a plan's base must be one commit so the integration
candidate contains only agent changes and the target branch can be checked before merge.
Two startup requests are added. `baseline_status` returns `{clean, changes}` from
`git status --porcelain --untracked-files=all` (at most 200 lines). `commit_baseline
{revision}` requires the current workspace revision and a configured repository, runs
`git add -A` and `git commit -m "AgentConductor baseline: <goal summary>"` with the user's
own Git configuration and hooks, and returns the new HEAD; a clean repository is reported
as nothing to commit. The page calls `baseline_status` before **Generate plan** and **Write
plan manually**; when unclean it opens a dialog with the reason, the changed files and
**Commit and continue** / **Stop**. Rejected: automatic stashing, which hides the
operator's work from the agents and can conflict when restored.

### DD-51 Messenger connectors (proposed, not decided)

Trace: R-43 -> BG-18 -> SF-37 -> UC-34/UC-35 -> FR-28/NFR-17 -> QAS-35/QAS-36 -> DDR-14.

Status: proposed for the roadmap; to be confirmed before implementation.

- **Channel abstraction.** A `MessengerChannel` port with `notify(event)` and an inbound
  `decision(payload)` path, fed by the existing audit/outbox so messages are durable,
  bounded and retried like agentmemory delivery. Events: decision needed, run outcome,
  usage-limit wait, briefing on request.
- **Slack connector.** A Slack app using Socket Mode, so the local application needs no
  public URL or inbound port; interactive buttons arrive over the same socket. Bot and app
  tokens live in the macOS Keychain / Windows Credential Manager, never in workspace files.
- **Company messenger connector.** Phase 1: outgoing webhook with a configurable JSON
  template. Phase 2: decisions through either signed HTTPS callbacks to a relay the company
  hosts, or authenticated polling of the company API, chosen per messenger capability.
- **Authority.** Messenger decisions are limited to pending tool requests (allow/deny) and
  reuse the exact request binding, expiry and single consumption of DD-32. Plan consent and
  merge stay desktop-only. Approvers are linked per workspace (messenger user ID ↔ operator
  approval) and every remote decision is audited with that identity.
- **Data minimization.** Default payload is metadata only; per-workspace opt-in for request
  text. Redaction removes paths outside the repository, tokens and environment values.
- **Open questions.** Which company messenger and its API (webhook, bot, signing); whether
  a company relay is acceptable; whether teammates other than the operator may approve.

### DD-52 Local exclusion for OS metadata files

Trace: R-44 -> BG-13 -> SF-36 -> UC-36 -> FR-29 -> QAS-37 -> DDR-01.

`baseline_status` also reports `os_files`: untracked (`??`) entries whose file name is
`.DS_Store`, `Thumbs.db`, `ehthumbs.db`, `desktop.ini` or starts with `._`. The revision-
checked `ignore_os_files` request appends the matching name patterns (`.DS_Store`,
`._*`, …) that are not yet present to the repository's local exclude file, resolved with
`git rev-parse --git-path info/exclude` so linked worktrees work, and returns the new
status. The local exclude file is untracked, so ignoring needs no commit and leaves the
operator's `.gitignore` untouched; the dialog explains how to make it global
(`core.excludesFile`). Tracked metadata files are not offered because ignoring cannot hide
a tracked change. Rejected: editing `.gitignore`, which itself creates a change to commit.

Amendment (user, 2026-10-11): **Open existing workspace…** lists saved workspaces instead
of opening a folder dialog. `saved_workspaces` scans the data directory (`desktop` and
`workspaces/*`) for folders containing `startup.json` and returns `{path,name,registered}`;
`open_workspace {path}` accepts only such a folder and then uses the same lease-first
switch and re-registers it. This prevents turning an arbitrary folder (for example a source
repository) into a workspace by mistake; the folder picker remains only for the repository.

### DD-53 Planning progress from durable job metadata

Trace: R-45 -> BG-12 -> SF-38 -> UC-37 -> FR-30 -> QAS-38 -> DDR-07.

The stored startup job gains `started_ms` (serde default 0 for older files), recorded when
the job is launched. Every job snapshot — live or recovered from `owner.json` — carries
`started_ms` and `log`, the job's process directory. The page renders the review area
from that snapshot: running plan → coordinator name and elapsed time computed from
`started_ms` (so a reopened window shows the true duration); completed → a one-time ready
notice and `scrollIntoView` of the review section; failed/interrupted → exit status and
log directory. No new process or polling path is added; the existing 1-second snapshot
drives the timer.

### DD-54 Sectioned instruction format and auto-growing editor

Trace: R-46 -> BG-13 -> SF-39 -> UC-38 -> FR-31 -> QAS-39 -> DDR-07.

The planning prompt specifies the `prompt` field format: lines `Goal:`, `Inputs:`,
`Steps:`, `Expected output:`, `Done when:` each followed by `- ` bullet lines, written in
the language of the goal, with JSON `\n` line breaks. The format is a request to the
coordinator, not a validation rule, so plans from CLIs that ignore it (and hand-written
plans) remain valid; the editor never rewrites instruction text because the approval
digest covers it. The instruction textarea starts at six rows and resizes to its line
count (cap 30) on render and input.

### DD-55 Operator role rules constrain the planner

Trace: R-47 -> BG-19 -> SF-40 -> UC-39 -> FR-32 -> QAS-40 -> DDR-12.

The workspace `Config` gains `roles`, a map from `verification`, `development`,
`documentation`, `knowledge` and `testing` to an ordered list of registered agent IDs
(planning is the existing coordinator). Saving rejects unknown roles, unregistered agents
and lists longer than five. The startup job writes `roles.json` beside `profiles.json` and
passes it as the optional last argument of `plan`. The planning prompt lists the rules and
requires a `role` on every task and an `agent` from that role's list, first entry
preferred. `Task` gains an optional `role` (omitted when absent, so earlier plans and their
digests are unchanged). After parsing, the planner rejects any task without a role or with
an agent outside its role's list, naming both; review edits by the operator are not
constrained because the human decides. Approval and merge remain human; the verification
role is the default agent for quality review proposals.

Amendment (user, 2026-10-11): a role entry is `{agent, model?}`; a plain string is still
accepted for earlier rules. Saving validates the model against the agent's observed
catalog (DD-21 rules). The planning prompt shows the model next to the agent, and after
parsing the engine sets `task.model` from the matching entry whenever that entry names a
model, so the operator's model choice never depends on the coordinator echoing it. The role
table shows a model selector per agent with the catalog models or *CLI default*.

### DD-56 Agent list saved by its own request

Trace: R-48 -> BG-09 -> SF-41 -> UC-40 -> FR-33 -> QAS-41 -> DDR-13.

`save_profiles {revision, profiles}` replaces only `config.profiles` in the stored
workspace after the same profile validation as `save` (private environment values are
kept for unchanged agent IDs). If the coordinator is no longer registered it becomes the
first remaining agent; role entries naming removed agents are dropped and empty roles
removed. Any generated proposal is cleared because its profiles changed. The page sends it
from **Use profile** and **Remove**, then merges the stored list into its draft so other
unsaved fields stay as typed. Rejected: auto-saving the whole draft, which would also
commit a half-written goal.

### DD-57 Page-driven automatic model discovery

Trace: R-49 -> BG-10 -> SF-42 -> UC-41 -> FR-34 -> QAS-42 -> DDR-13.

Discovery already exists as a single startup job per agent (`models-discover`, metadata
only). The page triggers it automatically after each snapshot when no job runs, no plan is
being generated or started, and nothing is unsaved; a reviewed plan stays valid because
catalogs are observed metadata — settlement and import no longer clear the proposal and
execution consent compares agents with catalogs removed (executables, arguments and
permissions must still match); it picks the first built-in agent without a catalog that has not
failed in this page session. Jobs therefore run one at a time and the existing catalog
validation and settlement apply. `save` and `save_profiles` keep a stored catalog when the
submitted profile has none and the agent ID, parser and executable are unchanged, so an
edit started during discovery cannot erase observed models. Custom CLIs still import an
operator-observed catalog.

### DD-58 Plan-relevant comparison and quiet catalog updates

Trace: R-49/R-50/R-51 -> BG-10/BG-17 -> SF-41/SF-42 -> UC-40/UC-41 -> FR-34/FR-35/FR-36 -> QAS-42/QAS-43 -> DDR-13.

`save` keeps the proposal when goal, repository, workspace mode and every agent's
execution view (profile without `catalog`) are unchanged; the same execution view is what
consent compares. Automatic discovery runs only for built-in agents added in the current
page session (from the `save_profiles` result); **Models** triggers a refresh explicitly.
The role table is rebuilt only when the set of agent IDs changes; catalog arrivals update
model `<option>` lists in place, keep selected values, and skip the element that has focus.
Automatic discovery shows no notice. Agent cards use a two-line header (icon, name;
provider · sessions · models · capability · fallback) and a single truncated executable
line.

### DD-59 Workspace settings file for Git

Trace: R-52 -> BG-20 -> SF-43 -> UC-42 -> FR-37/NFR-18 -> QAS-44 -> DDR-15.

- **Format.** `agentconductor.workspace.json`, pretty JSON with LF, keys
  `format` (`agentconductor.workspace/v1`), `name`, `coordinator`, `workspace_mode`,
  optional `repository` (only `"."`), `agents` (profiles with `environment` and `catalog`
  removed) and `roles`. Goals, proposals, jobs, runs and registry entries are excluded.
- **Export.** `export_settings {revision, folder}` writes the file into a folder chosen
  with the existing folder picker (default: the configured repository). The file is meant
  to be committed; writing it makes the repository unclean until it is, which the clean-
  baseline dialog then explains.
- **Import.** `preview_settings {folder}` (read-only) returns the parsed settings and
  warnings: capabilities needing local verification, absolute executable paths, agents
  that will replace local ones. `import_settings {revision, folder, keep_verified}`
  validates profiles and role rules, keeps local `environment` and observed `catalog` for
  matching agent ID/parser/executable (DD-56/DD-57 carry-over), downgrades
  `external_sandbox`/`read_only` to `unverified` unless `keep_verified`, sets the
  repository to the folder when the file says `"."`, and stores through the same
  revision-checked path. A plan is kept only under the DD-58 rule.
- **Rejected.** Syncing the whole `~/.agentconductor` workspace through Git, which would
  share secrets, local paths and run evidence.

Amendment to DD-57/DD-58 (user, 2026-10-11): `create_workspace` copies observed catalogs from
the workspace being left onto agents with the same parser and executable, so a new
workspace shows model lists immediately; after any workspace switch the page queues its
remaining built-in agents without catalogs for one quiet discovery each.

### DD-60 Variable plan size and document-driven planning

Trace: R-53/R-54 -> BG-02/BG-19 -> SF-44 -> UC-43 -> FR-38/FR-39 -> QAS-45 -> DDR-02/DDR-12.

`Plan::validate` accepts 1–12 tasks (12 bounds the consent dialog, review evidence and UI);
scheduling is unchanged — dependency order and per-agent session limits already govern
parallelism. The planning prompt asks for the smallest number of tasks (1–12) with
disjoint owned paths. `plan … [roles.json] [--from <plan.md>]` reads the document (UTF-8,
≤64 KiB), stores a copy as `source-plan.md` in the planning directory and appends it to the
prompt as untrusted reference text with the instruction to plan only unfinished items.
The startup `read_plan {revision, path}` request mirrors `prepare` (clean repository,
coordinator, role rules) and fills an empty goal from the document's first heading. A file
picker (`pick_file`, macOS `choose file`, Windows `OpenFileDialog`, test override
`AGENTCONDUCTOR_PICK_FILE`) selects the document. Review offers **+ Add task** and
**Remove task**; edits are collected from the cards before re-rendering. Coordinator review
still revises at most three tasks per round.

### DD-61 Slugs and their trees

Trace: R-55 -> BG-21 -> SF-45 -> UC-44 -> FR-40 -> QAS-46 -> DDR-16.

The workspace config gains `slugs` (ordered list of `{slug, goal, created_ms}`) and
`active_slug`. A slug is derived from the goal (lowercase ASCII words joined by `_`,
≤48 chars, made unique with a suffix) and editable before creation. The tree root is
`<repository>/.agentconductor/<slug>/` or `<workspace>/<slug>/` (slug characters `[a-z0-9_]` cannot collide with `plan-…`/`run-…` folders); the repository
variant adds `/.agentconductor/` to the local exclude file (same mechanism as DD-52) so
cleanliness checks, candidates and merges ignore it, and owned paths may not point into it.

### DD-62 Phased plans executed one run per phase

The planning prompt asks for `analysis` (Markdown) and `phases` (1–6, each with a name and
1–12 tasks). The analysis is written to `plan/analysis.md`, the phase list to
`plan/plan.json`, each phase to `phase/<nn>-<name>.md`, and each task as
`task/<phase>/<task>.md`. Approving a phase starts a run with that phase's tasks only
after every task of the previous phase is in `completed/`. When the run's tests run the
phase's task files move to `verification/<phase>/`; after the candidate is merged they move
to `completed/<phase>/`. Moves are atomic renames.

### DD-63 Progress derived from the tree

`progress(slug_root)` scans the tree: no `plan/plan.json` → *no plan*; analysis only →
*analysis*; otherwise per phase the counts of tasks in task/verification/completed and the
phase stage (pending, task, verification, completed); the slug stage is the first phase not
completed, or *completed*. The verifier reports a task present in more than one stage, a
task file not declared in `plan.json`, a declared task missing everywhere, or a completed
task in a phase whose predecessor is not completed. Workspace snapshots and exports carry
this derived progress; it is never stored as independent truth.

### DD-64 Slug archives

Export writes `<slug>.zip` (store-only ZIP, CRC-32, no compression, implemented in-house to
avoid a new dependency) containing the slug tree with paths relative to the slug root, and
lists `{slug, goal, archive, progress}` in the settings JSON. Import requires every listed
archive next to the JSON, rejects absolute or `..` entries and entries outside the listed
slug, recomputes progress from the extracted tree and refuses a mismatch with the JSON;
extraction goes to a temporary sibling folder renamed into place.

### DD-65 Plan rules file with workspace override

`~/.agentconductor/plan-rules.md` is created with the default rules when missing and is
never overwritten afterwards. The workspace config gains `plan_rules` (`null` = use global,
otherwise the workspace text), exported in the settings JSON. The effective rules are
appended to the planning prompt as operator rules. The window offers **Plan rules** with a
global/workspace switch and an editor.

### DD-66 Passive Claude quota from owned streams

The existing progress reader receives the reviewed agent profile and persists valid Claude `rate_limit_event.rate_limit_info` metadata while reading the owned stdout spool. The official wire fields are camelCase `rateLimitType` and `resetsAt`; `utilization` is a fraction, converted to percentage. Known windows are five_hour, seven_day, seven_day_opus, seven_day_sonnet and overage. Missing utilization remains unknown, including rejected status; no synthetic 100% is inferred. Each event replaces the retained snapshot, avoiding freshness inflation of omitted windows. Invalid quota metadata is audited without cancelling valid task output. Claude Refresh only checks retained metadata and records that check, preserving its timestamp and the execution revision. No SDK, API key, authentication-secret read, or extra inference request is introduced. Official schemas: https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/types.py and `_internal/message_parser.py`.

### DD-67 Inline quota — Add a compact, nonshrinking badge to the agent-name row; preserve detailed windows in the inspector and tooltip. Workspace snapshot reads quota from its retained run using a read-only SQLite snapshot and current profiles, never launches a CLI. Missing observations stay unavailable. Trace: QAS-52.

### DD-68 MCP goal-source imports

Use the OpenCode `mcp` map as the connection schema, with workspace UI to add/edit/enable servers or paste/import JSON/JSONC. Separate `goal_sources` binds Jira and Confluence to server names and configured read tools; one server may serve both. Tool schemas are discovered, not hard-coded to an Atlassian connector. Do not require OpenCode installation or copy authentication tokens from its store. Existing connector authentication/OAuth stays distinct from model API keys.

The explicit goal grammar is `jira:PROJECT-123` and `confluence:PAGE_ID`; source chips insert these forms. Keep original goal text plus normalized typed references. Unqualified issue keys and reference-number lists produce suggestions requiring confirmation; unrelated numbers remain prose. Multiple tenant bindings require an explicit source selection.

Import is a separate preview/apply operation in workspace settings before planning: retrieve Story fields/acceptance criteria and selected page bodies with read-only tools, normalize to local Markdown snapshots, bind digests to goal/slug and plan evidence, and transmit only necessary context to the selected planning CLI. Surface partial failures and allow explicit optional-page exclusion. Do not automatically follow every link, import attachments, or publish raw company documents to agentmemory. LLM-wiki stores source-linked reviewed summaries with local-access policy. Bound pagination, page count, response bytes and deadlines; no background polling in the first increment. Portable settings keep connection definitions/bindings with secret references, not secrets or private source bodies.

Compatibility source: https://opencode.ai/docs/mcp-servers/ (checked 2026-10-11).

### DD-67 collection correction

Workspace quota has its own SQLite cache and background metadata refresh, independent of execution runs. Query supported providers once per open and on badge activation, preserve reviewed plans, and use the newest valid profile-bound workspace/run observation. Claude macOS `/usage` terminal collection is a separate read-only metadata channel; answer the folder trust prompt with its default at most twice (the query session has no tools, hooks or project settings) and stop with guidance if it keeps asking.

### DD-69 Per-agent idle metadata refresh

The visible workspace and room keep per-agent timestamps for a two-minute cadence, updating only the requested agent. Metadata jobs use the existing coordinator job/ownership infrastructure; queries serialize to minimize resources and defer while a worker/coordinator is active. Manual refresh is independent of this cadence. Workspace metadata persists in quota.sqlite and carries the profile digest. Rooms can reuse this cache only when startup.json explicitly binds their exact run path. Codex uses app-server metadata; macOS Claude uses /usr/bin/script with the verified screen-reader /usage command, disabled tools/hooks, inherited CLI login, 20-second/1-MiB bounds and owned process-group cleanup. Project trust is never accepted automatically. Other platforms retain Claude stream evidence until a terminal adapter is verified. Action icons use local inline SVG without a new dependency.

### DD-71 Adaptive refresh and on-demand icons

Trace: R-65 -> BG-10 -> NFR-05 -> QAS-58 -> DDR-06.

Both pages replace their fixed one-second `setInterval` with a self-rescheduling timer:
1 s while a job runs, a plan is being generated, an action is pending or edits are unsaved;
5 s otherwise; no polling while `document.hidden`, with an immediate refresh on becoming
visible. The startup snapshot carries icon data URLs only when the request sets
`icons: true`, which the page does on first load and whenever the set of agent kinds
changes; other snapshots omit them. Rejected: server push, which would add a channel and
a resident listener for little gain at these intervals.

### DD-70 Plan-aware quota groups

Preserve optional provider plan_type in snapshots and views; old persisted records deserialize as unknown. Codex planType comes from rate-limit metadata. Claude terminal parsing accepts an explicit Enterprise plan header; other accounts stay unknown rather than inferred. Enterprise primary/five_hour and secondary/seven_day receive Current/Total tooltip aliases. Render percentages alone as one slash-separated line; keep all period labels in the tooltip. Other labels derive from reported window minutes, with source labels retained when no duration is known. Order windows by duration, retain unknown values and model-specific windows, and keep raw metadata in the tooltip. A sibling quota-controls group combines badges and refresh beside the selection button, avoiding nested buttons and detached icon spacing.


Quota refresh is asynchronous: native requests dispatch off the UI thread; a short-lived metadata child performs bounded Tokio I/O, and snapshots poll with try_wait. No CLI is kept resident between refreshes. Per-agent scheduling deduplicates requests and serializes collection to limit host resources. The temporary child still consumes CPU/memory during collection; async does not imply zero resource use.


R-64 / UC-54 interaction refinement: keep a 5px gap between quota values and refresh. All enabled buttons, including their icon/text children, show a pointer cursor; disabled controls keep the default cursor.


DD-70 visual correction: retain the 5px gap and remove the shared quota-controls background; only the numeric badge has a background.

### DD-73 Agent preferences and embedded brand

Use a serde-default five-minute field on AgentProfile; exclude cadence from execution comparison and quota identity. Retain per-agent timestamp deduplication and asynchronous metadata collection. PATH discovery checks executable files without resolving symlink names or spawning tools. A native picker fills the form only. Share a project-owned AC asset across titlebar, Cocoa application icon and Windows window icon, with no image or web service at runtime. Scoped CSS standardizes 40px single-line controls while preserving multiline editing. Suppress quota job loading text. Trace: QAS-60.
### DD-72 Empty default registry with templates

Trace: R-66 -> BG-09 -> SF-41 -> QAS-59 -> DDR-13.

`Startup::open` creates a new workspace with `profiles: []` and `coordinator: ""`; the
provider templates stay in the snapshot's `templates`. `save` accepts an empty coordinator
only while no agents are registered. Saving fewer processes and lookups for unused CLIs
also serves R-65.

Amendment to DD-62 (2026-10-11): planning from the window always requests phases into the
active slug. A coordinator that returns only a task list is accepted as a single phase
`01-plan` with a recorded note that no analysis was returned, so older or non-compliant
CLIs keep working.

## DD-74 Provider switch replaces the profile and renames references

The agent dialog lists Provider before Agent ID. `provider.onchange` takes the provider template and sets ID (`uniqueId(base, others)`), executable (then `resolve_executable` for an absolute path), args, input, permission, max sessions and quota cadence, and drops the previous catalog. On save, the page sends `save_profiles {profiles, renames}`; the server rewrites coordinator, role entries and other agents' fallback agent through `renames` before removing unknown references. Alternatives rejected: keeping the old ID (confusing quota/role labels) and silently dropping references (loses role rules). Trace: R-68, QAS-61.

## DD-75 Reset time from the existing quota observation

`quotaBadge` (shared by Workspace and Agent Room) appends ` · <reset>` using the latest `resets_at` among windows with remaining ≤ 0 %, formatted as time today or short date and time otherwise, ` · reset unknown` when absent; the tooltip appends `resets …` per window. Uses data already cached, adds no polling. Trace: R-69, QAS-62.

## DD-76 ID-based message catalogs with a machine-wide language preference

`src/ui/language.rs` stores the saved language next to the workspace registry (`~/.agentconductor/preferences.json`), and native documents embed `<html lang>`. Stable IDs reference the UTF-8 data files `src/ui/messages/en.json` and `ko.json`; catalogs are exempt from the 500-line code limit. HTML explicitly marks interface text and attributes, JavaScript uses `acMessage(id, parameters)` with tracked DOM bindings, and native app-owned messages use `ui::messages`. Switching language rerenders only marked content and bindings, preserves unsaved input, code and raw evidence, and retains appended icons. Replacing rendered children discards obsolete text bindings. Named parameter values are substituted once as plain text; missing translations fall back to English. Windows picker captions/filters use environment values. See [interface-messages.md](interface-messages.md). Trace: R-70, QAS-63; operator amendment 2026-10-13.

## DD-77 Fallback models through the discovery queue

`fallbackModels()` builds options from the target's `catalog`, adds a catalog-less provider agent to `addedAgents` (the existing one-at-a-time discovery queue, DD-57) and labels the empty state; `poll()` re-renders the select while the dialog is open unless it has focus. The help text states that this is the fallback agent's model. Trace: R-71, QAS-64.

## DD-79 Messenger notifications, phase 1 (decides part of DD-51)

Trace: R-43 -> BG-18 -> SF-37 -> UC-34/UC-61 -> FR-28/NFR-17 -> QAS-35/QAS-66 -> DDR-14.

- **Module.** `src/messenger.rs` (settings, events, payload, secret store) and `src/messenger/send.rs` (delivery). Public settings live in the workspace `Config.messenger`; the URL lives in `messenger-secrets.json` next to the workspace registry (0600 on Unix), keyed by the workspace folder digest, and is never exported.
- **Event sources.** (1) `workflow::save` — every report transition — calls `messenger::notify_run(dir, report)`; the desktop links a run to its workspace by writing `<run>.messenger.json` (workspace key and settings, no secret) before launching `startup-run`. A sibling `<run>.messenger-sent` keeps the last notified state, so repeats are suppressed. (2) `Startup::settle` sends *plan ready* when a planning job completes.
- **Delivery.** The system `curl` (macOS built-in, Windows 10+ `curl.exe`) is started without a shell; URL, headers and body go through `--config -` on stdin, so the secret never appears in a process list. `--fail --max-time 10 --retry 2`; a background thread waits and records failures to the run's audit store; errors never change the workflow. No new Rust dependency, no resident process. `AGENTCONDUCTOR_MESSENGER_CURL` overrides the program for tests.
- **Payload.** Slack: `{"text": …}`. Company webhook: operator template (default `{"text":"{{text}}"}`) with JSON-escaped placeholders; the result must be valid JSON.
- **Deferred to phase 2 (DD-51):** inbound decisions, linked approvers, signatures, Socket Mode, OS credential store.
- Rejected: an HTTP/TLS crate (new dependency and binary size), polling from the window (stops when the window is hidden or closed, DD-71).
### DD-68 implementation constraints — 2026-10-11

The built-in Rust client uses contained stdin/stdout JSONL for local servers and short-lived system curl for Streamable HTTP JSON/SSE; there is no resident MCP daemon, model API key or new Cargo dependency. Reuse one session per server per import; DELETE remote sessions and reap local children. Initial support targets protocol 2025-06-18 (older protocol versions are accepted on stdio). Legacy HTTP+SSE endpoints and native OAuth authorization/discovery/refresh are unsupported; use an authenticated local company proxy or environment-backed remote headers. OAuth registration metadata is retained for configuration compatibility, without reading OpenCode tokens. Secret values must be environment references; URL credentials/query parameters are refused.

Each source binding is an operator assertion of an exact read-only tool; conflicting server annotations are refused. Only initialize, initialized notification, tools/list and the bound tools/call are sent. Server requests for sampling/elicitation/roots are not answered. Maximums: 8 servers, 12 references, 128 tools/4 catalog pages, 256 KiB response, 2 MiB session output, 16 KiB per source, 48 KiB combined body and two minutes per import. Source URL/title/version/update metadata is optional and captured only when supplied; missing metadata is not invented. Arbitrary prose numbers remain prose; explicit source controls confirm IDs.

Workspace snapshots request large MCP results only once for a completed operation. Applying selected evidence invalidates a reviewed plan, creates immutable private snapshots and a local wiki provenance index, and audits IDs/status/digests without document bodies. Planning stores approved context separately from existing-plan documents and labels it untrusted evidence. Execution rechecks the exact applied evidence. Portable settings retain connection definitions and bindings, but no import receipts or original document bodies. Company-server authentication and read-tool behavior still require operator-supplied live acceptance.


DD-68 source-binding amendment (2026-10-13): discovered tool rows provide explicit Jira/Confluence binding editors with schema-derived identifier templates and editable required arguments. Preview identifies missing document-reader bindings, opens configuration, and never implicitly calls an unselected tool. Registered-server discovery remains independent; existing bindings are preserved and stale catalogs refused.

DD-68 UI amendment (2026-10-11): configure MCP connections through the top-bar Settings menu; keep reference insertion and preview beside the goal. The Settings gear uses a 26 px glyph in a 32 × 32 px pointer target.


## DD-78 Windows launchers, ConPTY and provider badges
Prefer platform launchers over extensionless npm shell scripts. Resolve quota executables at launch so existing profiles recover without re-registration. Use Windows ConPTY with a contained job, bounded output and deadline for Claude interactive /usage; preserve macOS collector. Original provider monogram badges fill installed-artwork gaps; retain installed artwork when available, correct legacy zero alpha. No vendor images are bundled. Trace: R-72 → BG-28 → SF-60 → UC-60 → FR-56/NFR-28 → QAS-65.


## DD-80 Operator catalogs for custom CLIs without model discovery
Keep custom discovery unsupported unless a real CLI contract exists. Models accepts newline-separated IDs or observed JSON, tags manual entries as operator-configured, records configuration time and marks CLI version unavailable. Existing IDs populate the editor. Reuse catalog persistence and role/session selectors; {model} binding remains required. No schema/dependency/HTTP transport change. Trace: R-73 -> BG-29 -> SF-61 -> UC-62 -> FR-57/NFR-29 -> QAS-67.

## DD-81 Messenger decisions, phase 2 (decides the inbound part of DD-51)

Trace: R-43 -> BG-18 -> SF-37 -> UC-35/UC-63 -> FR-58/NFR-30 -> QAS-36/QAS-68 -> DDR-14.

- **Where.** `src/messenger/inbound.rs`. `startup-run` starts one thread for a linked run when `settings.decisions` is on; it wakes every 15 s, reads the run's tool ledger, and does network work only while a request is `pending`. It ends with the run process, so stop-and-resume requests (run process exited) are answered in the desktop as before.
- **Announce.** New pending request IDs get one message (task, tool, code); IDs are remembered in `<run>.messenger-inbound.json` with the reply cursor.
- **Slack.** `GET https://slack.com/api/conversations.history?channel=<id>&oldest=<cursor>` with `Authorization: Bearer <bot token>` through the same curl stdin config. Messages with `bot_id`/`subtype` are ignored; `user` must be in `approvers` (Slack sets `user` server-side).
- **Company relay contract.** `GET <relay URL>?since=<cursor>` → `{"decisions":[{"code","decision":"allow|deny","user","ts"}…],"signature"…}` per decision `signature = hex(HMAC-SHA256(secret, code|decision|user|ts))`; HMAC is computed with the existing `sha2` crate (RFC 2104), compared in constant time.
- **Apply.** The code must match exactly one pending, unexpired receipt; then `current_exe tool-approve|tool-reject <run> <id>` runs the unchanged DD-32 path, the run's audit store records `messenger_decision` with approver and message ID, and a confirmation is posted.
- **Settings.** `decisions`, `slack_channel`, `approvers` in `Config.messenger` (exported); bot token, relay URL and relay secret in `messenger-secrets.json`.
- Rejected: Slack Socket Mode / Events API (WebSocket or a public HTTPS endpoint and a new dependency), approving plan consent or merge remotely.

## DD-82 Automatic slug naming with Hangul romanization

Trace: R-74 -> BG-12 -> SF-46 -> UC-64 -> FR-59 -> QAS-69.

`slugs::suggest` splits the goal into words; each ASCII alphanumeric run is lower-cased; each Hangul syllable (U+AC00–U+D7A3) is decomposed arithmetically into initial, medial and final indices and mapped with the Revised Romanization tables (`g kk n d tt r m b pp s ss '' j jj ch k t p h`; finals in their unreleased form `k n t l m p ng`…). Assimilation and liaison rules are not applied — the result is a stable folder name, not a pronunciation guide. Words are joined by `_`, the total is cut at 48 characters on a word boundary, and an empty result becomes `goal_<yyyymmdd>` (local date). `start_planning` always names new slugs itself; the `slug` request field and the step-1 input are removed, and Goals & progress shows the slug. Rejected: asking the coordinator for a name (extra model call before planning) and transliteration crates (new dependency).

## DD-83 Enterprise allocation metadata
Add optional backward-compatible budget observations to quota snapshots/views. Parse the installed Codex 0.162.1 generated protocol individualLimit (limit, used, remainingPercent, resetsAt) and credits (balance, unlimited) without HTTP/API keys. Prefer actual remaining/allocated quantities when both exist; retain reported percentage and scope as evidence. Exact bounded decimal subtraction avoids rounding money/credit values. Missing allocation remains unknown; nonempty multi-bucket metadata remains authoritative with empty-map legacy fallback. Claude Enterprise output requires observed CLI evidence rather than guessed screen labels. Trace: R-75 -> BG-30 -> SF-62 -> UC-65 -> FR-60/NFR-31 -> QAS-70.


## DD-84 Desktop window ownership
Trace: R-76 -> BG-31 -> SF-63 -> UC-66 -> FR-61/NFR-32 -> QAS-71. Replace external macOS AppleScript pickers with NSOpenPanel scheduled on AppKit main queue. Activate the owning app and make the picker key before running its modal loop; preserve cancellation and UTF-8 paths. Compile Windows executable with GUI subsystem; attach an existing parent console for CLI invocation without allocating one. Centralize ordinary background std commands with CREATE_NO_WINDOW and add the flag to suspended Job Object subprocesses. Windows picker helpers receive the foreground app HWND captured before spawning and implement IWin32Window, replacing an unrelated invisible TopMost form. Keep ConPTY separate. No additional runtime or dependency.


## DD-85 Directory validation at save; baseline validation at planning
Trace: R-77 -> BG-32 -> SF-64 -> UC-67 -> FR-62/NFR-33 -> QAS-72. Settings canonicalize an existing directory without Repository::open. Planning preflights the Git root and HEAD before slug creation, with explicit guidance rather than raw rev-parse errors. Manual planning and baseline status share this guard. Quota keeps its independent workspace cache and subprocess collection. Non-Git execution and operator-approved initialization are separate possible product changes; neither is silently introduced.

DD-85 quota follow-up: use a separate transient Controller/job ledger for workspace quota, so it cannot occupy the planning slot or disable workspace editing. Reject duplicate refreshes without advancing another provider cooldown. Serialize quota collection to one worker, retain ownership/deadlines and cache provenance, and poll it alongside planning without introducing a resident service.


## DD-86 One asynchronous metadata lane
Trace: R-78 -> BG-33 -> SF-65 -> UC-68 -> FR-63/NFR-34 -> QAS-73. Use the existing transient metadata Controller for both catalogs and quota, a bounded persisted FIFO and isolated profile snapshots per catalog. Foreground planning owns a separate slot. Metadata job state and non-executable catalog observations do not alter edit revision; execution settings and exact reviewed plan still bind actions. Merge observed catalogs into only the matching current profile and preserve registration changes. Open edit forms synchronously and resolve executables in a guarded Promise; retain operator typing. Reuse installed CLIs and their login; no API keys, daemon or new dependency.


## DD-87 Explicit empty Git baseline

Status: unreleased prototype superseded by DD-88. Its initialization API/UI is removed; ordinary-folder operation is the selected product flow.
Trace: R-79 -> BG-34 -> SF-66 -> UC-69 -> FR-64/NFR-35 -> QAS-74. Offer a dedicated native-page consent dialog rather than infer initialization from worktree settings. Distinguish missing root/HEAD from nested/inaccessible repositories. On approval, git init only when needed; create an empty tree via hash-object and commit-tree with no parent, then compare-and-set HEAD through update-ref. This preserves staged changes, unlike commit --allow-empty, which could include the index. Existing file changes use the separate established baseline-file consent. Write a local approval receipt, refuse existing HEAD and keep final plan/run/merge consent separate.


## DD-88 Ordinary folders use private change history

Superseded by DD-133 for new execution. Read-only planning and legacy private-copy runs remain supported.
Trace: R-80 -> BG-35 -> SF-67 -> UC-70 -> FR-65/NFR-36 -> QAS-75.

For a selected folder without a committed Git root, create a unique bounded working copy under the workspace's folder-history directory. Initialize Git only there and bind its base to the original file fingerprints. Planning gets a disposable read-only export; execution reuses the existing shared/worktree engine. Preserve the original Git index and files, including an unborn repository. Final approval binds the folder origin and snapshot; Apply changes validates the original again, exports the verified candidate, retains backups and a durable receipt, writes files atomically and rolls back applied files on recoverable failures. An interrupted application receipt refuses automatic replay and identifies the backups for operator investigation. Exclude .git, .agentconductor, node_modules, target, venv/.venv, __pycache__, .ssh, .aws, .env/.env.* and OS metadata; reject aliases, hard-linked files and oversized snapshots. Git remains a documented runtime dependency; worktree selection controls execution isolation, not repository prerequisites. No provider HTTP, API key, daemon or new crate is introduced. Reject: silently initializing the user folder; an independent patch engine duplicating the mature approval/test path in the first fix.

DD-86 / DD-88 review amendment: settle metadata errors as durable failed observations without propagating them through every dispatch. Preserve child ownership when failure occurs while a collector is live. Prune superseded unstarted snapshots and safe orphans at planning entry; protect unknown/live ownership and all report-referenced copies. After approved folder application, discard working files but keep Git objects and run evidence; knowledge reads completed results from the original. Candidate staging uses a scope guard; rollback failures retain the initial error and backup path. Trace: QAS-76.


## DD-89 Static agent tools guide in Settings
Trace: R-81 -> BG-36 -> SF-68 -> UC-71 -> FR-66/NFR-37 -> QAS-77.

Add one button to the existing Settings dialog that opens a static dialog of commands, reusing the existing dialog, eyebrow and baseline-os styles and the existing i18n walker, which already skips code elements. Commands were checked against each project's README on 2026-10-12. Reject: installing or detecting tools from the app (adds subprocesses, privilege and platform logic for optional third-party software), external links (the embedded page does not navigate), and a separate help page (duplicates the manual).


## DD-90 UTF-8 picker output on Windows
Trace: R-82 -> BG-37 -> SF-69 -> UC-72 -> FR-67/NFR-38 -> QAS-78.

Windows PowerShell 5.1 writes [Console]::Out in the OEM code page, and with the UTF-8 system locale it can emit a BOM; either turns a picked non-ASCII path into an invalid one while typing works. Set [Console]::OutputEncoding to UTF8Encoding without BOM inside the shared owner script and strip a leading BOM in one parsing function. Reject: Base64 transport (needs a decoder for no benefit once the encoding is fixed) and lossy decoding (would save a wrong path silently).


## DD-91 Truthful planning start and activity
Trace: R-83 -> BG-38 -> SF-70 -> UC-73 -> FR-68/NFR-39.

The 15 s default bridge timeout fired while planning entry copied an ordinary folder, although the job then started. Give planning-entry actions a 300 s limit instead of making entry asynchronous (a larger change to the proposal lifecycle). Report activity from the coordinator's existing stdout/stderr files by metadata only. Under reduced motion keep an opacity-only blink, which signals liveness without movement. Reject: parsing coordinator output for percentages (CLI formats differ and expose content).


## DD-92 Coordinator names non-English goals
Trace: R-84 -> BG-39 -> SF-71 -> UC-74 -> FR-69/NFR-40 -> QAS-80. Supersedes the romanization of DD-82.

The coordinator already reads the goal to plan it, so ask it for an English snake_case slug in the same JSON result, validate it with slugs::validate, and rename the date placeholder folder when the plan becomes ready. Reject: romanization (not English, user rejected), a bundled dictionary or translation API (dependency, network, cost).


## DD-93 Enterprise allocation as a percentage
Trace: R-85 -> BG-40 -> SF-72 -> UC-75 -> FR-70/NFR-41 -> QAS-81. Amends DD-83's display.

Show the share remaining as N%/100% so it reads like the other windows; keep raw remaining/allocated in the tooltip. Credit balances without an allocation have no meaningful percentage and show —.


## DD-94 Windows Claude collector: plain working folder, explicit blockers
Trace: R-86 -> BG-41 -> SF-73 -> UC-76 -> FR-71/NFR-42 -> QAS-82.

Canonical Windows paths are \\?\-prefixed; cmd.exe (used for claude.cmd) rejects them and silently runs in System32. Reuse git::git_path, which already converts such paths for Git. The reported screen was Claude Code's first-run theme selection; answering it would change the operator's CLI configuration, so the collector stops with an instruction instead. Reject: auto-selecting a theme or other first-run answers.

DD-91 amendment (v0.1.7): the user still saw "Native response timed out" while planning. Planning entry holds the settings lock while copying the folder, so the concurrent 1 s snapshot poll waited and hit the 15 s default. While an action is in flight the poll uses the same 300 s limit; only one poll runs at a time, so requests do not accumulate. Trace: R-83, QAS-79.


### DD-95 Readable Windows project paths
Trace: R-87 -> BG-42 -> SF-74 -> UC-77 -> FR-72 -> QAS-83 -> DD-95.
Canonicalization must not expose extended drive/UNC prefixes in project fields. Preserve roots, Unicode and actual directory identity.
Reuse git::git_path at the validated project/picker boundary; retain canonical paths for internal containment checks.


### DD-96 Reliable named workspace creation
Trace: R-88 -> BG-43 -> SF-75 -> UC-78 -> FR-73 -> QAS-84 -> DD-96.
Create by button or Enter; wait asynchronously for admitted metadata without losing input or starting repeated queries; failures leave the current workspace usable.
Serialize one UI creation attempt, explicitly submit Create on Enter, and wait for existing metadata with bounded asynchronous polling.


### DD-97 Delete an unwanted workspace
Trace: R-89 -> BG-44 -> SF-76 -> UC-79 -> FR-74 -> QAS-85 -> DD-97.
Confirm exact saved workspace name/path, refuse live or unknown execution, archive only app-owned workspace data and preserve project sources. The current workspace must be switched away first.
Move a confirmed idle app-owned workspace to .agentconductor/workspace-trash using rename; retain recovery evidence, never remove project directories.


### DD-98 Register MCP servers in Settings
Trace: R-90 -> BG-45 -> SF-77 -> UC-80 -> FR-75 -> QAS-86 -> DD-98.
Settings registers arbitrary local/remote MCP servers in OpenCode format. Jira/Confluence document import is a separate optional binding, not the name of server registration.
Rename registration controls and translations; retain generic MCP definitions and optional document bindings.

DD-96 amendment: successful reaped startup command records must not probe a historical PID as current ownership. Preserve checks for live, failed, malformed and unknown records. Actual macOS failure was PID reuse by mediaanalysisd; no unrelated process was killed. Creation keeps its title and inline failure feedback until success.


### DD-99 Await profile saves and retain inline errors
Trace: R-91 -> BG-46 -> SF-78 -> UC-81 -> FR-76/NFR-47 -> QAS-87 -> DD-99.
Edit Claude, choose Codex as fallback and press Use profile. Await the validated native save and close only after success. Invalid JSON, cycles, native/revision errors and an already admitted action leave the dialog open with a readable inline message. Save is asynchronous and deduplicated; private environment values remain unchanged. Profile execution changes invalidate an earlier plan approval.


DD-94 / DD-26 amendment (PR #1): validate nonempty cwd before creating collector logs or spawning macOS/Windows Claude processes. Normalize named weekly labels independently of seven_day so later model entries cannot overwrite the all-model window. Retain readable weekly model tooltips. Surface the app-server ChatGPT-authentication error without adding API integration or credentials.


DD-08 / DD-29 clarification (2026-10-12): CLI interfaces expose agent execution sessions; Rust mediates their context and results. Local SQLite remains authoritative for task/session state, checkpoint instructions and approvals; Markdown remains the reviewable knowledge asset. agentmemory is an optional replica/recall integration. General peer signals can be implemented as a durable addressed inbox/outbox, but current checkpoint commands do not constitute a general A2A endpoint or a cross-machine message service. Receipt is not acceptance, and neither grants execution permission. See [session communication](../knowledge/wiki/session-communication.md) for source evidence, capability-dependent delivery and proposed verification.

Direct-direction clarification: the orchestrator originates instructions; hooks are one possible delivery boundary, not the authority or required storage. Current one-shot stdin closes after initial input, and Claude permission callbacks are not generic prompt steering. Add live direction only through a verified provider control adapter; retain checkpoint delivery or quiescent resume/new-attempt handoff otherwise. A provider session ID alone cannot deliver input to a running process.


### DD-100 Rust-owned signal routing with selectable memory preference
Status: implemented (cooperative::route): local app-owned recipients only; remote authenticated routes are not implemented, so no remote delivery is ever claimed. Use workspace settings modes Local Rust and agentmemory preferred, defaulting to Local Rust when unconfigured. Keep the durable inbox/outbox in SQLite and record preference separately from effective route. agentmemory REST or memory/signal MCP is an optional delivery adapter; no memory reply is execution authority.
Persist first, then attempt bounded delivery. Fail the preferred route within eight seconds; transition eligible local delivery to Rust while retaining exact message/recipient identity. Keep one external operation in flight per workspace. Acknowledge idempotently; reconcile late success after fallback rather than repeat the instruction. Invalid/stale identity or a replaced session remains rejected, not silently retargeted. Reuse operation outcomes and bounded retry/backoff for recovery; new messages may use the restored preferred adapter, acknowledged messages never replay.
Expose preference and effective transport/failure in Settings and briefing. Apply the new preference to subsequent attempts without resetting existing message identity. Local adapters must honor verified direct control, cooperative checkpoint or explicit quiescent handoff. Rust routing cannot create steering capability. Remote sessions need their own authenticated route; local fallback cannot claim remote delivery. Optional-service failure does not authorize tools, deployment, secrets, reassignment or merge. No Python orchestration framework or service is introduced.
Trace: R-92 -> BG-47 -> SF-79 -> UC-82 -> FR-77/NFR-48 -> QAS-88 -> DD-100.


### DD-101 Read the running app version
Show the compile-time Cargo package version beside the AgentConductor title in both Workspace and Agent Room, using small readable text. Never hard-code a release number or query a server.
Trace: R-93 -> BG-48 -> SF-80 -> UC-83 -> FR-78 -> QAS-89 -> DD-101. Source: user request, 2026-10-12.


### DD-102 Bound and clear disposable caches
Automatically bound completed disposable metadata/catalog/query data and remove unused private snapshots when safe. Add Settings -> Clear cache with a preview of eligible paths/bytes and explicit confirmation that removed cache cannot be referenced afterward. Protect current/referenced/live/unknown data, workspace settings, project sources, reviewed plans, audit/diffs/tests and recovery evidence. No periodic cleanup daemon; reuse lifecycle boundaries and run manual scans off the UI thread.
Trace: R-94 -> BG-49 -> SF-81 -> UC-84 -> FR-79 -> QAS-90 -> DD-102. Source: user request, 2026-10-12.


### DD-103 Abandon a blocked planning workspace
Allow confirmed deletion of the current app-owned workspace after Generate plan fails or stalls. Offer Stop planning and delete: request cooperative cancellation, await owned-process quiescence and metadata settlement, then switch the same window to a fresh workspace and archive only the abandoned workspace app data. Never kill unrelated/reused PIDs or erase the project. Refusal must preserve the current workspace and expose the reason. Active implementation runs retain explicit Stop/recovery requirements.
Trace: R-95 -> BG-50 -> SF-82 -> UC-85 -> FR-80 -> QAS-91 -> DD-103. Source: user request, 2026-10-12.


### DD-104 The workspace is the agent sandbox; the coordinator decides beyond it
Agents coordinate and develop inside the plan without waiting on routine approvals. Permission capability follows from the CLI (standard Codex template: workspace-write sandbox; standard Claude template: workspace policy callbacks; otherwise unverified) and is no longer chosen in the profile dialog. A guarded Claude command, search or read confined to the task workspace runs at once; one reaching outside (network, installs, history-changing Git, privilege, other processes, paths outside the workspace, secrets, agent metadata) is decided by the workspace coordinator , read-only and bounded to 300 s. A person is asked only when no coordinator is configured or it gives no valid answer. Decision time does not count toward the task timeout, and a task that still times out becomes blocked and is retried by Resume instead of failing. The command check is textual and conservative, not an operating-system sandbox.
Trace: R-96 -> BG-51 -> SF-83 -> UC-86 -> FR-81 -> QAS-92 -> DD-104. Source: user request, 2026-10-13.

### DD-105 Usage query accepts the Claude folder-trust prompt
The terminal collector recognises the trust screen ("Permission Required:", "Do you trust the files", "Enter y/n", "Please answer y or n"), answers with Enter or y as the screen asks, re-reads only output after its answer, and gives up after two answers. Supersedes the DD-94 rule that never answered the trust prompt.
Trace: R-97 -> BG-52 -> SF-84 -> UC-87 -> FR-82 -> QAS-93 -> DD-105. Source: user request, 2026-10-13.

### DD-106 Planning start and workspace naming feedback
The startup page tracks a local planning-start time and renders it until the prepare request returns; Save refreshes the titlebar label; delete_workspace takes an optional replacement name (trimmed, empty falls back to "New workspace").
Trace: R-98 -> BG-53 -> SF-85 -> UC-88 -> FR-83 -> QAS-94 -> DD-106. Source: user request, 2026-10-13.

### DD-107 Run and task status at a glance
One status-to-tone map (ok, bad, run, wait, idle) drives the banner, run label, task pill and session rows through CSS custom properties; status words are capitalised and translated; Resume switches from quiet to primary when enabled and is mirrored in the banner.
Trace: R-99 -> BG-54 -> SF-86 -> UC-89 -> FR-84 -> QAS-95 -> DD-107. Source: user request, 2026-10-13.

### DD-108 Goal folders directly in the project folder
slugs::root(repository, workspace, name, slug) returns <repository>/<slug>; the workspace name is only used to find the interim layout. Each goal folder carries a marker file that keeps it out of folder copies, /<slug>/ is added to .git/info/exclude, slug creation avoids names already on disk, and Start refuses task scopes inside any goal folder. Correction 2026-10-13: the operator meant the project folder, not the workspace display name.
Trace: R-100 -> BG-55 -> SF-87 -> UC-90 -> FR-85 -> QAS-96 -> DD-108. Source: user request, 2026-10-13.

### DD-109 Agent CLI list is a master list
The master list is ~/.agentconductor/agents.json (ordered profiles). App-owned workspaces under ~/.agentconductor load it on open and write it on every agent-list save; the first open seeds it from that workspace. Workspace references are not dropped on open; the startup page marks unresolved coordinator and role entries and blocks planning until they are reassigned. Workspaces outside the app folder keep their own list.
Trace: R-101 -> BG-56 -> SF-88 -> UC-91 -> FR-86 -> QAS-97 -> DD-109. Source: user request, 2026-10-13.

### DD-110 Reorder agent cards
Cards are HTML5 draggable with a visible handle; a drop reorders draft.profiles and saves through save_profiles (order is the list order, already persisted). Option lists are built from draft.profiles, so they follow automatically.
Trace: R-102 -> BG-57 -> SF-89 -> UC-92 -> FR-87 -> QAS-98 -> DD-110. Source: user request, 2026-10-13.

### DD-111 Readable file paths everywhere
Display-layer normalisation: a shared helper rewrites the prefixes in every rendered text node and in title/aria-label attributes of all desktop windows (startup, Agent Room, MCP), and messenger text is normalised before sending. Data values are not rewritten, so the engine still receives the exact paths it issued.
Trace: R-103 -> BG-58 -> SF-90 -> UC-93 -> FR-88 -> QAS-99 -> DD-111. Source: user request, 2026-10-13.

### DD-112 Activity tab reads like the CLI
The Activity renderer maps provider events to transcript entries (message, tool call, tool result, error, status) per provider; unknown events collapse to a short type label. The Raw logs tab keeps the original JSON.
Trace: R-104 -> BG-59 -> SF-91 -> UC-94 -> FR-89 -> QAS-100 -> DD-112. Source: user request, 2026-10-13.

### DD-113 Coordinator final step in the task list
The room derives the final step from the run report (task statuses, integration test results, run state) and renders it after the task rows with the shared status tones; selecting it loads the candidate review. No new engine state is needed.
Trace: R-105 -> BG-60 -> SF-92 -> UC-95 -> FR-90 -> QAS-101 -> DD-113. Source: user request, 2026-10-13.

### DD-114 Decide a live tool request from its window
The window uses the Tool requests list's rule (enabled when the run is idle or running, since the paused worker belongs to the running job) instead of disabling while any UI job runs; a per-window flag prevents double submission.
Trace: R-106 -> BG-61 -> SF-93 -> UC-96 -> FR-91 -> QAS-102 -> DD-114. Source: user request, 2026-10-13.

### DD-115 Coordinator decides routine requests and records each decision
Coordinator profile falls back to the coordinator in the run's workspace startup.json; the prompt prefers allow with an explicit deny list; one retry; the audit record carries input, attempts and time and is returned with task details for Activity. The command policy checks protected names on the workspace-relative remainder and recognises the workspace root in any spelling.
Trace: R-107 -> BG-62 -> SF-94 -> UC-97 -> FR-92 -> QAS-103 -> DD-115. Source: user request, 2026-10-13.

### DD-116 Task list without account usage
Session rows drop the quota controls; the inspector remains the single place for account capacity.
Trace: R-108 -> BG-63 -> SF-95 -> UC-98 -> FR-93 -> QAS-104 -> DD-116. Source: user request, 2026-10-13.

### DD-117 The coordinator is selectable and shows its work; one name
A coordinator request returns the run's audit history with summaries (excluding per-event agent progress), and integration test stdout/stderr tails. The room keeps a coordinator-view flag beside the selected task index; poll reloads whichever view is selected. Engine identifiers keep their names; only interface and document wording change.
Trace: R-109 -> BG-64 -> SF-96 -> UC-99 -> FR-94 -> QAS-105 -> DD-117. Source: user request, 2026-10-13.

### DD-118 Failure cause and the next action on a failed run
The public run report carries the last 2 KiB of a failed integration test's output; the banner and the coordinator view read it. The drafted direction goes through the existing targeted revision path, which already continues a stopped run.
Trace: R-110 -> BG-65 -> SF-97 -> UC-100 -> FR-95 -> QAS-106 -> DD-118. Source: user request, 2026-10-13.

### DD-119 The next step is always shown
A state-to-step table in the room maps run state and whether the current evidence is reviewed to a sentence and an action that triggers the existing control (review load, Approve, Apply/Merge, workspace return); the existing confirmation dialogs still apply.
Trace: R-111 -> BG-66 -> SF-98 -> UC-101 -> FR-96 -> QAS-107 -> DD-119. Source: user request, 2026-10-13.

### DD-121 Plan review and run completion are organised by phase
The run's phase link resolves a moved goal folder (direct <project>/<slug>, or a child folder holding <slug>/plan/plan.json) and rewrites the link; sync runs when the workspace and the Agent Room open. The room snapshot carries the phase status (linked phase, all phases with stages, current phase); the startup page renders a phase list above the tasks from the active goal's progress.
Trace: R-112 -> BG-67 -> SF-99 -> UC-102 -> FR-97 -> QAS-108 -> DD-121. Source: user request, 2026-10-13.

### DD-120 A blocking verification finding fails integration verification
After the task tests and the tree check, changed Markdown files are scanned by a conservative rule (heading naming blocking but not non-blocking, list items not none; or a count above zero). A finding is recorded as a failed test evidence entry (stderr holds the report path and findings) attributed to the task owning a path the finding names, else the report's owner.
Trace: R-113 -> BG-68 -> SF-100 -> UC-103 -> FR-98 -> QAS-109 -> DD-120. Source: user request, 2026-10-13.

### DD-122 The run's phase beside the workspace name
The room snapshot's phase status carries a description taken from phase/<id>.md (first prose line, else its task bullets, at most 200 characters); the shared titlebar and the standalone room titlebar render it; leaving the room hides it.
Trace: R-114 -> BG-69 -> SF-101 -> UC-104 -> FR-99 -> QAS-110 -> DD-122. Source: user request, 2026-10-13.

### DD-123 Readable Agent Room text and agent icons sized to their names
Room font sizes are scaled by 1.2 up to 14 px (minimum 11 px) and by +2 px above; startup sizes below 11 px become 11 px; icon rules override the task-list grid and avatar sizes.
Trace: R-115 -> BG-70 -> SF-102 -> UC-105 -> FR-100 -> QAS-111 -> DD-123. Source: user request, 2026-10-13.

### DD-124 Task cards in Review the plan fold to one line
Each task card is a details element whose summary holds the heading, the derived one-line description and the Remove button; fields stay inside the same card element, so edit collection is unchanged.
Trace: R-116 -> BG-71 -> SF-103 -> UC-106 -> FR-101 -> QAS-112 -> DD-124. Source: user request, 2026-10-13.

### DD-125 Review the whole phased plan up front
slug_progress::briefing reads plan/plan.json (phases with specs) and plan/analysis.md; a phase's briefing is the analysis list item naming it, else its phase file; the active goal in the startup snapshot carries it; the plan review renders a details element per phase.
Trace: R-117 -> BG-72 -> SF-104 -> UC-107 -> FR-102 -> QAS-113 -> DD-125. Source: user request, 2026-10-13.

### DD-127 A run survives closing or losing the desktop app
Launch the run coordinator outside the app's Windows Job Object (CREATE_BREAKAWAY_FROM_JOB plus its own process group; the app's job must allow breakaway) and detached from the app's process group on Unix; keep workers in the coordinator's KILL_ON_JOB_CLOSE job so a dead coordinator cannot orphan them. The app reattaches through the existing run directory, owner records and locks. The Agent Room footer text claiming the coordinator keeps running becomes true on Windows.
Trace: R-118 -> BG-73 -> SF-105 -> UC-108 -> FR-103 -> QAS-114 -> DD-127. Source: user request, 2026-10-13.

### DD-128 Interrupted runs recover and resume automatically
Startup open and Room open call a liveness check: report state running, no live coordinator lease holder, and every recorded owner pid absent (or reused, never killed). Then run workflow::recover, record auto_recovery in the audit, and start resume as a UI job when the workspace setting auto_resume (default true) is on and the consecutive automatic-resume counter is below 2. The interrupted attempt's prompt gets an interruption note; session reuse follows the existing session_established path.
Trace: R-119 -> BG-74 -> SF-106 -> UC-109 -> FR-104 -> QAS-115 -> DD-128. Source: user request, 2026-10-13.

### DD-129 Interruptions and silent workers are visible
room.js transcript() merges adjacent system/thinking_tokens events; the room snapshot carries per-task last-output time (task_activity document observed_at_ms) and owner liveness; the banner reads auto_recovery audit records. Blocked on Codex's i18n migration of room.js; new strings go through the message catalogs.
Trace: R-120 -> BG-75 -> SF-107 -> UC-110 -> FR-105 -> QAS-116 -> DD-129. Source: user request, 2026-10-13.

### DD-130 Reassign tasks that have not started while the run is working
The Room action for Reassign, when the run is running and the task is pending, records a durable reassign request (run directory, atomic file, audited) without the run lock, like live tool decisions; the scheduler applies a pending request (plan task agent and model, no handoff needed because no work exists) immediately before starting that task and deletes it. The Assign button is enabled for pending tasks during a run.
Trace: R-121 -> BG-76 -> SF-108 -> UC-111 -> FR-106 -> QAS-117 -> DD-130. Source: user request, 2026-10-13.

### DD-131 Agents run with the highest permission by default
New permission capability full_access with pinned templates: Claude -p --output-format stream-json --verbose --permission-mode bypassPermissions (user MCP configuration kept), Codex exec - --json --dangerously-bypass-approvals-and-sandbox. derive_permission raises the standard templates to full_access; validate_permissions accepts the bypass flags only in exactly these templates; coordinator::read_only_args maps full_access back to the read-only template.
Trace: R-122 -> BG-77 -> SF-109 -> UC-112 -> FR-107 -> QAS-118 -> DD-131. Source: user request, 2026-10-13.

### DD-132 The coordinator resolves verification failures
workflow::repair(dir, timeout, report) runs after run, run_reviewed, resume, resume-due and ui-resume. While the report is repairable (state failed, error starting with integration, every task completed) and review_round < 3, it reviews with apply=true as the run's recorded coordinator (orchestrator_profile: the profile beside the run, else the workspace coordinator), and on a revise verdict resumes with resume_due; accept or blocked verdicts and review errors end the loop with the report as saved. After three rounds review_error carries the reason. The review prompt tells the coordinator to resolve integration failures by revising owning tasks and to defer unresolvable findings through the verifying task.
Trace: R-123 -> BG-78 -> SF-110 -> UC-113 -> FR-108 -> QAS-119 -> DD-132. Source: user request, 2026-10-13.

### DD-133 Direct folder execution with external project history
Supersede DD-88 execution copies for new ordinary-folder runs; retain its read-only planning snapshots and legacy application compatibility. Clone the reviewed baseline without checkout into external project-history keyed by canonical source identity. Repository operations pass explicit Git metadata and work-tree paths; never add `.git` to the user folder. Codex invocations in direct folders and read-only exported snapshots add `--skip-git-repo-check`, preserving permission/model/session flags and the coordinator read-only sandbox. Persist the baseline commit before worker launch. Shared tasks write to the selected source root. Accept advances only external history; Reject journals an inverse application of the baseline after exact candidate-file validation, preserving concurrent edits. Completed-worker checkpoints and an active-owned-path notice warn about external edits without pretending to identify the writer. Explicit worktree mode retains isolation, and existing Git repositories retain their prior flow. No provider API or new dependency is introduced.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user workspace-path requirement and Claude review, 2026-10-13.

### DD-133 Deleting workspaces needs no replacement name
Room::delete_workspace switches to the first entry of the recent registry that is still a saved workspace, else any other saved workspace, else creates one named native.new_workspace. The delete dialog has no name field; the optional replacement field only names that empty fallback.
Trace: R-124 -> BG-79 -> SF-111 -> UC-114 -> FR-109 -> QAS-120 -> DD-133. Source: user request, 2026-10-13.

### DD-134 Reuse Markdown and existing package asset selection
Keep two Markdown manuals with literal command/JSON identifiers unchanged. Reuse README-linked asset packaging; add no documentation framework or dependency. Capture the actual native app, mark offline fixture screenshots explicitly, and distinguish full-access from guarded tool gates.
Trace: R-125 -> BG-80 -> SF-112 -> UC-115 -> FR-110 -> QAS-121 -> DD-134. Source: user release/manual requests, 2026-10-14.

### DD-135 The app updates itself from GitHub releases
src/update.rs queries https://api.github.com/repos/neodamul/AgentConductor/releases/latest with curl, picks agentconductor-<target>-*.zip and SHA256SUMS, downloads them to ~/.agentconductor/updates/<tag>, verifies SHA-256 and unpacks with tar (Windows) or ditto (macOS). Self-replacement: the running executable (or the enclosing AgentConductor.app) is renamed to *.old, which Windows allows for a running file, the new one is put in its place (restored on failure), the same executable is started with AGENTCONDUCTOR_UPDATE_WAIT_PID and this process exits; the new process waits for that PID and removes *.old. Room requests update_status, check_update and apply_update; apply refuses while a startup job, a run UI job, or any saved workspace run is running. startup.js checks on load and every 600 s. The CLI command update installs without restarting.
#130: src/update_source.rs keeps one optional `update_repository` preference (`owner/name` for github.com, or `https://host/owner/name`). `api_url()` maps github.com to api.github.com and any other host to `https://host/api/v3/repos/owner/name/releases/latest`; assets keep coming from that release and still land under `updates/`. Values are validated (HTTPS only, no credentials, query or fragment) before anything is saved; the checksum check is unchanged.
Trace: R-126 -> BG-81 -> SF-113 -> UC-116 -> FR-111 -> QAS-122 -> DD-135. Source: user request, 2026-10-14.


### DD-136 Durable run-bound watcher and a five-hour fallback horizon
Persist account availability deadlines and coordinator instructions in the run store. Parse an explicit Claude reset timezone, reflect the local resume time and countdown, and resume due work once under run ownership. A reviewed fallback is eligible if the reset exceeds five hours, or no reset is known and five hours elapsed since the original restriction. Reassignment removes the task reservation; Stop cancels automatic continuation; completed work never restarts. App exit does not own the watcher; reopening restores a missing watcher with an exclusive lease. Unknown zones, incomplete reset windows and profile/model changes never invent a deadline.
Trace: R-127 → BG-82 → SF-114 → UC-117 → FR-112 → QAS-123 → DD-136. Source: user, 2026-10-14.

## DD-137 Semantic color tokens with light and dark sets

Trace: R-128 -> BG-83 -> SF-115 -> UC-118 -> FR-113/NFR-49 -> QAS-124.

Every color literal in `room.css` and `startup.css` is replaced by a semantic token (`--bg`, `--surface`, `--surface-2`, `--fill`, `--line`, `--line-strong`, `--ink`, `--soft`, `--bar`, `--bar-ink`, `--accent`, `--accent-ink`, `--accent-soft`, `--accent-text`, `--ok`, `--ok-bg`, `--warn`, `--warn-bg`, `--err`, `--err-bg`, `--info`, `--info-bg`). `:root` holds the light set (design E: system-settings neutrals, blue accent, light titlebar, borderless shadowed cards, segmented step tabs) and `:root[data-theme="dark"]` the dark set (design F: the sage brand on dark surfaces). The theme preference sits beside the language in `preferences.json`; `embed()` writes `data-theme` on `<html>`, and `set_theme` saves it while the page swaps the attribute without a reload. The Agent Room session row (amended by the user the same day) leads with the task name; a 14px agent icon, the agent name and a compact 10.5px status tag share the next line, which wraps so the tag moves below instead of leaving the rail (`max-width:100%`, normal white space). Rejected: two separate stylesheets (duplicated rules drift) and following the OS appearance automatically (the user asked for an explicit choice).

### DD-138 Plans know whether the project uses Git, and verification commands that cannot work are replaced
planner.rs appends a Project version control line (Git repository or not, and no git in test_command when not). Repository::external_environment gives GIT_DIR/GIT_WORK_TREE for the external history; scheduler integration verification passes it to shared-mode test commands. coordinator::Revision gains optional test_command (bounded argv); review prompt allows it only when the command cannot work here or an agent reports it must change, never to weaken a runnable test; apply_typed_review replaces plan.tasks[i].test_command and records test_command_revised {task, from, to, reason}.
Trace: R-129 -> BG-84 -> SF-116 -> UC-119 -> FR-114 -> QAS-125 -> DD-138. Source: user request, 2026-10-14.

### DD-139 Tooltip-only reset times and available-width explanations

Update both existing quotaBadge implementations so reset information belongs to title/aria-label, never the visible percentage string. Keep the existing card and semantic color tokens; reduce badge/icon size and reserved drag-handle space. Move the workspace save explanation beneath its title, align the other introductions there, and replace the footer helper 230px maximum with flexible remaining width.
Trace: R-130 -> BG-85 -> SF-117 -> UC-120 -> FR-115 -> QAS-126 -> DD-139. Source: user, 2026-10-14.

### DD-140 Conditional port label and shared guide

Use one hidden label wrapper around the existing server port and syncSignalSettings on load/change. Settings labels use a flex column with 8px internal gap; inputs/selects have a 36px minimum height. Reuse the static tools dialog from the existing workspace menu.
Trace: R-131 -> BG-86 -> SF-118 -> UC-121 -> FR-116 -> QAS-127 -> DD-140. Source: user, 2026-10-14.

### DD-141 Custom CLIs that approve their own tools can run
New permission_mode operator_auto_approve: profile form checkbox start-self-approve; validate_profiles and validate_permissions accept it; derive_permission keeps it like read_only/external_sandbox (standard claude/codex still become full_access); share.rs LOCAL_CLAIMS includes it. Bypass-flag argv checks still apply.
Trace: R-132 -> BG-87 -> SF-119 -> UC-122 -> FR-117 -> QAS-128 -> DD-141. Source: user request, 2026-10-14.

### DD-141 amendment: no checkbox
User decision 2026-10-14 ("체크박스없이 그냥 되면 안되냐"): derive_permission gives every CLI other than the standard Claude/Codex templates, the guarded Claude callback and an operator read_only/external_sandbox claim the mode operator_auto_approve by default, matching full access by default (DD-131). The agent-form checkbox is removed and the mode is no longer a local import claim. A raw plan whose profile is still unverified is refused with a hint to save the agent list.
Trace: R-132 -> DD-141. Source: user request, 2026-10-14.

### DD-142 Reuse GitHub release body without HTML rendering

Release stores body as notes and update::status exposes it. The existing renderUpdate uses the text binding in a focusable pre element below update-notes; semantic tokens and bounded scrolling preserve readability. No Markdown/HTML renderer or additional endpoint.
Trace: R-133 -> BG-88 -> SF-120 -> UC-123 -> FR-118 -> QAS-129 -> DD-142. Source: user, 2026-10-14; next release after v0.1.18.

### DD-143 Coordinator routing with immutable Markdown and bounded summaries

MCP document preview uses the registered workspace coordinator to select the appropriate discovered read tool and schema-valid arguments from enabled server configurations. Server selection, saved configuration and tool results stay linked; changing selection or configuration never displays another server’s stale catalogue. Manual per-tool source bindings are not required in the normal UI. Without a registered/configured coordinator, show registration guidance before launching any document selection. Large Jira/Confluence documents are first preserved as immutable local Markdown with complete source metadata and original hash; the coordinator derives bounded summaries for goal planning while the original remains inspectable. Write/destructive tools, unknown servers/tools and unrelated references are rejected. Follow docs/design-guide.md semantic colors, system fonts, wrapping, and Korean light/dark checks.
Trace: R-134 → BG-89 → SF-121 → UC-124 → FR-119 → QAS-130 → DD-143. Source: user, 2026-10-14.


### DD-144 Job-scoped planning feedback and CLI opacity animation

Retrying plan generation clears the previous visible failure immediately, switches to Review, and shows the selected coordinator CLI icon pulsing with the existing agent-pulse opacity animation. No quota/status color is added. New job IDs control progress, completion and failure; retained logs remain available. CLI cards do not display the paid-fallback shortcut/tag; fallback stays configurable in Edit. Trace: R-135 → BG-90 → SF-122 → UC-125 → FR-120 → QAS-131 → DD-144. Source: user, 2026-10-14.


### DD-145 Snapshot-based review readiness and typed Claude API errors

Plans reverted to their saved values can be approved; actual workspace/plan edits and active jobs still block approval with localized reasons. Compare normalized editable task values against a saved snapshot, retaining null/default semantics and preserving the baseline across local task add/remove. Claude api_error usage_limit_reached / HTTP 429 is an Error even without type/is_error. Persist a quota restriction without inventing utilization or a reset from duration fields; explicit retry/reset timestamps bind the durable reservation and reviewed five-hour fallback. Show the usage-limit reason instead of an opaque telemetry JSON blob. Trace: R-136 → BG-91 → SF-123 → UC-126 → FR-121 → QAS-132 → DD-145. Source: user, 2026-10-14.


### DD-146 Codex reconnect notices / DD-147 bundled manual

Codex reconnect attempts with a valid n/m counter and explicit WebSocket-to-HTTP fallback announcements are visible Warning events. All raw evidence/session IDs remain; turn.failed, real authentication/quota errors, missing final output, nonzero exit and timeout remain failures. No global Codex config is silently rewritten. The shared workspace menu opens bundled English/Korean user manuals offline with headings, lists, code blocks and safe text nodes; language switches update an open manual. The guide uses model_providers.openai-https (plural) and supports_websockets=false in user-level config. Follow managed Enterprise policy.
Trace: R-137 → BG-92 → SF-124 → UC-127 → FR-122 → QAS-133 → DD-146; R-138 → BG-93 → SF-125 → UC-128 → FR-123 → QAS-134 → DD-147. User, 2026-10-14.
Official setting reference: https://learn.chatgpt.com/docs/config-file/config-reference

### DD-148 Workspace and Agent Room feedback refinements
startup.html: name-coordinator-row, repository row before the goal-field wrapper (goal-tags layer behind a transparent textarea). mcp.js: goalTags copies the textarea computed text metrics to the tag layer, tagAt/tagTitle hit-test tags on mousemove and set the textarea title from source_notes (snapshot, applied sources) or the latest preview; preview opens start-mcp-review with start-mcp-loading before requesting mcp_preview; source-status shows mcp.lookup_running with mcp-busy; failures show mcp.lookup_failed. startup.js runPhaseLabel maps run_state to state.* labels. room.js renderQuota shows budget share percentages with amounts below; room.css .session.active outline uses --accent.
Trace: R-139 -> BG-94 -> SF-126 -> UC-129 -> FR-124 -> QAS-135 -> DD-148. Source: user requests, 2026-10-14.

### DD-149 Runs keep going and stay visible
availability::transient_capacity → restriction 60 s (automatic resume, three retries); coordinator::run_json_within retries three times after 30 s. workflow::stray moves new untracked paths outside every scope (external history only, excluded names skipped) to <run>/stray-files and records stray_files_moved. room.js taskHeading shows taskLine(prompt) with the ID label. i18n.js AC_PRESS marks the clicked button (is-busy, aria-busy) while a request runs and holds it while its job (plan, start, mcp; room resume/approve/merge/recover/reject) runs. ui.rs routes requests marked "room": true to the run controller and all others to the workspace; Home keeps the controller, OpenRun replaces a stale one and returns its run, workspace switches close it; startup.js keeps the room running and restarts it only for a different run.
Trace: R-140 -> BG-95 -> SF-127 -> UC-130 -> FR-125 -> QAS-136 -> DD-149. Source: user requests, 2026-10-14.


### DD-150 Selected backend contract / DD-151 direct Windows selection

The planner receives the workspace-selected coordination backend after operator rules, and every worker receives it after its task/dependency/lesson context. Local Rust requires no agentmemory daemon or CLI/MCP tool; progress remains normal owned CLI output captured by Rust. Agentmemory preferred delivery is host-owned and retains local fallback. Optional tool installation guidance cannot require or select a transport. Only registered checkpoint_v1 workers get addressed polling/ack/report instructions and exact executable-path shell examples; no new profile capability, peer recipient or approval is fabricated.

Windows folder/document/executable pickers call IFileOpenDialog directly on the IPC worker STA with the current owner, existing localization and file filters. FOS_PICKFOLDERS selects folders; filesystem/existing-path flags preserve validation. Cancellation returns None and UTF-16 paths retain Hangul, spaces and supplementary characters before common path normalization. No per-click PowerShell process, Forms load or dynamic C# compilation. Selection only fills the field; saving still validates it. macOS picker and deterministic fixture overrides are retained. Actual UI responsiveness must be checked on the target PC; no universal latency bound is claimed.

Trace: R-141 → BG-96 → SF-128 → UC-131 → FR-126 → QAS-137 → DD-150; R-142 → BG-97 → SF-129 → UC-132 → FR-127 → QAS-138 → DD-151. User, 2026-10-14.
Source: https://learn.microsoft.com/en-us/windows/win32/shell/common-file-dialog


### Workspace transfer menu — DD-152

Use the adjacent localized menu actions Export workspace and Import workspace. The operation remains the existing reviewed workspace configuration/goal archive transfer with local secrets and verification claims handled as before; no raw execution history or extra permission is added by the label change. Native request routing is separated into ui/dispatch.rs to retain the 500-line source bound without changing dispatch semantics.
Trace: R-143 → BG-98 → SF-130 → UC-133 → FR-128 → QAS-139 → DD-152. User, 2026-10-14.

### DD-153 Generate trusted HTML and fetch it on demand

`src/manual.rs` renders the repository's Markdown subset in Rust. Committed HTML is bundled as native read-only data; `manual_document` selects only en/ko. The existing themed dialog imports the rendered body. The HTML is not agent/user-supplied; raw Markdown HTML is escaped and image paths are restricted to local PNG assets. Rust tests guard generation drift. Reject a new Markdown runtime framework and eager embedding into every workspace startup.
Trace: R-144 -> BG-99 -> SF-131 -> UC-134 -> FR-129 -> QAS-140 -> DD-153. Source: user, 2026-10-14.


### Saved-project Claude quota retry — DD-154

Saving a project folder after an empty-folder Claude usage attempt immediately schedules a fresh query. Empty project paths are excluded from automatic Claude queries; other providers retain their account refresh schedule. Folder changes invalidate only Claude scheduling context, never grant trust or execution permissions.
Trace: R-145 → BG-100 → SF-132 → UC-135 → FR-130 → QAS-141 → DD-154. User, 2026-10-14.

### Schema-constrained planning replies — DD-155

Planning requests specify an explicit JSON Schema: phases, tasks, owned paths, dependencies and command arguments remain arrays, with registered agent IDs, required fields and no unknown properties. Claude receives --json-schema; Codex receives --output-schema bound to a retained local schema file. Other CLIs receive the exact schema in the planning prompt. Strict phase/task, ownership, role, dependency and human-approval validation remains unchanged; invalid objects are not guessed or silently reordered.
Trace: R-146 → BG-101 → SF-133 → UC-136 → FR-131 → QAS-142 → DD-155. User, 2026-10-14.

### DD-156 Gate on work rather than retained UI presence

Replace the unconditional controller guard with Controller::busy plus report/task running checks. Keep startup metadata/ownership guards and close_run_after success-only cleanup.
Trace: R-147 → BG-102 → SF-134 → UC-137 → FR-132 → QAS-143 → DD-156. GitHub #4, user2026-10-14.

### DD-157 Share known prompt readiness and timeout stage

Replace duplicated dollar-character checks with one banner-plus-prompt predicate. Keep onboarding/trust checks before readiness and report the timeout stage. This addresses a reproducible collector defect; onsite Enterprise response/timing still needs operator evidence.
Trace: R-148 → BG-103 → SF-135 → UC-138 → FR-133 → QAS-144 → DD-157. GitHub #5, user2026-10-14.

DD-157 implementation note: the operator confirmed Windows Claude2.1.296 uses a > prompt and a Yes-I-trust-folder choice, with an explicit spent/allocated Enterprise line. Parse that declared allocation using existing exact-decimal subtraction; the display percentage is derived from remaining/allocated amounts. Date-only resets remain unscheduled. Committed fixtures use synthetic amounts; no operator account values are published.


### Active workspace navigation — DD-158

Create, open and switch workspaces while planning, metadata collection or execution is active. Keep the owned Startup, child handles, controller and window/run leases for each active workspace; restore that session when revisited and read its current report/logs. Settings and job descriptors already persist in startup.json; progress/evidence remains in the original run, so navigation neither stops workers nor overwrites CLI state. Idle sessions release their leases and reopen from disk as before. Serialize composite navigation, acquire the target first, and retain the original on target/registry failure. Completed parked sessions can release their leases for exact-name-confirmed deletion; active or unknown work remains protected. Ignore late workspace/closed-room responses, reset per-workspace refresh tracking, and request a fresh snapshot before editing the new view. Export/import continues to transfer the existing reviewed configuration archive; live process handles are not exported.
Trace: R-149 → BG-104 → SF-136 → UC-139 → FR-134 → QAS-145 → DD-158. User, 2026-10-15; supersedes the active-navigation restriction in DD-156.

### DD-159 Reuse slug records and enforce project ownership

Reuse Config.slugs, derived progress and existing native IPC. Add backward-compatible optional goal-local run links; imported goal archives receive no foreign run link. New/select operations retain profiles/project and clear approval/source state, while running or pending runs remain protected. Use the existing registry plus a brief lease scoped to the canonical project path so distinct projects can be saved concurrently; no service or framework. Keep legacy data intact and direct operators to the primary workspace.
Trace: R-150 → BG-105 → SF-137 → UC-140 → FR-135 → QAS-146 → DD-159. GitHub #6, user2026-10-15.

### DD-160 Replace project-owned non-Rust execution with Rust
Keep the Rust application and authenticated CLI process boundary. Migrate UI behavior, release/package/manual generation, and repository-owned checks to Rust; preserve external authenticated CLI tools as process boundaries. A framework is allowed only for a fully native Rust UI that removes the embedded JavaScript runtime and passes the existing resource/accessibility QAS. Convert one bounded path at a time; preserve every workflow and package check before removing its former implementation.
Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160. User, 2026-10-15.

### DD-161 Prototype a native Rust UI with eframe/egui
Use a bounded eframe/egui prototype to test replacement of Wry/WebView UI behavior with a single-process Rust desktop UI. Start with `default-features = false` and explicitly evaluate only `glow`, `accesskit`, `clipboard`, and system-font support; do not enable the default wgpu renderer, bundled fonts, or unrelated persistence/web features. This candidate is MIT OR Apache-2.0 and documents native AccessKit semantics and widget-tree tests. The repository now contains an optional `native-ui-probe` feature and a separate probe binary; default application and release builds do not enable or link eframe. The probe uses a one-slot Rust worker to call `Room::dispatch` off the UI thread and exposes an accessibility-tree test. Do not replace production screens until this probe proves the existing idle CPU/RSS budgets, Korean/English rendering and IME, keyboard use, screen-reader tree coverage, native window lifecycle, macOS/Windows builds, and a resource/license dependency audit. The isolated graph identifies ISC dependencies on macOS and ISC plus BSL-1.0 dependencies on Windows. Exact `libloading` 0.8.9 ISC and `clipboard-win` 5.4.1 BSL-1.0 texts are pinned by upstream commit and SHA-256 in `licenses/upstream/sources.json`; a Rust package test verifies both notices are bundled, including when the BSL text is absent from the crate archive. The release allowlist and notice now explicitly cover these permissive licenses. If any remaining UI/platform/resource gate fails, stop and reconsider the toolkit before replacing screens. Slint is not the first candidate because its runtime licensing requires either GPLv3 compatibility, royalty-free use with disclosure, or a commercial license; avoid adding that licensing condition without need.

References: [egui license and overview](https://github.com/emilk/egui#readme), [eframe features](https://github.com/emilk/egui/blob/main/crates/eframe/Cargo.toml), [egui accessibility and testing](https://github.com/emilk/egui/blob/main/docs/accessibility.md), [Slint license](https://github.com/slint-ui/slint/blob/master/LICENSE.md).

Implementation boundary: keep `Room`, `Startup`, `Controller`, validation, approvals, and leases as the Rust application authority. `Room::dispatch` already routes workspace and room requests into Rust handlers, so a native shell can call this boundary directly without a local HTTP server or WebView IPC. Preserve the separate Workspace and Agent Room windows/viewports. Any dispatch that can wait on a child process or filesystem work must leave the egui frame thread; apply completed results on the UI thread and render the same Rust snapshots. An isolated prototype's bounded single-request worker now calls `Room::dispatch` from a background thread and returns the startup snapshot; its test was observed failing before `RequestWorker` existed and passing after implementation. The native Workspace also exposes cache preview and clearing through the same worker: deletion requires the exact digest of a visible preview, while the Rust cache handler retains its protected-path checks. This proves the narrow request boundary only, not UI integration or production readiness. Replace screen behavior incrementally and remove WebView only after both surfaces retain their workflow and accessibility coverage.

Trace: R-151 → BG-106 → SF-138 → UC-141 → FR-136 → QAS-147 → DD-160 → DD-161. User, 2026-10-16.

### DD-162 Plan retry timeout and recovery
Keep plan preparation, its following snapshot, and automatic progress checks in one bounded planning operation. Preserve retry availability after an earlier failed plan and suppress stale failure feedback while a retry is preparing. Timeout guidance states that progress is checked automatically and recommends reopening the workspace or run if the view remains stale.
Trace: QAS-148.

### DD-164 Custom CLI parsing and quota configuration (#12)
Keep native Claude/Codex/OpenCode parsing and quota collection as their verified defaults. Custom profiles may declare `event_types` as raw `type` string → standard event kind, and `session_id_pointer` as a JSON Pointer. Optional `quota_command` contains direct argv, `timeout_seconds` (default 30, bounded 1–300), and optional `output_pointer`; `{workspace}` and `{agent_id}` placeholders are substituted before spawn. The process has a dedicated cwd and retained stdout/stderr logs; no shell is involved. Output is one JSON object matching `Snapshot` (`source`, positive `observed_at`, optional `plan_type`, `windows`, `budgets`, optional `restriction`), with 1 MiB and field/count bounds. The custom CLI or a local wrapper must normalize its provider-specific response to this schema; a general parser DSL is out of scope. Omitted fields preserve legacy custom parsing and disable quota polling. Shared workspace JSON export/import carries these settings without local environment values and model catalogs.

Example profile fragment:

```json
{
  "id": "aipro",
  "parser": "custom",
  "executable": "aipro",
  "args": ["--jsonl"],
  "event_types": {"working": "progress", "finished": "result"},
  "session_id_pointer": "/meta/session",
  "quota_command": {
    "args": ["quota", "--json", "--workspace", "{workspace}"],
    "timeout_seconds": 30,
    "output_pointer": "/account"
  }
}
```

The Rust quota-due snapshot and CLI refresh command support configured custom profiles. The production WebView quota controls still gate visibility to Claude/Codex; adding custom quota controls awaits the R-151 Rust UI migration. No JS/Python changes are included in this branch.

Trace: R-154 → BG-109 → SF-141 → UC-144 → FR-139 → QAS-150 → DDR-17 → DD-164. GitHub #12, 2026-10-19.

### DD-166 Keep finalization in the primary run board
Represent finalization with the existing workflow row and status banner: Verify, Merge, then Workspace. Treat a running merge/apply job as active progress even while the report is `approved`; keep the banner action visible and busy until the job settles. Preserve separate latest-diff review, approval, and final-apply confirmations. Derive status from the existing job snapshot without a new polling loop or service.
Trace: R-156 → BG-111 → SF-143 → UC-146 → FR-141 → QAS-152 → DD-166. GitHub #50, user, 2026-10-20.

### DD-167 Let SQLite bound transient writer contention
Use one ten-second busy timeout for run database connections. SQLite already retries busy operations without a new worker, retry loop, or lock protocol; a finite bound preserves responsiveness and surfaces persistent contention. Keep each transaction and its existing commit/error behavior unchanged.
Trace: R-157 → BG-112 → SF-144 → UC-147 → FR-142 → QAS-153 → DDR-18 → DD-167. GitHub #56.

### DD-168 Keep custom CLI behavior in validated profile data
Represent optional model discovery, execution option groups, result extraction, and error matching as profile data interpreted by generic Rust adapters. Execute discovery commands directly with a bounded timeout; never infer an endpoint or model command from an OpenAI-compatible interface. Resolve task overrides before profile defaults and revalidate on reassignment. Apply result extraction only to successful events so configured error events cannot become parse failures. Preserve the previous `payload` wrapper when no result pointer is configured.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62.
### DD-168 Keep custom CLI behavior data-driven
Keep the model catalog in `AgentProfile.catalog`, marking operator-entered catalogs as manual and retaining discovered catalogs as discovered. Run custom discovery only from an explicit argv configuration and map its JSON response with JSON Pointers; do not guess provider commands. Store argument templates, values, defaults, and per-model restrictions in profile option groups. Resolve task overrides first, then profile defaults, then session/model context; if an optional group has no value, append none of its tokens. Validate the effective model and option values in plan validation and again before reassignment. Resume reuses the persisted task settings and exact session ID. Apply a configured result JSON Pointer and pointer/value error condition before the standard event decoder. Rust UI and workspace serde formats use these same fields; built-in parsers retain their existing behavior. No shell, API key, HTTP dependency, provider enum variant, or model service is added.
Trace: R-158 → BG-113 → SF-145 → UC-148 → FR-143/NFR-53 → QAS-154 → DDR-19 → DD-168. GitHub #62, 2026-10-20.

### DD-169 Extend the task-activity record with the current tool
`append_task_events` already writes `task_activity/<task>/<attempt>` with the last sequence, kind and observation time. It additionally records `tool: {name, since_ms}` when a normalized event starts a tool and clears it when the matching result arrives, using one provider-neutral extractor (`src/activity.rs`). The Agent Room snapshot exposes that record plus the owned-process state per running task; the page renders elapsed time locally every second and pulses the agent icon of running rows unless reduced motion is set.
Trace: R-159 → BG-114 → SF-146 → UC-149 → FR-144/NFR-54 → QAS-155 → DDR-20 → DD-169. GitHub #100, user, 2026-10-21.

### DD-170 Decide the follow target from consecutive snapshots in the page
`followTarget(prev, next, selectedId, onCoordinator)` in the Agent Room script returns the next running session or the integration entry, or nothing; the poll applies it through the existing selection functions. No backend change.
Trace: R-160 → BG-115 → SF-147 → UC-150 → FR-145 → QAS-156 → DDR-21 → DD-170. GitHub #109, user, 2026-10-21.

### DD-172 Add scope recovery as read, preview and digest-approved apply
`workflow::scope_failure`, `recovery_preview` and `recovery_apply` (`src/workflow/scope_recovery.rs`) read the failure, build a validated proposal with its restore list, and apply it under the run lock. The room reads through `scope_failure` and `scope_preview` requests and applies through the `scope_recovery` action (broker command `scope-recovery`). The run is left `stopped` so the existing explicit Resume continues from completed tasks.
Trace: R-162 → BG-117 → SF-149 → UC-152 → FR-147/NFR-56 → QAS-158 → DDR-23 → DD-172. GitHub #122, user, 2026-10-22.

### DD-173 Widen `repairable` from integration failures to failed tasks
`workflow::repairable` (src/workflow/repair.rs) also accepts a failed run with a failed task when no task waits for availability (an `out-of-scope change:` failure is repairable too since #149: the coordinator decides it with a typed amendment, DD-179). The existing `repair` loop, evidence bundle (already carrying task status, error, stdout and stderr), review rounds and `resume_due` are unchanged, so every caller that already repairs after a run, resume or availability wake-up triages failures too.
Trace: R-163 → BG-118 → SF-150 → UC-153 → FR-148/NFR-57 → QAS-159 → DDR-24 → DD-173. GitHub #135, user, 2026-10-22.

### DD-174 Step lessons are a run document delivered by prompt, outbox and snapshot
`workflow::step_lessons` (src/workflow/step_lessons.rs) keeps the `step_lessons` document (newest 50, fields clipped to 1 KiB, de-duplicated by task and response) and records it with a summary so the memory outbox carries it. It is called when a coordinator revision or replaced test command is applied (review.rs) and when a scope recovery is approved (scope_recovery.rs). `prompt_section` adds up to 4 KiB to every worker prompt; the task detail returns `step_lessons` and the Agent Room lists them under the task's lessons. With the agentmemory transport preferred, the outbox is synced best-effort after recording.
Trace: R-164 → BG-119 → SF-151 → UC-154 → FR-149/NFR-58 → QAS-160 → DDR-25 → DD-174. GitHub #138, user, 2026-10-22.

### DD-175 A pure preference-and-decision step drives the inspector
`inspectorStep` (src/ui/room.js) takes `{pref, open, key}` and an event: a toggle flips `open` and records the preference; a snapshot with a new decision key opens the column; a snapshot without a key restores `open` from the preference. `decisionKey` derives the key from the snapshot. The workbench gets an `insp-closed` class that narrows the third grid column; the heading keeps the toggle and a warning dot with a CSS opacity animation that is disabled under `prefers-reduced-motion`. The preference lives under one local-storage key. docs/design-guide.md names the single allowed animation.
Trace: R-165 → BG-120 → SF-152 → UC-155 → FR-150/NFR-59 → QAS-161 → DDR-26 → DD-175. GitHub #146, user, 2026-10-22.

### DD-176 Charter, gates and amendments as one module family
`workflow::guardrail` will own the charter, gate checks, violation codes, the amendment validator and the amendment budget; existing patches (planning overlap correction, test preflight, failure triage, scope recovery) are routed through it in steps.
Trace: R-166 → BG-121 → SF-153 → UC-156 → FR-151/NFR-60 → QAS-162 → DDR-27 → DD-176. GitHub #149, user, 2026-10-22.

### DD-177 The decision list is derived from audit records
`workflow::decisions::list` maps audit records to rows; `Store::count_events` supplies the snapshot count; the coordinator request returns `decisions`, the task request returns `coordinator_decisions`, and the Agent Room renders them as judgment entries beside tool decisions.
Trace: R-167 → BG-122 → SF-154 → UC-157 → FR-152/NFR-61 → QAS-163 → DDR-28 → DD-177. GitHub #149, user, 2026-10-22.

### DD-178 Launch gate before the run folder
`launch_gate` runs in `run_inner` after the repository baseline is known and before `create_dir`, so a refusal leaves nothing behind. It only resolves executables; running commands to prove them (and the planned test dry run) is left out because test commands legitimately fail before the work exists and a dry run would mutate the project.
Trace: R-168 → BG-121 → SF-155 → UC-158 → FR-153/NFR-62 → QAS-164 → DDR-27 → DD-178. GitHub #149, user, 2026-10-22.

### DD-179 Amendments ride on the existing review step
`Revision` gains `add_paths` and `add_dependencies`; `apply_typed_review` hands them to `workflow::amendments::apply`, which validates with `guardrail::amend`, applies low-risk ones to `report.plan`, and records an `amendment` audit record (status applied, escalated or undone, previous values, reason). `repairable` no longer excludes out-of-scope failures, so the coordinator decides them first and the guided recovery stays as the human path. `Request::UndoAmendment` restores the previous values under the run lock.
Trace: R-169 → BG-121 → SF-156 → UC-159 → FR-154/NFR-63 → QAS-165 → DDR-29 → DD-179. GitHub #149, user, 2026-10-22.

### DD-180 Charter as an audit record, drift as a pure function
`guardrail::charter` builds the record, `run` writes it after the launch gate, `guardrail::drift` counts paths beyond it, `amendments::apply` escalates above `DRIFT_LIMIT`, the review evidence bundle carries the latest charter, and `recovery_apply` writes an updated charter. Decisions list charter records as judgments of kind charter.
Trace: R-170 → BG-121 → SF-157 → UC-160 → FR-155/NFR-64 → QAS-166 → DDR-30 → DD-180. GitHub #149, user, 2026-10-22.
