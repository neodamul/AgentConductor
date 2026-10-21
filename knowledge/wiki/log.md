# Knowledge and development log

## [2026-10-19] verify | Recursive repository planning rules (GitHub #8)

Rules below `<repository>/.agentconductor/plan-rules/` now load recursively in relative
lexical order when the inline workspace override is unset. Existing global/default rules
remain the base; inline workspace text remains exclusive. Symlink roots, nested symlinks,
non-Markdown files and files outside the root are excluded. Tests first failed on the
missing loader, then pass for empty/nested/multiple files, precedence and containment.
Plan-rules (5) and slug-flow (1) pass; source-layout, formatting and all-target Clippy pass.
The all-target suite is not green in this sandbox: unrelated cooperative-memory and folder
execution tests fail with `Operation not permitted`. No full-regression pass is claimed.

## [2026-10-10] verify | Native Codex subscription/session contracts

Installed Codex 0.162.0 with existing ChatGPT login passed actual stdin/JSONL generation and exact-ID context resume (7.74 seconds); two native sessions ran concurrently, retained distinct IDs and each recalled its own marker on concurrent resume (7.13 seconds). API key environment variables were removed; no model tool calls appeared. Nonterminal host role-definition error items are now visible warning events; terminal failures remain separate.

Regression: 135 passed, four ignored, 37 suites in 52.91 seconds; both new ignored generation tests ran explicitly above. Clippy/format/whitespace pass. Graphify 377 nodes/1,199 edges; CocoIndex located item normalization. Raw evidence remains local; hashes are in [native Codex evidence](native-codex-sessions.md). Agentmemory: mem_mv1ldpjn_b96c9112eb44. Native containment, scoped implementation, all-provider proof, functional UI and final packaging remain open.

## [2026-10-10] verify | Reviewed runtime lesson assets

Red → Green covers exact typed promotion, source-hash revalidation, offline next-attempt task-scoped handoff, changed-source exclusion, sensitive/alias refusal and wiki alias failure. SQLite remains authoritative when replica materialization fails; retry repairs mirrors without duplicating acceptance log entries. Compiled digest/accept CLI controls pass.

Regression: 134 passed, two ignored, 36 suites in 51.90 seconds. Clippy/format/whitespace pass. Graphify 377 nodes/1,199 edges, no LLM; CocoIndex located worker lesson injection. No runtime dependency added. Agentmemory decision: mem_mv1l3l0i_ea3708c1e9b9. Native lesson integration, confidence-aware/cross-run context, functional UI and native containment remain open. See [runtime lessons](runtime-lessons.md).

## [2026-10-10] verify | Live normalized intermediate reports

Red → Green established pre-completion SQLite queries, malformed-stream process cancellation, final unterminated record draining, immutable replay, separate attempts, compact memory summaries and bounded recent-event queries. Both worker transports are connected. Parallel native-worker rework exposed a deferred SQLite lock upgrade; immediate writer transactions fixed it.

Regression: 131 passed, two ignored, 35 suites in 50.14 seconds. Clippy/format/whitespace pass. Graphify 352 nodes/1,110 edges; CocoIndex located both observer calls. No runtime dependency added. Agentmemory bug/decision: mem_mv1kvcr1_7c00066988b4. Native authenticated execution, OS containment, functional UI, runtime lessons and knowledge revalidation remain open. See [live progress](live-progress.md).

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

- 2026-10-10: Embedded Agent Room checkpoint with 141 passing tests, four ignored; real browser fixture feedback/reassignment/model/resume/diff/test/approval/merge. Native window acceptance and full product remain open.

- 2026-10-10: Native WKWebView review and Cmd-W close verified; duplicate-window lease added after user cleanup correction. Quota UI fixture outcome remains unresolved and retained.

- 2026-10-10: Assetized DD-23 direct native Room transport and source/artifact fingerprints. Historical HTTP checkpoint retained as superseded evidence. Actual native feedback/resume/review/approval/merge verified; both owned desktop windows exited 0. Availability fixture failure remains open.

