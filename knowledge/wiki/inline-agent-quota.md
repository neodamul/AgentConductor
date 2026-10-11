# Inline agent quota

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

## Source fingerprints

- `src/ui/room.js` SHA-256 `c99d4fe515197d3d567d71ae5bdd9145b7e915fdde2c8a2566f6d6386aa2e26a`
- `src/ui/room.css` SHA-256 `3af1dbbb3af39047b7ca35c31f52c68cbdf3eecf61ee703bac8175a9ef081a6a`
- `src/ui/startup.js` SHA-256 `75ff9f19b4f2fe5d3df153e1d2f73475bd6f46f09d8f7a00947ae510b394fdb9`
- `src/ui/setup/quota.rs` SHA-256 `5a1de355f1e0568f7252ad88821a8cbf2d5ee1ff53751d1fb3f6d4bf0c4616a9`
- `tests/startup_quota.rs` SHA-256 `118b408dce6b9dfe6cfe91e14696d15a3084edfa4aeca27be23322ddfea47c1a`
- `tests/test_quota_badge.py` SHA-256 `9e2260da13c145ff3c084753729bc9a83259df78dd1c1de996b8f033d30e29c5`
