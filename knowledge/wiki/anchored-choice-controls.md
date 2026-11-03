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
Role-specific fallback and manual catalog wiring are subsequent #409 work, not complete yet.
