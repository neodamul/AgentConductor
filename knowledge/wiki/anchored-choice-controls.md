# Shared anchored choice controls

Source: GitHub #409; R-210 / DD-222. The native select remains the form state contract;
only its visible control is replaced. One popup serves dynamic Workspace/role/inspector/
Settings fields and is reparented inside the active native dialog to preserve top-layer focus.
MutationObserver tracks actual option/disabled/dialog changes; no polling timer is added.

Escape cancels the list without closing its modal. Selection commits through the existing
change event. Outside dismissal restores field focus. A manual model callback must complete
registration before the selected value changes; invalid IDs and registration errors do not
commit. Backend catalog validation remains authoritative.

Checks use the actual assembled simulator in two themes and widths 840/1280. Geometry/style
and crop comparison of isolated controls uses the same fixed width and pixel-aligned fixture
origin on both sides, so fractional parent placement does not masquerade as a font difference.
Shell geometry is checked separately under #408. Do not broaden pixel tolerances to hide a
real control difference. First unit: 80 behavior checks, eight control crops at 0.00% difference.
Role-specific fallback and coordinator/role manual catalog wiring are implemented in the next verified unit; final per-surface and coordinator-seeding acceptance remain open in #430 and #429.

Task-scoped fallback uses missing/null = inherit, false = disable, object = explicit first
hop. Always clear a consumed hop in the candidate plan before validating a new agent, not
only after transfer: otherwise the new target appears to fall back to itself. Retain the
policy for same-agent model edits. Runtime tests, not just serialization checks, caught this.
Pending-task changes and profile rename/removal must follow the same policy contract.

The I18N catalog is lexical (`const AC_I18N`), not `window.AC_I18N`. A popup fallback
through the window property silently prints message IDs; assert actual translated labels.
Observe option/disabled changes to update an open list, but do not dismiss it merely because
its accessible name was rebound. Preserve a missing saved primary for reassignment instead
of silently serializing an empty role after an unrelated edit.

A manual registration callback must capture its originating agent before awaiting the catalog request. A coordinator switch can happen while the bridge is pending; reject the late selection rather than applying the old model to the new coordinator.

CI cleanup lesson (#431): a waiting_approval report is a workflow result, not proof that its independent availability watcher exited. The CLI fixture now waits for a terminal ownership record AND OS-confirmed PID/process-group absence before deleting evidence. The scheduling and model assertions already passed; do not change product fallback policy to hide a cleanup race.