- 2026-10-10: DD-24 process outcome TDD and 33 targeted regressions passed. Replaced ps-based absence checks with kernel probes after sandbox denial. Historical fixture failure did not reproduce in a new run; retained old evidence.

- 2026-10-10: DD-24 full regression passed 144 tests with zero failures and four explicit ignores (70.88 seconds); Clippy/rustfmt clean. No runtime dependency added.

## [2026-10-10] verify | Native startup workspace

See [source-hashed native startup evidence](native-startup-workspace.md). Full regression: 148 passed, four ignored, zero failed. Real native fixture completed setup, planning, edits, consent, execution, review, approval and merge without a UI web server. Two owned windows exited zero sequentially; no overlapping verification windows were retained. Authenticated all-provider acceptance and dialog accessibility remain open.

## [2026-10-10] ingest | Claude CLI and native app acceptance

Source: [raw observations](../raw/2026-10-10-claude-native-acceptance.md) (sha256 `bbef81bd95104db3b467aa8673844edaedc0c936fea4539ba2a53d45ca1b568c`). Live Claude session/resume/concurrency tests and a three-worker guarded workflow passed against a subscribed CLI. Findings fed R-30–R-32, DD-32–DD-34 and the guard missing-file deny fix.

## [2026-10-10] verify | Live tool decisions (DD-32)

See [live tool decisions](live-tool-decisions.md). Commit 6fc77b6. Full regression 203 passed, 0 failed, 8 ignored.

## [2026-10-10] verify | Isolated planning snapshot (DD-33)

See [planning snapshot](planning-snapshot.md). Commit cbba143. Hostile planner left the target untouched; full regression 203 passed.

## [2026-10-10] verify | Korean input (DD-34)

See [Korean input](korean-input.md). Commit 75b121d. Native IME acceptance with the system input method remains open.

## [2026-10-10] update | Method documents

Added functional/non-functional analysis (`docs/04b`), design drivers with architecture view (`docs/05b`) and the engineering document index with traceability matrix (`docs/README.md`).

## [2026-10-10] verify | Workspace back navigation (DD-39)

Operator report ingested as R-33. See [workspace navigation](workspace-navigation.md). Native discard acceptance and full regression (208 passed) recorded. Windows WebView2 shell (DD-38) is pending native CI evidence.

## [2026-10-10] reconcile | Codex–Claude continuity

Reconciled 19 committed changes through 2211ba3 and preserved the older Codex working changes in a named stash. Windows CI 38037224504 passed compilation, containment and UI smoke but failed gate-test directory cleanup with an open SQLite store. Explicit close added; focused macOS gate regression passed 4/0/0. Native Windows correction verification remains pending. See [development continuity](development-continuity.md); agentmemory mem_mv25cgtz_2a8a07bee3cc.

## [2026-10-10] verify | DD-41 source layout

Layout Red on eight oversized files and three inline tests; split Green, maximum 485 lines. Full macOS regression 213/0/9 across 57 suites before canonical checkout migration; focused layout and format passed in Development checkout. Windows verification remains pending. See [source layout](source-layout.md); agentmemory mem_mv25wmpa_f63ae0e2a551.

## [2026-10-10] verify | Native packaging contract

Two Python tests cover platform-specific packages, exact binary checksums, public asset selection, SBOM/notices and fail-closed license checks. Source layout also passed. Target-filtered license declarations/text fingerprints retained locally. Real artifacts and Release acceptance remain pending; see [packaging contract](native-release-packaging.md).

## [2026-10-10] verify | Packaged workflow

Actual macOS da746b9 packaged binary passed parallel three-worker, diff/tests, refused unapproved merge and exact approved merge in shared/worktree modes. Integrity contract tests passed; native release CI artifact gates added. Windows execution still pending. See [packaged workflow](packaged-workflow.md).

