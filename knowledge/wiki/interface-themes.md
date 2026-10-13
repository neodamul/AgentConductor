# Interface themes (R-128 / DD-137)

- Two themes from one stylesheet: light tokens on `:root`, dark tokens on `:root[data-theme="dark"]` at the top of `src/ui/room.css`. The rules live in [design guide](../../docs/design-guide.md).
- Preference `theme` beside `language` in `preferences.json`; `language::embed` writes `data-theme`; `set_theme` saves; the page swaps the attribute without reload.
- Tokenization was scripted by role (property + luminance + hue) and then corrected by hand where context mattered (titlebar divider, run-phase chip, app version, selected session icon, ok vs accent backgrounds).

## Lessons
- A script must compute its output before opening the target for writing; one crash truncated `room.css` (restored from git).
- Hex checks must look at declaration values only (`#id` selectors are not colors) and include 4/8-digit alpha hex; named colors (`white`) slip past hex scans.
- Verify both themes in the native window: the dark capture exposed a `background:white` card the tests had not covered yet.
- The Agent Room status tag overflowed because it shared the name line with `nowrap`; giving it its own wrapping line keeps all information.
