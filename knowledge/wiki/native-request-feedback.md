# Native request feedback

## Current contract

Every foreground action gives immediate visual feedback. If it is still waiting after one second, Workspace and Agent Room name the operation and show elapsed time. Metadata reads, snapshots, and routine refreshes remain quiet.

The Rust implementation records the foreground `Pending` kind and a monotonic `Instant`. A changed request resets the timer. The native view schedules a local repaint while the request is pending; it does not dispatch another snapshot or add a polling worker. Existing request workers and event delivery continue to own asynchronous work and state updates.

## Upstream behavior and Rust parity

- Main #279 added time-aware feedback for long planning, chat, goal rewrite, MCP lookup, and update actions. Rust port #283 is already in this branch at `18224cda` through PR #284. Its Rust Workspace renders job elapsed time and stale-snapshot age.
- Claude's open PR #287 implements issue #285 in the JavaScript Workspace and Agent Room. It marks the pressed control immediately, names a direct request after one second, and suppresses routine refresh noise.
- The Rust #285 port adds immediate pending feedback, foreground operation labels, elapsed status after one second, and Agent Room rendering. It does not alter the JS UI or backend dispatch lanes and does not modify Claude's worktree.
- Rust backend work on the separate #255 branch is not merged into this #260-based worktree. The request-feedback implementation does not depend on #255's worker changes.

## Verification

Local checks on the Rust #285 working branch:

- `cargo test --locked --offline --features native-ui-probe --lib --test native_ui_probe --test native_room_poll --test source_layout`: 102 passed across four suites.
- Warning-denied targeted Clippy passed for the library and those three integration targets.
- `cargo fmt --all -- --check`, `git diff --check`, and the changed-file 300-line check passed.

These source tests do not establish exact packaged GUI acceptance. Issue #9 remains open and its acceptance count is unchanged. Full all-target Clippy, all-target tests, packaged macOS/Windows interaction, and Windows runtime acceptance are not claimed here.

## Tooling

Graphify was refreshed after this change without clustering: 11,885 nodes and 19,393 edges. CocoIndex is not installed in the current environment (`python3 -m cocoindex` reports `No module named cocoindex`); no Python runtime or application dependency was added.
