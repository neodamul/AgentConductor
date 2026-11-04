# Anchored choice implementation contract

Trace: GitHub #409, amends R-210 / UC-202 / FR-204 / DD-222.

Preserve native select values and change events as the existing form/model contract.
A shared event-driven anchored surface replaces their visible chrome for Workspace,
roles, inspector and Settings. Dynamically created fields and active native dialogs use
the same surface. It belongs to the active dialog's top layer, or the document otherwise.
No per-field polling timer, desktop window, web server or new dependency is added.

Anchor the list 6px below the field or above if needed; constrain it to the app viewport.
Use the approved 33px field, 6px radius, 12px text and 230–310px popup sizing. Current
selection has tint/checkmark. Selection commits immediately; Escape/outside dismisses
without change and restores the field focus. Keyboard arrows, Enter, Space and Tab
retain native semantics. Escape while a list is open must not close its enclosing modal.
Disable a field through its underlying select. Observe dynamic option/disabled changes
and reparent the single shared surface safely when a dialog closes.

Role rows retain Agent + Model → Fallback + Model. Rebuild model choices and reset the
model after an agent changes; clear incompatible fallback assignments. Distinguish an
explicit role fallback override from inherited profile defaults in reviewed execution;
reject self fallback, unavailable targets and cycles. Unsupported catalog entry uses CLI
default or explicitly confirmed manual ID with the existing operator catalog contract.
Do not silently pass unregistered/incompatible model IDs to a worker.

Verification: native backend unit/integration tests, assembled UI simulator interactions,
and approved mock parity in both themes at widths 840 and 1280. Browser checks prove
webview behavior only, not OS-native package/IME/screen-reader acceptance. Main merge
and releases need their own approval; these checks never tag or publish.

## Reviewed fallback contract

Role entries and reviewed Tasks accept an optional `fallback`: omitted/null inherits the
profile default, `false` explicitly disables it, and `{agent, model?}` overrides the first
hop for that task. `true`, self/same-account targets, unknown agents and incompatible models
are invalid. Operator role rules apply the Task field without changing shared profiles or
other roles. The fallback chain still checks cached account restrictions and stops at an
already-tried agent; mutually configured profiles remain compatible rather than looping.

After an actual agent transfer, the first-hop override is consumed before candidate-plan
validation and the target uses its own reviewed profile chain. A same-agent model edit keeps
its policy. Pending-task agent changes clear the old first hop too. Agent renames update the
nested role reference; removing its target turns that role policy into explicit `false`,
which cannot accidentally reactivate an inherited default. Plan review metadata names the
effective fallback, and editing unrelated Task fields preserves the policy.

## Role and manual catalog UI unit

`role_choices.js` renders the five existing worker roles without adding a new planning-role
key to the Rust schema. Each row stores its own primary/model and inherited/disabled/explicit
fallback/model. Agent changes reset models even when both catalogs contain the same ID;
self fallback is removed, and no fallback disables its model. Changing an inherited model
creates an explicit role override. Saved missing primaries stay visible for reassignment.
The internal inherit token cannot collide with a registered agent ID.

Coordinator and role manual model entry registers through the existing asynchronous
`catalog` request before applying the choice. Custom CLIs need a declared model argument;
registration failure or a changed source agent must not select a new value. Catalog refresh
updates options without a polling timer or an extra query. The lexical I18N binding is used
for popup labels, and accessibility-label updates alone must not dismiss an open list.

Design-owner decision (2026-11-04): the planning row is intentionally informational;
Workspace owns its coordinator/model, and the existing five worker-role keys are retained.
This is an explicit exception from the mock's editable planning/verification row, not an
unimplemented sixth backend role. Goal-specific confirmation/seeding is tracked in #429;
remaining inspector/per-surface manual-model acceptance is #430. Both stay in milestone
0.2.0 after #409 merges. Isolated atom parity does not certify the parent shell or native
Windows package.
