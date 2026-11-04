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

## Browser action settlement — #466

A logged `prepare` or `read_plan` request is not a settled action: its `finally`
path still polls state before re-enabling controls. A fixed 500ms delay made the
acceptance test click disabled Import Plan, producing no `pick_file` request and
a missing confirmation modal. Delaying the post-prepare snapshot by 1200ms
reproduces this deterministically; an idle-only 5200ms poll does not.

Keep that delayed snapshot in `tests/planning_confirmation_browser.cjs`. Wait
for request evidence **and enabled next controls** before the following action.
For the changed-baseline negative assertion, also wait for the actual rejection
message and settled controls; an unchanged request count while the action is
still pending is not evidence of a correct refusal. Production confirmation and
revision guards need no weakening for this reproduced failure.
