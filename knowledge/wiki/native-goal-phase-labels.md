# Native goal phase labels

The released Workspace labels the action for the next phase with its ordinal, for example `Plan phase 2`. Keep this label localized through `startup.plan_phase` so the visible button and its AccessKit name agree. The rendered goal-list regression checks the accessible button label, while phase-progress tests verify the action still dispatches for the currently active phase.

The goal view is kept below the repository's 300-line Rust source limit by storing the goal-stage and phase-action label helpers in `workspace/goals/labels.rs`.

This is a component parity check only. It does not establish packaged workflow parity or advance issue #9's 0/25 end-to-end acceptance count.
