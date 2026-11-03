# Architecture delta and trace

Status: approved design contract, 2026-11-04. Scope: issue #387 and its earlier draft PR #388.
These local `UI387-*` IDs describe this design delta; they do not renumber the main architecture register.
Implementation must link its registered requirements/tests back to these entries before claiming completion.

## Requirement → business goal → feature → use case

| Requirement | Business goal | Feature | Use case |
|---|---|---|---|
| UI387-R1 One Workspace with independent Goals | UI387-BG1 Preserve project context | UI387-SF1 Workspace/Goal lifecycle | UI387-UC1 Add a Goal, retain folder/coordinator, choose roles, plan, execute, review |
| UI387-R2 Consistent light/dark six-step screen | UI387-BG2 Reduce navigation and visual confusion | UI387-SF2 Shared shell and step mapping | UI387-UC2 Move through six steps without losing state |
| UI387-R3 Complete, compact status and human control | UI387-BG3 Make current work and intervention immediately understandable | UI387-SF3 Agent/task rail, quota icon, briefing | UI387-UC3 Inspect an agent, select a task, read factual briefing, intervene |
| UI387-R4 Docked details with independent toggles | UI387-BG4 Use available screen area efficiently | UI387-SF4 Left and right dock control | UI387-UC4 Open details while continuing work in the center |
| UI387-R5 Stable headers, footers, and constrained text | UI387-BG5 Keep actions reachable at smaller widths | UI387-SF5 Scroll body and hover disclosure | UI387-UC5 Scroll long content without moving the description/actions |
| UI387-R7 Preserve role Fallback/model selection and a shared choice-control standard | UI387-BG3 Preserve human control | UI387-SF7 Anchored agent/model/Fallback choices | UI387-UC7 Select coordinator or role/fallback at its field without a separate modal |
| UI387-R6 Coherent Goal list/result/folder/deletion | UI387-BG6 Manage saved work without accidental loss | UI387-SF6 Goal slide-over and result/delete dialogs | UI387-UC6 Open results/folder, or explicitly permanently delete one Goal |

Existing traces retained: R-29 status, R-30/R-106 decisions, R-38 Workspace settings,
R-45/R-196 progress, R-55/R-150 Goal history, R-102 ordering, R-104 activity,
R-105/R-109 integration, R-110 feedback, R-111/R-156 review/merge,
R-112/R-195 phases, R-118 close confirmation, R-120 thinking/interruption,
R-127 availability, R-159 activity facts, R-164 lessons, R-172 directions, R-180 useful information.
Those references identify capabilities to preserve; the mock alone does not verify their implementation.

## Functional / non-functional analysis

| ID | Contract |
|---|---|
| UI387-FR1 | Derive the displayed step from persisted Goal/phase/run state; do not store a competing UI lifecycle |
| UI387-FR2 | Separate Workspace configuration from Goal configuration and keep internal folder IDs out of workflow language |
| UI387-FR3 | Show task selection, assignment/model/schedule, status, evidence, and human approvals from existing backend data |
| UI387-FR4 | Retain existing factual briefing fields and source/freshness/error information |
| UI387-FR5 | Separate docked inspector, slide-overs, and centered result/delete dialogs |
| UI387-FR6 | Resolve and open the selected Goal folder through the native host, not by guessing a path in UI code |
| UI387-FR8 | Show Agent/Model → Fallback/Model per role; retain profile defaults and explicit role overrides; reset incompatible choices |
| UI387-FR7 | Confirm irreversible deletion; stop owned jobs; delete only the selected Goal and report failure accurately |
| UI387-NFR1 | UI event handling stays responsive during process, quota, model, MCP, Git, and filesystem work |
| UI387-NFR2 | Keep dependencies and host resource use minimal; do not add frameworks or polling to reproduce a visual treatment |
| UI387-NFR3 | Both themes/languages preserve information, accessible names, keyboard/focus behavior, and contrast |
| UI387-NFR4 | Layout remains bounded when both rails are open and labels are long |
| UI387-C1 | Existing CLI login/subscription authentication and CLI-first adapters remain authoritative |
| UI387-C2 | Native Rust implementation must not embed this documentation's HTML/JS as a substitute |

