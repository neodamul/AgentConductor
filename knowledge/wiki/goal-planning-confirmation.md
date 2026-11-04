# Goal planning confirmation

Trace: R-210 → BG-165 → SF-198 → UC-202 → FR-204/NFR-09 → QAS-209 → DD-222. GitHub #429.

New Goal inherits Workspace coordinator, per-profile planning model and worker-role defaults;
it clears the previous Goal proposal/run context. These are defaults, not authority to
silently start the planner. Generate Plan and Read Plan show the current Goal (or source
document), coordinator and explicit model/CLI default in the same confirmation dialog.
Cancel and Escape launch nothing. A missing coordinator or model outside its registered
catalog is rejected; it must not appear to run CLI default while retaining an invalid ID.

Confirmation captures the Workspace epoch, execution review key and model catalog IDs.
It rejects a changed revision before acceptance and rechecks the execution context after
save/baseline awaits. Native revision guards and model validation remain authoritative.
The review key includes `coordinator_model`: omitting it lets a planning model edit look
unchanged to UI approval checks. No additional polling, process or persisted confirmation
state is needed. Historical reviewed plan artifacts remain independent.

The existing five backend worker-role keys are unchanged. Planning is deliberately an
informational row; its coordinator/model controls remain in Workspace. This is an
intentional exception to editable mock role rows, not an unimplemented extra role.

Evidence: `tests/planning_confirmation_runtime.cjs` covers evidence, cancellation, Escape,
stale revision/Workspace/model and CLI default. `tests/planning_confirmation_browser.cjs`
checks the actual assembled page in both themes at 840/1280px, including focus, bounds,
pointer cursor, cancellation, changed Goal/baseline, document import and confirmed requests. `tests/goal_planning.rs`
records the real planner CLI argv, inherited defaults, alternate coordinator model,
invalid-model rejection and byte-identical prior Goal plan. The deterministic fixture
calls no provider service. Windows native UI acceptance is separate.