## [2026-10-10] fix | Release archive timestamps

Actual archive failed on pre-1980 registry file timestamps. Red/Green regression preserves bytes and executable mode while clamping ZIP dates. Five contracts passed. macOS archive generated with all CRCs and binary SHA validated; Windows packaged acceptance remains pending.

- 2026-10-10: Windows default startup HOME/USERPROFILE selection, Red/Green evidence and sequential native test lifecycle recorded in [native startup workspace](native-startup-workspace.md). Native Windows acceptance remains pending.

- 2026-10-10: DD-40 runtime icon asset added; five icon tests passed including actual macOS installed applications. Windows path/extraction/rendering acceptance remains open.

- 2026-10-10: Windows CI 38040929721 failed on omitted WebView2 crate license files after release compilation. Exact official MIT notices retained and bound to version/VCS/hash; seven packaging tests and actual 42/108 dependency closure pass.

- 2026-10-10: 0308003 (Windows home selection), 6646f2e (icon knowledge) and 6ccd4bf (upstream license texts) pushed. CI 38041844312 validates source 6ccd4bfe2684f5c7d7dbe32cf7b1aab6ffa35efc; full release acceptance remains open.

- 2026-10-10: native package prerequisite Red/Green completed; public Windows support notes and WebView2/WebKit manifest requirements updated. Seven package/smoke tests pass; source CI 38041844312 still in progress.

- 2026-10-10: Claude resumed after the Codex token stop: autocrlf license-byte fix (a78d5f5) and tag-driven release workflow (9ffc124) assetized in [native release packaging](native-release-packaging.md). Native Korean IME acceptance attempt found the app window in an English input source; not verified.

## [2026-10-10] verify | Native end-to-end with Claude workers

Source: [raw observations](../raw/2026-10-10-claude-native-e2e.md). Three DD-32 gaps found and fixed by TDD (Room listing, per-call binding, synchronous Room decisions); full native run merged candidate 917074f. See [live tool decisions](live-tool-decisions.md).

- 2026-10-10: First pre-release `v0.1.0-preview.1` published through the tag workflow; CRLF checksum defect found by downloading it and fixed (d9501f5). See [native release packaging](native-release-packaging.md).

- 2026-10-10: `v0.1.0-preview.2` published from d84b876; downloaded macOS and Windows archives and both inner binaries verify with `shasum -c`.

- 2026-10-10: QAS-25 native Korean IME acceptance passed (composition across polling, exact UTF-8 save). See [Korean input](korean-input.md).

## 2026-10-11 public docs (DD-42)

README rewritten as desktop-first introduction; docs/user-manual.md holds the full reference. public_docs 3/3, release contracts 13/13, package ships manual and screenshots. Commit 39a55f0.

## 2026-10-11 v0.1.0-preview.3

Release run 38069707257 green on macOS and Windows; archives contain README, user manual and screenshots; SHA256SUMS verified locally. Regression 221/9/0.

## 2026-10-11 reviewed handoff (DD-43)

Red→Green cooperative_handoff; regression 224/10/0.

## 2026-10-11 v0.1.0-preview.4

DD-43 shipped; Release 38072717471 green on both platforms; checksums verified.

## 2026-10-11 memory discovery (DD-44)

Red→Green discovery fixture + real daemon; regression 225/10/0.

## 2026-10-11 v0.1.0-preview.5

DD-44 shipped. CI 38074008190 failed on a cold PowerShell start exceeding the 8 s containment-test timeout (test timing, not product); raised to 30 s; CI 38074563237 and Release 38075480798 green; checksums verified.

## 2026-10-11 workspace UX and planning

DD-45…DD-60 assetized: manual plans, workspace settings/switching/names, release notes, baseline dialog, OS-file ignore, progress, readable instructions, role rules with models, immediate agent save, model lists, plan retention, compact agents, settings export/import, 1–12 tasks, Read plan. Regression 260/11/0.

