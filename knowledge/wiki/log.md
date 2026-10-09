# Knowledge and development log

## [2026-10-10] verify | Worker-bound human decisions

Opt-in Claude worker binding and CLI list/approve/reject/refresh controls now have
compiled mock-protocol evidence. Persisted session UUIDs, approval epochs, paused
workspace revalidation, single consumption and separate merge approval are verified.
Red → Green covered exact-ID lookup beyond 1,000 history records, payload tampering,
foreign/replayed callbacks, expiry/refresh, rejection/revision and stopped explicit
resume. Shared and optional worktree modes pass.

Regression: 127 passed, two ignored, 34 suites in 49.61 seconds; Clippy, formatting
and whitespace checks pass. Actual Claude metadata-only initialization: 0.90 seconds,
no generation prompt. Graphify: 341 nodes/1,083 edges, no LLM. CocoIndex found the
operator/scheduler receipt reads. Agentmemory decision: mem_mv1klk1z_adfd73a70fd3.
No runtime dependency added. Native enforcement, OS containment, functional UI and
runtime knowledge remain open. See [worker decisions](native-worker-decisions.md).

## [2026-10-10] verify | Asynchronous context verification

Red → Green established asynchronous callback cancellation and normal final-frame
draining. Owned verifier descendants terminate even when they do not observe the
stop marker. The compiled internal command derives binding from current stored
plan/session and Git state; both workspace modes pass. Ignored sensitive targets
use metadata rather than content reads. Verification never grants permission.

Regression: 121 passed, two ignored, 34 suites in 43.06 seconds; Clippy, formatting
and whitespace checks passed. Actual Claude metadata-only initialization passed
separately in 0.90 seconds without a generation prompt. Graphify: 321 nodes/987 edges;
CocoIndex located the verifier. No new runtime dependencies or API keys.
agentmemory records the design reason and cancellation/ordinary-exit distinction.
See [verifier evidence](tool-context-verification.md). Native worker/operator
integration, runtime knowledge, actual native enforcement and functional UI remain open.

## [2026-10-10] verify | Bounded worker policy components

Red → Green exposed approval consumption before duplicate tool identity validation
and evaluation after input closure. The corrected controller rejects both first.
An explicit bounded file classifier protects owned edits, metadata/settings, sensitive
paths and filesystem aliases; a replaced workspace directory also invalidates it.
The classifier bridge keeps commands/searches behind the human evaluator and refuses
unsupported inputs before evaluation. Mock control integration confirms retained raw
logs, terminal/quiescent ownership, no pending side effect and one-shot consumption.

Regression: 116 passed, two ignored, 32 suites in 38.76 seconds; Clippy, formatting
and whitespace checks passed. Graphify: 308 nodes/941 edges; CocoIndex located the
classifier and evaluator bridge. agentmemory recorded replay/lifecycle and policy
decisions with reasons. No new runtime dependency, API key or LLM indexing call.
Ponytail guidance remains applied; no callable tool/CLI is available locally.
See [policy evidence](tool-approval-policy.md) for source hashes and limitations.
Production worker/context/operator integration, live native proof, runtime knowledge
and functional UI remain open; the full goal is active.

## [2026-10-10] verify | Pre-execution control components

Installed Claude 2.1.286 predates the documented fail-blocking command-hook option;
ordinary hook failure is not enforcement proof. Bounded bidirectional subprocess
control retains stdin until explicit idle, spools raw output before decisions, and
kills/reaps the owned group on malformed output or callback errors. Typed approvals
start pending, bind exact context/input, expire within 900 seconds, and have atomic
single consumption with audit/outbox updates. Mock subprocess integration proves no
command side effect before approval and no replay after its first use. A Red native
controller test exposed loss of pending permission decisions after a result while work
was active; preserving them repairs the late callback without reevaluating approval.

