# Behavior contract

Authority: [approved design](README.md), 2026-11-04. English labels below describe the behavior;
the reference images intentionally retain the reviewed Korean UI.

## Workspace, Goal, and work products

A Workspace links a working directory and stores the coordinator and shared agent configuration.
It normally starts with one Goal and can contain more. Each Goal has its own description, role choices,
plan, phases, task state, audit trail, and result. Internal Goal folders use stable English identifiers.
Do not show "slug" as a workflow concept, invent phonetic transliterations, or rename existing folders
when display text changes. Backend folder metadata is authoritative.
Approved commit/merge results apply to the Workspace's linked working directory.
Git worktrees are optional; a non-worktree/non-Git workspace must not acquire a Git-only prerequisite.

## Titlebar and docked layout

Left-to-right order:

1. App icon, app name, version.
2. Divider exactly aligned with the left rail's right boundary.
3. Three-line menu icon, left rail toggle, Workspace name.
4. Flexible remaining space.
5. Downward update icon, existing coordinator-chat icon, existing settings gear, right inspector toggle.

The version is right-aligned immediately before the divider. Use the existing chat/gear artwork.
Toolbar buttons have transparent resting surfaces, a quiet hover fill, and a pointer cursor.
The menu opens below its actual button position, including after resizing.
Workspace name opens Workspace settings. Workspace sharing/export/import and agent installation help
remain menu actions; the mock's short menu is illustrative, not the complete menu inventory.

The left toggle hides/restores the whole rail and reallocates its space to the center.
The right toggle opens/closes a docked task inspector and reallocates center width.
The inspector starts **closed**, has no internal close button, no backdrop, no slide animation,
and never blocks interaction with the center or left rail. Both rails toggle independently.
Maximizing/fullscreen expands content space without scaling icons, type, or the 1.5px status border.
Its body scrolls independently. Main body, rails, and dialogs must not scroll one another.

## Left rail

Tabs appear as **Agents** then **Phases & tasks**. Steps 1–2 initially select Agents;
planning, execution, verification, and review initially select the task view.

Agent cards preserve Add, edit, model catalog/reload, delete, permissions, fallback/coordinator facts,
model count, drag reordering, and up/down ordering. Agent removal uses the same red 18px trash
SVG as Goal removal, not the platform emoji. Executable paths remain editable in profile settings
but are not printed on cards. Do not show a fixed session-count capacity as current activity.

Agent/task status is shown by the avatar border: green for running/completed, yellow for waiting,
red for failure/attention. Only running avatars pulse. Hover/focus or activation exposes status text;
a small attention marker signals a decision/error. Do not insert a permanent status sentence in cards.
Reduced-motion mode removes pulses. Meaning is also available as text/accessibility labels.

Quota is a dual-ring icon beside the agent name. Each window has a muted, distinct color
shared with that window's label dot and meter in the clicked information popup. The windows come from provider data: do not assume
5-hour/week or manufacture an Enterprise allocation. Error/unsupported states must be distinguishable
from a real zero. Click opens actual values, window labels, reset time, observation time, and refresh;
errors show the real message and retry. Avatar status and quota activation are separate actions.
Quota updates stay asynchronous; default interval is five minutes, editable per agent.

The task rail shows the **current Phase and Task**, task completion count, and elapsed time.
Each row shows a small agent avatar, task number, task description, and ownership/dependency context.
The agent avatar carries status; do not replace the task description with a standalone agent name.
Task rows also retain compact Completed/Running/Waiting tags (and blocked/approval states),
so progress/dependency order is readable. Tags use green for running/completed, yellow for
waiting, and red for blocked/refused (displayed as 막힘 in the Korean reference). The existing dependency text still distinguishes parallel work.
A trailing chevron, quiet row separator, hover fill and selected accent edge make every task selectable.
Selecting a task updates the center and an already-open inspector without losing the chosen subview.

## Center and scrolling

The Goal header is compact. Step 1 shows Workspace settings while the Goal is being typed;
do not echo each keystroke as a second Goal title. Later steps use the saved Goal name. Their Goal list entry uses the same leading list icon, outlined button and
Goal-count badge as the Workspace Goal section, never a plain text link. All badges update together
when Goals are saved/deleted; an unsaved draft is not included.
The information row below is fixed-height, including when empty. Team briefing is an icon button immediately to the right of Goal list in the same action area,
including the Workspace Goal section. Its tooltip/accessibility name is Team briefing.
Completion counts and elapsed time belong to the left task rail, not a duplicate right header.

One stepper has six tabs, with neutral desktop-style tabs and a shared, subdued background for each
prepare/run/finish group. Prepare, Run, and Finish are visible icons inside three background groups
(1–3, 4–5, and 6), with no extra caption row above. Tooltips and the selected step description also
retain the group name. Avoid large pastel tab backgrounds. Selected tabs visually join their content. Tab labels remain single-line:
use ellipsis when constrained, with full text on hover; preserve complete accessible names.

