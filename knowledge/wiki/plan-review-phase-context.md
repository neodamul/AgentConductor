# Plan review Phase context

Issue: #944. Design: DD-125 amendment.

The ready proposal is the authoritative editable task list. Do not repeat its task IDs in another Phase box or an editing instruction above the cards. The Plan eyebrow carries Phase position/name; the existing position line carries the selected task.

Rail navigation temporarily hides the editor while displaying another Phase. Move the same editor nodes back when returning: rebuilding them would lose operator input. Completed/future Phase summaries remain read-only. Suppress only task-ID-only briefs, keeping meaningful historical explanation visible.

Evidence: `tests/plan_phase_pick_runtime.cjs`, `tests/design_parity/plan_review_spec.cjs` and `docs/evidence/944-plan-review.md`.

## Pending re-plans (#958)

A proposal digest alone cannot identify a pending plan. Include target Phase, directory and proposal lifecycle in the renderer key. Bind the pending heading before the no-plan early return, clear obsolete editable rows/tools, and reset historical Phase selection when the proposal target changes. Reuse the existing planning progress/elapsed output and job guards. Do not reset edits for unrelated metadata jobs.
