# Interface themes (R-128 / DD-137)

- Two themes from one stylesheet: light tokens on `:root`, dark tokens on `:root[data-theme="dark"]` at the top of `src/ui/room.css`. The rules live in [design guide](../../docs/design-guide.md).
- Preference `theme` beside `language` in `preferences.json`; `language::embed` writes `data-theme`; `set_theme` saves; the page swaps the attribute without reload.
- Tokenization was scripted by role (property + luminance + hue) and then corrected by hand where context mattered (titlebar divider, run-phase chip, app version, selected session icon, ok vs accent backgrounds).

## Lessons
- A script must compute its output before opening the target for writing; one crash truncated `room.css` (restored from git).
- Hex checks must look at declaration values only (`#id` selectors are not colors) and include 4/8-digit alpha hex; named colors (`white`) slip past hex scans.
- Verify both themes in the native window: the dark capture exposed a `background:white` card the tests had not covered yet.
- The Agent Room status tag overflowed because it shared the name line with `nowrap`; giving it its own wrapping line keeps all information.

## Readability and focused controls (2026-10-14)

The shared disabled-button opacity reduced primary text contrast to 1.91:1; disabled primary buttons now use the existing soft/fill pair at full opacity. Active primary buttons retain accent/accent-ink. Notices use bar/bar-ink; error notices use err/err-bg. Contrast regression measures actual CSS tokens and opacity on both panel surfaces, requiring 4.5:1 in both themes.

R-130/DD-139 keeps percentages and refresh compact, with all reset details in accessible tooltips. Step introductions sit below the title; footer explanations use remaining width instead of a 230px cap. R-131/DD-140 spaces Settings labels and controls, hides the retained server port for Local Rust and opens the static tool guide from the workspace menu. No additional polling or dependencies.

[Verification evidence](../../docs/evidence/2026-10-14-ui-readability.json) records genuine latest-source native captures, three overlapping fixture workers and approval enforcement. Published v0.1.17 predates these source changes; screenshot captions distinguish that release from the source build. Live Windows UI remains unverified.
