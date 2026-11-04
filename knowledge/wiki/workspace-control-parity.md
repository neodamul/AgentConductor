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
