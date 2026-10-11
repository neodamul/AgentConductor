# Compact quota windows

R-64 → BG-24 → SF-54 → UC-54 → FR-50/NFR-24 → QAS-57 → DD-70.

The operator requested complete period values and an adjacent refresh icon instead of
one minimum percentage and a detached button. Both agent lists now group window values
with one accessible refresh button. Rows show one compact numeric line such as `64%/100% ↻`;
period labels stay in the tooltip, leaving the agent name readable. Selection and refresh remain separate sibling controls. Values and refresh have a 5px gap. Enabled buttons and their icon/text children use a pointer cursor; disabled controls use the default cursor.

Snapshots retain optional provider plan_type, defaulting to unknown for older persisted
records. Codex supplies planType in rate-limit metadata. Claude accepts only an explicitly
recognized Enterprise plan header; a company subscription is never inferred. Enterprise
primary/five_hour and secondary/seven_day use Current/Total presentation aliases.
Other accounts use reported durations: 300 minutes → 5h, 1440 → Daily, 10080 → Weekly;
other durations and provider labels remain distinct. Model-specific windows are retained.
Percentages represent remaining account capacity, with unknown and stale evidence visible.

TDD Red confirmed the old minimum-only helper and absent plan metadata failed. Regression
cases cover Enterprise, plus, daily/weekly, another duration, reordered Claude windows,
missing percentages and persisted metadata compatibility. The related startup integration
exposed a race: quota.sqlite existed before documents schema publication. A deterministic
empty-database test reproduced it. Readers now check sqlite_master in their read-only
snapshot; unfinished initialization is unavailable, while unrelated SQLite errors surface.

Full Rust regression: 282 passed, 12 ignored (before the final numeric-only JS and async-test refinement). Final focused Rust verification: 14 passed, 1 ignored. Python/Node regression: 1 passed. Formatting, Clippy and optimized builds passed.

A slow-provider fixture holds collection until explicitly released: launch returns, snapshots remain available and a duplicate refresh fails while the provider is waiting. Native Cocoa/WebView2 dispatch requests off the UI thread; the coordinator polls children with try_wait and the temporary metadata process uses bounded Tokio I/O. No provider CLI remains resident between refreshes.
Actual Enterprise account collection is unverified; fixtures establish presentation and
parsing only. See [workspace quota evidence](workspace-quota-refresh.md) for the actual
macOS authenticated collection and cleanup evidence. No screenshot QA is claimed.

## Source fingerprints

- `src/quota.rs` SHA-256 `03e25794fdcaec5e1f4e1dea9dffd625e68e8b26b2ebc8478926d78396b2866e`
- `src/quota/terminal.rs` SHA-256 `9acfe18358b811a62d7b99763eb26e77400f9c0091d1ea1ed01ec50cd36bbe51`
- `src/quota/workspace.rs` SHA-256 `21c954173f259c8df60a3234f776d195a53421477bc6aecbaa0dd01416da5dc1`
- `src/ui/room.js` SHA-256 `d99a7fe600770428223dbefad420553112b8cca53df6c478611adb3ae5f123e9`
- `src/ui/room.css` SHA-256 `d1ba3f036027dbc862e21b62441f3aaa0af95b946718a20102a85515d89b1745`
- `src/ui/startup.js` SHA-256 `5743a3013c6a3bb5d693960f789366899a3ed71332bdbb9b2056ed1349f27248`
- `tests/quota.rs` SHA-256 `78b105ff71642fd6fcccad7de293089f3a1585dcfa618613086703e34633bb7c`
- `tests/claude_usage.rs` SHA-256 `4e17456510f93ba602b7c375dcfdfda8fd9a3ac5a3915bf90058827f115fdc99`
- `tests/startup_quota.rs` SHA-256 `3866da64e0711ed1826bf0a7f139ce715244365d6f397508e83d18b49167376b`
- `tests/test_quota_badge.py` SHA-256 `55cc11611bf2c22a07bf3bbd17070f7b4b8d27b111fb8e771e76cfc5e000e934`