Regression: 107 tests pass, two opt-in/helper tests ignored, 30 suites in 37.64 seconds; Clippy/formatting clean. Native metadata-only SDK initialization
also passes with Claude 2.1.286, registered PreToolUse callback IDs, zero user frames and
terminal/quiescent ownership. Raw stdout SHA-256:
`935b6d736d8562ab968ac4a6fa440f34f46f0a00f3d3753e09e488a84857f5f1`.
The current Claude CLI reports no login. Native generation/control tests require the
existing subscription login; no API-key fallback is introduced. Production worker
permission integration, all-provider enforcement, runtime knowledge and functional UI
remain open. Graphify: 284 nodes/869 edges without LLM; CocoIndex verified consumption.

## [2026-10-10] verify | Structured coordinator review

A registered CLI coordinator receives bounded diff/test/log excerpts with exact source
hashes. Typed final documents reject invented task/evidence IDs and generated command
fields. Saved proposals bind to current task state and evidence; exact-hash application
makes no additional model call. Acceptance never creates merge approval. Stop preserves
raw review logs, interrupted ownership remains recoverable, and unexpected edits revoke
the candidate without rebaselining. TDD also exposed stopped reviews consuming a round;
they now refuse launch before changing state. Regression: 96 tests, one intentionally
ignored subprocess helper, 27 suites in 37.42 seconds; Clippy and formatting clean.

Graphify: 234 nodes/737 edges, no LLM. CocoIndex structural grep verified CLI/workflow
saved-review integration. No runtime dependency added. Native provider contracts and
pre-execution approval enforcement remain unverified; full-product acceptance is open.

## [2026-10-10] verify | Review feedback and dependency refresh

The worktree rework test first failed because dependent retries retained old predecessor
contents. Versioned dependency context and preserved-checkout refresh now propagate a
reviewed upstream revision through a three-task chain in both workspace modes. Scoped
partial files and old native session evidence remain available; refreshed worktrees use
fresh sessions. Patch conflicts leave destination bytes/index unchanged and block launch.
Compiled `revise` control, failure repair, transitive invalidation, preserved unaffected
tasks and fresh candidate approval pass. Regression: 82 tests; Clippy clean.

## [2026-10-10] verify | Durable ownership and checkpoint boundaries

TDD exposed orphaned child processes after coordinator SIGKILL, a claim created too
early during dirty-repository validation, and mock session IDs reused across independent
workspaces. Kernel leases now keep stable inodes; durable repository/process claims
require explicit recovery when ownership is unknown. Native session leases use the
provider/account/session identity, and paused working trees and indexes are checked
before resuming. All 78 tests pass (one subprocess helper is intentionally ignored in
normal discovery and invoked by its parent test); Clippy reports no warnings.

Graphify extracted 206 nodes/612 edges without an LLM. CocoIndex structural grep
confirmed SessionLease declaration and workflow integration. No new runtime dependency
or model API key was introduced. Full-product acceptance remains open in
[the audit](../../docs/09-completion-audit.md); this is an incremental verification checkpoint.

## [2026-10-09] plan | Requirements and OSS research

Investigated CLI subprocesses, existing authentication, worktrees, human approval.
Added Rust/TDD, agentmemory, LLM-wiki, RTK/Ponytail/Graphify/CocoIndex constraints.
Recorded OSS comparison and headless/resume contracts in [protocols](cli-protocols.md).

## [2026-10-09] task | Design documents

Linked requirements → BG → SF → UC → QAS → DD → TDD implementation plan.
This was a documentation task, not the start of product execution implementation.
Initial architecture: Rust/SQLite/CLI-pager; optional memory/indexing sidecars.

## [2026-10-09] verify | Licenses and contracts

Excluded ao code reuse for unconfirmed license. Distinguished agentmemory Apache-2.0
from iii Apache-2.0 AND Elastic-2.0. Recorded conduct text output and Vibe Kanban
app-server/serve differences from requested exec/run. At this stage, product execution,
TDD tests, performance and transitive distribution checks had not been performed.

## [2026-10-09] complete | Research and design

Verified 15 Markdown files, zero broken local links, seven stage documents.
Research/design complete; T-01–08 implementation not complete. Saved findings and
outstanding verification to shared agentmemory.

## [2026-10-09] complete | T-01 Profile and plan foundation

Red: five failed, three passed. Green: eight passed. cargo test and clippy passed.
Commit 6391f7d; design commit 33a3edf. Created private neodamul/AgentConductor repository.

