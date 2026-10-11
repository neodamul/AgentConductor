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

`CHANGELOG.md` keeps a `## vX.Y.Z[-pre]` section per release with *Added*, *Fixed* and
*Known limitations*. `scripts/release_notes.py <tag>` prints exactly that section and exits
non-zero when it is missing or empty. The publish job writes it to a file and passes
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
