# Exact Workspace controls

Trace: R-210 / DD-222, GitHub #435.

The approved mock is immutable. Compare the whole control box: width/height, padding,
line-height, flex/gap, radius, border, fill and the real label. A fixed height can hide
incorrect padding; copy the mock's natural box model instead of adding a tolerance.

Normalize variable names only in test inputs: a Workspace called `sim` cannot have the
same natural width as `Auth Service`. The product still shows its actual name. Inspect
child boxes too: the missing 4px left margin on the Goal-count badge caused the remaining
button-width difference after its own padding already matched. The MCP action container
has stronger CSS rules than a plain ID selector plus an element; inspect actual computed
styles rather than assuming a later rule wins. A 3px rail gap splits its width exactly.

Verification: light and dark, 91 local parity entries each, zero differences. Ten local
icon/DOM/Node/architecture tests passed. This unit does not claim Windows package acceptance
or resolve the separate #431 Windows choice-field crop difference.

## Windows-only failures

Equal computed parent fonts do not prove equal child line boxes. A direct flex text item
and a translated child span can lay out differently with fractional font sizes and platform
fallbacks. Keep the mock's text structure, then inspect child bounds and actual platform
fonts (`CSS.getPlatformFontsForNode`) on the failing platform. The failure collector is
read-only and writes JSON alongside comparison crops.

The Windows avatar crop difference is still unconfirmed locally. A fractional-origin
perturbation did not reproduce it on macOS, even with zero channel tolerance. Do not turn
that candidate into a diagnosis or loosen the comparison; retain failing platform crops.

The confirmed import-row defect was the cascade, not a different translated child font:
`#start-mcp-controls .small-button` fixed height at 26px and outranked the later
`#start-mcp-controls button` natural height. Removing the stale MCP selector allows
the parent stretch behavior to work. A real-browser regression varies the tallest
label's line-height to reproduce the Windows fallback condition on macOS; it asserts
natural content height and equal sibling heights without changing the approved mock.
