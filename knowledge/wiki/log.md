# Knowledge and development log

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