## Quality attribute scenarios

| ID | Stimulus / environment | Required response / measure |
|---|---|---|
| UI387-QAS1 | Operator opens right details while an agent works | Center shrinks; no overlap/backdrop/inert state; center and left remain interactive |
| UI387-QAS2 | Operator scrolls a long card at a short viewport | Header/footer coordinates remain unchanged; body scroll position changes |
| UI387-QAS3 | Long Korean tab/path at constrained width | No column overflow; ellipsis where specified; full text on hover and complete accessible name |
| UI387-QAS4 | Operator changes light ↔ dark | Same elements and geometry; critical text/background contrast ≥4.5:1 |
| UI387-QAS5 | Provider reports a limit/error/unsupported quota | Show supplied windows/reset/errors honestly; no fabricated balance; UI remains interactive |
| UI387-QAS6 | Operator cancels permanent Goal deletion | No files, records, jobs, or list entries change |
| UI387-QAS7 | Operator confirms deletion of a Goal with owned jobs | Cancel and clean up those jobs, verify the owned root, then delete or report failure; unrelated data remains |
| UI387-QAS8 | Slide-over opens/closes/reopens rapidly | No horizontal frame scroll; no stale close timer or invisible input blocker |
| UI387-QAS9 | Goal result is opened from its list | Same result popup as completion; close leaves the original list stable |

## Design drivers and decisions

| Driver | Decision |
|---|---|
| UI387-DDR1 Shared backend state and minimum runtime overhead | UI387-DD1 Reuse current domain snapshots, commands, and approval protocol; introduce no parallel orchestration model |
| UI387-DDR2 Clear information hierarchy | UI387-DD2 One shell, six steps, task-centric left list, optional docked inspector |
| UI387-DDR3 Predictable intervention | UI387-DD3 Fixed description/action areas, asynchronous command completion, explicit exceptional states |
| UI387-DDR4 Safe saved-work lifecycle | UI387-DD4 Goal list slide-over; shared centered result; separately confirmed native deletion |
| UI387-DDR5 Same visual contract across implementations | UI387-DD5 Freeze semantic tokens, dimensions, interaction matrix, paired images, and known preview limits |

## Backend ownership boundaries

The UI renders state and sends intents; Rust owns process lifecycle, capability detection, task scheduling,
quota/model queries, native file opening, deletion, approval records, and repository changes.
No synchronous external process, HTTP/MCP, Git, or filesystem work may block a UI callback.
Use the existing worker/event mechanisms; do not create a single blocking request queue for unrelated work.
Background snapshots must not starve interactive commands. Deliver state changes through events/listeners
where supported and independently bounded refresh where polling is necessary.

Native folder opening resolves persisted Goal metadata. Deletion checks the canonical owned path,
handles links safely, cancels workers, serializes conflicting lifecycle actions, and reports partial failures.
No browser `file:` URL or preview DOM mutation is evidence that these backend capabilities exist.

Quota state keeps provider/window semantics, observation time, reset time, availability, and error separate.
Model discovery respects CLI capabilities and existing login; do not require an OpenAI-compatible API key
for CLI agents or invent a `/models` command for a CLI that does not provide one.

Task/Goal outputs still require exact diff/tests and human approval before merge. The new layout does
not relax dangerous-command, external-transfer, secret-access, deployment, or merge approval gates.

The anchored selection list is shared by Workspace/roles/Fallback, task inspector, and Settings.
It is a nonmodal choice surface, distinct from a Goal list slide-over or irreversible confirmation.
Normal selection updates the local draft immediately; persistence and live execution remain separate intents.
Model/fallback catalog queries must not block opening/using other controls.

## What the approved design changes

The earlier PR #388 text placed details inside a body tab, retained executable paths/fixed sessions,
added permanent progress text, and offered a new-Goal shortcut in the result body. Those decisions are
superseded by [behavior](behavior.md). Its original screenshots are not this design's acceptance images.
The general design-guide motion restriction is amended only for the approved running-avatar pulse.
Other architecture/security/process requirements continue to apply.
