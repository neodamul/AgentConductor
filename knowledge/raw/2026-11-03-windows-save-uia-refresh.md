# Windows desktop Save UI Automation refresh — 2026-11-03

## Plan

The exact-head Windows desktop test typed a new goal, read the expected value through UI Automation, invoked the Save button, but did not observe the value in the isolated `startup.json`. The driver had captured the Save button AutomationElement before editing. Reacquiring the button after the text edit is a test-driver hypothesis; the exact Windows package result is required before calling it the cause.

## Task

Added a source regression that required the post-edit lookup. The regression failed first. The PowerShell driver now refreshes the current window's UI Automation tree, locates the visible enabled Save button, then uses the existing InvokePattern and pointer fallback.

## Verify

On source head `85f0f82e099f7b7c9b410e8986121fcd97a933d4`, local `cargo test --test windows_desktop_driver` passed 16 tests; `cargo fmt --check` and `git diff --check` passed. The commit was pushed to PR #22.

Exact-head Platform Validation run `41520505534` is still in progress: macOS validation passed, including all-target/native UI tests, formatting, and Clippy. Windows build and source layout passed; all-target tests are running. No Windows package or acceptance credit is claimed yet. Issue #9 stays at 22/50 (22/25 packaged workflows; 0/25 shipped/default Rust UI).


## Knowledge tooling

Graphify refreshed the code graph to 13,089 nodes, 19,604 edges, and 963 communities. CocoIndex was checked but its CLI is not installed in this host; no CocoIndex result is claimed.
