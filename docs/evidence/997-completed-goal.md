# Completed Goal immutability — #997

## Plan / architecture trace
- Requirement: a completed Goal is immutable; changes belong to a new Goal. A completed non-final Phase may advance through the existing next-Phase flow.
- Business goal: preserve approved delivery evidence without preventing investigation.
- System feature: one read-only policy at the native boundary and in both UI surfaces.
- Use case: reopen a completed Goal; click stages 1–6, inspect saved settings/roles/plan/tasks/logs/diffs/tests/results/history/briefing/quota, ask the coordinator questions, or add/delete a Goal.
- Functional analysis: block Goal/run edits, task assignment/models, schedule/recovery/feedback, tool decisions, approval and re-merge. Retain account metadata refresh and unrelated global settings. Do not treat chat as a run review request.
- QAS: stale enabled controls or handcrafted native requests cannot launch a mutation on completed data; informational actions work in light/dark and English/Korean.
- Design driver: consistency and evidence integrity, without polling threads or new dependencies.
- Design decision: share a small UI read-only policy, enforce completed-report checks before native mutation dispatch, and keep the existing workflow guards for races and direct CLI calls.

## Task checklist
- [x] Plan: user reviewed the completed-Goal mock and requested implementation.
- [x] Task / Red: native mutation matrix and actual-browser completed-Goal regression.
- [x] Task / Green: shared UI policy, native guards and stage navigation.
- [ ] Verify: targeted native/UI tests, existing regression, local lint/build.
- [ ] Completed: PR with audit checklist, merge and issue closure after verification.

## Audit contract
- Workspace/Goal text, roles, per-Goal model and source application: locked.
- Plan/task edits, auto-merge and phase regeneration: locked for the completed Goal.
- Room feedback/review request, reassignment, schedule/resume/recover/stop, tool decisions, approve/reject/merge and amendment undo: locked.
- Plan/task expansion, session/log/history views, diffs/tests/results, quota refresh, copy, folder opening and briefing: available.
- Coordinator chat/history/reset: available; questions cannot mutate the completed run.
- New Goal and confirmed deletion: available.
- Global configuration and forward continuation to a remaining Phase must not accidentally unfreeze an archived run.

## Verification so far
- Native Red: completed Save accepted, Room revise launched, coordinator review request lacked early completed-state refusal.
- Native Green: dispatch refusal matrix plus existing completed chat state-drift fixture passed.
- Actual DOM: completed Goal mutations, saved stage clicks, logs, lessons, evidence, results, quota availability, chat send, Undo lock and editable New Goal passed in light/dark × English/Korean.
- Documentation: 110 reference captures and 34 contrast pairs passed; 47 Python checks passed.
- Whole-target Clippy passed (a pre-existing Rust 1.100 needless-range-loop warning in fallback test was replaced with enumerate).
- Full local native/parity regression remains in progress; no Windows runtime acceptance or release is claimed.

Final focused native follow-up: 15 tests passed across completed_goal, chat_state_drift and goal_row_actions. The one planning-cancellation revision failure in the loaded full run did not reproduce in this serial follow-up. Native completed/chat/automatic-merge follow-up also passed 10 tests across four suites.

Joined journey follow-up: goal_lifecycle_runtime now expects a read-only completed Workspace rather than an enabled Save button. It passed 32 continuous intervention checks plus six restart/restore checks in each light/dark × 840/960/1280 cell (228 checks). This is an intentional #997 expectation change, not a relaxed assertion: the test now requires Goal.readOnly and disabled Save after completion.