Within every step card, keep the eyebrow/title/explanation header and action footer stationary.
Only the middle body scrolls. Long content must not push actions below the visible card.
Workspace and Goal are separate sections, each with breathing room above its heading.
Action explanations may wrap within the footer; do not silently cut a warning or required decision.

## Step flow

| Step | Body and transition |
|---|---|
| 1 Workspace | Name, working directory, coordinator, optional worktree; separate Goal section with description, import/optimize, Goal list and + New Goal; Save → 2 |
| 2 Agent roles | Goal-specific roles, agents, session models; prior roles are only a starting point; confirm → 3 |
| 3 Plan | Generate/progress/review, current Phase/Task, editable task scopes/dependencies/tests; approve → 4 |
| 4 Tasks | Selected task header; Activity, Changes and tests, Raw logs, Lessons; direction composer and context-appropriate actions |
| 5 Verify | Integration tests with outcomes/exit codes, coordinator judgments, failure feedback/rework; success → 6 |
| 6 Review and merge | Review changes (변경 검토), exact candidate, changed files, tests, human review/approve/reject/revise, then merge/accept |

Do not introduce steps 7–10 because the reference has ten scenarios. Scenario numbers are preview tooling.
A completed merge opens the result popup while preserving the step-6 context. If phases remain,
continue the same Goal through its next phase instead of treating a phase as a new Workspace.
New Goal starts at step 1 with the Workspace folder/coordinator preserved and prior roles available.

## Common anchored selection standard

All choice fields use the same anchored popup list: Workspace, coordinator/model, role assignment,
Fallback/model, task inspector, and Settings. Click the field itself; place the list immediately below
it (6px gap), or above if needed, keeping it inside the app frame. Do not open a separate assignment
modal or require an Edit → dialog → dropdown sequence. Current choice has a checkmark/tint.
Selecting a normal item updates the visible draft and dismisses the list immediately. The enclosing
Workspace/Goal save action persists the draft; choosing a value is not permission to change a live run.
Click outside/Escape dismisses without changing the value; Escape restores focus. Arrow keys move
through enabled options, Enter/Space select, and Tab dismisses to continue field navigation.
A nested choice list consumes Escape before its containing slide-over.

Step 1 offers coordinator Agent and Model side by side. Step 2 shows, per role:
**Role | Agent | Model → Fallback agent | Fallback model**. The muted 18px arrow separates primary
and alternate pairs. The UI display name is **Claude Code**; internal ID/CLI command stays `claude`.
Fallback must remain available, including self-hosted CLI choices. No fallback disables its model field.
Do not allow the primary agent as its own fallback; validate fallback cycles/availability in the backend.
Preserve the existing agent-profile fallback and model behavior when adding Goal-specific overrides.
Resolve an explicit role override before an inherited profile default; do not mutate unrelated roles.

Changing Agent rebuilds its supported model choices and resets the model to CLI default;
changing Fallback resets the fallback model. Never silently retain an incompatible model.
Catalog discovery remains asynchronous. Unsupported catalogs offer CLI default or explicit model-ID
input in the same popup; only that manual-entry branch needs an Apply button. Empty IDs are invalid.
A new Workspace coordinator seeds the planning role, subject to Goal-specific confirmation.
Catalog labels and models in the reference are fixtures, not a claim about current provider support.

## Task inspector contents

The right dock replaces the old task inspector without losing its capabilities:

| Section | Data/actions |
|---|---|
| Ownership and task | Owned paths, task/session IDs, instruction, dependencies, verification command |
| Assignment and model | Reassign to a registered agent, select/apply a supported session model, catalog availability |
| Availability | Actual restriction/reset information, manual resume schedule, countdown where available |
| Review candidate | Candidate revision, tests, exact evidence, approve/reject/merge gates |
| Tool requests | Pending commands/inputs and their existing human decision flow |
| Account usage | Selected account, provider windows, observation/source, refresh, honest unsupported/error state |
| Session metrics | Reported tokens, elapsed time, completed tasks, reported cost; never infer subscription cash |

Before a plan/task exists, show an empty-state explanation. Candidate actions stay disabled until
the relevant evidence/review conditions are met. Opening a panel must not approve, resume, or execute anything.
A running CLI without steering receives changes through the established checkpoint/memory protocol.

## Slide-overs and centered dialogs

Goal list, briefing, chat, and settings open from the right **over** the unchanged content.
Only one slide-over is active at a time. Confine it to the app frame; prevent animation/focus-induced
horizontal scrolling. On close, restore focus without scrolling the underlying app.
Backdrop/Escape/close controls apply to these slide-overs, not to the docked inspector.

