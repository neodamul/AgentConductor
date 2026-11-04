# Compact agent actions

Status: the grid implementation below is historical. The newer approved rail in #436
supersedes it; integration must preserve that natural-width layout. #424 remains open
for the shared mock/app 167px narrow-strip wrap. Current tests compare exact button
boxes to the approved mock and retain icon-fit/accessibility checks.

Source: GitHub #424; R-210 / UC-202 / DD-222.

The assembled agent strip had 7px horizontal text padding, including the reorder controls. In the reported 187px strip the fifth action wrapped. Fixed button sizes alone do not reserve usable icon space.

Keep 22px border-box height, 13px non-shrinking SVGs, 4px horizontal padding for labeled actions and zero for remove/reorder actions. The 23px remove and 22px reorder targets remain centered. Scope these rules to `.startup-profile .profile-tools` so titlebar and dialog buttons retain their own geometry.

`tests/agent_button_geometry.cjs` measures the actual assembled controls in both themes at 840/1280, checks the reported strip width, names, pointer cursors and SVG bounds. It uses temporary browser profiles, never real agent processes. Exact reported-page verification is a temporary style injection and does not modify Claude's running server.

Windows verification found a remaining font-width dependency after the first padding fix. The final strip uses two minmax(0,1fr) label columns plus fixed 23/22/22px icon targets; labels are separate ellipsis-capable spans. A 2px letter-spacing stress case reproduces wider glyph metrics locally without relaxing the single-row or icon-bound checks.
