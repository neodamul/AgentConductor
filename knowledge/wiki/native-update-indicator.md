# Native update indicator

The title-bar update control is rendered in `src/native_ui_probe/appearance.rs`. Keep the visible
glyph aligned with R-126 and GitHub #127: use the downward arrow `⬇`, not the upward `↑`. The
localized accessible name still includes the available version, and the Settings menu keeps its
separate **Check for updates** action.

`tests/native_ui_updates.rs::available_update_has_a_localized_titlebar_accessible_name` renders the
control, checks its accessible name, and inspects the actual painted text. This catches glyph drift
that an accessibility-only assertion misses. On 2026-10-22 the test failed with `↑`; changing only
the rendered glyph made it pass. This is component parity evidence and does not add a packaged
end-to-end workflow to issue #9's 8/25 acceptance count.
