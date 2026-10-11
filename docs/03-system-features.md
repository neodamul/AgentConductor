# System features

| ID | Feature | Goal | MVP |
|---|---|---|---|
| SF-01 | Registered subprocess profiles and parsers | BG-01 | Included |
| SF-02 | Three-task plan and dependency scheduler | BG-02 | Included |
| SF-03 | Shared workspace or optional worktrees, ownership, integration candidate | BG-02/03 | Included |
| SF-04 | Logs, results, diffs, test evidence, resume | BG-03 | Included |
| SF-05 | Dangerous-action controls, human approval, merge | BG-03 | Required; enforce provider/sandbox capabilities before execution |
| SF-06 | SQLite state, audit, recovery | BG-03 | Included |
| SF-07 | agentmemory, local wiki, lessons | BG-04 | Minimal integration |
| SF-08 | CLI status, review, approval list | BG-03/05 | CLI foundation verified; functional interaction UI remains required |
| SF-12 | License/SBOM and TDD verification | BG-05 | Development/release procedures |

Automatic planning runs the coordinator CLI once to produce JSON, then validates it
in Rust. The coordinator reuses its login. Manual plans use the same validation path.

## Availability and session controls

- SF-09 (BG-07): Local persisted reset detection, manual scheduling, due-work execution, bounded retries, and low-idle-cost watch mode.
- SF-10 (BG-08): Multiple sessions per profile, active-session exclusion, account quota grouping, per-profile concurrency, and audited reassignment.
- SF-11 (BG-09): Native catalog discovery/import and per-task session model controls with approval invalidation.

### SF-18 — Native startup and exact-plan execution consent

Persist a versioned local agent registry and goal workspace, discover observed catalogs, generate read-only proposals, render editable task scopes/tests/models, and require the exact reviewed plan digest for execution. Retain coordinator ownership and logs across UI lifecycle. Trace: BG-09 / R-21.

### SF-19 Account quota summary

Normalize provider-reported quota windows with remaining percentage, reset timestamp and freshness; show independent windows and unavailable states in native agent summaries. Query metadata through the registered CLI without generation prompts or API-key setup.

### SF-20 Cross-platform build and release

Commit a Windows development handoff, execute platform build/test jobs, implement native Windows safety primitives and publish verified platform artifacts with checksums and notices.

### SF-21 Deterministic supervision and reviewed fallback policies

Separate process supervision from optional semantic AI review. Register per-agent fallback targets/models, validate their permission/account boundaries, retain durable restriction observations and transfer work only after active attempts drain. Non-steerable CLIs receive scoped next-attempt handoffs. Trace: BG-11.

### SF-22 Evidence-based team briefing

Generate a bounded local briefing across agents and sessions using task/attempt/process evidence and intermediate memory reports. Include observation timestamps, unresolved conflicts, approvals and next actions. An optional coordinator may verbalize the same factual bundle. Trace: BG-12.

### SF-23 Agent status indicators

Render deterministic status-to-light mapping directly from observed task/session state. Preserve selection and pulse continuity across polling. Trace: BG-12 / R-29.

### SF-24 Cooperative memory control channel

At declared checkpoints, capable CLI sessions publish intermediate reports and pull addressed directives from agentmemory. Local command identity, acknowledgment and task/process reconciliation make this channel auditable across agents. Trace: BG-11/BG-12 / R-27.

### SF-25 Live tool decision gate

Hold a guarded worker's pre-tool callback while an exact ledger request awaits a human decision; decide it transactionally from UI/CLI while the coordinator owns the run; consume an approval once in the same attempt; refuse only the call on rejection. Trace: BG-13 / R-30.

### SF-26 Isolated planning snapshot

Materialize the reviewed base's tracked files into a disposable directory outside the repository, run the coordinator there, discard it and verify the target workspace is unchanged. Trace: BG-13 / R-31.

### SF-27 Input-method-safe native forms

Preserve composition state and focused values during polling/re-rendering and keep UTF-8 Korean text intact end to end. Trace: BG-13 / R-32.

### SF-28 Workspace navigation and draft discard

A persistent titlebar action returns to the current run's Agent Room, and a discard action restores the last saved workspace and plan. Both are local view operations; discarding requires confirmation when edits exist. Trace: BG-09 / R-33.

### SF-29 Provider application icons

Resolve the installed desktop application for each built-in provider, render its icon as a bounded PNG data URL in UI snapshots and fall back to letter avatars. Trace: BG-14 / R-34.

### SF-30 Public product documentation

A concise README (introduction, screenshots, install, desktop walkthrough) and a complete user manual covering the desktop workflow, every CLI command, configuration and safety behavior, packaged with releases. Trace: BG-15 / R-36.

### SF-31 Manual plan authoring

Create an editable three-task plan draft in the workspace without launching a coordinator; reuse plan editing, validation, digest approval and repository-base binding. Trace: BG-16 / R-37.

### SF-32 Workspace settings and folder picker

A labeled settings section (goal, coordinator, repository with native folder picker, worktree option); saving does not require a repository. Trace: BG-17 / R-38.

### SF-33 Workspace menu

Current workspace shown in the title bar with a menu of recent workspaces, open folder and new workspace; switching happens in place under the one-window-per-workspace lease. Trace: BG-17 / R-39.

### SF-34 Workspace names and registry

Workspace name field above the goal; registry of named workspaces with create-by-name, add-folder, switch and remove-from-list. Trace: BG-17 / R-40.

### SF-35 Release change summaries

`CHANGELOG.md` holds one section per version tag; the release workflow publishes that section as the release notes and stops when it is missing. Trace: BG-10 / R-41.

### SF-36 Baseline commit prompt

Before planning, report repository cleanliness and changed files; on the operator's choice, commit all changes as the baseline and continue, or stop without changes. Trace: BG-13 / R-42.