## [2026-10-09] verify | T-02 State foundation

Red: three failed, one passed. Green: four passed. Transitions and audit events share
one transaction. Verified resume transitions, duplicate events, database reopen.
Single-executable requirement: bundled SQLite, separate optional sidecars.
Commit 37e7860 pushed; total 12 tests passed. Full recovery scope remains pending.

## [2026-10-09] plan | Documentation and interface preview

User requested README UI, English documents/Korean communication, and exclusion of
internal decisions/development plans from README. Scope: translate all Markdown, retain
source references/IDs and evidence, add a labeled mockup. No implemented UI is claimed.

## [2026-10-09] task | English migration and public README

Translated documentation. Replaced README with product-facing content. Added original
SVG and a PNG preview. An intermediate README referenced an image before its file existed;
corrected the broken reference. Git history preserves pre-migration log/source wording.

## [2026-10-09] verify | Preview and document checks

Rendered the SVG locally and inspected the PNG. Initial Quick Look aspect handling cropped
the image; rendering on a square canvas and cropping blank padding preserved the full design.
Verify local links, English-only text, XML structure, image dimensions, and Git diff before commit.

## [2026-10-09] complete | Documentation and preview

Verified 15 English Markdown files, zero broken local links, valid SVG XML, 1280×880 PNG,
and clean Git diff whitespace. Visually inspected all preview panels. Saved requirements and
progress to shared agentmemory mem_mv0xsakb_73793144f244. This completes the documentation
change, not the UI or orchestration implementation.

## [2026-10-09] plan | Optional worktrees and human controls

User changed worktree separation to an option and requested a less web-like interface with
visible intervention controls. Shared is the default; worktree is explicit. Shared-mode
policy requires disjoint scopes, a clean baseline, no worker Git mutations, quiescent tests,
and aggregate review with unverified per-worker attribution.

## [2026-10-09] task | Workspace plan schema and console concept

Added workspace_mode (shared/worktree), backward-compatible shared default, and unknown-mode
rejection. Redesigned the preview as a terminal console with task selection, goal/feedback
entry, pause/resume/revise, workspace selection, and inspect/approve/reject controls.

## [2026-10-09] verify | Workspace schema

Red: two new tests failed while eight existing tests passed. Green: 14 tests passed overall;
clippy passed. Actual workspace execution/scheduling and UI controls remain unimplemented.

## [2026-10-09] complete | Workspace option and revised preview

Plan schema, English documentation, and revised console illustration verified. All local
links resolve; SVG parses; PNG is 1280×940 and all interaction controls are visible.
Shared/worktree execution and interactive controls still require later implementation.

## [2026-10-09] plan | Additional interface concepts

User requested more design mockups and approved the addition of intervention controls.
Produce three visually distinct layouts retaining targeted feedback and human approval.

## [2026-10-09] task | Three interface mockups

Created Agent room (conversation/inspector), Dispatch desk (stage/assignment), and Review
cockpit (diff/evidence). Added original SVGs, PNG previews, gallery, and README comparison.
All artifacts explicitly label illustrative data and unimplemented controls.

## [2026-10-09] verify | Visual review

Rendered each SVG and inspected all three PNGs for clipping, overlaps, readable labels,
and visible intervention controls. Validate XML, dimensions, English text, and local links.
No runtime code changed; visual/document checks apply instead of new unit tests.

## [2026-10-09] complete | Interface gallery

Three 1280×860 PNGs and corresponding valid SVGs verified. English-document and local-link
checks passed. Gallery and README updated. These mockups do not complete runtime UI work.

## [2026-10-09] complete | Push cadence preference

User clarified that pushing every time is unnecessary. Keep small commits; batch pushes
at meaningful checkpoints or for remote review. Updated the implementation workflow.

## [2026-10-09] complete | Preferred interface palette

User preferred the second mockup's colors. Applied the Dispatch desk light/sage/forest
palette to Agent room, Review cockpit, and console overview. Preserved feedback, pause,
resume, revision, and approval controls. Inspected rendered previews and corrected pale
action/status text for contrast. Saved preference to agentmemory; commits stay local
until a meaningful push checkpoint.

## [2026-10-09] plan | T-03 Process supervisor

