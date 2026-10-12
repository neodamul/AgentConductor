# System features

| ID | Feature | Goal | MVP |
|---|---|---|---|
| SF-01 | Registered subprocess profiles and parsers | BG-01 | Included |
| SF-02 | Scoped plan (1–12 tasks, R-53) and dependency scheduler | BG-02 | Included |
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

Workspace name field above the goal; registry of named workspaces with create-by-name, open-saved (listing workspaces stored in `~/.agentconductor`), switch and remove-from-list. Trace: BG-17 / R-40.

### SF-35 Release change summaries

`CHANGELOG.md` holds one section per version tag; the release workflow publishes that section as the release notes and stops when it is missing. Trace: BG-10 / R-41.

### SF-36 Baseline commit prompt

Before planning, report repository cleanliness and changed files; on the operator's choice, commit all changes as the baseline and continue, or stop without changes. Trace: BG-13 / R-42. Untracked OS metadata files can be excluded locally instead (R-44).

### SF-37 Messenger connectors (planned)

A provider-neutral notification and decision channel with a Slack connector and a generic company-messenger connector (outgoing webhook plus signed inbound callbacks or polling), per-workspace channel mapping, redaction policy and linked approvers. Trace: BG-18 / R-43.

### SF-38 Planning progress indicator

Job state with start time and log location; review area shows coordinator, elapsed time, ready notice with scroll, and failure reason. Trace: BG-12 / R-45.

### SF-39 Readable plan instructions

Sectioned, line-broken instruction format required from the planning coordinator in the goal's language; auto-growing instruction editor (minimum six rows). Trace: BG-13 / R-46.

### SF-40 Agent role rules

Workspace role table (planning = coordinator; verification, development, documentation, knowledge, testing → ordered registered agents, each with an optional observed model) passed to the planning coordinator; task `role` field; rule enforcement on generated plans; role shown in plan review; numbered steps 1 Workspace settings → 2 Agent roles → 3 Review the plan. Trace: BG-19 / R-47.

### SF-41 Immediate agent registry persistence

A dedicated, revision-checked request stores only the agent list, repairing the coordinator and role rules when an agent is removed. Trace: BG-09 / R-48.

### SF-42 Automatic model catalog preparation

Page-driven sequential discovery for built-in agents without a catalog while no plan proposal exists and nothing is unsaved; catalog preservation on save. Trace: BG-10 / R-49.

### SF-43 Workspace settings export/import

Versioned `agentconductor.workspace.json` written to a chosen folder (default: the repository); import from a folder with preview, local-state preservation and capability re-verification. Trace: BG-20 / R-52.

### SF-44 Variable-size plans and plan-document import

Plan validation for 1–12 tasks, coordinator prompt for the smallest disjoint split, add/remove task controls in review; **Read plan…** converts a chosen Markdown document through the read-only planning path with role rules. Trace: BG-02/BG-19 / R-53, R-54.

### SF-45 Slug work trees

Slug creation from goals (accumulating), per-slug folder tree in the repository's `.agentconductor/` (locally excluded from Git status) or the workspace data folder. Trace: BG-21 / R-55.

### SF-46 Phased planning and execution

Analysis document plus ordered phases of tasks from the coordinator; one run per phase; task files move task → verification → completed with the run's lifecycle. Trace: BG-21 / R-56.

### SF-47 Progress model and tree verification

Progress derived from file locations; workspace and export record the derived state; a verifier reports any disagreement. Trace: BG-21 / R-57.

### SF-48 Slug archives in export/import

Store-only `<slug>.zip` archives beside the settings JSON, listed with progress; import checks presence, names and contents before restoring. Trace: BG-20/BG-21 / R-58.

### SF-49 Plan rules file

`~/.agentconductor/plan-rules.md` (created with defaults, editable in the window) and a workspace override in the settings; sent to the planning coordinator. Trace: BG-21 / R-59.

### SF-50 Passive Claude quota collection

Normalize owned `rate_limit_event` messages into quota snapshots during execution; display source, freshness, independent reset windows and unknown utilization. Claude Refresh checks retained observations without launching a worker. Trace: BG-22 / R-60.

### SF-51 Inline quota badges — Compact capacity badge beside paid agent names in workspace and Agent Room lists. Trace: R-61 / BG-22.

### SF-52 Configured MCP goal imports

OpenCode-compatible local/remote MCP registration in workspace settings, source bindings, goal-reference chips, previewed read-only import and source-linked planning. Trace: BG-23 / R-62.

### SF-53 Per-agent quota refresh and compact actions

Independent per-agent refresh deadlines, manual refresh icons and accessible agent action icons. Trace: R-63 / BG-22.


### SF-54 Compact plan-aware quota control

A single visual group contains slash-separated percentages on one line and the per-agent refresh button. Period labels stay in the tooltip. Provider plan metadata and window durations survive cache persistence. Trace: R-64 / BG-24.

### SF-55 Native agent preferences

Persist per-agent cadence, resolve executable paths through filesystem lookup, offer an OS file picker, and embed shared AC artwork in the native executable. Trace: R-67 / BG-25.

### SF-56 Provider-driven agent profile

Selecting a provider fills ID, resolved executable, arguments, input, permission, sessions and cadence from that provider's template; renaming an agent rewrites references (coordinator, roles, fallbacks). Trace: R-68 / BG-26.

### SF-57 Exhausted-quota reset time

The quota control appends the latest reset time among exhausted windows; the tooltip lists every window's reset. Trace: R-69 / BG-26.

### SF-58 Language setting and UI translation

A machine-wide preference (`~/.agentconductor/preferences.json`, `language`: `en` | `ko`) is embedded in the page at load; a Settings dialog changes it at runtime. A translation table maps exact English interface strings (text, placeholder, title, aria-label) and known prefixes; untranslated strings stay English. Trace: R-70 / BG-27.

### SF-59 On-demand fallback catalog

The fallback model select reflects the chosen fallback agent's catalog, queues one model fetch when missing and refreshes in place. Trace: R-71 / BG-26.


### SF-60 Windows provider metadata parity
Resolve runnable Windows launchers, collect Claude /usage through a short-lived ConPTY session, retain existing Codex app-server collection and show original built-in provider badges when installed artwork is unavailable. Trace: BG-28.


### SF-61 Explicit operator model catalogs
Custom Models opens a compact ID editor with optional observed JSON import, explicit provenance and no subprocess/network query. Trace: BG-29.

SF-37 phase 2: inbound decision channel by bounded polling (no public URL, no resident process): Slack `conversations.history` with a bot token, or a company relay returning HMAC-SHA256-signed decisions; linked approvers per workspace; decisions applied through the existing exact tool-decision path.

SF-46 amendment (R-74): automatic slug naming with Hangul romanization and uniqueness; no slug input.

### SF-62 Budget-aware quota normalization
Collect CLI-reported budgets independently of rolling usage windows, preserve scope, allocation, reset and provenance, and render remaining/allocated compactly. Trace: BG-30.


### SF-63 Native window ownership
Use an in-process AppKit picker on the main thread and Windows GUI subsystem with console-free subprocess creation. Trace: BG-31.


### SF-64 Separate directory settings from execution baseline validation
Validate selected directories at save time, and validate Git plus committed baseline only at planning entry. Quota does not depend on either. Trace: BG-32.
