# Visual specification: light and dark

Authority: [approved design](README.md). Theme colors follow the existing
[design guide](../../design-guide.md); this specification changes the unified screen geometry
and explicitly allows the reviewed running-avatar pulse.

## Geometry at the reference size

All dimensions are logical CSS pixels in the documentation reference. Native implementations
must reproduce the visual proportions with their own scale-factor handling, not embed the HTML.

| Element | Size / treatment |
|---|---|
| App frame | 1180 × 800 inside the capture viewport; thin neutral outline, 10px outer radius |
| Titlebar | 52px high |
| Title brand column | Same width as the left rail; divider aligns with rail boundary |
| Left rail | 250px; 230px below 900px preview viewport, 190px below 600px |
| Right inspector | Up to 330px / 35% of shell; hidden by default; 1px left separator |
| Side-rail padding | 22px vertical / 18px horizontal; both sides identical |
| Header Goal title | Compact single line, 17px semibold, ellipsis and full-text tooltip |
| Header information row | Fixed 36px, never collapses when empty |
| Six step tabs | 36px high; three background-grouped desktop tab sets with integrated labels; no upper caption row |
| Rail/activity tabs | 29px high |
| Selected tab | Rounded upper corners, thin side outline, accent upper edge, open lower edge joining content |
| Step card | Neutral surface, thin outline; fixed header/footer; flexible scrolling body |
| Workspace / Goal section | 24px top padding; visible section division |
| Agent card | 144px fixed reference height; 8px radius; compact facts/actions |
| Agent avatar | 29px; 1.5px semantic status border |
| Task avatar | Smaller compact counterpart in task list; same semantic state treatment |
| Quota hit box / graphic | 30px / 27px, two concentric rings |
| Titlebar hit boxes / SVGs | 32px / 18px; same vertical center and baseline across the cluster |
| Statusbar / its glyphs | 30px bar; 14px glyphs centered in 24px internal rows |
| Goal list row | 12px padding, 10px column gaps; compact, no deletion footer |
| Goal trash hit box / glyph | 26px / 18px; red, far-left column; folder glyph also 18px |
| Preview pager | 440px max, 40px high, fixed 32px arrow slots; normal flow below image |

Prepare/run/finish are grouped by one subdued background per set, not framed boxes or separate
label badges. Light backgrounds: `#dce5ef` / `#dce9df` / `#e9e1d2`; dark backgrounds:
`#22333c` / `#25342b` / `#363024`. Each background contains a 14px group icon (clipboard, connected task nodes, checked document) and steps
(1–3, 4–5, and 6) on a single 36px row. Keep the selected tab clear on the card surface.
Do not restore detached captions, group outlines, or a second group-label row.

The two side panels share background, spacing, separators, heading proportions, and control radii. The right "Task details" heading is a plain panel
title, not a tab: no accent edge, selected background, or tab-shaped border.
The inspector must look like a docked rail, not a floating modal or a different settings application.
No X/Close is present in the inspector. The titlebar toggle remains reachable while it is open.

## Typography

Use system sans: `-apple-system, BlinkMacSystemFont, Apple SD Gothic Neo, Segoe UI, sans-serif`.
Use system monospace for paths, hashes, task identifiers, and raw logs. No web-font dependency.
Use tabular numerals for percentages, counts, timings, and other aligned metrics.

| Text | Reference treatment |
|---|---|
| App name / version | 14px semibold / 10px secondary; version right-aligned before divider |
| Main Goal | 17px, weight 650 |
| Step title | 18px semibold |
| Rail title | 18px semibold |
| Selected left-rail tab | 12px, weight 650 |
| Right inspector title | Plain 18px heading, weight 650; no selected-tab styling |
| Section heading | 13–14px, weight 650 |
| Main body / compact metadata | 12–13px / 11–12px |

Tab/header/list text that cannot fit stays on one line with ellipsis. Hover exposes the complete text;
accessible text must remain complete. Do not use clipped screenshots as evidence of preserved information.
Instructions, errors, confirmation warnings, logs, and evidence wrap or scroll instead of losing content.

## Color tokens

The same component uses the same semantic token in each theme. Theme changes must not move elements,
remove controls, change interaction, or change the meaning of status colors.

