# Inline agent quota

The retained-run-only workspace behavior below is historical and was corrected by [Workspace quota collection](workspace-quota-refresh.md).

R-61 / BG-22 / SF-51 / UC-50 / FR-46 / NFR-21 / QAS-52 / DD-67.

Both left lists place a compact, nonshrinking quota badge on the same row as the paid
agent name. The lowest remaining reported window is shown, with all reported windows,
source and observation time in the tooltip. Unknown shows an em dash; stale shows Stale.
Names truncate before the badge; detailed account capacity remains in the inspector.

The workspace snapshot reads only its most recently retained run, inside its own root,
with a read-only SQLite transaction and current profile digests. Rendering performs no
CLI queries and never grants execution authority. An unqueried workspace has no invented
percentage. Account data is not money or context-window capacity.

TDD: the missing badge function failed before implementation. The display regression
executes both actual JavaScript helpers in Node and checks multiple windows, stale and
missing data, source tooltips and both name rows. A Rust workspace regression verifies
retained 73% evidence and profile-change invalidation. Focused Rust checks: 18 passed;
Python/Node display regression: 1 passed. Both JavaScript syntax checks, formatting,
Clippy with warnings denied and optimized macOS build passed. CI includes the display
regression; Windows runner execution remains blocked by GitHub billing.

## Rust quota badge rendering — DD-160

The startup snapshot now carries localized `quota_badges`, calculated by
`src/quota/display.rs`. The UI only renders the prepared text and tooltip; it no
longer sorts quota windows, calculates Enterprise remaining percentages, or
formats reset/observation timestamps in JavaScript. Timestamps use the host's
local UTC offset. Rust tests cover Enterprise 64%/100%, compact windows, stale
and missing observations, unlimited credits, reset-unknown, localized labels,
and startup snapshot wiring. The duplicate room formatter was unused and was
removed. Two Python/Node display suites were replaced by Rust integration tests.

## Current source fingerprints

- `src/quota/display.rs` SHA-256 `9bdcb9f7e5ac436844d13f6eed1ab9cc70e1c0ec9f0de54618d3dee7aa1ea2ab`
- `src/quota.rs` SHA-256 `06cf95decc348aca51f86eda609cb04014335df864e83c0ab1e53f80f871afe8`
- `src/ui/startup.js` SHA-256 `de14dde8f50d549f9677f172b0b8a54a7b444831493a788cc19d7f3bf6277173`
- `src/ui/room.js` SHA-256 `221853084e1e12e97c39e47926d2566b31ca2f946b43fd411647eb0f6cc9f66c`
- `src/ui/setup/snapshot.rs` SHA-256 `209cba09a547fddf8f89bdbde0b6b5c70df4c143451f23aa53d151a0123d9ccf`
- `tests/startup_quota.rs` SHA-256 `5bba477107f5e86dc6e3bce783b9e59c4b64cb6ffe795edc1783a62b99f92d79`
- `tests/quota_display.rs` SHA-256 `3432a2a9a4da5b97d31932eadc5e8f2b50e6f9e1a68ee99db525d0e23cfd3379`
