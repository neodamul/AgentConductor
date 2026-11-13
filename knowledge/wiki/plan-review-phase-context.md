# Plan review Phase context

Issue: #944. Design: DD-125 amendment.

The ready proposal is the authoritative editable task list. Do not repeat its task IDs in another Phase box or an editing instruction above the cards. The Plan eyebrow carries Phase position/name; the existing position line carries the selected task.

Rail navigation temporarily hides the editor while displaying another Phase. Move the same editor nodes back when returning: rebuilding them would lose operator input. Completed/future Phase summaries remain read-only. Suppress only task-ID-only briefs, keeping meaningful historical explanation visible.

Evidence: `tests/plan_phase_pick_runtime.cjs`, `tests/design_parity/plan_review_spec.cjs` and `docs/evidence/944-plan-review.md`.