- 2026-10-11: Codex reconciled clean Development checkout to Claude source 318f930, reviewed R-36–59/DD-42–65, confirmed 16 focused tests and actual preview.6 assets; latest CI jobs did not start due GitHub account billing/spending restriction.

- 2026-10-11: DD-66 Claude quota stream collection; six regressions and local full-suite evidence recorded in [Claude subscription quota](claude-quota.md).

- 2026-10-11: DD-67 inline quota badges and retained workspace observation tests.

- 2026-10-11: R-62/DD-68 planned MCP goal imports; workspace quota collection regression reproduced and corrected.

- 2026-10-11: DD-69 per-agent refresh/action icons and independent workspace collection; live account and cleanup evidence in [Workspace quota collection](workspace-quota-refresh.md).

- 2026-10-11: DD-70 compact multi-window quota presentation and deterministic cache-initialization race correction.

- 2026-10-11: resource constraints reaffirmed, saved to shared agentmemory and signaled to Claude; final compact quota runtime replaced.
## 2026-10-11 slug work trees and idle budget

DD-61–DD-65, DD-71, DD-72 assetized. Regression 287/12/0 on 51f5bb6.

## 2026-10-11 agent preferences and native AC identity

R-67/DD-73/QAS-60 assetized with [native acceptance evidence](../raw/2026-10-11-native-settings-acceptance.md). Default five-minute per-agent configurable refresh, executable discovery/Browse, consistent fields, silent quota jobs and AC application identity are implemented. Actual operator screenshots and English manual updated. Final integrated regression: 293 passed, 12 ignored, 0 failed; 22 Python checks and macOS shared/worktree packaged smoke passed. The OS picker opened outside the app-scoped capture; preserve the working chooser. Windows verification and stable publication remain blocked by GitHub runner billing.

## 2026-10-11 provider-driven identity and quota reset time

R-68/R-69, DD-74/DD-75 assetized in [agent-identity-and-reset](agent-identity-and-reset.md). Repository made public (user decision) so GitHub Actions runs again; first real Windows run exposed a 3 s timing assumption in `claude_quota` stream test, now 20 s with the observer's own error reported. Regression 299/0.

## 2026-10-11 interface language and UTF-8

R-70/DD-76 assetized in [interface-language](interface-language.md). Windows CI exposed cp1252 decoding in Python tests and a one-second race in the Claude quota stream test; both fixed. Regression 303 Rust / 26 Python.

## 2026-10-11 v0.1.1 released (stable)

First stable release after Codex's v0.1.0, published by GitHub Actions from 3cea042 with verified macOS (aarch64) and Windows (x86_64) packages and SHA256SUMS. Lesson: the first real Windows CI run surfaced four Windows-only problems (cp1252 Python I/O, release notes encoding, a one-second test race, an unused `mut` that only exists without the Unix block); validate on both platforms before tagging.

## 2026-10-11 messenger notifications phase 1

R-43 activated by the user; DD-79 implemented and assetized in [messenger-notifications](messenger-notifications.md). Live delivery verified locally over HTTPS. Regression 310 Rust / Python OK. Inbound decisions remain phase 2 (company messenger API to be chosen).


## 2026-10-11 stable v0.1.2: MCP, Windows metadata and custom catalogs

