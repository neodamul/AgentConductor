# Completed Goal immutability

Verified: 2026-11-15. Sources: [raw observations](../raw/completed-goal-997.json), [architecture and audit](../../docs/evidence/997-completed-goal.md), [read-only chat](read-only-chat-state.md).

A completed Goal is an archive, not an editable plan. Both the native request boundary and the UI enforce this rule. Disabling buttons alone is insufficient: a stale page, a direct handler call or a handcrafted native packet could otherwise launch work.

Keep navigation separate from mutation. Steps 1–3 expose the saved workspace/Goal, roles and plan. Logs, diffs, tests, results, session/history views, briefing, quota refresh and copy remain available. Questions use chat/history/reset, not the coordinator review-request endpoint. New Goal and confirmed deletion remain separate permitted paths. Global server/account configuration does not rewrite an archived run.

The UI policy runs only from existing render/update events. Weak maps retain control baselines without retaining detached task elements. Text inputs use read-only mode to preserve selection and copying. Native guards run before bookkeeping or child launch; workflow checks retain defense against state changes between request acceptance and execution.

Lesson: a direct variable change in a UI test does not prove tabs work. Click the actual tabs in the current browser, verify the selected panel, and test the same path after returning from the Room. A previously open mock can still hold an old script; versioned mock assets and an explicit reload are part of visual verification.

Revalidate when adding any mutating request/control, changing Goal/Phase completion semantics, modifying chat behavior, or changing document assembly. Physical Windows acceptance remains separate from local DOM/native tests.
