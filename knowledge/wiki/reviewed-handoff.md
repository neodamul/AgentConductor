# Automatic reviewed-target handoff (DD-43)

Sources: `src/workflow/assignment.rs` (`apply_handoffs`), `src/workflow/execution.rs`,
`src/workflow/scheduler.rs`, `tests/cooperative_handoff.rs`.

- A completed cooperative `handoff` marks the task `handoff` (transient), not `stopped`.
- After the JoinSet drains, the scheduler checks every attempt's ownership evidence is
  quiescent, then follows the task agent's reviewed `fallback` chain past restricted
  accounts and calls `transfer_role` (bounded handoff document, new session, dependent
  invalidation). No target, stop request or live process → `stopped`, as before.
- Recovery normalizes a persisted `handoff` to `stopped`.
- Lesson: authority for an automatic transfer must come from what the human reviewed in
  the plan, never from the directive text.
