# Ownership-checked Goal row actions

Source: GitHub #402, R-211 / DD-223. Backend branch: `codex/issue-402-goal-actions`.

Read historical Goal evidence through the startup bridge instead of switching the active Goal
or replacing the run controller. Reuse the Controller public report, diff/test evidence and task
knowledge/lesson data; verify the report revision again after collection.

Permanent deletion requires the current startup revision, registered Goal ID, exact confirmed
Goal text and folder. A direct-child ordinary folder with the app marker is eligible; an alias,
reparse point, missing marker or external run binding is refused. Request owned planning/run
stop and return `stopping` until drain and run/process ownership checks pass. Never sleep while
holding settings. During slow filesystem removal, snapshots remain available while competing
mutations receive an explicit retry error. Registry removal follows successful folder deletion;
filesystem/registry failures stay visible. Central run/audit evidence remains.
Retained runs get a goal-deleted marker under their locks; stale requests cannot resume them.

Keep UI cancellation separate: Cancel sends nothing; default focus is Cancel. A stopping result
must not optimistically remove a row. Retry only the same confirmed Goal/path with a fresh
revision; changed confirmation content requires review. A historical Result reply includes
`report`, `evidence` (diff/test_output) and `details` (task lessons/knowledge).

Local evidence: tests/goal_row_actions.rs, tests/unit/goal_deletion.rs. Native Finder/Explorer,
Windows reparse points and complete webview acceptance require package testing; the backend
unit does not establish those outcomes.
