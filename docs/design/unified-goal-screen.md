# One goal screen with a single stepper (GitHub #387)

Status: design, approved for handoff 2026-11-03. Owner of the Rust UI port: Codex (#9 split: #365, #370, #373, #374). The JS UI follows the same design.

## Problem
The workspace screen (1 Workspace settings · 2 Agent roles · 3 Review the plan) and the Agent Room (Plan · Tasks · Verify · Merge) are two screens that count the same goal twice, use different layouts (two vs three columns), and scatter actions over four places.

## Design
- **One screen per goal, one stepper:** 1 Workspace → 2 Agent roles → 3 Plan → 4 Tasks → 5 Verify → 6 Review and merge, grouped as prepare (1–3), run (4–5) and finish (6). Done steps show ✓, the current step is highlighted, and done steps open read-only while a run works. There is no "← Workspace" switch.
- **One left list:** the CLI agents list stays in every step. Each card shows the agent, its quota and one live line (idle, the task it works on, waiting, done).
- **One card per step:** each step uses the same card: eyebrow "Step n · name", a title, a one-line explanation, the body, and one action row at the bottom right. Only the next action is primary (Approve and run, Send direction, Open changes to review, New goal).
- **Step 4:** activity, changes and tests, raw logs, lessons and details (today's task inspector) are tabs of the selected task card. The Activity tab keeps R-120's folded thinking line and R-159's facts.
- **After step 6:** the card becomes "Goal done" (result, tests, lessons). The primary action is "New goal", or "Plan the next phase" when phases remain (R-56, R-195). A new goal keeps steps 1–2 (workspace-level settings, ✓ and editable) and starts at step 3. Finished goals stay listed under the title (R-55, R-150).

## Screens
Rendered from `unified-goal-screen.html` (the app's own color tokens and sizes; arrows switch the states):

| State | Image |
|---|---|
| Step 3 · plan review | ![](1-plan-review.png) |
| Step 4 · tasks | ![](2-tasks.png) |
| Step 6 · review and merge | ![](3-review-merge.png) |
| Goal done | ![](4-goal-done.png) |
| New goal · step 3 | ![](5-new-goal.png) |

## Traces it touches
R-38 (workspace settings), R-45/R-196 (visible progress), R-99/R-111 (next step always shown), R-104 (activity), R-112/R-195 (phases), R-118 (close dialog), R-120/R-159 (activity facts), R-165 (inspector folding is replaced by tabs), R-180 (only useful information). A requirement entry (next free number on `main`, see docs/README.md reserved block) is added when implementation starts.

## Implementation order (suggested)
1. Single stepper and card on both pages.
2. One agent list with live lines.
3. Remove the screen switch (the run renders in steps 4–6 of the same page).