The Goal list shows Goal name, state, current Phase/Task, and resume/result action.
A folder icon beside that action opens the exact persisted Goal folder in Finder/Explorer.
The small red trash icon is at the far left of the same row, separated from the result/folder actions;
there is no dedicated deletion row. Its icon matches the folder glyph size. Compact spacing is required.

Trash first opens a centered confirmation naming the Goal and exact folder. Warn that the entire
folder and descendants are permanently deleted, without backup, recovery, or moving to Trash.
Cancel is the default. Cancel changes nothing. Actual deletion must stop/cancel owned jobs first,
finish cleanup, and delete only that Goal's verified folder; failure remains visible.
Never delete a Workspace root, unrelated folder, or follow a link outside the owned Goal root.
Deleting the last Goal leaves a usable empty list with + New Goal. The preview never deletes disk files.

The centered result popup immediately includes summary, changed files, tests, commit/target directory,
and lessons. Do not hide these behind "Result details" or add a "Start new goal…" shortcut here.
Both automatic completion and Goal-list Result use this same popup. Closing a result opened from
the list restores the unchanged list without another sliding animation.

Team briefing preserves snapshot/state/time, per-session current work, session/model/attempt,
process/observation, last activity, memory report/source/freshness, restrictions/attention, next action,
and approval summary. It is read-only and reflects recorded facts, not generated guesses.
Chat and settings keep all existing functions even where the preview uses only sample controls.

## Statusbar and preview controls

Bottom left: connection/local-run status. Bottom right: actual worktree/folder identity and branch,
with small aligned folder/branch icons. Full path/name is available on hover. If worktrees or Git
are absent, report that state honestly; sample path/branch values are not backend defaults.
The update icon belongs in the titlebar, not this statusbar.

Preview arrows sit **below the app image**, in normal document flow. Their fixed-width label slot
keeps left/right arrow positions stable across all scenarios. They are not viewport-fixed overlays.
They are not application controls and must not be shipped.

## Profile editing and closing an active run (Claude review)

The agent-card Edit button opens the profile editor. The executable command/path remains visible
and editable there, with Browse, even though it is omitted from compact cards. Preserve provider,
agent ID, JSON argument arrays, read-only arguments, stdin/task-file input, concurrent sessions,
quota refresh interval, fallback agent/model and the existing CLI-permission explanation.
Apply validates and closes; Cancel changes nothing. This reference stores fixture edits only,
and Browse illustrates the host file picker without accessing or executing files.

R-118 is inherited from merged PR #383, not a new implementation in this design. When the actual
backend reports an active run on window close, show a centered confirmation with three choices:
Cancel, Stop the run and close, Keep running in the background. Keep running is the focused default.
Cancel leaves the window/run/drafts untouched; Keep closes only the window and preserves the run;
Stop requests the durable stop before closing. Reopening restores the recorded run.
The merged host's fallback for a second close within 10 seconds or an unresponsive page remains
backend behavior, not an extra mock feature. The menu's Close action opens this preview safely;
choosing an option never closes the browser or stops a real process in the reference.

## Plan approval presentation

GitHub #830: Plan approval proceeds directly from the reviewed plan into execution.
Do not display the legacy Exact plan approval evidence disclosure or reopen a separate
confirmation dialog. The start request still carries the exact reviewed plan digest;
backend validation of plan identity, permissions, and scope remains mandatory.

## Task removal hint

GitHub #833: expanded Plan tasks show a localized tooltip on Remove task:
"Only manually added tasks can be removed." The hint also applies when the
control is disabled and must update when the UI language changes.

## Plan introduction freshness

GitHub #831: when a Goal or Workspace has no current plan proposal, show the
instruction to generate a plan. Clear the previous Goal's task-count introduction
before planning begins; only describe proposed tasks once that proposal exists.

## Goal-wide Phase position

GitHub #832: the Plan heading and selected task summary use the current proposal's
Phase in the active Goal's full Phase list. Task position remains local to that
Phase. Single-Phase plans without Phase metadata retain Phase 1/1.

## Menu entry points (0.2.4)

The Workspace group retains recent/New/Open/Export/Import actions. Workspace settings are opened through step 1. There is no Agents or View group: use Add in the Agents rail and the titlebar panel toggles. Plan rules, Messenger, Settings, Help and Close remain available. These changes remove duplicate navigation only.

## Bug reports

Help → Report a bug opens a prefilled issue composer in the default browser. The
application target shares the updater repository preference, including Enterprise
HTTPS hosts. GitHub handles login and final submission; automatic context is limited
to version, OS and architecture. The preview demonstrates the same Help action.

## MCP registration and automatic tool selection (#882)

Normal MCP settings lists registered servers, discovery status and errors, and the add/edit form. It has no Jira/Confluence tool-binding selectors, argument form or duplicate text catalog. The workspace coordinator chooses tools and schema-valid arguments for Goal source preview; compatible legacy binding data remains internal and round-trippable through advanced JSON.
