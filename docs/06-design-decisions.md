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
