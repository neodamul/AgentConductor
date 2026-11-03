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
