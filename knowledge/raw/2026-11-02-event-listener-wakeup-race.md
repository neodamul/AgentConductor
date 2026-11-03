# Preserve Report Changes During Workspace Wakeup

Date: 2026-11-02

## Failure evidence

- Exact Windows validation run `41431944692` passed the build and source-layout gates, then failed one Rust library test: `ui::events::tests::startup_changes_wake_workspace_and_run_changes_wake_both_surfaces` timed out waiting for `RunChanged`. The remaining Windows tests passed.
- The failure exposed an ordering race in `RunChangeListener::listen_startup`: publishing `WorkspaceChanged` invokes subscribers synchronously, but the listener captured the report fingerprint after publishing. A subscriber could update the report before that baseline was recorded, swallowing the run event.

## Test-first fix

- Added `tests/native_ui_probe/events_listener_race.rs`. Its wake callback changes `report.json` during `WorkspaceChanged` publication. The new test failed before the fix with `Err(Timeout)` while waiting for `RunChanged`.
- `src/ui/events/listener.rs` now records the report fingerprint before publishing the workspace event. A report update during the callback is then observed and broadcast to both Workspace and Agent Room subscribers.

## Local verification

- Deterministic regression: 1 passed after the ordering fix.
- Release library tests: 111 passed.
- Native UI probe integration tests: 13 passed.
- Source/test layout tests: 2 passed; new file is 59 lines and affected production files remain under 300 lines.
- `cargo fmt --all -- --check`: passed.
- Warning-denied all-target release Clippy: passed.
- Graphify refreshed after the source fix: 12,983 nodes, 19,462 edges, 972 communities.
- CocoIndex was checked but unavailable: the CLI and `cocoindex` Python module are absent; no dependencies were installed.

## Remaining

The exact Windows head run failed on the pre-fix commit. The fix is pushed and exact-head platform validation run `41437912566` passed on macOS at `a27f2d6a`; Windows validation is still running. This does not establish Windows packaged UI acceptance or complete issue #9. The follow-up is recorded in issue #257 and root PR #22.

The fix was pushed in commit `fdf83c64`, and both issue #9 and issue #257 plus PR #22 were updated. AgentMemory record: `mem_mvz6idls_525815ce0e09`.
