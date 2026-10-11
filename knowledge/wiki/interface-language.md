# Interface language and UTF-8

Trace: R-70 → BG-27 → SF-58 → UC-58 → FR-54/NFR-27 → QAS-63 → DD-76.

## Design
- Preference: `~/.agentconductor/preferences.json` `{"language":"en"|"ko"}` (`src/ui/language.rs`); `embed()` rewrites `<html lang="en">` in both page builders; `set_language` is a room-level navigation request.
- Translation: `src/ui/i18n.js` swaps exact English strings (text nodes, `placeholder`, `title`, `aria-label`) using `src/ui/i18n_ko.js`; keys ending with a space are prefixes (`"Local workspace · " + path`). The inverted table switches back without reload. A `MutationObserver` runs only for non-English.
- Never translated: textarea text (placeholder is), `PRE`, `CODE`, input values, user/agent content.

## Adding strings or a language
- New interface text in English source automatically stays English until the table gets an entry; `tests/interface_language.rs` rejects stale keys that no longer exist in the sources.
- A new language = one more `i18n_<lang>.js` table, `tables` entry, `SUPPORTED` entry and `<option>`.

## UTF-8 lessons (Windows CI, 2026-10-11)
- Python on Windows defaults to cp1252: always pass `encoding="utf-8"` (guarded by `tests/test_utf8_sources.py`); CI also sets `PYTHONUTF8=1`.
- `python -I` ignores `PYTHON*` variables — a test that sets `PYTHONIOENCODING` must not use `-I`.
- Time-based tests must view observations at the current time; a slow runner crosses a second boundary (`claude_quota` race).
