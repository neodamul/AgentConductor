# Isolated Workspace screenshot capture — 2026-11-03

## Context

The documented Rust Workspace image showed an empty first-run screen, while the available main UI reference showed a populated saved workspace. The two images could not support a same-state parity judgment.

## Change and verification

- Added a regression proving that a manual screenshot run suppresses automatic quota scheduling.
- The screenshot-only guard is active only when the `manual-screenshot-capture` feature and output path are both present; normal app quota scheduling remains enabled.
- Captured the current v0.1.57 Rust Workspace from an isolated HOME with the same workspace name, goal, three profile IDs, and 1280×892 viewport. The eframe callback closed the window automatically.
- Local quota scheduler tests pass 5/5; source-layout passes 2/2; formatting, diff checks, and the 300-line source/test limit pass.

## Limits

The temporary fixture had no retained quota observation and the Rust screen still differs from the legacy screen in quota/status content. The capture is not a documentation asset, not packaged evidence, and does not raise issue #9 UI acceptance (still 0/25).

## Knowledge tools

Graphify code-only extraction completed with two workers: 9,052 nodes and 17,032 edges, without semantic LLM extraction or clustering. Graphify traced `refresh_quotas` to the new capture-aware `due_quota` call. CocoIndex CLI is unavailable in this host.

## Broader local verification boundary

Warning-denied Clippy passes on all targets with `manual-screenshot-capture`. The full `all-targets` test run was intentionally interrupted after 897 tests passed and 14 were ignored across 268 suites; it is not recorded as a complete suite pass. Focused quota, source-layout, public-doc, and knowledge tests passed.