Implement direct argv execution, inherited auth environment/cwd, stdin EOF, separate disk
spooled stdout/stderr, exit capture, and bounded POSIX process-group termination. No retries
or real paid-model calls. Explicit user stop and parser normalization remain later tasks.

## [2026-10-09] task | T-03 Foundation

Red: process-contract test target failed because supervisor API did not exist. Green: six
process cases pass, including multi-megabyte stdin/stderr, nonzero exit, spawn failure,
timeout of descendants, unread stdin, and ignored interrupt/termination escalation.

## [2026-10-09] verify | T-03 Foundation

20 tests passed; clippy passed. CocoIndex structural grep inspected ProcessGroup; Graphify
code-only extraction found 41 nodes/72 edges across four source files. Derived graph moved
to temporary storage. Inventory captured 33 host-target dependency license declarations;
full source-notice and multi-platform release checks remain pending.

## [2026-10-09] complete | T-03 Foundation

Supervisor foundation complete on Unix. Windows containment and explicit user-stop API
remain unimplemented; no full MVP completion is claimed.

## [2026-10-09] complete | Initial single-binary CLI

Red: all three CLI tests failed with an empty entry point. Green: help, plan validation,
and invalid-input handling pass. Full suite: 23 tests passed after fixing parallel test
temporary-directory collisions with an atomic sequence. Release build and help smoke pass.
otool shows only host system libraries (libiconv/libSystem), no dynamic SQLite dependency.
The CLI currently validates plans; it does not yet run the orchestration workflow.

## [2026-10-09] plan | T-04 JSONL normalization foundation

Normalize stored Claude/Codex/OpenCode/custom events without launching providers. Preserve
unknown payloads, session IDs, sequence and byte offsets; bound line size and reject malformed
JSON/UTF-8. OpenCode step completion must not imply whole-task completion.

## [2026-10-09] task | Parser and normalize command

Red: missing protocol module; normalize CLI test failed before command implementation.
Green: six synthetic parser tests and four CLI tests pass. Preserve raw provider payloads
and expose normalized JSONL through the single native executable.

## [2026-10-09] verify | Normalization

30 tests passed overall; clippy passed. Fixtures are synthetic contracts, not captured or
version-pinned live CLI evidence. Live provider smoke and resume capability checks remain
pending. No API keys or paid-model calls were used.

## [2026-10-09] complete | T-04 Foundation checkpoint

Added five adapter tests for literal file argv, native initial/resume flags, exact session
selection, invalid-session rejection, and custom-profile registration requirement. Full suite:
35 tests passed; clippy and release build passed. Release normalize smoke preserved session
and terminal result. Current CLI commands are validate and normalize only. Scheduling,
workspaces, approval/merge execution, recovery, and live-provider checks remain unfinished.

## [2026-10-09] complete | Progress checklist

User requested stage-by-stage progress in the plan. Added evidence-based checklists for
T-01–08 with plan/task/verify/complete and explicit distinction between foundations and
full completion. Current Git tests pass; scheduler/approval E2E is still in the red stage.

## 2026-10-10 — Availability/session extension

- Added R-18–20 traceability from goals to design decisions and T-09.
- TDD verified explicit reset parsing, manual scheduling across restart, role transfer, model approval invalidation, concurrency limits, and bounded retries.
- Native Claude/Codex metadata discovery succeeded with inherited authentication; OpenCode returned an empty configured catalog.
- Graphify extracted 173 nodes/482 edges; recovery links to drive, permission validation, and exclusive locking. CocoIndex structural grep verified all session-edit entry points use the run lock.
- No new framework, API key, indexing daemon, or provider generation request was introduced. Ponytail's small-scope verification discipline continues to guide implementation.

- Release binary E2E: three mock agents in optional worktrees, 1.148 seconds, all integration tests pass, unapproved merge denied, exact approved SHA merged. Artifact is 3,170,736 bytes with startup/help RSS 2,195,456 bytes. Live CLI workloads are not included in that memory figure.
- Packaged 42 pinned Rust dependency license texts, SQLite disclaimer, SBOM, checksum and manifest. Broader product backlog remains unchecked in the implementation plan.