| Token | Light | Dark |
|---|---|---|
| Page / rail `--bg` | `#f2f2f6` | `#141a19` |
| Card / input `--surface` | `#ffffff` | `#1a2220` |
| Secondary `--surface-2` | `#ebebf0` | `#111716` |
| Quiet fill `--fill` | `#e6e6ec` | `#222c29` |
| Separator `--line` | `#dcdce2` | `#2a3431` |
| Strong separator `--line-strong` | `#c4c4cc` | `#3a4844` |
| Primary text `--ink` | `#1c1c1e` | `#dde6e1` |
| Secondary text `--soft` | `#606067` | `#8e9c96` |
| Titlebar `--bar` | `#e9e9ee` | `#0f1413` |
| Title text / divider | `#1c1c1e` / `#cfcfd6` | `#dde6e1` / `#2a3431` |
| Action `--accent` | `#0a66d8` | `#8cc9a8` |
| Action text `--accent-ink` | `#ffffff` | `#0f1a15` |
| Selected tint `--accent-soft` | `#e3edfb` | `#23382e` |
| Accent text | `#0a56b8` | `#a9dcc1` |
| Success text / fill | `#1f7a3a` / `#e3f3e8` | `#8cc9a8` / `#1f3328` |
| Warning text / fill | `#9a5a00` / `#fff1dc` | `#e2c06a` / `#3a2f1c` |
| Error text / fill | `#c4312a` / `#fdecea` | `#e07a6a` / `#3a2220` |
| Short-window quota / long-window quota | `#637d91` / `#9183a3` | `#91a6b6` / `#b0a1bd` |
| Information text / fill | `#2b5fa8` / `#e4ecf8` | `#8fbfdc` / `#1c2a33` |

Critical body/secondary text and primary/warning/error pairs must meet 4.5:1 contrast.
In particular, dark-theme sage buttons use dark text, not white text. Disabled primary controls
use secondary text on the quiet fill at full opacity. Do not use a quota percentage as cash balance.

All choice fields use the shared **anchored list**, never a separate assignment modal.
Field height is 33px, popup radius 8px, menu width 230–310px, and anchor gap 6px; clamp to the app
frame and open above when necessary. Use the same neutral surface, checkmark/tint, 12px text,
hover/focus treatment, and keyboard behavior in Workspace, roles/Fallback, inspector, and Settings.
The primary and fallback Agent/Model pairs are separated by a muted 18px right arrow.
Clickable buttons, icon buttons, links, and choice fields use a pointer cursor. Disabled controls
remain visibly disabled and use the normal cursor. No unrequested extra modal is introduced.

## Icons and surfaces

- Keep the existing app mark, coordinator chat SVG, and settings gear identity.
- Agent-list Claude Code/Codex avatars use pinned local color SVG brand assets (Claude Code orange, Codex blue/violet), not Cl/Cx initials or executable extraction.
  See `brand-icons/provenance.json` and its retained MIT notice. Task-list initials are a separate scope.
- Menu: familiar three horizontal strokes. Left/right panel glyphs mirror which column they control.
- Update: downward arrow/download shape, before chat and settings in the right titlebar cluster.
- Use 1.6px strokes on the 24-unit SVG grid; align the graphics inside identical hit boxes.
- Titlebar buttons have transparent background/border at rest, quiet fill on hover/selection.
- Goal and Agent removal share the same red trash SVG path and stroke. Goal glyph: 18px;
  agent-card glyph: 13px in a 22px control, balanced with 11px surrounding tool text. No emoji.
- Trash is the deliberate red exception; do not color unrelated actions red.
- Keep focus outlines visible. Active, hover, disabled, loading, error, and unsupported are distinct states.

## Status and motion

Green: running/completed. Yellow: waiting/needs attention. Red: failed/refused/blocked.
Only a running avatar border pulses (reference period 1.8s); completion is a steady border.
Keep the icon legible throughout the pulse. Status text is available on hover/focus/activation.
An attention badge signals unresolved decisions without adding a progress sentence to every card.
Honor reduced motion. Theme changes do not add motion or background polling.

Slide-overs use a short approximately 200ms translation inside the app frame; they may use a dim backdrop.
Focus must not scroll the clipped app container horizontally as the panel enters. Restore focus with
scroll prevention. A closing panel cannot leave an invisible blocker, and rapid reopen cancels its close timer.
The docked inspector does not slide and never dims or disables the work area.

## Full-window behavior and quota-window identity

Maximizing or entering fullscreen changes the available content area, not the logical sizes of
icons, 1.5px status borders, typography, hit targets, or rail spacing. Keep the left divider aligned
and the titlebar controls at their fixed sizes. Reflow the center; do not scale the entire UI.
The preview can exercise full available width/height with `?fullscreen=1`.

Outer/first quota window uses muted blue-gray; inner/second uses muted lavender-gray.
Use those same colors for the corresponding information-popup label dot and meter fill.
These are window identities, not success/error status. Always retain actual provider window names
and percentages, so color is never the only way to distinguish windows. Unknown/errors do not
borrow a successful ring value. The palette remains subdued in both themes.

## Theme operation and evidence

The existing Settings → Theme choice remains the production entry point; preserve saved preferences.
The reference supports `?theme=light` / `?theme=dark` and its Settings sample selector.
Reference images use the same viewport, crop, state, and layout for both themes.
See [gallery](gallery.html) for paired screenshots, including quota errors, dialogs, and the docked inspector.

No external fonts, network image hosts, new styling library, or runtime framework is needed.
The image set is documentation evidence, not an additional desktop process or app dependency.

## Evidence summaries

Review/result summaries are compact document metadata on one ruled line, not dashboard KPI tiles.
Use readable 14px labels with 16px semibold values, no separate card fill/radius and no oversized numbers.
The actual changed-file list, test evidence and lessons remain the main content.
