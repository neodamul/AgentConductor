# AgentConductor design guide

This guide keeps later UI work consistent with the chosen look (R-128, DD-137). Read it before
changing `src/ui/*.css`, `startup.html`, `room.html` or the rendering code in `startup.js` and
`room.js`. `tests/interface_theme.rs` enforces the rules marked **(tested)**.

## Principles

1. **Information first, decoration never.** Themes change color only. Every theme shows the same
   elements in the same places. A redesign must not drop, truncate or hide information.
2. **Native and light-weight.** The app replaces three heavy desktop apps
   ([minimal resources](../knowledge/wiki/resource-budget.md)): no web fonts, images,
   background animation or new dependencies for styling. Use the system fonts below.
3. **One token, one meaning.** Colors are named by role, never by hue. Status colors mean status
   only and are never decorative.
4. **Nothing leaves its container.** Long, translated or user-provided text wraps or ellipsizes
   inside its column. Korean text is often longer than English; design for it.

## Themes

| Theme | Design | Selector | Default |
|---|---|---|---|
| Light | E "native settings": system-settings neutrals, blue accent, light titlebar | `:root` | yes |
| Dark | F "night": the sage brand on dark surfaces | `:root[data-theme="dark"]` | no |

The operator chooses the theme in ⚙ Settings → Theme. It is stored as `theme` in
`~/.agentconductor/preferences.json`, written onto `<html data-theme="…">` by `language::embed`,
and switched at runtime by setting `document.documentElement.dataset.theme`.

## Color tokens

Both blocks live at the top of `src/ui/room.css`, which both pages load. **(tested)** The dark block
may only redefine tokens that the light block defines.

| Token | Role | Light | Dark |
|---|---|---|---|
| `--bg` | Page background behind panels | `#f2f2f6` | `#141a19` |
| `--surface` | Cards, inputs, dialogs, step panels | `#ffffff` | `#1a2220` |
| `--surface-2` | Rails, sidebars, secondary areas | `#ebebf0` | `#111716` |
| `--fill` | Quiet buttons, neutral chips, hover | `#e6e6ec` | `#222c29` |
| `--line` | Dividers and default borders | `#dcdce2` | `#2a3431` |
| `--line-strong` | Input borders, emphasized dividers | `#c4c4cc` | `#3a4844` |
| `--ink` | Primary text | `#1c1c1e` | `#dde6e1` |
| `--soft` | Secondary text, captions, eyebrows | `#606067` | `#8e9c96` |
| `--bar` / `--bar-ink` / `--bar-line` | Titlebar background, text, separators | `#e9e9ee` / `#1c1c1e` / `#cfcfd6` | `#0f1413` / `#dde6e1` / `#2a3431` |
| `--accent` | Primary buttons, selected tab, focus ring | `#0a66d8` | `#8cc9a8` |
| `--accent-ink` | Text on `--accent` | `#ffffff` | `#0f1a15` |
| `--accent-soft` | Selected rows, active tab tint | `#e3edfb` | `#23382e` |
| `--accent-text` | Accent-colored text on surfaces | `#0a56b8` | `#a9dcc1` |
| `--ok` / `--ok-bg` | Completed, passed | `#1f7a3a` / `#e3f3e8` | `#8cc9a8` / `#1f3328` |
| `--warn` / `--warn-bg` | Waiting, needs a decision, stale | `#9a5a00` / `#fff1dc` | `#e2c06a` / `#3a2f1c` |
| `--err` / `--err-bg` | Failed, blocked, refused | `#c4312a` / `#fdecea` | `#e07a6a` / `#3a2220` |
| `--info` / `--info-bg` | Neutral information, coordinator | `#2b5fa8` / `#e4ecf8` | `#8fbfdc` / `#1c2a33` |

Pairs always travel together: text on `--accent` uses `--accent-ink`; a status tag uses the
status color on its `-bg`. Status tags get their pair through the existing `.tone-*` classes
(`--tone` / `--tone-bg`); prefer those classes to new rules.

## Rules for CSS

- **No color literals outside the two token blocks. (tested)** No `#hex` in a declaration value and
  no `background:white`, `color:white`, `…black`. Shadows may use `rgba(0,0,0,a)`, which reads on
  both themes.
- **A new color needs a token in both blocks**, a row in the table above, and a reason a current
  token cannot serve. Check body text contrast against WCAG AA (4.5:1) in both themes.
- Never pick a color by hue for meaning that already has a token (for example green for "done":
  use `--ok`).
- Keep the stylesheet structure: shared rules in `room.css`, workspace-only rules in
  `startup.css`, MCP dialog rules in `mcp.css`.

## Typography

- System sans only: `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`; the OS supplies
  Korean glyphs (Apple SD Gothic Neo, Malgun Gothic).
- Monospace for executables, paths, hashes and raw output only: `Menlo, Consolas, monospace`.
- Numbers that line up (quota, counts, durations) use `font-variant-numeric: tabular-nums`.

## Layout rules that protect information

- **Agent Room session rows**: line 1 is the task name (13px, semibold). Line 2 is one compact
  line: a 14px agent icon, the agent name (11.5px, ellipsized) and the status tag (10.5px,
  semibold, tinted). The line wraps (`flex-wrap:wrap`), so a long or translated tag moves below
  instead of leaving the rail; the tag keeps `max-width:100%` and `white-space:normal`.
  **(tested)** Keep the icon in every row. Status tags stay small and quiet; the strong colored
  treatment belongs to the run status banner only.
- Any text that can come from the user, an agent or a translation (names, goals, statuses, paths)
  needs `min-width:0` on its flex/grid item and either wrapping or an ellipsis with the full text in
  a tooltip.
- Do not shrink an element by removing information. Prefer wrapping to truncation for statuses and
  actions; truncate only long identifiers (paths, hashes) that stay available elsewhere.

## Checking a change

1. `cargo test --test interface_theme --test interface_language --test room_task_rows`.
2. Open both screens in both themes and in Korean. Capture the windows; do not judge from
   screenshots of a single theme.
3. Look for: white or dark leftovers, unreadable text on tags, controls that disappear on the
   titlebar, tags or buttons leaving their column.
