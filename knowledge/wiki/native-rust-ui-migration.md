# Native Rust UI migration

Status: active migration; the shipped application still uses its existing WebView UI.

## Architecture boundary

Keep Room, Startup, Controller, validation, approval decisions, and persisted state in Rust. A native desktop view calls the existing Room::dispatch boundary through a bounded background request worker; process and filesystem waits stay off the UI frame thread. Preserve separate Workspace and Agent Room windows and the existing user-visible workflows.

The eframe/egui implementation remains behind the opt-in native-ui-probe feature. The production binary and release package do not enable that feature. Do not switch the shipped UI until cross-platform workflow parity, accessibility, keyboard/IME behavior, visual layout, idle resources, window lifecycle, and dependency-license checks pass.

## Verified prototype progress

- ec1339c adds asynchronous repository-folder selection to the Workspace probe. Run 38799761260 passed macOS and Windows validation and exact release-package smoke.
- 8683862 adds a Shared-folder default with optional Git worktree isolation. Run 38804235226 passed macOS and Windows validation and exact release-package smoke.
- d39e173 adds profile registration from the existing Rust templates, collision-free profile IDs (up to the existing 16-profile limit), and coordinator selection. The focused nine-test suite, feature Clippy, and formatting pass locally. Exact-source CI run 38808098099 passed after a failed-job-only retry: macOS and Windows validation, native UI probe, formatting/Clippy, bilingual manual rendering, and both exact release-package smoke jobs passed. The initial Windows UI Automation `NO_HEADING` was transient and did not reproduce.
- A follow-up resource slice removes a full workspace snapshot clone from every egui frame by borrowing the snapshot alongside the editable Team fields. The existing nine tests, feature Clippy, and formatting pass locally. Exact-source run 38812385807 passed macOS and Windows validation, optional probe, formatting/Clippy, bilingual manual rendering, and exact release-package smoke.
- The profile and coordinator save uses the existing Rust dispatcher, which retains profile validation and carries private environment values from stored profiles.

### Verification migration: localization contracts

The next Rust verification slice preserves duplicate-ID rejection, English/Korean catalog-key and placeholder parity, explicit source references, localized static markup, and removal of the legacy text-matching helper. Keep the live language-switch and user-content preservation regression until the production WebView UI is migrated; moving its Python wrapper alone does not prove the JavaScript UI has been removed.

The slice is complete in `tests/i18n_contracts.rs`: its ownership test was first observed failing while the Python module existed, and then all 5 Rust contract tests passed after removal. The existing `tests/messages.rs` runtime test remains and passes. Full Rust verification excluding the documented slow source-layout test passed 515 tests (14 ignored, 1 filtered); the source-layout test passed separately, as did all 38 remaining Python tests, formatting, all-target Clippy and the 500-line bound (new Rust test: 307 lines). A scoped Graphify extraction of the two Rust test files and message module produced 42 nodes, 65 edges and 8 communities with no import cycles; it showed the catalog parser, source-reference scanner and markup checker as isolated test helpers. CocoIndex was not available as a CLI or enabled connector in this environment, so this slice did not use it or add a dependency.

Exact-source CI run 38817585058 passed macOS and exposed a Windows timeout in the independent `mcp_remote` SSE fixture: its 3-second per-request limit expired before `discover` completed. The local focused regression passes. Increase only that fixture's deadline while keeping the server stream open until DELETE, so a client waiting for EOF still fails the test.

The prototype currently covers only the Workspace identity, repository path, goal, isolation mode, and basic Team registration/coordinator selection. It does not replace the shipped Workspace setup, planning review, Agent Room, settings, MCP, approvals, logs, diffs, or workflow controls.

### Agent Room task-status rendering

`src/native_ui_probe/render.rs` reads the real `Controller::snapshot` envelope (`report.state`, `report.plan`, and `report.tasks`) as well as the startup snapshot (`run_state`, `proposal.plan`) and top-level fixture form. A regression using the real envelope first failed because the renderer only read top-level fields. The view renders the goal/state and each planned task's assigned agent and current status. Status colors are Rust-side: running alternates between two green levels on 500 ms repaint boundaries, waiting approval and blocked are amber, failed is red, completed is green, and pending/unknown are gray. Repaint is scheduled only while a task is running, so an idle view does not continuously redraw. The earlier pulse test caught that a sine wave could repeat at repaint boundaries; half-second color alternation now passes. The native UI suites pass 12 tests; all-target compile, source-layout, focused feature Clippy, rustfmt and diff checks pass. Graphify extraction of the touched renderer/test produced 10 nodes and 13 edges. CocoIndex was unavailable as a CLI or enabled tool, and no project dependency was added.

This slice fixes the snapshot projection only. The probe is still opt-in and currently launches the Workspace path; it has not replaced or been connected to the shipped Agent Room window. Production integration, workflow/accessibility parity and cross-platform UI acceptance remain open.

### Real Agent Room probe path

The existing opt-in binary also accepts `--room <run-dir>`. It opens the exact run with `Room::open`, then `Probe::connect_room` consumes `Controller::snapshot` through the existing bounded worker. The Room surface hides Workspace edit controls and exposes an accessible refresh action. Snapshot refresh is asynchronous: one second for a running job/task and five seconds while idle, matching the shipped Room poll cadence; completed worker replies trigger repaint. This probe path does not call the WebView launcher or construct a second Room/controller; `Room::open` retains its standard recovery/auto-resume policy. Locally, an integration regression opens a real SQLite-backed completed run, checks the actual Room snapshot envelope and verifies the Room-only accessibility controls. The 15 native UI tests, optional probe binary check, default all-target compile, source-layout, feature Clippy and rustfmt pass. This does not yet create the app's production native window or establish macOS/Windows UI/resource acceptance.

Run the opt-in probe against an existing run with `cargo run --features native-ui-probe --bin agentconductor-native-ui-probe -- --room <run-dir>`; the default invocation still opens the Workspace probe.

## Structural map

Graphify was run on an isolated copy of the native probe source and tests. The scoped graph contained 44 nodes and 101 edges; it showed Probe::show_team calling the template ID helper, with the integration test separately proving persisted profiles and coordinator through the Rust workspace dispatcher. This graph is limited to the copied probe files and is not evidence of full production UI coverage.

## Remaining acceptance

- Port every shipped screen and interaction to the native Rust application while preserving existing workflow behavior.
- Verify Korean IME composition, keyboard navigation, screen-reader accessibility, responsive layout, and separate-window lifecycle on macOS and Windows.
- Measure idle CPU and memory for the packaged native application against the existing resource budget.
- Remove project-owned JavaScript and Python runtime/build/verification paths only after equivalent Rust behavior and checks pass.
- Audit shipped dependencies and license notices, then build and smoke-test exact macOS and Windows release archives.
