# AgentConductor design guide

This guide keeps later UI work consistent with the chosen look (R-128, DD-137). Read it before
changing the Rust views under `src/native_ui_probe/` or shared UI behavior under `src/ui/`.
Native appearance and interaction tests enforce the rules marked **(tested)**.

## Principles

1. **Information first, decoration never.** Themes change color only. Every theme shows the same
   elements in the same places. A redesign must not drop, truncate or hide information.
2. **Native and light-weight.** The app replaces three heavy desktop apps
   ([minimal resources](../knowledge/wiki/resource-budget.md)): no web fonts, background
   animation or new dependencies for styling. Use the platform's system fonts.
3. **One token, one meaning.** Colors are named by role, never by hue. Status colors mean status
   only and are never decorative.
4. **Nothing leaves its container.** Long, translated or user-provided text wraps or ellipsizes
   inside its column. Korean text is often longer than English; design for it.

## Themes

| Theme | Design | Native preference | Default |
|---|---|---|---|
| Light | E "native settings": system-settings neutrals, blue accent | `light` | yes |
| Dark | F "night": the sage brand on dark surfaces | `dark` | no |

The operator chooses the theme in ⚙ Settings → Theme. It is stored as `theme` in
`~/.agentconductor/preferences.json` and applied by the Rust eframe appearance layer.

The native appearance test checks panel, window, selection, and active-control colors against
the palette (**tested**).

## Color tokens

Shared surface colors live in `src/native_ui_probe/appearance.rs`; status foreground/background
pairs live in `src/native_ui_probe/render.rs`. Keep light and dark values paired. **(tested)**

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

Primary controls use the accent foreground/background pair. Active, disabled, panel, and rail text
contrast is checked at 4.5:1 (**tested**).

Pairs always travel together: text on `--accent` uses `--accent-ink`; a status tag uses the
status color on its matching background. Errors and notices use semantic colors. The native tests
check the actual theme colors for at least 4.5:1 text contrast **(tested)**.

## Rules for native styling

- **Use shared palette values for surfaces and controls.** Add a paired light/dark value when a
  semantic role needs a new color; explain why an existing value cannot serve.
- Check body text contrast against WCAG AA (4.5:1) in both themes.
- Never use color alone to communicate task state; keep the localized text and accessible label.
- Keep shared appearance behavior in the appearance layer and workflow-specific layout in its
  existing Rust view modules.
- Workspace navigation uses three equal-width numbered steps with a visible active underline.
  Keep the Workspace and Agent Roles save actions at the lower right of their content cards; the
  primary action uses the theme's accent/ink pair. **(tested)**
- The Workspace page eyebrow identifies the selected step in English and Korean. Repeat that
  marker inside the active section card to match the established Workspace flow; keep its title and
  description beneath the marker.

## Typography

- Prefer platform system fonts so Korean glyphs use the installed OS fonts.
- Use monospace only for executables, paths, hashes and raw output.
- Align quotas, counts and durations where the native widget supports tabular figures.

## Layout rules that protect information

- **Agent Room session rows** keep the task, agent icon/name, and localized status visible together.
  Long names and translated statuses must wrap or remain available through an accessible tooltip.
  **(tested)**
- Agent Room shows the app brand/version and current workspace/phase context; its content does not
  repeat the native window title.
- Any user, agent, or translated text (names, goals, statuses, paths) must fit its native layout;
  preserve full text when truncation is necessary.
- Do not shrink an element by removing information. Prefer wrapping to truncation for statuses and
  actions; truncate only long identifiers (paths, hashes) that stay available elsewhere.

## Checking a change

1. `cargo test --offline --locked --features native-ui-probe --test interface_theme --test interface_language --test native_ui_appearance --test native_ui_bilingual --test native_ui_room_language --test native_room_layout --test native_room_task_selection --test native_ui_settings_journey`.
2. Open Workspace and Agent Room in both themes and in Korean. Capture the native windows; do not
   judge from screenshots of a single theme.
3. Check contrast, text wrapping, keyboard and IME input, accessible labels, and controls at the
   edges of each window.

Every custom dialog exposes a localized, keyboard-accessible close or cancel action inside its
content. A title-bar close control alone is insufficient for keyboard and screen-reader parity.
