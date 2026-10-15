# Native Rust UI migration

Status: active migration; the shipped application still uses its existing WebView UI.

## Architecture boundary

Keep Room, Startup, Controller, validation, approval decisions, and persisted state in Rust. A native desktop view calls the existing Room::dispatch boundary through a bounded background request worker; process and filesystem waits stay off the UI frame thread. Preserve separate Workspace and Agent Room windows and the existing user-visible workflows.

The eframe/egui implementation remains behind the opt-in native-ui-probe feature. The production binary and release package do not enable that feature. Do not switch the shipped UI until cross-platform workflow parity, accessibility, keyboard/IME behavior, visual layout, idle resources, window lifecycle, and dependency-license checks pass.

## Verified prototype progress

- ec1339c adds asynchronous repository-folder selection to the Workspace probe. Run 38799761260 passed macOS and Windows validation and exact release-package smoke.
- 8683862 adds a Shared-folder default with optional Git worktree isolation. Run 38804235226 passed macOS and Windows validation and exact release-package smoke.
- d39e173 adds profile registration from the existing Rust templates, collision-free profile IDs (up to the existing 16-profile limit), and coordinator selection. The focused nine-test suite, feature Clippy, and formatting pass locally. Exact-source CI run 38808098099 passed macOS validation; Windows build/layout passed, but its desktop UI Automation test timed out with `NO_HEADING`. A failed-job-only rerun is queued; this remains unresolved.
- The profile and coordinator save uses the existing Rust dispatcher, which retains profile validation and carries private environment values from stored profiles.

The prototype currently covers only the Workspace identity, repository path, goal, isolation mode, and basic Team registration/coordinator selection. It does not replace the shipped Workspace setup, planning review, Agent Room, settings, MCP, approvals, logs, diffs, or workflow controls.

## Structural map

Graphify was run on an isolated copy of the native probe source and tests. The scoped graph contained 44 nodes and 101 edges; it showed Probe::show_team calling the template ID helper, with the integration test separately proving persisted profiles and coordinator through the Rust workspace dispatcher. This graph is limited to the copied probe files and is not evidence of full production UI coverage.

## Remaining acceptance

- Port every shipped screen and interaction to the native Rust application while preserving existing workflow behavior.
- Verify Korean IME composition, keyboard navigation, screen-reader accessibility, responsive layout, and separate-window lifecycle on macOS and Windows.
- Measure idle CPU and memory for the packaged native application against the existing resource budget.
- Remove project-owned JavaScript and Python runtime/build/verification paths only after equivalent Rust behavior and checks pass.
- Audit shipped dependencies and license notices, then build and smoke-test exact macOS and Windows release archives.