DD-68, DD-78 and DD-80 assetized; concurrent DD-79 messenger changes preserved. Source 5cefa6c passed all four [native CI jobs](https://github.com/neodamul/AgentConductor/actions/runs/38134378978): macOS 320 Rust tests / Windows 135, no failures, 32 Python contracts each. Exact packaged binaries passed shared/worktree parallel execution, tests and approval/merge checks. Both archives, SHA256SUMS, license notices and SBOM published as stable v0.1.2. Existing app replaced with the verified macOS binary and the user workspace/plan preserved. Company MCP auth and real Windows subscription acceptance remain operator checks.

Lessons: a functioning custom CLI does not imply catalog discovery; keep explicit manual model provenance. Windows cmd.exe argv[0] must use native separators/literal cmd.exe; terminal output needs explicit null standard handles under ConPTY. Compile and lint platform-specific tests on the actual platform before release.

## 2026-10-11 messenger decisions phase 2

DD-81 implemented: Slack replies or signed company relay decisions answer live tool requests through the exact DD-32 path. Live round trip verified locally over HTTPS with the system curl. Regression 323 Rust / Python OK. Disk was full during the session; both build caches were cleaned (user approved for the main checkout).

## 2026-10-11 automatic slug names

R-74/DD-82: slug input removed; Hangul romanized for readable unique folder names. Regression 323/0. Lesson: chain doc scripts and commits with `&&` so a failed edit never ships.

## 2026-10-11 v0.1.3 released (stable)

Published by GitHub Actions from 12448c9 after macOS/Windows validation and native packaging: messenger decisions (phase 2) and automatic slug names. Windows MCP remote test was flaky once before the release run (rerun passed).


## 2026-10-12 session communication and optional memory

Assetized the distinction between CLI execution interfaces, A2A-style message exchange, authoritative local state and LLM-wiki knowledge. Local checkpoint operation already works without agentmemory. Documented a general durable signal inbox as a proposal, with receipt/acceptance separation and remote transport limits; no runtime transport was added. Public README contains only product behavior. Source observation pins local hashes and official source links. Another Claude machine owns the in-progress v0.1.9 build; this documentation does not change its version or publish a release.

Follow-up requirement R-92/DD-100: selectable Local Rust or agentmemory-preferred signals, automatic local fallback on memory server/MCP failure, bounded retry and duplicate/late-reply handling. Full BG/SF/UC/QAS/design trace and unchecked TDD implementation checklist added. Runtime support remains pending.

## 2026-10-12 build identity, storage and blocked planning

R-93–R-95/DD-101–DD-103: compile-time version beside both titles; narrowly owned disposable metadata cache preview/cleanup and bounded idle pruning; stop-marker planning cancellation and same-window current-workspace deletion. Focused fixtures and Python48 pass before final follow-up; full compact regression/native/platform verification is pending. Initial full debug compilation exhausted disk; cargo clean removed 22 GiB of generated target artifacts. User workspace/project evidence was preserved.

- 2026-10-12: audited and published stable v0.1.10; verified downloaded platform packages and retained existing reviewed workspace. Reviewed the subsequent signal adapter, reproduced uncertain-restart duplication and added a durable attempt claim plus kernel-owned lease (post-release work).

## 2026-10-13 delivery closed at v0.1.3

The user declared the goal complete with v0.1.3. Open follow-ups need the operator's environment: a real Windows PC run and a live Slack/company messenger round trip.

- 2026-10-19: GitHub #16 default-path follow-up shows localized `Starting agent execution…` directly below the approve button immediately after plan approval and holds the run progress indicator while the start request resolves and Agent Room opens. The native Rust path continues to use its bottom feedback panel and exact-run report gate. Local runtime, message, UI, pending-state and formatting checks pass; package-level macOS and Windows validation remains outstanding.

## 2026-10-19 local company build version

GitHub #13: kept `Cargo.toml` numeric and added the empty tracked `company-suffix.txt` local build input. The validated suffix appears in both titlebars through `update::current()`; Cargo/MCP protocol versions and semantic release comparisons remain numeric. Rust package assembly rejects either a changed suffix file or a previously suffixed binary, preventing an accidental public company-labelled build. Architecture trace R-155 → DD-165 and the implementation checklist are in the system docs. Focused version, UI, release-package, source-layout, formatter, and Clippy checks pass locally. AgentMemory decision saved as `mem_mvezq58i_752e54236f97`.
## 2026-10-19 custom CLI profile behavior (#12)

Added backward-compatible per-profile event-name/session-pointer mappings and opt-in custom quota commands. Commands use the registered executable and argv without a shell; bounded JSON output must follow the normalized `Snapshot` schema. Existing native provider parsers/quota readers remain unchanged. AgentMemory decision/verification memories: `mem_mvf02oy9_66b41cb2e24d`, `mem_mvf0u5jd_1df56fa0eec3`; maintained detail: [custom CLI profiles](custom-cli-profiles.md). The Rust quota-due snapshot and `quota-refresh` command support configured custom profiles, but the production WebView still gates quota UI controls to Claude/Codex; that UI capability awaits the Rust UI migration under R-151. No JS/Python source/test changes are included relative to `origin/main`. After merging current `origin/main` with #44, full local Rust verification passed (180 suites, 542 passed, 0 failed, 14 ignored); focused Rust/source-layout 47/47, all-target Clippy, formatting and diff checks passed. PR #48 and issue #12 remain open for review.
## 2026-10-19 GitHub #44 timeout recovery

R-153 / DD-163: keep a wall-timeout visible while automatically retrying from retained artifacts; cap the backoff at 60 seconds; honor Stop before relaunch; expose terminal blocked causes through the report-level error used by the existing banner. Focused timeout and Stop regressions pass locally. Full verification on integrated #12 branch: 180 suites, 542 passed, 0 failed, 14 ignored; Clippy, rustfmt, and 300-line source-layout checks pass. The separate Rust-native port remains in progress.

## GitHub #52 generated phase overlap recovery

The coordinator returned duplicate `score:` ownership paths in phase `01-game-selection`, which correctly failed validation but ended planning without a correction attempt. Added one bounded re-plan with the exact validation feedback; overlap remains rejected if the corrected response is still invalid. Deterministic retry, phased planning, schema and MCP planner tests pass. See [planning overlap repair](planning-overlap-repair.md). Main-first merge and Rust-native port are still pending.

## 2026-10-20 read paths must not open SQLite write-capable (#65)

PR #55's macOS failure (`active_feedback` timing out after 20 s) was `workflow::run` ending at start-up with `Err("database is locked")`. `workflow::load` used `Store::open`, which runs `PRAGMA journal_mode=WAL` and `CREATE TABLE IF NOT EXISTS` on every call; polled while a run starts, those statements race the run's own database set-up and SQLite returns BUSY at once (the busy timeout does not apply to lock-upgrade conflicts). `load` now uses `Store::read_only_snapshot`. Lessons: a read must never open the database write-capable; a test that waits for a state must also watch the task that should produce it, or an early error looks like a timeout; keep failed-run directories (`$TMPDIR/ac-flow-*`) as evidence; four threads opening and closing the same database in a tight loop made SQLite mutexes stall for minutes, so concurrency tests need bounded polling.

## 2026-10-20 v0.1.31 release preparation (#68)

Prepared the v0.1.31 release from `main` (WebView line): CHANGELOG section, English/Korean manuals (finalization progress, overlapping-scope retry, #62 profiles, SQLite contention, read-only polling), README and Cargo version. Local gate: fmt, clippy `-D warnings`, full suite; `timed_out_worker_resumes_automatically_and_keeps_timeout_evidence` (1 s timeout) failed once under full-suite load and passes alone. Release is gated on PR #55 (Rust port of #50) being merged into `rust` and #50 closed; tag `v0.1.31` only after that. The native #9 UI is excluded until its gates pass.

## 2026-10-20 messenger de-duplication reset on state change (#72)

`messenger::notify_run` remembered only the last notifiable state, so `failed → running → failed` sent one message. It now records every reported state (when the link is enabled) and sends only for notifiable ones; repeated saves of one state remain silent. Red-first test: `tests/messenger_renotify.rs`; existing messenger suites unchanged.

## 2026-10-20 send feedback while a task runs (#95)

`room.js` disabled Send feedback whenever the run was `running`, although `active_feedback::queue` accepts feedback for a running task. New `feedbackEnabled(task)` gate; red-first `tests/room_feedback_gate.cjs` run through `tests/room_feedback_gate.rs`. Manuals updated.
## 2026-10-20 quota reset zones (#82)

`reset_time` knew only Seoul/Tokyo/UTC. New `src/quota/zones.rs` is a built-in table with US/EU/AU/NZ rules (identical on macOS and Windows). Unknown zones still return nothing; extend the table to add one. Red-first `tests/quota_reset_zones.rs`.
## 2026-10-20 update downloads are cleaned (#80)

`update::install` left `updates/<tag>` (15-30 MB each) forever. After a successful replace it now calls `clear_downloads`. Red-first `tests/update_downloads.rs`.
## 2026-10-20 MCP header prefixes validate (#84)

`Settings::parse` validated headers with the bare-reference check, so the documented `Authorization: Bearer {env:TOKEN}` was refused although `expand` supports it. Headers now allow the Bearer/Basic prefix. Red-first `tests/mcp_header_references.rs`.
## 2026-10-20 case-insensitive scope overlap (#79)

`plan::overlaps`, the `.git`/`.agentconductor` component check and the tool write-scope check compared bytes. They now fold case. Red-first `tests/plan_scope_case.rs`.
## 2026-10-20 command policy fails closed on unplaceable forms (#83)

`Policy::command` matched whole words against the outside-program list, so `c''url`, `${x}rl`, `node -e`, `sh -c` and `python -m http.server` were Automatic. New `src/guard/forms.rs` sends glued quotes, expansions and inline-code interpreters to a human. Red-first `tests/guard_command_forms.rs` (both directions: ordinary quoting and `python3 src/tool.py` stay Automatic).
## 2026-10-20 messenger approval parser is explicit-command only (#81)

`inbound::parse` accepted an allow/deny pair anywhere in a reply, so `don't allow CODE` approved. It now takes the decision word as the first word after mentions. Red-first `tests/messenger_decision_parse.rs`.

## 2026-10-20 queued agent and model change for pending tasks (#103)

`workflow::reassign` and `set_model` refused any edit while the run was running, and the room disabled the buttons. A change for a pending task is now queued (`src/workflow/pending_edit.rs`) and applied by the scheduler before launch; running or finished tasks are still refused. Red-first `tests/pending_edit.rs` and `tests/room_edit_gate.cjs`.
## 2026-10-20 live agent activity in the room (#100)

Requirement chain R-159 to DD-169. `task_activity` gains the current tool (`src/activity.rs`), the snapshot exposes a per-running-task `activity` map, and the task header renders elapsed times locally every second. Red-first Rust and Node tests: `activity_tool`, `activity_record`, `room_activity_snapshot`, `room_activity_text`.

## 2026-10-20 coordinator chat (#115)

New `chat` module and worker job (`src/chat.rs`, `src/commands/chat.rs`, `src/ui/setup/chat.rs`), slide-over panel (`chat.html/css/js`). Reuses the coordinator's read-only invocation and the single job slot. Red-first `chat_prompt`, `startup_chat`, `chat_ask` (scripted coordinator end to end), `chat_ui` and `chat_runtime.cjs`. Browser check was not possible; the panel logic is covered by the Node test.
## 2026-10-20 room focus follows the running session (#109)

The room kept the finished session selected, so the activity showed old work while the run awaited approval. `followTarget` (room.js) now moves the selection from consecutive snapshots; reviewed finished sessions are never taken away. Red-first `tests/room_follow_focus.cjs`.
## 2026-10-20 activity line layout jitter (#117)

The line toggled hidden and could wrap as its numbers changed, moving the tabs below. It is now a fixed 18 px one-line slot hidden with visibility only, with an ellipsis and tabular numbers. Extended tests/room_activity_text.cjs.
