# Native Rust UI migration

## Windows signal port input synchronization — issue #9 / 2026-10-28

Exact Windows package run `40595627014` failed before saving the Signal Settings preference: the UI Automation value was `7` while the expected port was `7333`. The per-character synchronization fix was followed by run `40608159958`, where PowerShell failed to resolve `[ushort]`; the driver now uses `[System.UInt16]`. Later exact run `40628759322` passed briefing, desktop, and schedule but failed while typing the second port digit: `PORT_CHARACTER_NOT_APPLIED: expected=73 actual=7`. It stopped before Save, so neither signal persistence nor tool-decision was verified. The test had reused the UIA edit object captured before typing; a regression now requires it to reacquire the visible edit before every `ValuePattern` poll. The regression failed before that change and now passes; dispatch/source-layout suites pass 13 combined, warning-denied Clippy, formatting, diff, and the 300-line check pass. Stale-element behavior is a hypothesis until a corrected exact package passes the durable settings assertion. Workflow 23 remains uncredited; candidate acceptance remains 22/25. Evidence: `docs/evidence/issue9-windows-signal-port-input-20261028.json`.

## Windows packaged schedule input must produce a state change — issue #9 / 2026-10-28

Exact Windows package run `40602847814` on `e1a8b052` passed Windows validation, the packaged briefing, and default desktop save. `InvokePattern.Invoke()` returned and the driver logged `method=uia`, but the button remained enabled and the run stayed `waiting_availability` / `manual_required` with no `available_at`. The driver now waits for the app to disable the schedule control after UIA or keyboard input; if the app does not react, it falls back to a foreground- and process-checked pointer click. A source-contract regression failed before the fallback check was added, then the schedule dispatch suite passed 6/6. Native scheduling/interaction pass 6/6, source layout 2/2, formatting and diff checks pass. Exact Windows package verification is pending, so this workflow remains uncredited. AgentMemory: `mem_mvrd9wr9_c01569e70a04`. Evidence: `docs/evidence/issue9-windows-schedule-noop-uia-20261028.json`.

## Windows packaged tool-decision modal focus — issue #9 / 2026-10-28

Exact Windows package run `40617964910` confirmed that Signal Settings persisted correctly but the tool-decision journey failed when the pointer hit Explorer (PID 928) at `(394, 757)` instead of the app (PID 5380). The driver trusted `IsOffscreen=False` without checking the button rectangle against the app window, so it attempted to click a control below/outside the viewport. A TDD source-contract regression failed before a bounded maximize/scroll helper was added. The helper now verifies the full button rectangle is within the app window and retains same-process foreground and `WindowFromPoint` checks for scroll and click actions. Decision-dispatch tests pass 3/3, source-layout 2/2, Clippy, rustfmt, diff checks, and the 300-line limit pass. A new exact Windows package run is required; workflow 23 remains uncredited and candidate acceptance remains 22/25 until its receipt/effect assertions pass. Evidence: `docs/evidence/issue9-windows-tool-decision-focus-20261028.json`.

## Windows packaged tool-refresh timeout evidence — issue #9 / 2026-10-28

Exact Windows package run `40622684398` no longer failed on pointer ownership; it reached the refresh journey but the original receipt did not become `superseded` within the test's 10-second wait. The packaged test deleted its temporary run on panic, so the run artifact did not retain the child action's owner record, stdout/stderr, workflow report, or ledger receipt needed to distinguish a refresh failure from a polling race. Commit `3a037b61` adds bounded timeout diagnostics and leaves the deadline unchanged. Its first formatter used `Debug` on `Receipt`, which is not implemented; corrected in `61b5f722` with a shared formatter and cross-platform compile regression that constructs the actual type. Local dispatch/source-layout tests pass 7 combined, warning-denied Clippy, formatting, and diff checks pass. The Mac host has no installed Windows Rust target. Run `40627667365` is validating prior commit `3a037b61`; a new exact Windows package run on `61b5f722` is required. The refresh cause remains unknown. Workflow 23 remains uncredited; candidate acceptance remains 22/25 and shipped Rust replacement remains 0/25. Evidence: `docs/evidence/issue9-windows-tool-decision-focus-20261028.json`.

## Windows signal preference Save control — issue #9 / 2026-10-28

Exact Windows package run `40539263133` validated the `7333` port field before Save but did not create `signal-transport.json`. The source path maps the UI workspace to `startup.root()`, so the path is not the apparent cause. The failure log did not expose which same-name Save control was clicked, its enabled state, or the asynchronous worker result. A test-first driver correction now chooses only a visible enabled Save button, invokes UI Automation with a checked mouse fallback, and records the accessibility tree/process state if persistence still fails. Local dispatch/layout checks pass 11 tests; exact Windows package verification is pending. Candidate UI acceptance remains 22/25; shipped/default Rust replacement remains 0/25.

## Legacy browser asset cleanup audit — 2026-10-24

The authored application, test, script, and example source inventory has no JavaScript, Python, CSS, or HTML UI files; the `source_layout` test passes the Rust-only and 300-line contracts. English and Korean `docs/user-manual*.html` files are Rust-generated offline manual outputs and remain release documentation. Retain `tests/native_ui_assets.rs` as a negative regression that checks deleted WebView assets remain absent. This closes the stale cleanup checklist item only; it does not prove the native UI is shipped or add workflow acceptance. Evidence: `docs/evidence/issue9-rust-source-inventory-20261024.json`.

## Exact-package named-workspace lifecycle recheck — 2026-10-24

On exact macOS package source `f23381cf`, a temporary HOME and the exact app bundle path isolated the UI review. Two named workspaces were created through the menu, each saved with a separate folder and goal, listed, switched, and reopened with values intact. Removing the inactive first entry from the recent list kept its workspace directory and it remained openable from the saved-workspace list. No CLI, MCP, external service, plan, task, archive, or user workspace was touched. The single process and temporary tree were cleaned. This repeats already accepted UC-30/UC-31 coverage; no new workflow credit. Packaged acceptance remains 15/25; shipped/default Rust UI remains 0/25. Evidence: `docs/evidence/issue9-f23381cf-named-workspaces-20261024.json`.

## Restore the Settings gear control — R-70 / 2026-10-24

The Rust Workspace used a text-only Settings menu entry, conflicting with the existing R-70 amendment. It now shows a 20-point gear glyph only; the localized Settings name remains available through AccessKit and the hover tooltip. The regression test first failed on the missing visible gear, then passed with the icon-only control. Six focused suites pass 22 tests; targeted warning-denied Clippy, formatting, and source layout pass. Source and test files remain 262 and 288 lines. Exact macOS package validation and all four shared/worktree × Git/folder CLI smoke cases pass for source `f23381cfb8cf4165a9207a0a67805c9c89de4d8a`. Evidence: `docs/evidence/issue9-settings-gear-f23381cf-20261024.json`.

The exact macOS package was later opened: the gear-only Settings control was visible in light and dark themes, and the Settings menu opened with its accessible name. No screenshot was retained; tooltip hover was not checked. A follow-up launched the package binary with a temporary `HOME` and disposable `ui-new` root, but CUA display-name lookup resolved older package `91393d7b` and showed saved user workspace content. Testing stopped immediately; no fields or settings were changed and no task, approval, or merge was started. The stale app was quit. Use the exact `.app` bundle path for a future package review. This earns no UI credit. Packaged acceptance remains 15/25; shipped/default Rust UI remains 0/25. See `docs/evidence/issue9-settings-gear-f23381cf-20261024.json`.

## Current-head macOS package smoke — issue #9 / 2026-10-24

The exact pushed PR head `c10121cee4a94e4d0af701779459a34cc3a684f4` builds an offline aarch64 macOS package with the Rust `package-release` command. Package validation and ZIP integrity pass; the archive contains its SBOM and dependency-license notices. The package smoke covers shared-folder and worktree execution with both Git and plain-folder sources: four cases, three parallel tasks and tests per case, approved merges verified, and unapproved merges refused. The local package archive SHA-256 is `6ec2aed9dece072a390906416bd6a4288d40e8f698af404c961fb5c71b031f94`; evidence is `docs/evidence/issue9-local-macos-package-c10121ce-20261024.json`.

This CLI smoke creates only disposable Rust fixtures. It does not count as visual UI acceptance or prove Windows behavior. Packaged native UI acceptance remains 15/25 and shipped/default Rust UI remains 0/25; issue #9 stays open. CocoIndex is unavailable in this host, so no CocoIndex result is claimed.

## UTM guest reset follow-up — issue #9 / 2026-10-24

The user approved the UTM reset after the warning that unsaved guest changes would be lost. UTM still reports the VM as `started`. The display showed UEFI boot output, then `Display output is not active`; a later `utmctl ip-address` returned IPv4 `192.168.64.6` and IPv6 addresses. A read-only `utmctl file pull` for `C:\Windows\win.ini` aborted with signal 6 (exit 134), as did an earlier staged-package read. A temporary guest marker was attempted, but its cleanup could not be verified after the same failure. No further power action was taken. Windows and `Z:` verification remain open; see `docs/evidence/issue9-utm-windows-state-20261024.json`.

## Current Windows UTM staging — issue #9 / 2026-10-24

The existing guest `Z:` share maps to `/Users/neodamul/Downloads/UTM`; no share setting or VM power control was changed. The current PR source archive and a Windows x64 package are staged under `Z:\AgentConductor\`. The package came from successful platform validation/package run `39968582271`, with inner archive SHA-256 `b9540618139f279d22cbfe7515618ad0d5db04f3b67da8be3c9838ef4835c281`; its ZIP integrity passed and it includes the bilingual manuals, license notices and SBOM. Source code under `src/`, `tests/`, `Cargo.toml`, and `Cargo.lock` has no changes since the package's PR source commit. The latest guest recheck cancelled the Windows restart-apps prompt; UTM remains started at `192.168.64.6` and the display is black. A read-only PowerShell query through `utmctl exec` fails with OSStatus `-10004`, and macOS denies listing the host share folder. UTM's Restart control warns it resets the VM and loses unsaved guest changes; the reset has not been confirmed. Guest `Z:` visibility, package launch and Windows UI behavior remain unverified. Evidence: `docs/evidence/issue9-utm-windows-state-20261024.json`.

At PR head `7806b18e2ca6a3b8ce493e81664a3e267df4679d`, local locked/offline release tests pass 959 tests with 14 ignored across 319 binaries; warning-denied all-target release Clippy, rustfmt, public docs/source-layout (13 tests), and diff checks pass. Graphify was refreshed after the current docs/evidence update to 9,180 nodes / 14,377 edges without clustering. Platform Validation run `39983599833` is validating macOS and Windows; its test jobs remain in progress. This local result does not verify the Windows guest or add packaged UI acceptance. AgentMemory: `mem_mvl4tx5n_79ab32014652` records the prior full regression; `mem_mvl63tpj_d85ad6415cc8` records the latest Windows guest boundary.

The pushed PR head `c67425bfa897e8f7a30efe989c7ce132ec28e4e8` also builds and runs the default Rust CLI offline with `--release -- --help`, without opening another desktop window. The 18.0 MiB executable at `target/release/agentconductor` has SHA-256 `3fa44f63193cdb47ec6d06db450a090804b1241eddb4938e3c2da4451beeaa65`. This validates a local executable only; the exact source still needs packaged release artifacts and Windows guest/runtime verification. AgentMemory: `mem_mvl4xlbl_400fc4fb5a23`.

### PR #22 macOS lint correction — issue #9 / 2026-10-23

Actions run `39956497286` passed macOS build, tests, native UI probe, formatting and layout checks but warning-denied Clippy rejected an equality search using `iter().any()` with `manual_contains`. The test now uses `labels.contains(&result)`. On local Rust 1.99, its focused suite passes 7/7, all-target Clippy and rustfmt pass, and the 300-line source-layout suite passes 2/2. Graphify refreshed without clustering (9,159 nodes / 15,548 edges). CocoIndex's CLI and Python module are unavailable. The pre-fix Windows job was still running when observed and does not validate this change; rerun/check both platforms on the pushed fix head. Evidence: `docs/evidence/issue9-pr-clippy-fix-20261023.json`; AgentMemory: `mem_mvkzmzgs_5e9246d8aa0a`. Packaged UI acceptance remains 15/25 and shipped/default Rust UI remains 0/25.

## Live UTM Windows screen recheck — 2026-10-23

UTM reports the Windows VM as started. The Windows restart-apps confirmation is visible and lists UTM Guest Tools Installer, Network File Explorer, and Task Manager. The existing Z: mapping and UTM share settings were not changed. Input capture was temporarily enabled for a Cancel attempt and restored off; the prompt remained visible. No restart, shutdown, or Windows build occurred. Guest-side Z: visibility remains unverified. Evidence: `docs/evidence/issue9-utm-screen-recheck-20261023.json`.

## UTM Z: share transfer checkpoint — 2026-10-23

The existing Windows UTM `Z:` mapping uses the host folder `/Users/neodamul/Downloads/UTM`. The exact branch-head archive `AgentConductor-c4cbd2a.tar.gz` was copied there and is visible in Finder. Only the source archive checksum is recorded; destination checksum and guest access remain unverified. The VM stayed started, input capture was restored off, and no restart or shutdown was selected while a Windows restart-apps confirmation remained visible. This does not count as Windows build, package, UI, or workflow acceptance. Evidence: `docs/evidence/issue9-utm-z-share-transfer-20261023.json`.

## Windows packaged schedule UI — 2026-10-25

### Click target diagnosis — 2026-10-26

Exact package run `40241618513` passed both platform validation jobs, macOS package verification, Windows package build/smoke, Windows briefing, and default desktop save. The schedule button was visible and enabled at `(720,684)-(819,704)`; the script moved to `(770,694)` and sent the timed mouse click, but the report stayed `manual_required` and no coordinator action file appeared. Therefore the added delays did not solve dispatch. TDD now requires checking the actual foreground HWND and the HWND under the click point against the packaged app PID before sending input, with PID diagnostics. Focused local suites pass 16 tests; exact Windows schedule persistence remains unverified. Evidence: `docs/evidence/issue9-windows-schedule-40241618513-20261026.json`.

### Native click timing follow-up — 2026-10-26

Exact package run `40237790745` passed both platform validation jobs and macOS/Windows package builds/smokes. The schedule button was visible at `(720,684)-(819,704)` and the native mouse click script ran, but no coordinator action was recorded and the report remained `manual_required`. TDD adds 150 ms after cursor placement and 100 ms between mouse-down and mouse-up, with enabled-state and target-point diagnostics. Sixteen focused tests pass locally with Clippy, rustfmt, and diff checks. Exact Windows persistence remains the acceptance gate. Evidence: `docs/evidence/issue9-windows-schedule-40237790745-20261026.json`.

### Click activation after native scrolling — 2026-10-26

Exact-package run `40234021794` passed macOS/Windows validation and package build/smoke; Windows desktop save also passed. Native cursor positioning plus wheel input successfully moved the schedule button into the viewport at `(748,684)-(847,704)`. UIA `Invoke` then failed with `ElementNotEnabledException`, so no schedule was saved. TDD now requires an enabled-state check after scrolling and a real mouse down/up at the visible button center. Sixteen focused tests, warning-denied all-target Clippy, rustfmt, and diff checks pass locally. Exact Windows persistence remains the gate. Evidence: `docs/evidence/issue9-windows-schedule-40234021794-20261026.json`.

### Native wheel delivery follow-up — 2026-10-26

Exact package run `40230530142` passed macOS/Windows validation and package builds/smokes, and the Windows default desktop save passed. The schedule button stayed clipped at `(748,792)-(847,812)` below window bottom `728`, even after `WM_MOUSEWHEEL` was sent to the top-level HWND using coordinates inside the inspector. No schedule action or persisted timestamp resulted. TDD now requires cursor positioning and a native wheel event over the inspector; the contract failed before replacing the direct message. Fifteen focused tests, warning-denied all-target Clippy, rustfmt, and diff checks pass locally. The next exact Windows package run must persist the expected timezone-qualified schedule. Evidence: `docs/evidence/issue9-windows-schedule-40230530142-20261026.json`.

### Inspector scroll target correction — 2026-10-26

Exact-package run `40226959980` passed platform checks, package builds/smokes, Windows desktop save, and briefing. The schedule journey still left its button below the window, and UIA `RangeValuePattern` was unsupported. Source inspection found the native wheel event used the window center, over the activity column; the schedule field is in the right-hand inspector. TDD now requires wheel coordinates at the inspector's visible lower-right region. The first regression run failed against the old coordinates; after the change, 15 focused tests, warning-denied all-target Clippy, rustfmt, and diff checks pass. The exact Windows package retest must still persist `manual_schedule=2099-01-01T09:00:00+09:00`. Evidence: `docs/evidence/issue9-windows-schedule-40226959980-20261026.json`.

### Exact-package retest — native wheel ineffective, 2026-10-26

Run `40223438114` again passed both platform-validation jobs and exact macOS/Windows package builds and smoke. The Windows package's desktop save and briefing passed, but schedule remained clipped at `(748,792)-(847,812)` with window bottom `728`; the `-360` `WM_MOUSEWHEEL` message had no effect. No action or manual schedule was recorded. The next test revision uses the right-edge vertical UIA scrollbar's `RangeValuePattern` when available, logs provider support, and retains the bounds guard. Local schedule/docs/layout tests pass 15 tests; exact-package verification remains pending. Evidence: `docs/evidence/issue9-windows-schedule-40223438114-20261026.json`.

### Exact-package retest — briefing recovery and clipped schedule, 2026-10-26

Run `40219585922` passed both platform-validation jobs and both exact package build/smoke jobs. The Windows package's default-desktop save and read-only briefing journeys passed; briefing completed in 12.31 seconds after the UIA no-effect click fallback. Schedule failed before dispatch because the original button remained clipped at `(720,762)-(819,782)` while the maximized window ended at `y=728`. No `manual_schedule` timestamp was persisted. The new TDD contract requires a three-notch negative wheel delta (`-360`) while retaining a bounds-based stop condition. Exact Windows package retest remains pending. Evidence: `docs/evidence/issue9-windows-schedule-40219585922-20261026.json`.

### Exact-package retest — briefing driver follow-up, 2026-10-26

Run `40215446452` passed macOS/Windows validation, both package builds, and package smoke, but the Windows UI gate failed in `windows_briefing` before reaching schedule setup. UI Automation's `Invoke()` returned without an exception, yet the briefing menu never appeared; the original button remained visible after the 40-second wait. The driver had used a physical click only when `Invoke()` threw, so it did not recover from a successful-but-ineffective invocation. A new regression contract failed before the change. The driver now checks for the menu for two seconds and sends a real click only if the menu is still absent. Local regression, docs/layout tests pass 14/14; warning-denied all-target Clippy and formatting pass. Exact package retest remains required; no schedule acceptance is claimed. Evidence: `docs/evidence/issue9-windows-briefing-402154-failure-20261026.json`.

Exact-package run `40193058379` passed macOS and Windows validation, both package builds, and the Windows default save and briefing UI journeys. The schedule journey still failed: its UIA button reported `IsOffscreen=false`, but the button rectangle (`834,878–933,898`) was below the native window bottom (`857`). UIA activation returned and emitted a marker without creating a coordinator action or changing the persisted report. Never count an activation marker or `IsOffscreen=false` as success; compare the control rectangle with its window and require `manual_schedule` plus the exact timestamp. The next test revision maximizes the isolated window before discovering controls and rejects a button rectangle outside the window; package verification remains pending. Evidence: `docs/evidence/issue9-windows-schedule-40193058379-20261025.json`.

Exact-package run `40196728804` confirmed a second geometry failure after maximizing: the schedule button was at `(748,792)-(847,812)` while the maximized window ended at `y=728`. The native Agent Room had no outer vertical scroll area, so lower controls were rendered below the viewport. The fix wraps the room page in a stable vertical scroll area and calls UI Automation `ScrollItemPattern.ScrollIntoView()` before checking the button bounds. Local dispatch, room-render, and source-layout tests pass 15/15; Windows package acceptance must still prove persisted `manual_schedule` and the exact requested timestamp. Evidence: `docs/evidence/issue9-windows-schedule-40196728804-20261025.json`.

The first scroll-area driver revision was run against the exact Windows package in `40200012590`. Both platform validation jobs and macOS packaging passed; Windows release build, launch, desktop save, and briefing passed. The schedule test then confirmed that the UIA provider does not implement `ScrollItemPattern` on the button (`Unsupported Pattern`). The driver now walks the original button's control ancestors for `ScrollPattern` and scrolls until its bounds fit the window, keeping the bound guard and persisted-state assertion. Focused tests pass 26/26 and all-target Clippy/rustfmt/diff checks pass; exact package acceptance remains pending. Evidence: `docs/evidence/issue9-windows-schedule-40200012590-20261025.json`.

The follow-up exact-package run `40203935608` confirmed the same Windows provider exposes no `ScrollPattern` on the button's ancestor chain. Platform validation and macOS package execution passed; the Windows release binary launched and passed the default desktop and briefing checks before schedule setup failed. The test now uses bounded native mouse-wheel input over the foreground Agent Room window, checking the button bounds after each tick before dispatch. Local public-docs, source-layout, schedule-dispatch, and room-render tests pass 26/26; warning-denied Clippy, rustfmt, and diff checks pass. Exact-package durable schedule verification remains pending. Evidence: `docs/evidence/issue9-windows-schedule-40203935608-20261026.json`.

Run `40207836777` showed that global `mouse_event` injection did not scroll the foreground Windows package: the schedule button stayed at `(748,792)-(847,812)` while the window ended at `y=728`. The next driver sends `WM_MOUSEWHEEL` directly to the room HWND with screen coordinates and retains the per-tick bounds check. Both platform validation jobs and macOS packaging passed; Windows package launch, desktop save, and briefing passed; exact schedule persistence remains unverified. Evidence: `docs/evidence/issue9-windows-schedule-40207836777-20261026.json`.

Exact-head run `40211646053` passed both platform validation jobs, macOS and Windows release packaging/smoke checks, and Windows desktop-save and briefing UI tests. The schedule UI still failed its clipping guard: the button was at `(748,792)-(847,812)` below the window ending at `y=728`. Inspection found the direct `WM_MOUSEWHEEL` driver sent a positive delta, which scrolls upward while the schedule button is below the viewport. The test contract now requires a negative delta; exact package acceptance remains gated on persisting `manual_schedule` with the requested timezone-qualified timestamp. Evidence: `docs/evidence/issue9-windows-schedule-40211646053-20261026.json`.

## Current packaged-acceptance checkpoint — 2026-10-23

The PR #22 metadata recorded below was last observed at branch head `8b50d0bd`: open/draft against `main`, merge state `DIRTY`, with no checks reported. At that observation, `origin/main` was `602ec118`, v0.1.37 was published and the Rust branch package version was 0.1.36; recheck before packaging. A read-only merge-tree reported 39 conflicts and no merge was attempted. The newer UTM transfer state is recorded above: the current source archive is copied to the user's existing Z: host share, while guest-side access is still unverified. This adds no UI workflow credit.

Workflow 15 is accepted at **15/25 (60%)** after exact v0.1.36 package UI verification with a
disposable HOME and direct executable binding. The package showed the isolated workspace root,
zero registered agents, the embedded Agent Room manual image, and scrollable v0.1.37 release notes.
The bilingual Agent Room manual images are now retained source-built Rust UI captures; exact-package
element-by-element comparison against the design guide remains open. The capture adds no workflow
credit. Issue #9 remains open; shipped/default Rust UI replacement remains 0/25. Evidence:
`docs/evidence/issue9-workflow15-ui-isolated-20261023.json` and
`docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.

### Safe manual-link backend — R-144 / 2026-10-23

The native manual emits egui `OpenUrl` commands only after the existing HTTP(S) allowlist accepts the target. The native runner must compile with eframe's `links` feature; without it, egui-winit deliberately drops the request and logs that browser opening is disabled. Regression coverage checks this build contract alongside safe-link interactions. Enabling the feature brings in `webbrowser 1.2.5` (MIT OR Apache-2.0); retain its license texts in generated release bundles. Exact macOS arm64 package `d539e86f1d099279e7a80fe8f9753b9065621631` validates with 242 dependencies and passes all four packaged smoke cases. Its binary grows by 235,136 bytes (18,705,600 → 18,940,736) compared with the previous package at `ec1fc3c1ac06`. A later exact-package test opened the official safe HTTPS reference in Safari; unsafe-scheme UI remains source-test-only.

### Exact-package UI binding and isolation — 2026-10-23

A direct launch of package `d539e86f1d09` with temporary HOME and `ui-new` opened an empty workspace and the bundled offline manual. Rebinding through CUA by app identity can start another GUI instance without the launch arguments or environment; that instance loaded the user's saved workspace and refreshed quota metadata. It was stopped without editing or saving workspace fields or running a task, approval, update, or merge; temporary files were removed. A quota refresh may have occurred.

The initial safe HTTPS link click did not yield an observable browser destination because the CUA binding had reopened the default-profile app. A later exact-package link check opened the expected official reference in Safari; an isolated direct launch then proved Workflow 15's data root. Unsafe-scheme UI behavior remains source-test-only. AgentMemory: `mem_mvkohxhn_1d474190b4f1`; issue: [#9](https://github.com/neodamul/AgentConductor/issues/9).

### Package verification for native manual safe links — 2026-10-23

The pushed `8c8a08a8` source built as a checksum-valid macOS arm64 v0.1.36 package. Its 214 SBOM components all have license expressions, and the package contains dependency-license and third-party-notice files. The isolated smoke matrix passes all four shared/worktree × Git/folder combinations with three parallel tasks and tests per case; approved merges succeed and unapproved merges are refused. Evidence: `docs/evidence/issue9-native-manual-links-package-20261023.json`. The package was not launched because the only visible AgentConductor window had returned to the user's saved workspace. This is package/build evidence, not exact-package manual-link interaction or workflow acceptance.

### Safe links in the app-menu manual — R-144 / 2026-10-23

The native manual now renders Markdown links as clickable UI links in paragraphs, lists, table headers, and cells, using the same URL allowlist as the standalone offline HTML renderer. Interaction tests click each safe-link context and confirm unsafe schemes never emit an open-URL command. The Markdown remains the source for both native blocks and generated standalone HTML; no browser or local web server is introduced. TDD evidence is `docs/evidence/issue9-native-manual-safe-links-20261023.json`. Exact-package safe-link dispatch is verified. The embedded Agent Room image was later replaced with bilingual source-built Rust captures; that documentation update added no workflow acceptance.

### Issue #9 PR-head checkpoint synchronization — 2026-10-23

PR #22 remains open and draft at `cba882af115dacbefadb8a137802bb57f957e78a`. The issue and completion audit now use the same live head. The current-head regression, public-doc, and source-layout suites pass 14 tests; formatting and `git diff --check` pass. The PR-check query returned no statuses, so no remote CI pass is claimed. Packaged operator workflow acceptance remains 14/25, and shipped/default Rust UI replacement remains 0/25. This documentation synchronization adds no workflow credit. Evidence: `docs/evidence/issue9-pr-head-status-cba882af-20261023.json`.

### Exact-package manual and release-note access check — issue #9 / workflow 15 slice

In the existing v0.1.36 macOS package, the Workspace menu opened the offline manual; closing it returned to the unchanged workspace. The Settings update panel then exposed installed v0.1.36, latest v0.1.37, the GitHub release link, release-note text underneath it, and a scrollbar. The content included release items #162, #165, #149, and #156. The update control became enabled after its check. No install/restart, workspace save, agent, or provider operation occurred. Evidence: `docs/evidence/issue9-f438820-manual-update-ui-ax-20261023.json`.

This is an accessibility-tree component check only. It does not verify visual layout, screenshot persistence, HTML manual rendering, actual scroll movement, or install behavior, and it adds no packaged workflow acceptance. Workflow 15 remains partial.

### Current PR checkpoint — issue #9 / 2026-10-23

GitHub PR #22 is open as a draft. At the start of this documentation synchronization its head was `b6b44f7c4e59e3955ae2742d3029d3a7af791067`; the synchronization itself added a follow-up commit. Packaged native UI acceptance remains 14/25; shipped/default Rust UI replacement remains 0/25. GitHub's PR-check query returned no checks or statuses for the preceding revision, so no CI pass is claimed. Evidence source: the GitHub issue/PR snapshots and `docs/09-completion-audit.md`.

### Native UI accessibility and keyboard regression checkpoint — issue #9 / 2026-10-23

At source head `f5a7ed6d`, focused offline release-mode tests pass 15/15 across the task editor, MCP keyboard flow, and profile contrast suites. They verify localized accessible task controls, overlay layout, Tab/Enter behavior, and minimum 4.5:1 contrast for profile names and quota values in light and dark themes. This is automated source-level evidence only; screen-reader behavior and Korean IME remain unverified, and no packaged workflow acceptance is added. Evidence: `docs/evidence/issue9-native-ui-accessibility-keyboard-20261023.json`.

### Current PR-head docs/layout verification — issue #9 / 2026-10-23

At pushed head `608eb741`, the offline locked release-mode `public_docs` and `source_layout` test suites pass 13/13. Formatting and `git diff --check` pass. This verifies repository documentation and the 300-line source/test guard only; it adds no packaged workflow acceptance and does not close platform or release gates. Evidence: `docs/evidence/issue9-current-head-doc-layout-20261023.json`.

### Windows guest access probe — issue #9 / 2026-10-23

The existing UTM Windows ARM64 VM was stopped before the probe and configured with 10 GB RAM. It booted to the Windows lock screen; no guest session was authenticated, no credentials or project files were accessed, and no Windows build/package/UI test ran. The VM was then stopped and verified stopped to return host memory. Windows validation still requires an authorized guest login or another Windows build host. Evidence: `docs/evidence/issue9-windows-vm-access-20261023.json`.

### Windows login gate retry — issue #9 / 2026-10-23

The selected Windows App device `neodamul-PC` could not be found (`0x104`). Retrying through the existing UTM Windows ARM64 VM reached its sign-in screen after a recovery screen reported a network failure. No password was entered and no project files were accessed. UTM's Stop action warned of possible guest-disk damage, so it was cancelled and the VM was paused; it may retain its configured 10 GB RAM. Windows build, package, and UI checks remain pending an operator sign-in. Evidence: `docs/evidence/issue9-windows-utm-login-20261023.json`.

### Current source-layout and artifact checkpoint — issue #9 / 2026-10-23

The latest product-code source and exact macOS package baseline is `9845db79af260b631bde7ade2104bae2400cc649`; the branch was verified at docs head `12d6d808`. The complete locked/offline native-feature all-target suite passes 938 tests, with 14 ignored and 0 failures across 310 test binaries (363.89 seconds). Warning-denied all-target Clippy, rustfmt, diff checks, and the combined public-doc/source-layout checks pass. Evidence: `docs/evidence/issue9-full-regression-20261023.json`. The Rust source/test inventory has no file above 300 lines. Packaged UI acceptance is 14/25 after workflows 13 and 14; shipped/default Rust UI replacement remains 0/25. Windows UI, accessibility/Korean IME, current screenshots, resource measurements, and final-source license review are still open.

At the earlier read-only checkpoint, the existing v0.1.36 window was backed by the `cff95dd` bundle and was not treated as exact-head workflow evidence. With no Cargo/Rust compiler process active, only regenerable `target/debug/incremental` and `target/debug/tmp` files were removed: measured `target/` size fell from 11 GB to 8.7 GB. Debug dependencies, release output, the active app, and packaged artifacts were preserved. AgentMemory records the cleanup as `mem_mvk94ev1_7dfbd3b09cfb`.

### Worktree cleanup after run finalization — GitHub #143 / 2026-10-22

Run-owned Git worktrees are removed after candidate merge or rejection, before run-cache deletion,
and before workspace archival. `Repository::remove_worktrees_under` canonicalizes the run boundary,
removes only linked checkouts beneath it, and prunes entries whose folders have vanished; unrelated
worktrees are preserved. Cleanup failures after finalization are recorded as `worktree_cleanup`
and do not reverse the final run state. Existing diff/report evidence stays in the run directory.

The upstream change initially grew `src/git.rs` past the project 300-line limit. The method now lives
in `src/git/worktrees.rs` while callers retain the same `Repository` API. Four focused regressions
cover merge/reject cleanup, preserved evidence, dangling entries, and isolation of unrelated
worktrees. The focused suite and source-layout gate pass; the full native-feature suite passes 873
with 14 ignored. This backend lifecycle fix does not add packaged UI workflow acceptance; issue #9
remains 8/25 until another exact-package user journey is accepted.


### Exact merged worktree-cleanup package — 2026-10-22

Commit `cd9935de0d598612af1f52c1a11b699c1714de26` includes main's run-owned worktree cleanup and keeps the 300-line source guard by placing the helper in `src/git/worktrees.rs`. The full native-feature suite passed 873 tests with 14 ignored; warning-denied Clippy, formatting, and diff checks passed.

The Rust renderer generated both offline manuals. The exact macOS package passed manifest, binary checksum, `--help`, and deterministic fixture workflow smoke checks. Shared-folder and worktree modes each ran against Git and non-Git folders; every case completed three parallel tasks and tests, rejected merge before approval, and merged only after approval. No real CLI or UI was launched. Package evidence: `docs/evidence/issue9-cd9935de-macos-package.json`.

Graphify reports 8,479 nodes / 14,574 edges. Cargo cleanup reclaimed 10.8 GiB and a superseded macOS package pair; the exact `cd9935de` package is retained. Exact-head GitHub run `39740270740` passed all macOS and Windows validation and package jobs, including exact-binary smoke execution. Retained artifact IDs: macOS `12409469483`, Windows `12409824890`; their digests are in the package evidence. UI acceptance remains 8/25.


### Exact-source macOS package checkpoint — 2026-10-22

The pushed `ead895e51bcf92b6d843447cc8c3c00790958a8b` head packages as `aarch64-apple-darwin` with the binary and manifest naming the same source commit. Package validation, packaged `--help`, internal binary SHA-256, and archive CRC pass. The archive hash is recorded in `docs/evidence/issue9-ead895e-macos-package.json`.

The GUI was not replaced because the current application showed an unsaved-workspace warning. Windows exact-head package/runtime and a complete packaged GUI workflow remain unverified; issue #9 stays at 8/25. This checkpoint does not count as UC-28 acceptance.


### Exact-head macOS package verification — 2026-10-22

Issue #9 head `9443a666096bb68e590ad757dbb08a852de4b75c` builds a validated
`aarch64-apple-darwin` package containing one 18,356,880-byte native executable and 214
dependency records. The packaged `--help` command and built-in package smoke pass. The smoke
covers shared-folder and worktree execution over Git and plain-folder inputs; each case runs
three parallel tasks and tests, refuses an unapproved merge, then verifies the approved merge.
The exact package was opened in the existing single app slot. A disposable workspace saved a
three-task manual plan with disjoint paths and distinct Codex/local-agent assignments. The review
screen displayed digest `4059a27f9ed4c8bf48b85a1c0d50fc4b87d5e5af8d54dd0eb45640ee54314b26` and
the execution-consent dialog showed each profile, permission, writable path, argv and test command.
The UI refused both an overlapping path and an empty task prompt. The final confirmation explicitly
starts the registered CLI processes, so it was canceled before execution. The temporary workspace
was archived, its project folder removed, and the original workspace restored. This does not accept
UC-28 or advance packaged native UI acceptance beyond 8/25. Evidence:
`docs/evidence/issue9-9443a66-macos-package.json`.

### Previous local native-feature suite verification — issue #9 / 2026-10-22

At branch head `696b2f8`, the full feature-enabled all-target suite passes 845 tests with 14
ignored and no failures across 289 suites. The source code was last changed in `214e359`; later
commits only updated verification documentation. This local suite does not prove current Windows
runtime/package parity and does not advance the 8/25 packaged workflow acceptance count. Graphify's
no-cluster code refresh reports 8,170 nodes and 14,029 edges. CocoIndex Code cannot start on this
host because its launcher cannot import `mcp.server.fastmcp`.

### Workspace switcher overlay versus layout movement — 2026-10-21

When reviewing the top-bar workspace switcher, the tall opaque popup made the page look as if it
moved downward. The regression now mirrors the production outer frame, left agent roster, and
central panel, includes one saved workspace, then compares both panels' AccessKit y-coordinates
before and after opening the switcher. It also checks that the saved entry's Open/Archive controls
remain exposed. The panel coordinates stay stable: the popup covers underlying content while open,
but does not reflow the page. Keep this distinction explicit when diagnosing visual menu feedback.

The follow-up audit initially tested the phase-details menu without its production parent scroll
area. That was insufficient, so the regression now mirrors the workspace scroll container, uses a
480px viewport, eight phases, and a long final-phase popup. It opens that last menu and checks that
the leading content anchor's y-coordinate stays fixed while the details are visible. The popup does
not reflow or scroll the page in this case.

The Coordinator analysis popup is also inside this production scroll area. Its rendered regression
uses a long analysis body, confirms that the popup remains viewport-bounded, and checks that the
workspace anchor does not move when it opens. Both tested goal-page popups preserve the page scroll.

The platform workflow keeps an explicit list of native UI suites, so the new workspace and goal-menu
regressions were initially absent from cross-platform execution. Added both to the macOS/Windows UI
test and Clippy commands, plus `tests/platform_validation.rs` to assert both registrations. Focused
menu and workflow tests pass locally.

### Keep current acceptance separate from historical snapshots — 2026-10-21

The current native packaged-workflow count advances in `docs/07-implementation-plan.md`, while
completion-audit and evidence sections intentionally preserve counts from their original dates.
When that current count changes, update only assertions describing the current plan; retain earlier
counts where they record historical audit or package evidence. Actions run `39606419496` caught a
stale test expecting the plan to remain at 6/25 after Workflow 8 raised it to 8/25. Both platform
jobs failed on this same documentation assertion; the Windows UI Automation save test itself passed,
including the previously reported goal-value case.

### Keep Markdown assertions independent of Windows line endings — 2026-10-21

Actions run `39543488348` passed Windows compilation, layout, and applicable native tests, then
failed `public_docs` because one assertion matched a literal LF across Markdown lines. Windows checkout
used CRLF. Normalize included Markdown with `replace("\r\n", "\n")` before multiline assertions;
keep single-line content checks unchanged. Re-run exact-head platform validation after this test fix.

### Exact package provenance and Rust cache cleanup — 2026-10-21

For native UI checks, select the app bundle by its full path and verify its process command line
before interacting. A generic display-name lookup can launch an older installed bundle. If the
wrong process opens, do not click or save; close it and retry only after identifying the exact
bundle. The `de032e0` macOS package was opened by full path with isolated workspace data; its
Workspace, Agent Roles, and Review empty states were inspected without saving. This is visual spot
check evidence, not a new accepted workflow.

A task worktree's `target/` grew to 61 GB while no Cargo/Rust compiler was active. Removing only
`target/debug/deps` and `target/debug/incremental` freed about 59 GB; keeping the linked debug and
release executables, release dependencies, source, and current package left a 2.2 GB target.
Rebuild the debug cache only when a debug test run is needed.

### Windows picker compile regression — 2026-10-21

Keep the picker intent (`folder` and `executable`) threaded from `picker::pick` into the Windows
Common Item Dialog; the file filter still depends on `executable`. The Windows-only
`tests/windows_console.rs` includes `src/ui/picker.rs` in a test-local `ui` shim, so it must
re-export both `agentconductor::ui::language` and `messages`. Actions run `39542757398` verified
the production picker compiled and exposed the shim omission. Commit `de21a88` adds the re-export;
run `39543488348` is the exact-head Windows/macOS revalidation. Local targeted verification passed
29 tests across six suites, macOS-target compilation, formatting, and source/test layout.

### OpenCode MCP server registration — 2026-10-21

The exact v0.1.32 package saved and reloaded a local MCP server in Settings using the OpenCode
JSON/JSONC structure, then discovered `get_issue` from the Rust fixture. The request trace
contained initialization and `tools/list` only; no `tools/call`, provider, or remote service ran.
This distinct server-management journey advances packaged native UI acceptance to **8/25 (32%)**.
Evidence: `docs/evidence/issue9-256db1a-mcp-server-registration-ui.json`.

### UC-79 exact-package archive recheck — 2026-10-21

The exact v0.1.32 package displayed the target workspace's saved name/path and stated that
project sources would be preserved. After switching to a disposable landing workspace and
confirming, it removed the target from the active list and wrote a recovery record in
`workspace-trash`; the separate project directory remained. No provider or external service ran,
and the temporary app was closed. This rechecks Workflow 2 without adding another distinct
acceptance. See `docs/evidence/issue9-256db1a-uc79-archive-ui.json`.

### Workspace settings export/import — 2026-10-21

`FolderPurpose` gives project, export, and import pickers distinct localized titles through the
existing async worker. The exact macOS package from source `256db1a` exported settings, previewed
the selected file, required explicit confirmation, and restored the workspace name. Evidence:
`docs/evidence/issue9-256db1a-settings-share-ui.json`. This distinct user journey advances
packaged UI acceptance to **7/25 (28%)**. The UI used an isolated HOME and disposable repository;
the user's workspace and external services were not touched. No screenshot was retained. Windows
GUI and exact-head Actions run `39541153826` remain open; that run was in progress for the
preceding source checkpoint.

## Exact PR-package revalidation — issue #9 / 2026-10-21

PR head `d799ad6` merged by Actions as `d386e6a` passed macOS and Windows validation and package
jobs. The exact merge-result macOS app completed an isolated Workspace-to-plan-to-three-parallel-
tasks-to-test-to-review-to-human-approval-to-merge journey. All three fixture tests passed; the
reviewed diff added only the three scoped files, and the app reached Completed after merge. This
revalidates Workflow 1 and does not add a distinct packaged workflow; acceptance remains
**6/25 (24%)**. The deterministic Rust fixture CLI and temporary HOME/repository avoided real
provider calls and user data. No screenshot was retained; Windows GUI, Korean IME/accessibility,
and visual parity remain open. See
`docs/evidence/issue9-d386e6a-native-ui-plan-to-merge.json` and the completion audit.

Cross-platform package artifact digests and executable checksums are recorded in
`docs/evidence/issue9-d386e6a-pr-merge-cross-platform-packages.json`.

## PR merge-result packages — issue #9 / 2026-10-21

Actions run `39524574228` passed both macOS and Windows validation and native-package jobs. It was
triggered for PR head `806bf32` and checked out synthetic merge result `9e36175`; the artifact
names confirm the built commit. Each package rendered the English and Korean manuals, executed
successfully, and was retained. Artifact IDs, sizes, and digests are in
`docs/evidence/issue9-9e36175-pr-merge-cross-platform-packages.json`.
This is packaged CLI evidence, not a GUI acceptance: packaged native UI remains **6/25 (24%)** and
the public release does not yet ship the Rust UI.

## Exact-head cross-platform and package checks — issue #9 / 2026-10-21

Actions run `39520326287` passes the macOS and Windows validation jobs, including the Rust UI
launch-path check, formatting and warning-denied Clippy. The macOS native package passed release
build, bilingual manual rendering, package execution and artifact upload. The Windows package also
passed all four steps. Both platform artifacts are retained with digests in
`docs/evidence/issue9-b714887-cross-platform-packages.json`. In addition, the exact macOS v0.1.32
executable passed a three-task quota-fallback and human-approval smoke in a disposable repository.
This verifies packaged orchestration but not a native UI journey; packaged UI acceptance remains
**6/25 (24%)**, and the public release still does not ship the Rust UI. See
`docs/evidence/issue9-b4ad2f5-quota-fallback-cli.json` and the implementation checklist.

## Full local regression — issue #9 / 2026-10-21

At branch head `7a6256d`, `rtk cargo test --all-targets --no-fail-fast` passes **805 tests,
14 ignored, 0 failed across 272 suites** (216.14 seconds of Cargo test execution). All-target
Clippy with `-D warnings`, formatting, and `git diff --check` pass. This complete local run
includes the previously CI-flaky approval/return-to-Workspace/Agent-Room journey and did not
reproduce its timeout. Exact-head GitHub Actions run `39517629909` remains in progress, so
cross-platform status is not yet established. Component and regression tests do not increase
packaged end-to-end workflow acceptance, which remains **6/25 (24%)**.
Graphify was refreshed after this checkpoint: 7,733 nodes / 12,227 edges.

## macOS start-journey CI triage — issue #9 (2026-10-21)

Actions run `39507118252` completed with Windows passing and the macOS
`review_approval_shows_start_feedback_then_opens_the_started_run` test failing while waiting for
the enabled `Open Agent Room` button after returning to Workspace. Ten consecutive local runs on
the current source and the focused test pass, so the CI failure is not yet explained or considered
fixed. The wait helper now includes accessible button enabled/disabled states and visible labels in
timeout diagnostics. Recheck the next exact-head macOS run before accepting this navigation path.
This test-only change does not advance packaged workflow acceptance (6/25).

An optimized exact-source macOS package from `0935f481fba1b89b329bca604ebb1c0131035992` also
passes `package-release`, manifest validation, packaged `--help`, ZIP integrity, and packaged
smoke. Binary SHA-256: `a5887a30c8475f9700c1e7e853a5e9b739ff07de219cc630fbfb8fac44e86ba4`; archive
SHA-256: `07d41c8297da13cbe4067b53a024caf812c913aa53cd0507a8c69e05d4591ab1`; 214 dependency
records. The current app window shows a saved user workspace, so the new package was not launched
and visual parity remains open. Evidence: `docs/evidence/issue9-0935f48-macos-package.json`.

## Agent Room header context — issue #9 / R-17 / DD-137 (2026-10-21)

The v0.1.32 WebView titlebar shows the app/version, repository, linked phase, and local
CLI-authentication context. Rust repeated “Agent Room” inside the page under the native window title
and omitted those context fields. Red-first rendered assertions failed in English and Korean. The
native header now restores the brand/version and localized workspace/phase/auth labels, shows the
repository name with the full path as a hover label, and avoids repeating the window title. Focused
header, localization, layout, rendering, and source-layout checks pass (21 tests across five
suites), as do all-target warning-denied Clippy, formatting, diff checks, and the 300-line guard.
Graphify refreshed to 7,719 nodes / 12,210 edges. This fixes component parity but does not add a
packaged workflow; acceptance remains 6/25. Exact-current package screenshots and platform parity
remain unverified.

The follow-up geometry regression inspects rendered `TextShape` visual bounds instead of AccessKit
nodes: passive `ui.label` text is visible but is not represented as a labeled AccessKit control.
English and Korean layouts at 792px and 1140px pass, including the brand, version, workspace,
phase, and local-auth text. This is component geometry evidence only and leaves package acceptance
at 6/25. AgentMemory `mem_mvhb36q0_2892617eb052` records the reusable test rule; Graphify is at
7,724 nodes / 12,215 edges.

## Workspace navigation parity batch — issue #9 / R-17 / DD-137 (2026-10-21)

The Workspace stage row now follows the current-main hierarchy: equal-width numbered buttons,
accent selection underline, and clear left-to-right stage order. Workspace and Agent Roles save
actions align to the content card's right edge; the primary action uses the shared theme accent and
ink colors. Tests cover both locales, 480px/836px widths, action placement, and at least 4.5:1
button-text contrast in both themes. The real start journey test and the accessible Workspace
controls test pass on the current source. The 300-line file limit, source-layout test, warning-denied
Clippy, formatting, and diff checks pass. A broad all-target run was stopped after more than ten
minutes; the latest serial retry on `d215949` was interrupted after 586 tests passed and 14 were
ignored across 193 suites, with no failure reported before interruption. That partial run is not
a full-suite pass. The earlier stale text-count assertion and start-journey click failure were
corrected, and the affected UI tests pass on current code. No current package screenshot was
captured; packaged acceptance stays 6/25 and shipped/default Rust UI replacement stays 0/25.

## Sidebar guidance parity — issue #9 / R-17 / DD-137 (2026-10-21)

The Rust agent rail now shows the localized `YOUR TEAM` eyebrow above `CLI agents`, followed by
the existing executable/authentication hint. These labels reuse existing English/Korean message
keys and do not change agent registration or card state. A rendered regression verifies both
labels in both languages. Thirteen tests pass across sidebar, density, icon, header, start-journey,
and source-layout suites; warning-denied Clippy, rustfmt, diff checks, and the 300-line limit pass.
The latest complete native-feature suite is still the earlier 797/14 result; the current retry
was partial only. This change adds no packaged workflow, and no exact-current package screenshot
was retained. Graphify refreshed to 7,582 nodes / 13,037 edges. Packaged acceptance remains 6/25;
shipped/default Rust UI remains 0/25.

## Current checkpoint — 2026-10-21

The exact v0.1.31 macOS package built from `ca5c3f4` was launched under a disposable `HOME` and
checked in one 1140×772 window. Workspace, Agent Roles, and Review each show the active step in
both the page eyebrow and its card, matching a checked-in same-size capture with a v0.1.24 label.
Screenshot provenance identifies that reference as an earlier Rust build, so this verifies Rust
component consistency only, not parity with the published WebView UI. This supersedes the interim
single-marker rule below; `docs/design-guide.md` now records the behavior. Focused UI and
source-layout tests pass 19/19 across three suites, and formatting/diff checks pass. Graphify is
at 7,557 nodes / 11,988 edges. CocoIndex remains unavailable because its environment cannot
import `mcp.server.fastmcp`. Packaged acceptance is 6/25; shipped/default Rust UI remains 0/25.
Screenshots were inspected but not retained. The complete native-feature/test-fixtures all-target
suite passes 797 tests, with 14 ignored and 0 failures across 263 suites (220.67s). A Rust-only
source gate now rejects JS/TS, Python, HTML/CSS, and shell code under the application/build roots;
its first run found and removed 20 ignored `.pyc` caches, and the synthetic/source-layout suite
passes 2/2. The focused UI/docs/source-layout batch passes 27 tests across 4 suites; all-target
Clippy, rustfmt and diff pass. Windows UI,
IME/accessibility, retained current screenshots, the other 19 packaged workflows, and full parity
remain open. Clarification: Rust UI source coverage is 25/25 and Cargo enables it by default in this
branch; packaged end-to-end acceptance is 6/25. Stable v0.1.32 targets `main` and draft PR #22 is
unmerged, so public shipment of the Rust UI is still pending. The 6/25 count measures accepted
workflows, not UI conversion progress. Full visual/workflow parity remains open for 19 workflows.

## Interim selected-step heading — issue #9 / R-17 / DD-137 (superseded)

The page eyebrow was changed to follow the selected stage, and an interim implementation removed the active card's repeated marker. A later same-size comparison with the checked-in v0.1.24-labeled capture found the marker in both positions, but screenshot provenance identifies that capture as an earlier Rust build; this did not prove published-WebView parity. The marker has been restored in the current package; see “Active Workspace card step markers” below. The earlier test count, screenshot audit, and acceptance figures in this entry are historical and do not describe the current package.

## Historical grouped settings checkpoint — 2026-10-21

The grouped settings-dialog audit found and fixed the workspace-sharing popup's left-anchored compact default. Its egui window now centers in the viewport and opens at 480px, matching the local cache dialog; a rendered AccessKit regression failed at x=30 before the change and passes after it. The sharing, MCP, Messenger, cache, settings-journey, and theme/layout suites pass 18 tests; all-target Clippy, rustfmt, public-doc, source-layout, and diff checks pass. Exact package source `97a16bb4ca40070760d6052e5bd095ef6b1c4a3c` built and passed manifest validation, packaged `--help`, and ZIP CRC checks; binary SHA-256 is `2f9402e0f14b9c90083a7c897211e0e7968be08039543178d12b43ac4ea94eaf`. In one isolated test window, share was visually verified centered in both themes and MCP, Messenger, cache, profile registration, role assignments, and empty Review were inspected; no screenshot file was retained. Evidence: `docs/evidence/issue9-97a16bb-macos-package.json`. At that checkpoint exact-package acceptance was 5/25; this is historical and is superseded by the 6/25 checkpoint in `docs/09-completion-audit.md`.

Follow-up on branch head `b204eb5d5d770873db954c4c9606b4e6e7ff014c`: the settings-journey language wait fix passes with the compact-header and workspace-start suites (6 tests / 4 suites); all-target Clippy, rustfmt, source-layout, and diff checks also pass. CI run `39452437163` actually checked merge commit `11588965` with PR head `c5dac47`, so it is stale relative to `b204eb5`; both OS jobs failed only the settings-journey test. The equivalent test now passes locally on `b204eb5`. Full-suite validation remains incomplete: this follow-up run was interrupted at 67 tests / 18 suites, and the earlier run stopped at 666 tests / 213 suites. Exact-head CI, Windows UI, and remaining acceptance checks are still open.

The subsequent all-target run on `b204eb5` completed with one failing target: `native_session`. The test had written a live owner record before its first `SessionLease::acquire`, so the implementation correctly refused that acquisition. The fixture now acquires an empty lease first, writes the simulated live owner while the lease is held, releases the lease, and verifies reuse is refused; its isolated target passes 3/3. After the correction, the complete all-target suite passes **794 tests, 14 ignored, 0 failed across 263 suites** in 258.50 seconds. Formatting, source-layout, public-doc, all-target Clippy, and diff checks pass.

## Historical checkpoint — 2026-10-20

The latest product-code batch reconciled here is `4a114ec6a76141c48ebe8138694398572cae1acd`; documentation/evidence heads are pushed on the issue branch, and PR #22 remains draft. Exact local macOS package passed manifest validation and packaged `--help`; its isolated UI keeps Generate plan disabled when the coordinator is missing. The grouped settings audit covered the empty Review, MCP, workspace sharing, Messenger, and cache preview screens without saving or deleting data. A complete positive-path UI run registered a local Rust CLI, generated/reviewed a plan, ran and tested a task, reviewed/approved the diff, merged, and reached Completed. Follow-up package checks covered phase analysis, candidate evidence, scrollable update notes, and dark-theme readability. Evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`, `docs/evidence/issue9-4a114ec-settings-batch.json`, `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`, `docs/evidence/issue9-4a114ec-review-updates-batch.json`, and `docs/evidence/issue9-4a114ec-dark-theme-audit.json`. The 21 native UI suites plus docs/layout checks pass 122 tests across 23 suites; the Review/Updates five-suite batch passed 23 tests; the theme/layout five-suite batch passed 15. Packaged UI acceptance is 4/25. Component checks and source-level tests do not count as UI workflows. The full all-target run did not finish. Platform runs `39429494121` and `39431587615` were cancelled on both OSes during native build/test; no Actions result has been reported for current docs head `459ba84`.

The exact package was also opened once in a disposable workspace for a Korean/light component check. The MCP editor, Jira/Confluence reference dialog, workspace sharing, Messenger, and cache all exposed an in-content `닫기` control and returned to the underlying Workspace after a click. Evidence: `docs/evidence/issue9-a8be85f-packaged-dialog-ui.json`. This is not an end-to-end workflow and does not change 2/25 acceptance; no screenshot file was retained.
Graphify was refreshed on this source (7,422 nodes / 12,771 edges, no clustering). CocoIndex Code
is installed but cannot start because its environment lacks `mcp.server.fastmcp`.

Verified native UI source coverage is 25/25, but exact packaged native UI acceptance is **2/25**;
the Rust replacement is not yet released. The current design guide command now names the actual
test targets: `interface_theme`, `interface_language`, `native_ui_appearance`,
`native_ui_bilingual`, `native_ui_room_language`, `native_room_layout`, and
`native_room_task_selection`. The corrected offline command passes 16 tests across seven suites.
The exact packaged macOS UI rendered Workspace in light and dark themes during visual review,
with the original light preference restored. No JavaScript-baseline screenshot was captured, so
the visual review does not establish parity or advance the 2/25 acceptance count. Platform run
`39366430380` for PR head `26ca4a9` is historical; exact-head validation for the current PR remains open.

The exact macOS package from code source `7419557` was subsequently inspected in one isolated
window. Workspace, Settings/MCP, agent registration, and Agent Roles screens rendered; a delayed
Rust mock coordinator exercised planning progress and the confirmed stop-and-delete path, which
returned to Ready and a fresh workspace. The package UI interaction evidence is
`docs/evidence/issue9-7419557-packaged-ui-audit.json`. This does not add a complete workflow to
the conservative 2/25 count; Windows, provider-authenticated flows, public screenshots,
accessibility/IME, and resource verification remain open.

## Native Settings dialog close accessibility — issue #9 / R-17 / DD-160

A grouped native-probe audit covered all six native custom dialogs. The profile editor already provides Cancel and cache provides Close; localized in-content Close actions now cover the MCP server editor, MCP source-reference editor, workspace sharing, and Messenger settings. Sharing combines its inner close request with the window-title state so either path dismisses the dialog. Pointer-event regressions exercise the sequential Settings journey and the Jira reference flow. Full local validation passes 768 tests (14 ignored across 260 suites), Clippy, formatting, and source-layout/public-doc checks. This does not establish native macOS VoiceOver behavior or increase packaged acceptance beyond 2/25.

## Batch native UI parity regressions — issue #9 / `cb49fa1`

One audit found four related defects and fixed them together: a running Agent Room lacked a route back to Workspace; returning discarded the selected Review stage; an active plan had no stop-and-delete path; and the profile editor clipped at a 640px viewport. The new route preserves the running session and selected stage. Stop-and-delete requires confirmation and waits for the planning job and metadata work to become idle before archive. The profile editor fits a narrow window and stacks its fields. The all-target native-feature suite passes 759 tests (14 ignored, 0 failed across 258 suites); 25 focused tests across 10 suites, source-layout/public-docs, all-target Clippy, rustfmt, and diff checks pass. Commits `cb49fa1` and `7419557` are pushed. This component batch does not increase packaged UI acceptance beyond 2/25.

## Exact-source macOS package — issue #9 / `7419557`

The optimized macOS package was built from exact issue head `7419557168ddc3ffe76c446cf292152cc9fc1932`, passed `validate-package`, packaged `--help`, archive checksum/CRC, and all four shared/worktree × Git/folder mock smoke cases. It records 214 dependency notices and bundled SQLite. Binary SHA-256: `6988a2dd852b23665be6f43f4d2d7e20092dbef5e20db49574cd6d1982aa6836`; archive SHA-256: `610f3987658acd5e8c319d827539ab0fe2745d45ee28d2dced754c50365becfb`. Evidence: `docs/evidence/issue9-7419557-macos-package.json` and `docs/evidence/issue9-7419557-macos-smoke.json`. No GUI was opened; exact packaged UI, Windows package, and CI remain unverified. Acceptance stays 2/25.

## Finalization progress parity — GitHub #50 / issue #9

The default JavaScript implementation was merged to `main` in PR #51. The Rust-native issue #9 run board now carries the same Verify → Merge → Workspace progression as an additional parity change: it derives active merge state from the existing job snapshot, keeps the apply explanation and busy action in the primary status banner, and disables duplicate merge requests. Regression coverage checks the rendered stages and Verify → Merge → Workspace transitions. Commit `b81caa5` plus the test-gate correction passed all-target serialized tests, 17 focused UI tests, the native feature-gate guard, Clippy, formatting, diff checks, and the changed-file 300-line check. The first full run caught a missing `native-ui-probe` gate on the new integration test; the corrected rerun passed.

This component-level Rust parity is not itself a packaged workflow acceptance; see the current
checkpoint above for the verified packaged workflow count.

## Exact-source macOS package — issue #9 / `ca17284`

The optimized 17,627,872-byte native executable from source `ca17284b68e531bfb16476981d80844b53bbaab` has SHA-256 `0652485f9bbf9d2964e50b9b7585194889898f94acb2f21b7d235f8d31aed85a`. The exact package contains the bilingual Markdown/HTML manuals, README, design guide, MCP reference, assets, SBOM, and 214 reviewed dependency licenses. Package smoke exited 0 for shared and worktree modes on both Git repositories and plain folders; every case ran three parallel mock tasks and tests and verified both refusal before approval and successful merge after approval. Archive SHA-256: `786e83c54de8194bbda019f96aaac62d70a5583498638ad3a0b535254d5de933`. The evidence summary is tracked at `docs/evidence/ca17284-macos-package.json`.

This is package/CLI evidence on macOS, not visual acceptance of the desktop UI or Windows package. Cross-platform native UI acceptance remains `0/25`.

Status: active migration. Published v0.1.24 still uses WebView; the issue branch defaults to the Rust eframe UI and has removed project-owned JavaScript/Python source and test files. Exact macOS/Windows package acceptance and final native hardware checks remain open. The serial full Rust suite passes: 719 passed, 14 ignored, across 247 test targets (2026-10-20).

## Exact-source macOS package — issue #9 / `fd177ea` (2026-10-20)

The optimized Rust package built from exact source `fd177ea145326407d485d7f18ccd1d916f1fc00e`. `package-release` completed all four isolated Git/folder × shared/worktree smoke cases; each used three parallel mock tasks and tests and verified that unapproved merge is refused while approved merge succeeds. `validate-package` confirms 214 dependency license records, binary SHA-256 `6bd00e6f520e0172a0b1f912c15fe558695d4a3a6c2d80477395b9da7cae6c88`, and archive SHA-256 `b4d17e6b8ecb152e4056aa2e39d1cabe4e9f7880d8398435c3bb2d132578e6e5`. One isolated package window showed the native empty Workspace and was closed. This is local macOS packaging evidence, not a release or full UI acceptance; #9 remains 0/25 and Windows acceptance remains open. The serial all-target suite subsequently passed 719 tests with 14 ignored across 247 targets.

## Reviewing-phase heading — issue #9

In v0.1.30, a phase proposal changes the phase panel heading from “Next” to “Reviewing phase n/N”; completed stale proposals are ignored and the next phase remains available. Rust continued to label a phase as next even when its proposal was under review. A rendered regression failed on that contradiction. Native Rust now uses the localized reviewing heading and marks any non-completed proposed phase as under review. Two goal-list tests, nine phase-progress tests, the native approval/start journey, source-layout, formatting, and warning-denied native-feature Clippy pass. Exact packaged UI acceptance remains open; end-to-end acceptance is 0/25.

## Goal-list identity and stage — issue #9

The v0.1.30 goal list identifies each saved goal with its slug and localized stage, shows the goal description, and gives inactive goals a separate Select action. Native Rust previously displayed only the active goal's description and rendered inactive descriptions as buttons. A rendered regression failed on the omitted identity/stage; Rust now shows those fields and preserves the existing revision-bound selection event. The goal-list and nine phase-progress tests, native approval/start journey, review copy, source-layout, formatting, and warning-denied native-feature Clippy pass locally. Exact package layout/accessibility and full workflow acceptance remain open (0/25).

## Preserve warnings when progress has no phases — issue #9 / DD-63

`slug_progress::progress()` can return `phases: []` plus a mismatch when `plan/plan.json` is unreadable. The native Goals & Progress card previously returned before rendering, hiding the integrity warning. A rendered empty-phase regression failed; the card now remains visible when warnings exist and avoids falsely reporting that zero phases are complete. All nine phase-progress tests, the source-layout guard, formatting, and warning-denied native-feature Clippy pass. Exact package appearance and accessibility remain open; acceptance remains 0/25.

## Goal-progress mismatch warnings — issue #9 / DD-63

Legacy `renderGoals()` displays each mismatch derived from phase/task files below the goal. Native Rust omitted these integrity warnings. A rendered regression failed on the missing text; the native card now renders every mismatch in the warning color with a visible marker. All eight phase-progress tests, review-copy, source-layout, formatting, and warning-denied native-feature Clippy pass locally. Exact packaged macOS/Windows appearance and accessibility checks remain open; workflow acceptance stays 0/25.

## Coordinator analysis disclosure — issue #9

The shipped v0.1.30 phase panel shows the coordinator's whole-goal analysis in a collapsed disclosure before the phase list. Rust omitted it. A rendered regression failed on the missing localized disclosure; native Rust now reads the saved plan analysis and reveals it through the existing localized collapsible control. All seven phase-progress tests, the native approval/start journey, three parallel mock tasks through shared/worktree approval and merge, review-copy/source-layout checks, formatting, and warning-denied native-feature Clippy pass locally. Exact package and keyboard/accessibility checks remain open; end-to-end acceptance remains 0/25.

## Workspace phase briefs and task details — issue #9

The v0.1.30 Workspace expands the current or upcoming phase by default and exposes its brief plus planned task ID, first-line instruction summary, assigned agent, and task state. The Rust Goals & Progress card previously showed phase names and aggregate counts only. Rendered regressions failed for the missing brief/task details and verified that a phase under review keeps the existing editing notice rather than duplicating its editable task list. Six phase-progress tests, the native approval/start UI journey, and the three-task shared/worktree approval-and-merge workflow pass locally. The UI journey test now follows the rendered accessible task headings and summaries and scrolls to offscreen controls. Exact packaged macOS/Windows and keyboard/screen-reader checks remain open; this does not advance end-to-end acceptance from 0/25.

## Review Plan task-role badges — R-47 / UC-39 / UC-106

The released Review Plan task summary includes a role badge when a task has a role, lays out its two-digit task ID/role above the instruction summary, and labels its removal control `Remove task`. The native card omitted the role, combined the ID and summary into one line, and shortened the action to `Remove`. Rendered regressions caught each difference; Rust now restores the two-line hierarchy, localized role/remove labels (including the special `knowledge_assetization` key), raw-role fallback, and trailing ellipsis after the shipped 160-character limit. AccessKit confirms the role badge remains inside a 480px review viewport. Exact packaged macOS/Windows comparison and the 0/25 workflow acceptance remain open.

## Review Plan snapshot-base cue — R-21 / UC-15

The shipped v0.1.30 Review Plan displays the first 12 characters of the plan's captured repository base and prompts the operator to inspect executables, permission modes, scope, and tests before starting. Native Rust previously showed this only after opening exact execution consent. An AccessKit regression failed because the cue was missing from Review Plan; the native view now uses the existing localized warning and snapshot base before the task list. The consent dialog still presents its full repository-base evidence. Five Review Plan unit tests and ten focused task-card/edit/review-copy/source-layout tests pass, along with warning-denied Clippy and formatting. Packaged macOS/Windows confirmation remains open; end-to-end acceptance stays 0/25.

## Current acceptance checkpoint — 2026-10-20

The native UI source-coverage inventory remains **25/25 workflows implemented (100%)**, **0 partial**, and **0 not started**. Packaged operator acceptance is **4/25 (16%)**. The v0.1.31 macOS package completed a three-task parallel shared-folder plan with overlapping task intervals, three passing tests, human review and candidate approval, then app-level integration to Completed. Evidence: [`issue9-4a114ec-parallel-ui-plan-to-merge.json`](../../docs/evidence/issue9-4a114ec-parallel-ui-plan-to-merge.json). This rechecks the existing parallel workflow rather than adding a fifth acceptance item. The folder was not a Git repository, so it does not establish a Git merge. An enabled approval action and the Completed panel were readable in dark mode; Activity, Raw logs, Lessons, and read-only team briefing also showed task-specific results and final state. The mock CLI has no provider-native session or model IDs, so those remain unverified. Light mode was restored. The current product source also passes 770 tests with 14 ignored / 0 failed across 260 native-feature suites, plus all-target Clippy with warnings denied. Windows UI, authenticated provider CLIs, accessibility/IME, screenshots, and the remaining 21 workflows remain open. The shipped Rust replacement remains 0/25 until a release ships. Keep implementation coverage, verified workflow acceptance, and shipped replacement as separate metrics.

## Historical acceptance checkpoint — 2026-10-19

The native UI source-coverage inventory marks **25/25 workflows implemented (100%)**, **0 partial**, and **0 not started**. This does not establish cross-platform end-to-end parity. Source/control presence does not prove functional parity: verified native end-to-end parity remains **0/25 (0%)**, and the shipped Rust replacement remains **0/25 (0%)**. Published v0.1.24 still uses WebView; exact packaged Windows, full accessibility/IME, and complete cross-platform workflow acceptance remain open. Component tests and package smoke checks do not count as accepted end-to-end workflows. Keep these metrics separate.

## Exact-head platform and package verification — `de95fc5` (2026-10-20)

Run `39271534953` attempt 2 passed every macOS and Windows platform-validation step. Windows release compilation, Rust-rendered bilingual manuals, exact package execution/smoke, and artifact upload also passed; the uploaded Windows package artifact has SHA-256 `0bc32e45402eb7a87aff17a8fa5f21018f42862158e65f7256d35b1a4a50696d`. The macOS native-package job was cancelled while queued without starting any step, so no exact-head macOS release-package result is claimed. The platform timeout correction is verified; this evidence does not advance native UI acceptance from 0/25. Exact current app screenshots, full native UI workflow comparison, and live IME/accessibility checks remain open.

## Clickable native release-notes link — issue #9 (2026-10-20)

The native Updates panel previously rendered “Release notes” as a button that copied the URL to the clipboard, even though it appeared to be a link. Regressions first failed because there was no AccessKit link node and an arbitrary external origin could be linked. The Rust view now uses egui's native hyperlink only for `https://github.com/neodamul/AgentConductor/releases/` and retains its localized accessible name plus the existing keyboard-focusable, read-only, bounded release-body view. Local verification passed `native_ui_updates` (7/7), `native_ui_update_links` (2/2), `source_layout` (1/1), formatting, diff checks, and focused Clippy-driver checking with warnings denied. The exact guarded Rust source was rebuilt (binary SHA-256 `9582c6cf9396c2b1baf20a4c8c7a794d861a4e78fbdc6e2326904fc95ac5cc4c`) and opened in one isolated macOS app window; Settings → Updates displayed the actual v0.1.30 release notes and trusted accessible link, and the check resolved to v0.1.30. The app and isolated HOME/bundle were closed and removed without opening the browser link or changing workspace data. Exact packaged browser activation, keyboard traversal, and scrolling on both platforms remain unverified; overall acceptance remains 0/25.

## Exact-head Windows validation attempt 2 — 2026-10-19

For run `39265015072`, macOS validation passed every step. Windows build, the source/test layout guard, and all applicable tests passed. The 20-minute job limit then cancelled the opt-in Rust UI compile shortly after it started; formatting and Clippy therefore did not run, and the native-package jobs were skipped. The platform-validation job timeout is raised to 45 minutes for the exact-head rerun. This is CI-duration remediation only and does not change the 0/25 end-to-end UI acceptance count. Local Rust compilation remains unavailable on this host because Homebrew LLVM requests the missing `libz3.5.1.dylib`.

## Rust-only source and layout checkpoint — 2026-10-19

On issue branch `codex/issue-9-native-ui-parity` at `a11848e`, `git ls-files` finds no project-owned Python, JavaScript, or TypeScript source/test files. The historical estimate of 19 Python contract modules is stale. The locked offline `source_layout` integration test passes 1/1 and verifies source/test separation plus the 300-line per-file limit. This is repository hygiene evidence only; it does not advance packaged UI acceptance from 0/25.

## Agent Room goal context — 2026-10-19

The active `report.plan.goal` now appears under the localized `CURRENT GOAL` label above the run-status banner, matching the released Agent Room's information order. AccessKit layout coverage failed before the header was added and passes with the full goal exposed before status. The optimized prototype was not involved: the exact locally built Rust app bundle was replaced in place, then one isolated fixture window showed the goal and its three completed tasks while the candidate remained unapproved and unmerged. No subscription CLI or user workspace was used. Graphify re-extracted the code graph to 6,720 nodes / 11,655 edges. CocoIndex Code remains unavailable because its installed environment lacks `mcp.server.fastmcp`; no dependency was added. This corrects one visible omission; packaged/native workflow acceptance remains 0/25.

## Agent Room integration verification summary — 2026-10-19

Restored the released Agent Room's integration-verification cue below the native session list. The Rust view derives passing count and failed/running/completed status from the existing report test records, and renders only the summary; stdout/stderr paths remain private. An egui render regression failed before the panel existed and passes with the localized summary. Full offline all-target tests and Clippy pass, and the exact rebuilt isolated macOS app's AccessKit tree shows the summary for three passing task tests. Graphify now records 6,736 nodes / 11,677 edges. This is one visible-parity slice, not end-to-end acceptance; issue #9 remains 0/25 and open.

## Agent Room Human Review rail — 2026-10-19

Restore the compact left-rail Human Review section with a Review candidate link, pending tool-request count, and current report status. The link selects the Review tab; it requests evidence only if the cached revision is missing or stale, and never approves or merges. The red-first egui pointer/AccessKit regression passes, together with 22 focused tests, the source-layout guard, all-target Clippy, rustfmt, and a local binary build. The full offline all-target suite also passes after commit `2ec925b`. An isolated instance of that build's actual Agent Room reports the task-test summary and exposes `HUMAN REVIEW`, `Review candidate`, `Tool requests · 0`, and `Waiting approval` in AccessKit. Graphify has 6,747 nodes / 11,693 edges. A screenshot could not be captured from this display, so layout pixels are not visually evidenced. This is a parity slice, not end-to-end acceptance; issue #9 remains open at 0/25 until cross-platform workflows are accepted.

## Native sidebar provider icons — 2026-10-18

The Rust sidebar now renders bounded provider artwork from the existing native icon
snapshot contract. A rendered-widget regression failed before the icon texture path
was added and passes with it. An optimized macOS build showed Claude and Codex icons
with isolated fake CLI profiles; the custom profile retained its monogram. This does
not verify Windows discovery/rendering or alter the 0/25 workflow acceptance count.
The committed executable SHA-256 matches the visually inspected build; package smoke
passed shared and worktree modes against Git and ordinary folders, including approved
merge and rejected unapproved merge. See [runtime icon evidence](runtime-icons.md).

## Startup screen parity checkpoint — 2026-10-18

Compared the pre-migration startup layout at `ce5b58c` with the exact Rust app. The Rust startup flow now retains the same shared goal headline and three-stage tabs, the Workspace settings card, the five-column role table, Generate Plan on Agent roles, and a Review Plan card with an explicit empty-plan state. The header and role-table assertions were added red-first, then passed after the layout was restored. The three stages were inspected at a 1140×772 macOS viewport without changing the saved workspace. The reviewed-plan state, live agent operations, full accessibility/platform acceptance, and remaining operator workflows are still pending; this checkpoint does not change verified workflow progress from 0/25.

## Equal-width Workspace stage tabs — 2026-10-19

Legacy `v0.1.21` uses three equal-width stage tabs across the Workspace content column. A rendered-bounds regression reproduced unequal Rust tab widths at 480px (`142/178/205px`). The Rust header now allocates the content width evenly, wraps long labels inside their cells, and draws a full-width selected underline; localized button names and click handling remain unchanged. The native-header and 300-line source-layout suites pass (4 tests), along with rustfmt and diff checks. Graphify re-extracted the issue branch at 7,021 nodes / 12,186 edges. CocoIndex Code is installed but could not start because its environment lacks `mcp.server.fastmcp`; no dependency was added. Windows/macOS package visuals have not been re-inspected, so this component correction does not advance workflow acceptance from 0/25.

## CI-safe mock start journey wait — 2026-10-19

Platform run `39256513641` showed the mock plan-to-Agent-Room test can take 6.13 seconds on a hosted macOS runner, exceeding its five-second final UI-state wait. Only the post-start transition now has a 15-second deadline; immediate UI-state checks remain at five seconds. The source/test line guard caught the test file growing to 307 lines, so the reusable wait loop was moved to `tests/native_ui_start_journey/ui_wait.rs`. Local start-journey, header, and source-layout suites pass (5 tests). This only stabilizes CI timing; it does not change the app's runtime timeout or advance packaged workflow acceptance from 0/25.

## Shared Workspace page eyebrow — 2026-10-19

Legacy `startup.html` presents `Prepare a goal` above the stage tabs on all three screens, with the selected section's stage eyebrow appearing inside its card. The Rust page header instead repeated `STEP 1/2/3` above the heading while the active card repeated the same label. A bilingual rendered regression failed on English step 1 because the shared page eyebrow was absent. The header now uses the existing localized `startup.prepare_a_goal` entry; four focused header/source-layout tests, focused Clippy, rustfmt, diff checks and the 300-line limit pass. Graphify re-extracted the updated branch at 7,022 nodes / 12,187 edges. The checked-in screenshot has not yet been refreshed from a visually reviewed exact Rust build; workflow acceptance remains 0/25.

## Accessibility probe after restoring the shared Workspace eyebrow — 2026-10-19

Exact-head Platform run `39259634447` confirmed the new rendered header shows the step label once, but the older `native_probe_controls_have_accessible_names` test still expected the duplicated page/card label count of two. Its failure is a stale assertion, not a lost accessible name. The test now expects one rendered step label. macOS build and source-layout checks passed before the stale assertion failed; Windows build was still in progress at diagnosis. Rerun the exact-head platform checks after this test-only correction. The screenshot/manual refresh and end-to-end acceptance remain open at 0/25.

## Exact linked-branch preview — 2026-10-18

Issue #9's development branch was created from the issue's Development section.
After its PR was opened, GitHub removed the branch from the linked-branch list and
shows the PR there instead; an empty `gh issue develop --list 9` is expected at this
stage. The current draft PR is #22 with head `codex/issue-9-native-ui-parity` and
base `main` ([GitHub behavior](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-a-branch-for-an-issue)).

Built source HEAD `0c5b7d9` as a macOS release binary (SHA-256
`78f835c8b6766c3f6367ce7838951aff88c5c8e806131d68a70bfee63db31ef9`). Stopped and
verified exit of the previous preview process before replacing the bundled executable;
the copied executable hash matched, and only one app window was relaunched. At 1140×772,
inspected Workspace, Agent roles, and the empty Review Plan states. The saved workspace
values remained untouched and no generation request was sent. These three screen
observations do not establish all 25 workflows, the generated-plan state, Windows parity,
live IME, or release readiness. Public README/manual screenshots were not replaced.
The same exact build's Settings menu, MCP JSONC editor, messenger form, and cache preview
were also inspected through AccessKit and screenshots. No settings were saved and no
cache was cleared; the preview listed 8 removable candidates (0.0 MiB) and 4 protected or
review-required items. Update details, other agent-profile/quota actions, generated-plan/live
Agent Room states, and platform-specific controls remain unverified.

### Runtime quota refresh — 2026-10-18

On the same packaged binary, opened and cancelled the Claude profile editor without
saving. Codex quota refresh displayed progress immediately and then reported 65%/79%
(5-hour/weekly remaining). Claude refresh also displayed progress immediately, then
reported the CLI-observed weekly 0% and budget-wait status. No generation request was
sent. This verifies asynchronous refresh for these two configured profiles only; it
does not establish quota parity for other providers or release acceptance.

## Transactional registration checkpoint — 2026-10-18

R-48/UC-40/DD-56: sidebar **+ Add** now opens the profile modal directly,
starting with the same custom `aipro` default as shipped v0.1.24.
Keep Add/Edit drafts in the editor and send a separate candidate registry to the
existing revision-bound `save_profiles` worker. Only confirmed persistence updates
the live list and coordinator/role renames. Rejection keeps the editor and refreshes
the authoritative registry/revision for retry; it does not save the draft goal.
The redundant Team registration menu is removed.

Five UI-to-Room tests cover Add, Cancel, save/reopen, stale-revision retry and
coordinator renaming. Three editor boundary tests cover capacity/duplicate IDs,
provider switching and concurrent removal. Actual macOS interaction verified
Cancel, custom-profile saving, the saved notice, first-agent coordinator selection,
and retained unsaved name/goal. Native inspection also found an untranslated
None value and an invisible modal heading; a two-language/two-theme rendered
regression now covers the corrected title and None value. Final native follow-up
and full-suite evidence are recorded in the raw registration audit.
This fixes part of workflow 3; it does not establish full visual or cross-platform
parity, and verified replacement progress remains 0/25.

### QAS-124 — agent-card quota contrast

The compact quota percentage must use the active theme's readable text color.
The sidebar's strong monospace label defaulted to white on the light surface; a
rendered-widget test caught the 1:1 contrast failure. The sidebar now explicitly
uses `ui.visuals().text_color()`. The focused light/dark widget regression passes;
full native workflow parity remains 0/25.

### Exact-source macOS package — 2026-10-18

Commit `2996dc6af777dfe2b511d207e02e83ffdabf81c6` builds a 17,411,584-byte
`aarch64-apple-darwin` binary. Package validation confirms its source commit and
SHA-256; the archive includes the README, both HTML manuals, dependency license
texts, and CycloneDX SBOM for 214 dependencies. The archive SHA-256 is
`c97f3e174336abce9de48e4607800b16c4205c318ae1972998312942915be91d`. Packaged
smoke passes Git and non-Git folder projects in shared and worktree modes, with
three tasks/tests and approval required for merge. This is a local package only;
Windows, final UI acceptance, and release publication remain open.

### Workspace keyboard acceptance — 2026-10-18

On the exact packaged Rust binary (SHA-256 `c4618e60f777de0f96e21c69864fbe34267686a70f771eebf435eb921cc36d7f`), a clean isolated Workspace screen was traversed with Tab through every enabled control. Disabled coordinator and import-preview controls were skipped. Space changed the isolation radio selection; the temporary Git worktree selection was returned to Shared without saving. This covers Workspace keyboard traversal only. A public screenshot could not be saved: macOS `screencapture` returned `could not create image from display`; the CUA screenshot was visually inspected but its bytes were not written to the repository. Public screenshots and the bilingual manuals therefore remain pending.

### Native Workspace visual review — 2026-10-19

The source-build Rust executable at `2ec925b` was opened in one temporary Workspace window. CUA screenshots of the actual window were inspected in Korean light and dark themes at 1140×772, and the AccessKit tree exposed the corresponding fields and controls. The empty Workspace form, step navigation, Jira/Confluence reference buttons, isolation radios, save action, and settings import/export stayed inside the viewport and readable in both themes. This verifies only the empty Workspace surface; populated agent cards, profile/MCP dialogs, Agent Room pixels, keyboard/IME, Windows, and end-to-end workflows remain unverified. The CUA screenshots were not persisted as repository assets, so README/manual images remain unchanged and issue #9 remains 0/25. Graphify refreshed to 6,749 nodes / 10,780 edges / 527 communities. CocoIndex CLI is unavailable in this host.

## Historical source checkpoint — 2026-10-17

- The 25 operator workflows have native source-build implementations. This is prototype parity, not release conversion; v0.1.24 remains unchanged.
- The 300-line source-layout guard passes across Rust sources and tests. Legacy browser assets and obsolete JavaScript/Python verification were removed; subprocess fixtures are Rust.
- Local verification: 596 tests passed, 14 ignored, 0 failed; all-target check and Clippy with warnings denied pass; rustfmt and diff checks pass.
- The optimized macOS executable builds and its `--help` CLI smoke exits successfully without opening a desktop window.
- Not verified: exact macOS/Windows release packages, a Windows Rust target on this host, final screenshot review, Korean IME, accessibility on real windows, and idle resource measurements. Keep issue #9 open until those checks pass.

## Architecture boundary

Keep Room, Startup, Controller, validation, approval decisions, and persisted state in Rust. A native desktop view calls the existing Room::dispatch boundary through a bounded background request worker; process and filesystem waits stay off the UI frame thread. Preserve separate Workspace and Agent Room windows and the existing user-visible workflows.

Cargo enables `native-ui-probe` by default, so the actual `ui-new` and `ui` commands open the shared eframe launcher in a source build. The WebView shell and project-owned browser assets have been removed from the issue branch. The published v0.1.24 package is unchanged. Do not publish the new UI until exact cross-platform package acceptance, accessibility, keyboard/IME behavior, visual layout, idle resources, window lifecycle, and dependency-license checks pass.

For GitHub #50, the default WebView behavior was merged to `main` in PR #51. PR #55 carries the same finalization feedback into the optional Rust UI as supplemental parity; do not report the issue as Rust-only or treat the Rust port as the shipped default.

## Production launch-path pilot

The `ui-new` and `ui` commands still identify their `Workspace` or `AgentRoom` surface and pass the existing `Room` into one shared eframe launcher. Cargo's default feature now selects this path without creating another controller. A local macOS build launched successfully. Exact macOS/Windows package UI acceptance, screenshot review, Korean IME, accessibility, idle resource comparison, and license/resource review are still pending.

## UI conversion denominator

Measure native UI development against the 25 end-to-end operator workflows below, drawn from the published Workspace, Agent Room, settings, and help surfaces. Each workflow counts once and is accepted only after its native controls and complete result are verified against the released behavior, design guide, and platform requirements. Partial and unverified workflows receive no credit. Current accepted parity is 0/25. The table is a historical source implementation inventory; its entries do not establish parity. Published release conversion also remains 0/25.

| # | Operator workflow | Historical source inventory (acceptance pending) |
|---:|---|---|
| 1 | Save workspace name, goal, repository, and isolation mode | Complete |
| 2 | Browse for a repository folder | Complete |
| 3 | Register, edit, and remove agent profiles | Partial on audit — transactional Add/Edit save corrected; complete registry/UI/platform acceptance pending |
| 4 | Assign roles and choose a coordinator | Complete |
| 5 | Discover models and select role models | Complete |
| 6 | Generate a plan and inspect live progress/failure logs | Complete |
| 7 | Edit a plan or write task instructions manually | Complete |
| 8 | Review consent and start execution | Complete |
| 9 | Create, open, switch, and delete workspaces | Complete |
| 10 | Import and export workspace settings | Complete |
| 11 | Change language and theme | Complete — language and theme persist; Workspace, Agent Room, status, quota roster and accessibility labels use English/Korean catalogs |
| 12 | Configure MCP servers and import referenced sources | Complete |
| 13 | Configure and test messenger delivery | Complete |
| 14 | Preview and clear local cache | Complete |
| 15 | Open manuals, installation guidance, and release updates | Complete |
| 16 | Inspect run/task status and agent activity lights | Complete |
| 17 | Inspect and refresh per-agent quota | Complete |
| 18 | Send task feedback and reassign work | Complete |
| 19 | Select a task session and model | Complete |
| 20 | Schedule work after an availability restriction | Complete |
| 21 | Inspect task activity, logs, and lessons | Complete |
| 22 | Read a factual team briefing | Complete |
| 23 | Allow, deny, or refresh a tool request | Complete |
| 24 | Inspect diff/tests and approve, reject, or merge a candidate | Complete |
| 25 | Stop, resume, or recover a run | Complete |

Update this table after each completed parity slice; do not report the prototype percentage as release conversion.

## Agent card ordering correction — R-102 / UC-92 / DD-110

Evidence: [native UI ordering audit](../raw/2026-10-18-native-ui-ordering-audit.md).

The old UI exposed drag handles and accessible move arrows, but the native rail omitted both. The native implementation now emits the same bounded move action from either arrow clicks or a typed egui drag payload. It moves the existing profile without reconstructing it and uses the existing revision-bound `save_profiles` request on the background worker. Coordinator, roles, fallback references, private environment and observed catalogs remain attached to their existing profiles. The first Up and last Down controls, and all reorder controls during profile edits/pending work, are disabled.

`tests/native_ui_agent_order.rs` reproduces the missing action in Red and now covers English/Korean boundary labels, actual arrow/drag pointer events, disabled controls, saved order after reopening, reference/secret retention, invalid moves, and stale-revision refusal. Local widget and dispatcher evidence is not native Windows or complete operator acceptance. Graphify refreshed the local AST graph to 6,247 nodes/10,839 edges; CocoIndex could not start because its installed environment lacks `mcp.server.fastmcp`.

Native-window inspection then caught defects that synthetic checks had missed: overflowing framed tabs, 0px input strokes with the expected color, a missing font icon, and an ineffective label-based drag handle. Dedicated egui drag sources, vector icons, real stroke widths and bounded/wrapping tabs now pass 20 focused regressions. In an isolated macOS window, actual dragging and process restart retained gamma/alpha/beta and coordinator beta. This evidence is limited to that surface; the full replacement remains unaccepted.

The audit also exposed a shared persistence defect: comparing serialized profile arrays treated presentation order as changed execution settings and discarded the reviewed proposal. The execution view now sorts its clone by agent ID, leaving the stored display order intact. `tests/model_catalogs.rs` first reproduced `ready` becoming `null`; after the fix it verifies the same proposal digest after reorder/reopen and invalidation after a real argv change. The six affected persistence/consent test targets pass 24 tests. Registration's Add → editor → immediate-save path remains a separate outstanding correction under R-48/UC-40.

## Verified prototype progress

- The native Review screen now edits bounded task plans (1–12): task ID, registered agent, observed session model, instructions, owned paths, dependencies, and JSON argv test command. Assignment/model choices come from registered agent profiles. The editor refuses generation/start while drafts are dirty, validates the complete typed plan before saving, binds edits to the visible revision and plan digest, and requires review of the newly digested plan before execution consent. The Rust dispatcher rechecks the submitted digest against the persisted `review.json` before replacing it. Local verification: 54 tests across six native UI suites, all-target feature Clippy with warnings denied, source layout, rustfmt, and diff checks pass; the largest changed file is 496 lines. Prototype parity is 13/25 (52%), one partial and 11 not started; release conversion remains 0/25.
- After exact-plan consent, the native Review surface now shows “Starting agent execution…” while the start request and report are pending. It polls asynchronously and opens Agent Room only when `run_available` refers to the exact run path returned by the approved start; an older run's existing report cannot trigger the transition. Agent Room receives its snapshot through the existing `Room::dispatch` boundary. A real dispatcher integration test checks the approved run path, open-run route, and resulting report snapshot. Local validation passed 57 native UI tests, 11 compatibility tests, feature Clippy (`-D warnings`), source layout, formatting, and diff checks. This remains an opt-in prototype; GitHub bug #16 stays open until the production Rust UI replaces the shipped WebView.
- The native plan-review panel now uses the saved English/Korean catalogs for its heading, connection state, coordinator activity, elapsed/output details, fallback error, logs, generation action, and save guidance. Agent-provided error text, coordinator IDs, and log paths remain untouched. TDD catalog/interpolation tests pass, as do 202 tests across the native-feature all-target run (4 ignored) with the separate source-layout guard excluded, all-target Clippy, formatting, and diff checks. The guard now reports 35 files above the 300-line limit. Workflow 11 remains partial, prototype parity remains 24/25, and shipped/default conversion remains 0/25.
- Workspace navigation now localizes its saved-workspace list, create/open/archive actions, confirmation prompts, and dirty-workspace loss warnings. The new workspace name field has a localized accessible name; archive wording continues to state that project source files are preserved. The former 498-line navigation module is split into the request/state module and bounded view, confirmation, context, and helper modules. Fourteen focused navigation/i18n tests and native-feature all-target Clippy pass; the strict guard now reports 35 remaining oversized files. Workflow 11 and parity remain unchanged at 24/25 prototype and 0/25 release/default conversion.
- When the task-plan draft differs from the reviewed plan, the native Review surface now shows “Plan changes need review” and an accent-filled “Save changed plan and review again” button. The next step is clear, and the save action remains disabled when the draft is unchanged or a save is pending. Its AccessKit regression verifies both dirty-state labels. This improves prototype feedback only; GitHub bug #15 remains open until this behavior is in the shipped UI. The 25-workflow parity count is unchanged.
- During native plan generation, Review now displays an accessible coordinator activity light using the same pulsing running-state color as Agent Room. It only appears while the plan job is running; the existing 500 ms repaint cadence animates it. A regression verifies the indicator color changes over time and disappears for completed jobs. This remains prototype work; GitHub bug #11 stays open until shipped UI behavior is verified.
- A newly pending tool request now opens an accessible native modal with the tool, task, reason, and exact input. Allow/deny remain two-step, revision-bound decisions through the existing Rust dispatcher. “Decide later” suppresses only that request; any other pending request can then surface, while all receipts stay in the inline list. Regressions cover the popup and per-request deferral; five live Claude callback integration tests pass. GitHub bug #14 remains open until the default shipped UI has the Rust implementation. This refines workflow 23 without changing the 25-workflow completion count.

- ec1339c adds asynchronous repository-folder selection to the Workspace probe. Run 38799761260 passed macOS and Windows validation and exact release-package smoke.
- 8683862 adds a Shared-folder default with optional Git worktree isolation. Run 38804235226 passed macOS and Windows validation and exact release-package smoke.
- d39e173 adds profile registration from the existing Rust templates, collision-free profile IDs (up to the existing 16-profile limit), and coordinator selection. The focused nine-test suite, feature Clippy, and formatting pass locally. Exact-source CI run 38808098099 passed after a failed-job-only retry: macOS and Windows validation, native UI probe, formatting/Clippy, bilingual manual rendering, and both exact release-package smoke jobs passed. The initial Windows UI Automation `NO_HEADING` was transient and did not reproduce.
- A follow-up resource slice removes a full workspace snapshot clone from every egui frame by borrowing the snapshot alongside the editable Team fields. The existing nine tests, feature Clippy, and formatting pass locally. Exact-source run 38812385807 passed macOS and Windows validation, optional probe, formatting/Clippy, bilingual manual rendering, and exact release-package smoke.
- The profile and coordinator save uses the existing Rust dispatcher, which retains profile validation and carries private environment values from stored profiles.

### Workspace planning and role assignment

The opt-in Rust Workspace probe now presents the numbered Workspace settings, Agent roles, and Review the plan steps. The Team step edits coordinator and registered profiles, assigns up to five allowed agents to each existing role, and pins an optional model only from that agent's observed catalog. Saving uses the existing workspace dispatcher and RoleRules validation. The Review step sends a revision-bound plan request asynchronously, refreshes running coordinator progress and output size, and renders the resulting proposal or failure log. Local verification passed 26 focused native UI tests, the 500-line source-layout test, feature Clippy with warnings denied, formatting, and diff checks; the largest changed Rust test file is 492 lines.

This remains an opt-in eframe probe. The production application still uses its WebView UI; this work does not establish production workflow parity, platform UI/IME acceptance, or resource measurements.

### Per-profile runtime preference editor

The opt-in native Team screen can edit a saved agent's executable, JSON argument vector, stdin/file input mode, session cap (1–3), and quota-refresh cadence (1–60 minutes). The executable picker uses the existing asynchronous operating-system picker worker; selecting a path does not launch a CLI. Save sends only the profile registry through the existing revision-bound `save_profiles` operation, preserving unrelated unsaved Workspace drafts and retaining backend validation, derived permission behavior, private environment carry-over, and reviewed proposals when only quota cadence changes. A changed executable clears its observed model catalog so stale models cannot be selected; the existing execution-settings comparison invalidates a reviewed proposal and requires review. The accessible editor is covered by a UI tree regression, a real Room persistence regression, and a regression for executable-change catalog invalidation. Local verification passed 33 focused native UI tests, 2 quota-preference tests, source-layout, feature Clippy with warnings denied, formatting, diff checks, and the 500-line source bound (largest touched file: `workspace.rs`, 377 lines). This remains opt-in and does not complete the shipped UI migration.

### Native provider switching

The native profile editor now offers the observed built-in templates plus the existing custom `aipro` profile shape. Switching providers replaces the profile with the selected template, assigns a collision-free provider-based ID, clears the prior model catalog, and asynchronously resolves the suggested executable through the existing Rust Room dispatch on the bounded worker (no CLI process is launched). The profile-only save sends the old→new ID map; backend `save_profiles` rewrites stored coordinator, role and fallback references, while the native draft rewrites its in-memory references so unsaved role edits survive. Tests cover template replacement, suffix generation, exact rename payloads, local coordinator/role/fallback migration, stale asynchronous path results and the existing backend provider-switch persistence suite. Local verification passed 36 native UI tests, 5 provider-switch tests, source-layout, feature Clippy with warnings denied, formatting, diff checks, and the 500-line source bound (largest touched file: `profile_tools.rs`, 388 lines). The flow remains in the opt-in probe.

### Native agent profile removal

The opt-in Team screen now exposes an accessible trash action with an explicit confirmation step. Removal updates the in-memory profile draft, clears a deleted coordinator (selecting the first remaining profile), removes deleted IDs from role and fallback assignments, cancels an editor targeting that profile, and submits the remaining registry through the existing asynchronous `save_profiles` dispatcher. Removal is disabled while profile editing or that agent's quota/catalog work is active. Local verification passed 12 native profile tests, 5 existing provider-switch persistence/reference tests, default all-target check, feature all-target Clippy with warnings denied, changed-file formatting, diff checks and the 500-line count. CocoIndex indexed 525 files / 5,549 chunks; scoped Graphify extraction contains 714 nodes / 1,458 edges. This completes prototype workflow #3: prototype parity is 15/25 (60%), with 0 partial and 10 not started; the shipped app/release remains 0/25.

### Native workspace lifecycle

The opt-in Workspace screen now lists registered workspaces and supports named creation, opening/switching, removing an entry from the list, and archival deletion after an exact-name confirmation. Operations use the existing asynchronous Room dispatcher and refresh persisted workspace state after create/switch/archive. Backend ownership and busy-state checks remain authoritative; archival preserves the project source folder. Local verification passed five navigation contract/integration tests, four workspace registry tests, eight release-workflow tests, default all-target check, native-feature all-target Clippy with warnings denied, source-layout, formatting, and the 500-line file bound (largest changed navigation file: 439 lines). Exact-source macOS/Windows validation and native-package checks passed for `7bfad6e` in [Actions run 38844468171](https://github.com/neodamul/AgentConductor/actions/runs/38844468171). This completes prototype workflow #9: prototype parity is 16/25 (64%), with 0 partial and 9 not started; the shipped app/release remains 0/25.

### Per-profile model and quota refresh

The native Team step shows observed model counts, existing per-profile quota badges/tooltips, and accessible model/quota refresh actions. A profile with no observed catalog displays the localized “Models not queried” / “모델 미조회” state instead of suggesting that zero models are available. Claude/Codex profiles with no quota observation show an accessible localized unavailable marker, matching the main agent list rather than leaving a blank gap. For paid profiles, the quota value and refresh icon share a compact one-line group with a five-pixel gap; the icon uses the small button style and the model action no longer separates the pair. Requests carry the current startup revision and use the existing Rust metadata queue, which runs the installed authenticated CLI rather than an API-key client. The view refreshes every second only while metadata work is queued/running and returns to five-second idle refresh; it does not poll or redraw continuously while idle. Metadata replies update quota badges and merge a catalog only into the matching saved profile (same ID, parser, and executable), retaining unsaved/new profile drafts. The issue #9 branch also includes the manual custom-CLI catalog editor described below. This remains branch-level native-probe work and does not count as accepted shipped workflow parity or a release.

### Manual custom-CLI model catalog editor (issue #9)

Custom CLI profiles can enter model IDs manually, one per line, or paste a previously observed `ModelCatalog` JSON object. The flow deliberately does not call `/models`, discover endpoints, or require an API key. Saved entries retain operator provenance and use the existing revision-bound local Rust metadata queue; a successful save refreshes the profile snapshot before closing the editor. Input is bounded to 256 KiB, model IDs to 256 characters, and the catalog to 1,024 unique IDs. Focused catalog, sidebar, persistence, profile, localization, layout, and plan tests pass; native-probe Clippy with warnings denied and formatting pass. Physical native-window visual review, Windows package validation, and exact packaged-app persistence remain open acceptance gates.

### Bilingual profile controls and bounded modules

The native Team and profile editor now localize role/profile labels, model and quota refresh actions, removal confirmation, profile fields, and accessible names in English and Korean. The existing `profile_tools::show` entry point remains an English wrapper, while the production Team view passes the selected language through the new view entry point. Profile removal, metadata queueing, validation, save requests, and the existing Rust dispatcher retain their behavior. Focused verification passes 20 tests across five suites; native-feature all-target Clippy passes with warnings denied. The broader all-target run reported 533 passed and 14 ignored across 158 suites before its RTK runner was interrupted after the summary (exit 130). Changed Rust files stay below 300 lines after extracting profile metadata and pending-status helpers. The repository-wide 300-line guard still fails on 34 other files; prototype parity remains 24/25 and shipped/default conversion remains 0/25. Graphify was refreshed with 6,284 nodes, 10,530 edges, and 478 communities.

### Verification migration: localization contracts

The next Rust verification slice preserves duplicate-ID rejection, English/Korean catalog-key and placeholder parity, explicit source references, localized static markup, and removal of the legacy text-matching helper. Keep the live language-switch and user-content preservation regression until the production WebView UI is migrated; moving its Python wrapper alone does not prove the JavaScript UI has been removed.

At the original DD-160 checkpoint, all 5 Rust i18n contract tests passed after removal of the Python catalog module; its 307-line test file met the then-current 500-line cap. R-158 later changed the ceiling to 300 lines, and the current source-layout regression caught the oversized test. Its independent legacy-content assertion now lives in `tests/i18n_legacy_compatibility.rs`: the contract file is 294 lines and the new test file is 14. All five tests pass, while the repository-wide 300-line gate still reports 51 other files. The earlier 515-test and 38 Python-test results belong to that historical checkpoint, not current verification.

Exact-source CI run 38817585058 passed macOS and exposed a Windows timeout in the independent `mcp_remote` SSE fixture: its 3-second per-request limit expired before `discover` completed. The local focused regression passes. Increase only that fixture's deadline while keeping the server stream open until DELETE, so a client waiting for EOF still fails the test.

The prototype covers Workspace identity, repository path, goal, isolation mode, Team registration/coordinator selection, role-agent/model assignment, plan generation/progress/review, plus the separate Agent Room probe slices below. It does not replace the shipped Workspace setup, settings, MCP, approvals, logs, diffs, or production workflow controls.

### Agent Room task-status rendering

`src/native_ui_probe/render.rs` reads the real `Controller::snapshot` envelope (`report.state`, `report.plan`, and `report.tasks`) as well as the startup snapshot (`run_state`, `proposal.plan`) and top-level fixture form. A regression using the real envelope first failed because the renderer only read top-level fields. The view renders the goal/state and each planned task's assigned agent and current status. Status colors are Rust-side: running alternates between two green levels on 500 ms repaint boundaries, waiting approval and blocked are amber, failed is red, completed is green, and pending/unknown are gray. Repaint is scheduled only while a task is running, so an idle view does not continuously redraw. The earlier pulse test caught that a sine wave could repeat at repaint boundaries; half-second color alternation now passes. The native UI suites pass 12 tests; all-target compile, source-layout, focused feature Clippy, rustfmt and diff checks pass. Graphify extraction of the touched renderer/test produced 10 nodes and 13 edges. CocoIndex was unavailable as a CLI or enabled tool, and no project dependency was added.

This slice fixes the snapshot projection only. The probe is still opt-in and currently launches the Workspace path; it has not replaced or been connected to the shipped Agent Room window. Production integration, workflow/accessibility parity and cross-platform UI acceptance remain open.

### Real Agent Room probe path

The existing opt-in binary also accepts `--room <run-dir>`. It opens the exact run with `Room::open`, then `Probe::connect_room` consumes `Controller::snapshot` through the existing bounded worker. The Room surface hides Workspace edit controls and exposes an accessible refresh action. Snapshot refresh is asynchronous: one second for a running job/task and five seconds while idle, matching the shipped Room poll cadence; completed worker replies trigger repaint. This probe path does not call the WebView launcher or construct a second Room/controller; `Room::open` retains its standard recovery/auto-resume policy. Locally, an integration regression opens a real SQLite-backed completed run, checks the actual Room snapshot envelope and verifies the Room-only accessibility controls. The 15 native UI tests, optional probe binary check, default all-target compile, source-layout, feature Clippy and rustfmt pass. This does not yet create the app's production native window or establish macOS/Windows UI/resource acceptance.

Run the opt-in probe against an existing run with `cargo run --features native-ui-probe --bin agentconductor-native-ui-probe -- --room <run-dir>`; the default invocation still opens the Workspace probe.

### Confirmed stop action

The Room probe exposes Stop only while the run is running, ready, waiting for availability/tool approval, or already stopped. The operator must confirm and provide a nonempty reason no longer than 16 KiB. The worker sends the current snapshot revision and stop action through `Room::dispatch`; the Rust coordinator records/processes the request. The UI refreshes the snapshot after acceptance and shows the coordinator message, so acceptance is not represented as an instantaneous process kill. If a refresh shows the run has become terminal, the pending confirmation is cleared. A real SQLite-backed run regression verifies `waiting_availability` transitions to `stopped`; UI contracts cover accessible confirmation controls, reason input, terminal-state hiding/clearing, and empty/oversize reason refusal. Seventeen focused native UI tests, default all-target compile, source-layout, focused Clippy, formatting and diff checks pass locally. Graphify's isolated probe graph now contains 85 nodes and 179 edges. The change remains opt-in; full production UI parity and native platform acceptance remain open.

### Read-only review evidence

The native Room probe can load review evidence on demand through the existing Rust `Controller::evidence` dispatch. It uses the bounded background worker, shows the candidate commit, per-task exit/timeout status, captured test output, truncation notice and exact diff, and rejects evidence whose revision differs from the current snapshot. A later refreshed snapshot clears cached evidence when the revision changes. UI rendering and stale-revision contract tests pass; the existing real `Room::dispatch` diff-review test now also verifies candidate and test output. Local validation passed the default all-target suite (333 passed, 14 ignored), 21 focused native Room/UI/diff tests, feature Clippy, formatting and source-layout. The action is still part of the opt-in probe; production UI migration and native platform acceptance are not complete.

### Candidate approval, rejection, and merge

The native Agent Room now exposes explicit candidate decisions. Approve and merge are enabled only after current-revision evidence has been loaded; each action requires a second confirmation, and rejection asks for a nonempty reason capped at 16 KiB. Requests include the exact candidate commit and current run revision. A state/revision change clears an approval or merge confirmation, and the Rust controller remains authoritative for the approved candidate, diff/test evidence, revision and merge checks. TDD observed both a missing decision request and a stale confirmation that remained actionable after the run revision changed; both regressions now pass. Across five native suites, 41 tests pass with native-feature Clippy, source-layout, formatting and diff checks. Changed source/test files stay within 500 lines (`room.rs` 448; `native_room_render.rs` 392; `native_ui_probe.rs` integration tests 497). Scoped Graphify extraction of the launch/decision slice reports 197 nodes and 383 edges. CocoIndex remains unavailable in this environment. These controls are currently available only in the opt-in native build; the shipped WebView still has additional Room controls, including task reassignment, model/session selection, availability scheduling, tool approvals, activity/log/lesson panels, and team briefing.

### Resume and recover

The native Room now mirrors the shipped Resume state gate and revision-binds the action through the bounded worker. Recover appears only for a running run, requires an explicit confirmation, and is cleared if a fresh snapshot changes the run out of `running`. The Rust recovery implementation still refuses live or unknown recorded processes; the native confirmation does not weaken that guard. UI tests cover the available controls, confirmation state, and state-change cancellation; request tests cover the allowed run states and exact revision. The five native suites now pass 43 tests, feature Clippy, source-layout, formatting, and diff checks. The scoped Graphify graph contains 214 nodes and 408 edges. These remain opt-in pilot screens; full workflow parity and release cutover are still outstanding.

### Human-gated tool requests

The native Room renders the current tool receipt's tool, reason, task, status, and complete bounded input from the Rust snapshot. Pending receipts expose one-time allow, deny, and refresh actions; each requires a second confirmation and sends the exact receipt ID plus current revision through the Rust controller. Approved receipts expose refresh only, matching the shipped WebView. The request builder rejects malformed IDs, missing receipts, ineligible receipts, or incomplete request/context evidence. Active runs can receive a live decision while the worker is paused; idle approvals resume only after the `tool_approve` job succeeds and the refreshed run is ready/stopped. Idle decisions remain disabled while another controller job is active. Stale confirmations are cleared when request status or revision changes. Verification now passes 45 tests across five native suites, all-target feature Clippy, source-layout, formatting and diff checks; largest changed files remain below 500 lines (`room.rs` 461, `room_actions.rs` 334, `native_ui_launch.rs` 235). Scoped Graphify is 220 nodes / 432 edges; CocoIndex is unavailable in the environment. This does not port the Agent Room's other panes, assignment/model/scheduling controls, team briefing, or settings navigation, so the shipped WebView remains the release UI.

#### Exact-package Windows decision check — 2026-10-28

The existing dialog and request-contract tests do not prove that Windows UI Automation can submit a human decision to a live worker. `tests/windows_tool_decision.rs` now runs an isolated mock Claude worker and drives Refresh → Allow and Deny through the native dialog's two-step confirmation. It checks the superseded, consumed, and denied ledger receipts and the corresponding mock file effects. The existing exact-package Windows job now includes this ignored package test. Local native regressions, Clippy, formatting, source layout, and the 300-line guard pass; exact Windows package execution is still required before workflow 23 can receive candidate-package acceptance credit. This test does not change shipped Rust replacement acceptance.

### Read-only team briefing

The native Agent Room now requests the existing Rust `briefing` response through its background worker and renders a collapsible, read-only summary. It displays observed session/task/work state, session/model/attempt, process and activity observations, availability restrictions, attention/next action, and recorded approvals. Timestamps are converted to local time when the platform offset is available, with a UTC fallback. The view does not send commands to agents or modify run state. The TDD regression covers rendered session, work, activity, attention, and approval details; a real completed-run fixture verifies the background worker receives the read-only briefing response through the Agent Room dispatcher. Five native suites pass 47 tests; feature all-target Clippy, source-layout, formatting, and diff checks pass. The scoped Graphify graph contains 232 nodes and 446 edges. CocoIndex is unavailable. The feature remains opt-in and does not complete Agent Room or Workspace parity.

### Reviewed execution consent

Workspace Review now opens explicit consent only when the saved proposal is ready. It shows the goal, repository/base commit, workspace mode, plan digest, and every task's agent/model/executable/arguments/permission/fallback, writable paths, and test command. The operator must separately confirm or cancel; a revision, digest, or phase change clears confirmation. Starting sends the current revision and digest through the existing Rust startup dispatcher, which retains its source-context, permission, repository-cleanliness/base, reviewed-plan and digest checks. Tests cover stale/invalid request rejection, exact consent evidence, and starting a generated plan through the native worker against a real Room. Five native suites pass 50 tests; all-target feature Clippy, source-layout, formatting and diff checks pass. Scoped Graphify: 247 nodes / 471 edges. CocoIndex is unavailable, and the feature remains opt-in.

### Strict workflow parity score

Use the 25-workflow denominator above, not the total system use-case count: many system use cases have no operator UI. The issue branch implements 25/25 Rust UI source workflows, while packaged end-to-end acceptance is 6/25 (24%). Workflow 11 language and theme controls and Workflow 12 MCP discovery/source import have source implementations; Workflow 15 includes the native manual and screenshot viewer, installation guidance, and automatic release updates. Source presence does not prove behavior or visual parity. Stable v0.1.32 on `main` still ships the WebView UI, and PR #22 is unmerged, so the externally shipped Rust replacement remains 0/25. Keep the source-coverage, packaged-acceptance, and public-shipment measures separate; update packaged acceptance only from exact-package workflow evidence.

## Native local-cache maintenance

The native Workspace probe exposes disposable-cache maintenance without a WebView or local service. It requests the existing Rust Startup preview on the background `RequestWorker`, displays the bounded inventory using a virtualized list, and reports the retained protected/uncertain count. Clear is enabled only when the preview contains entries and sends the exact preview digest back to the existing Rust cache handler, which revalidates the inventory before deletion. Reviewed run evidence remains protected. Tests cover the payload contract, empty previews, accessible Workspace entry, preview rendering, and the asynchronous Room-to-Startup preview/clear path against a disposable cache fixture.

## Workspace settings import and export

The native Workspace settings step now exposes import and export through the existing asynchronous request worker. Export stays disabled while workspace fields, role/profile editing, or task instructions have unsaved changes. Import first previews the selected folder, shows the settings summary and backend warnings, defaults local capability claims to unverified, and requires a separate confirmation before applying; the backend retains machine-local secrets and model catalogs. Import refreshes the workspace snapshot after success. Native request-contract and accessible-preview tests pass alongside the existing real Startup import/export tests. This completes workflow 10 for the opt-in prototype; the shipped app and release conversion remain 0/25. Exact-source CI initially caught the older Workspace accessibility contract's static button list; it now includes both accessible sharing actions. Run 38852463148 passed all macOS/Windows validation and exact native-package jobs for `a69132b`, including bilingual manual rendering and execution of both exact release packages. The corrected test file stays under the 500-line limit. Prototype parity remains 17/25 (68%); shipped/default conversion remains 0/25.

### Bilingual import/export and result handling

This UI-only slice extends UC-42 and R-52/DD-59 under DD-160 without changing the preview/apply request contract. TDD first failed on untranslated sharing labels and English-only asynchronous results. The opt-in Workspace now localizes the unsaved-change warning, folder preview, agent/coordinator summary, permission-claim checkbox, import confirmation, and import/export results. Result state stores the outcome and path rather than pre-rendered text, so changing the selected language updates a result already on screen. The short English preview wording is preserved while its accessible name stays descriptive. Verification passes 13 tests across three sharing/workspace/i18n suites plus 13 native-probe accessibility checks; native-feature all-target Clippy, formatting, and diff checks pass. Changed Rust files are below 300 lines; the global source-layout gate reported 34 other oversized files at that checkpoint, reduced to 33 by the following messenger-view split. Workflow 10 remains complete; workflow 11 is still partial because other controls remain untranslated. Prototype parity is 24/25; shipped/default conversion is 0/25.

### 300-line refactor — native Workspace messenger renderer

The failing source-layout gate identified 34 violations before this refactor, including the 339-line native messenger module. The existing messenger regression passed before the move. `State::show` and its rendering helpers now live in `workspace/messenger/view.rs`; `messenger::State::show`, request builders, events, and dispatcher entry points remain unchanged. The messenger suite still passes (2 tests), native-feature all-target Clippy is clean, formatting and diff checks pass, and the two files are 135 and 213 lines. The repository-wide gate still fails on 33 other oversized files. Graphify now contains 6,307 nodes, 10,539 edges, and 478 communities. CocoIndex could not refresh because its installed CLI fails to import `mcp.server.fastmcp`; no new CocoIndex index is claimed. This organization slice does not change prototype parity (24/25) or shipped/default UI conversion (0/25).

### 300-line refactor — native Room task controls

The strict source-layout gate identified `room/task_controls.rs` at 417 lines. Existing feedback, reassignment, model, and scheduling tests passed before extraction. Rendering now lives in `room/task_controls/view.rs`, revision-bound request validation lives in `request.rs`, and the existing `task_controls::show` and `task_controls::request` API paths remain available. All 6 task-control/scheduling tests pass after extraction; native-feature all-target Clippy, formatting, and diff checks pass. The three files are 41, 281, and 104 lines. The global 300-line gate still fails on 31 other files. Graphify now contains 6,318 nodes, 10,542 edges, and 483 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. Prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### 300-line refactor — Workspace settings sharing handlers

The source-layout gate identified the 326-line workspace sharing handler. Existing workspace-sharing, archive, messenger-settings, and MCP integration tests passed before the move. Export, preview, and import/archive staging now live in separate Rust child modules while retaining the `Startup` dispatch methods and their setup-level visibility. All 6 focused tests pass after extraction; native-feature all-target Clippy, formatting, and diff checks pass. The files are 41, 70, 37, and 192 lines. The global 300-line gate still reports 30 other violations. Graphify now contains 6,336 nodes, 10,543 edges, and 480 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. This is a source-organization change only; prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### 300-line refactor — native Agent Room view

The source-layout gate identified the 497-line Agent Room module. Existing render, language, tool-dialog, and candidate-decision tests passed before extraction. The `show` and decision-control renderers now live in `room/view.rs`; callers keep the original `room::show` path and state/event contracts. All 19 focused tests pass after extraction; native-feature all-target Clippy, formatting, and diff checks pass. The facade is 237 lines and the view module is 268 lines. The repository-wide gate still reports 29 other oversized files. Graphify now contains 6,342 nodes, 10,543 edges, and 489 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. This source-only refactor does not change prototype parity (24/25) or shipped/default conversion (0/25).

### 300-line refactor — native Room tool controls

The strict source-layout gate identified the 491-line Room actions module. Existing request-validation, tool-dialog, and render tests passed before extraction. The tool-request view now lives in `room_actions/tool_view.rs`; the `room_actions::tool_controls` path, request validation, action enums, and dispatch remain intact. All 27 focused tests pass after extraction, and native-feature all-target Clippy, formatting, and diff checks pass. The facade is 211 lines and the view is 286 lines. The repository-wide 300-line gate still reports 28 other files. Graphify now contains 6,349 nodes, 10,545 edges, and 484 communities. CocoIndex remains unavailable because its installed CLI cannot import `mcp.server.fastmcp`. Prototype parity remains 24/25 and shipped/default conversion remains 0/25.

## Native MCP registration and source imports — workflow 12 complete in prototype

The shared Settings menu opens a native editor initialized from the workspace's `mcp` and `goal_sources` maps. It accepts the existing OpenCode-compatible JSON/JSONC format, saves revision-bound settings through the Rust worker, and discovers tools only on explicit request. The Workspace goal editor inserts explicit `jira:KEY-123` and `confluence:PAGE_ID` references. Preview requires the saved goal, a configured MCP server, and a coordinator; it uses the existing revision-bound Rust dispatcher and server-generated preview. The bounded review shows each record's source, tool, title, URL, version, update time, retrieval time, hash, summary, and collapsible full content. Jira records remain required and unresolved Jira references block apply; Confluence records can be deselected. Apply accepts only the exact displayed preview ID and selected records, then refreshes the saved snapshot.

Verification passed 32 tests across six native UI and MCP suites, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks. Every changed Rust source/test file remains at or below 300 lines. The full native-feature all-target run passes all targets except the strict `source_layout` guard, which reports 36 files above the 300-line requirement. This completes workflow 12 in the opt-in prototype and raises prototype parity to 24/25; the shipped WebView UI and release packages remain 0/25. Live GUI acceptance was not performed. CocoIndex still cannot start in the installed environment because `mcp.server.fastmcp` is missing; no new CocoIndex index is claimed.

## Native messenger settings and delivery — workflow 13

The native Settings menu opens an asynchronous messenger editor using the existing Rust settings dispatcher. It supports Slack Incoming Webhook and company webhook settings, event selection, optional goal inclusion, JSON body templates, approvers, and masked secret fields. Save requests are bound to the visible workspace revision; saved secrets are never read back into the UI, and blank secret fields preserve machine-local values. The explicit test action works with a newly entered or already-saved webhook URL and displays a localized result. No message is sent until the operator selects **Send test message**.

The exact v0.1.36 package at `2ef5c3b` exposed a first-use bug: selecting the company webhook rendered an unset template as an empty string, so the test action failed JSON parsing before invoking its local transport. The editor now initializes that field from the same `{"text":"{{text}}"}` default already used by `messenger::payload`. A red/green rendered-value test also passes the value through the production payload renderer. The fixed exact package at source `9845db79af260b631bde7ade2104bae2400cc649` then displayed the default JSON and reported “Test message sent”; an injected loopback fake captured the HTTPS URL and JSON body without sending a request or saving settings. Workflow 13 is accepted at 13/25 in `docs/evidence/issue9-messenger-default-template-20261023.json`. No external endpoint or saved secret was used.

Verification: 28 tests across seven targeted suites, native-feature all-target Clippy (`-D warnings`), `source_layout`, bilingual message-catalog checks, formatting, diff checks, and the 500-line source limit pass locally. The exact-package Workflow 13 acceptance is macOS-only and uses a fake transport; Windows GUI, accessibility/Korean IME, and live company-webhook delivery remain unverified. Shipped/default UI conversion remains 0/25. Overall packaged acceptance later advances to 14/25 with Workflow 14; see `knowledge/wiki/packaged-workflow.md` for the current checkpoint.

## Native Workspace roster and goal header — workflows 11/16 partial

The opt-in native Workspace now keeps registered agents visible beside the workspace form. Each roster card shows status, session count, queried model count, quota details with a full tooltip/accessibility name, coordinator marking, and the existing edit/register actions. Quota refresh remains asynchronous; the only scheduled repaint is while planning is active. The form now restores the goal-first headline and stage context and uses the guide's surface and secondary-text colors. Bilingual accessibility tests preserve the original numbered stage names and cover the goal headline.

Verification: 43 tests across nine native/UI/catalog/source-layout suites, feature all-target Clippy (`-D warnings`), native probe build, formatting, diff checks, and the 500-line bound pass locally. The rebuilt single macOS preview visibly shows all three agents and the goal-first Workspace hierarchy with the existing draft intact. Prototype parity remains 18/25 (72%), 2 partial, and 5 not started; shipped/default conversion remains 0/25. The native screen is still a prototype: comparison against the complete prior screenshots in both themes, Korean IME/keyboard/screen-reader acceptance, and Windows verification remain open.

## Native task feedback and reassignment — workflow 18

The Agent Room task inspector now selects a planned task, edits per-task direction, and reassigns it to a registered agent. Requests are bounded, tied to the displayed run revision, and validated against current task/profile IDs; assignments stay disabled during an active run. Feedback goes through the existing Rust dispatcher and resumes an eligible stopped run after the revision is accepted. Prior evidence remains available.

Verification: three focused task-control tests plus the native Room, rendering, language, launch, status, tool-dialog, probe, and source-layout suites pass (50 tests across nine suites). Native-feature all-target Clippy (`-D warnings`), `cargo fmt --check`, offline native binary build, and the 500-line limit pass. Controller integration verifies reassignment and feedback persist in the run report. Prototype parity is 19/25 (76%), 2 partial, and 4 not started; shipped/default conversion remains 0/25. Live-window keyboard/IME and Windows acceptance remain open.

## Native task session and model selection — workflow 19

Selecting a task in the native inspector identifies its existing provider session and displays the captured session ID. The model selector offers only that task agent's observed catalog; changing the model requires an idle, editable run and sends the revision-bound action through the existing Rust controller. The Rust workflow validates the catalog, clears stale approval/evidence and requires a fresh attempt before approval. No provider session IDs can be typed or substituted.

Verification: native task-control, Room, rendering, language, launch, status, tool-dialog, probe, and source-layout suites pass (51 tests across nine suites). The Room-dispatch integration verifies reassignment, model selection, and feedback persist. Existing native-session/recovery tests (8) verify session ownership, invalid-catalog rejection, model-change approval invalidation, and a new attempt. Native-feature all-target Clippy (`-D warnings`), formatting, offline native build, diff checks, and the 500-line limit pass. Prototype parity is 20/25 (80%), 2 partial, and 3 not started; shipped/default conversion remains 0/25. Live GUI, keyboard/IME, and Windows acceptance remain open.

## Native manual scheduling after an availability restriction — workflow 20

When an Agent Room task is waiting on a temporary provider restriction, the task inspector shows its recorded reason, an explicit ISO 8601 timestamp field with timezone, and a schedule action. It prefills a known reset time; when none is known, the operator enters the time. The revision-bound action is accepted only for a waiting task with a valid timestamp, and the existing Rust scheduler persists it across the affected quota group without starting work before the scheduled time.

Verification: the bilingual task-control and real Room-dispatch integration tests cover the visible controls, malformed timestamp/state rejection, and persisted `manual_schedule` time. The relevant 11 suites pass (61 tests), along with native-feature all-target Clippy (`-D warnings`), formatting, offline build, source layout, diff checks, and the 500-line bound. Prototype parity is 21/25 (84%), with 2 partial and 2 not started; release conversion remains 0/25. Live-window, Korean IME/accessibility walkthrough, and Windows acceptance remain open.

## Native task activity, logs, and lessons — workflow 21

The Agent Room task inspector now has localized Activity, Raw logs, and Lessons tabs. Selected-task details load asynchronously through the existing bounded Room request worker and refresh with the Room snapshot cadence. Claude, Codex, and OpenCode events are shown as concise transcript entries; tools, results, warnings, and errors are distinguished, unknown event types retain their type label, and provider JSON is reserved for Raw logs. That tab shows bounded recent stdout/stderr. Lessons remain task-scoped and show only the reviewed lesson status, response, source hashes, and revalidation note returned by the existing Rust knowledge service.

Verification: 66 tests across 12 native UI/session/recovery suites cover provider normalization, unknown events, both language catalogs, raw-log/lesson visibility, and a real Room-dispatch details response. Native-feature all-target Clippy (`-D warnings`), formatting, offline build, source layout, diff checks, and the 500-line bound pass. Scoped Graphify extraction reports 57 nodes / 125 edges across 4 Room-control files; CocoIndex CLI is unavailable in this environment. Prototype parity is 22/25 (88%), with 2 partial and 1 not started; release conversion remains 0/25. The feature remains opt-in; live GUI, Korean IME/accessibility walkthrough, and Windows acceptance remain open.

## Native appearance selector — workflow 11 partial

The shared Workspace/Agent Room navigation now exposes Settings, English/Korean, and Light/Dark choices. Language and theme use the existing machine-wide preference file; cached state localizes navigation labels and applies egui's active theme without per-frame disk reads. Workspace cards use the active panel fill and border. The macOS preview verifies Korean menu/step labels and persisted selections after restarting the probe. Language persistence/accessibility, appearance, workspace, native probe, catalog-integrity, and existing interface-language tests pass; feature all-target Clippy, changed-file rustfmt, diff checks, and the changed-source 500-line bound pass. Workflow 11 remains partial while the rest of Workspace/Agent Room content is not ported; the shipped UI remains WebView.

The native egui theme now maps the design guide's light E and dark F palette to panel/window surfaces, secondary text, selection, borders, and active controls. TDD verified the exact palette after first exposing egui's stock dark panel color. Eleven tests across native appearance, theme/language contracts, and source layout pass, along with feature all-target Clippy (`-D warnings`), formatting, diff checks, and the offline probe build. Workflow 11 remains partial and release conversion remains 0/25. Visual review of the rebuilt preview in both themes is still required before this styling slice is complete.

The native Workspace form now renders localized labels and accessible names for workspace name, project folder, goal, execution isolation, shared-folder/worktree choices, save, and import/export settings actions. It reuses the existing English/Korean catalog and preserves draft values. Regression coverage now checks these labels under the persisted language and the native control contract derives expected labels from the active catalog. Focused Workspace, language, appearance, native-probe, i18n, interface-language suites and native-feature all-target Clippy pass. This remains part of partial workflow 11; other Agent Room content and shipped/default UI conversion remain outstanding.

The native plan and Agent Room task list now display compact localized status tags with accessible task identity. Running adds a pulsing green dot and schedules repaint only while active. Waiting states use amber, failures/blocked states red, and completed states green; semantic foreground/background pairs meet WCAG AA in both themes. Status colors and text use the existing theme and bilingual catalogs without disk reads. Two status tests cover translated AX labels, unknown-state fallback, active pulse and contrast across seven states in both themes. Room/render, launch, tool-dialog, probe, i18n tests and native-feature all-target Clippy pass. The rebuilt Korean preview was checked in light and dark themes. Scoped Graphify reports 456 nodes / 718 edges and CocoIndex indexed 532 files / 5,639 chunks, reprocessing 11 with no errors. Workflow 11 is still partial because other Workspace/Agent Room controls remain; shipped/default conversion is 0/25.

The native Agent Room refresh, team-briefing, stop-confirmation, and review-evidence controls now use the selected English/Korean catalog for visible text and accessibility names. Stop confirmation retains the existing bounded-reason validation and Rust dispatcher; no new polling or background work was added. Focused verification passed 35 tests across Agent Room language/render, status, launch, tool-dialog, and i18n suites; feature all-target Clippy, formatting, and the 500-line bound also pass. A previous English-only test assumption was updated to follow the saved language, so Korean users can run the same regression suite. The single macOS preview build is being refreshed for inspection. Workflow 11 remains partial while other native screens and shipped/default conversion remain outstanding.

Native candidate decisions now use bilingual visible labels and accessible names for approve, reject, and merge, plus their confirmation prompts, rejection reason, cancel, and confirm controls. The exact current-candidate/evidence gate, separate approval and merge steps, and bounded nonempty rejection reason remain unchanged. A Korean AccessKit test covers all three decisions and the rejection field. Focused Agent Room language/render, status, launch, tool-dialog, and i18n verification passed 33 tests; focused Clippy with warnings denied passed. `room.rs` remains below 500 lines at 492 and the new test is 86 lines. Workflow 11 remains partial; other Agent Room content and shipped/default UI conversion are still outstanding.

Native tool-request cards and their decision modal now localize the request count, task/status context, allow/deny/refresh actions, confirmation prompts, and accessible action names. A Korean AccessKit regression first failed on English labels and now covers the localized decisions while preserving current revision and pending-request checks. Nine focused tool-dialog, Agent Room language, candidate, and i18n tests pass; one unrelated broad source-reference test was excluded after it stalled reading the source tree. Rustfmt and diff checks pass, and changed Rust files remain below 500 lines. The refreshed opt-in macOS preview opens as one window with the existing Workspace data intact. Scoped Graphify extraction reports 258 nodes / 325 edges; CocoIndex indexed 534 files / 5,662 chunks, reprocessing 7 with zero errors. Native-feature Clippy and full source-layout verification remain pending; the shipped WebView path is unchanged.

Native Agent Room Resume and Recover actions, their accessibility names, and the recovery safety explanation now follow the selected language. Existing Rust state gates remain authoritative: Resume stays unavailable in terminal/approval states, Recover is limited to a running run and still refuses live or unknown processes. The Korean AccessKit regression first failed on the old English labels; after localization, the UI regression and bilingual catalog key/placeholder contract pass. A disk-full linker failure exposed 50.9 GiB of stale Rust build output; clearing only Cargo's generated `target` cache restored build space. The rebuilt native probe reopened as one macOS window. Scoped Graphify reports 256 nodes / 323 edges; CocoIndex indexed 534 files / 5,667 chunks, reprocessing 7 with no errors. Native-feature Clippy remains pending.

The read-only Agent Room team briefing now receives the selected language and localizes its heading, captured state, session metadata, process status, availability message, approval summary, and missing-value fallbacks. Known run states use the shared status catalog, while agent-provided work and notes remain unchanged. A Korean AccessKit rendering regression first failed on the English-only briefing; both Agent Room language tests and the bilingual catalog ID/placeholder contract now pass. Rustfmt, diff checks, and a debug-info-disabled native probe build pass; the changed Rust files remain below 500 lines (`room.rs` 493, `room_briefing.rs` 179, test 135). The refreshed opt-in macOS preview is a single window and retains the current Workspace values. Scoped Graphify reports 54 nodes / 111 edges; CocoIndex indexed 534 files / 5,674 chunks, reprocessing 7 with zero errors. Native-feature Clippy and full source-layout verification remain pending. Prototype parity remains 17/25 (68%); shipped/default UI conversion remains 0/25.

The native Agent Room heading and read-only review-evidence panel now use the selected language. Revision, candidate, test exit/timeout, stdout/stderr, and truncation labels are localized; captured test output and diff content remain byte-for-byte user data. The existing English `render_evidence` API remains available, while the Room passes its current language through the new localized renderer. Korean regressions first failed for the missing heading and language-aware renderer. Verification passes 3 Room-language tests, 12 Room-render tests, the bilingual catalog contract, native-feature all-target Clippy (`-D warnings`), the complete source-layout test, rustfmt, diff checks, and a native probe build. Scoped Graphify reports 129 nodes / 283 edges; CocoIndex indexed 534 files / 5,686 chunks, reprocessing 9 with zero errors. The rebuilt macOS preview remains one window with existing Korean Workspace values intact. Prototype parity remains 17/25 (68%); shipped/default UI conversion remains 0/25.

## Structural map

The latest Graphify update rebuilt the repository graph with 6,163 nodes, 10,349 edges, and 461 communities. CocoIndex was invoked but could not start because the installed `cocoindex-code` entry point cannot import `mcp.server.fastmcp`; earlier successful indexing numbers refer to an older source snapshot. Both are local analysis tools, not application runtime dependencies; the graph is code-structure evidence, not proof of production UI coverage.

### Safe OpenCode MCP examples — GitHub #17 / DD-163

When no MCP server is registered, the native Settings editor displays commented local and remote examples using OpenCode's `mcp` field shape; registered servers suppress both examples and their explanatory note. The local command is deliberately not installed, the remote host is `mcp.example.invalid`, and credentials use `{env:COMPANY_MCP_TOKEN}`. Existing `goal_sources` values remain in the editor when examples are shown. The untouched JSONC parses and saves without creating a job or starting a process. Saved server definitions still render from the workspace snapshot. The UI is an opt-in native preview; the shipped application continues to use WebView and workflow 12 still lacks native discovery/import review/apply.

### MCP source import review layout

The MCP source-selection preview is a page-level action but its potentially long result list must not be inserted into the Workspace scroll flow. Render the list in a centered, scrollable window and keep an explicit reopen control after dismissal. The regression fixture showed a 116 px shift when the list was inline. Individual full-source records remain short anchored menu popups. This is native component coverage only; it does not add an accepted packaged end-to-end workflow.

### Native agent tools guide — workflow 15 complete in prototype

The native Workspace menu opens an inline, scrollable installation guide using the existing English/Korean message catalog. Each static command is selectable and has an explicit copy button with an accessible name. Copying only writes to the system clipboard; the guide does not install tools, start processes, or make network requests. It keeps optional setup out of Settings and uses no additional runtime or crate.

Verification: 11 tests across native appearance, workspace navigation, and language suites cover the menu item, localized close/copy controls, and all 12 commands. Native-feature all-target Clippy (`-D warnings`), formatting, offline probe build, source layout, diff checks, and the changed-file line limit passed for this slice. Scoped Graphify extraction reported 30 nodes / 61 edges. CocoIndex was unavailable at that earlier checkpoint. Together with the native manual/screenshots and release-update slices below, workflow 15 is complete in the opt-in prototype; shipped/default conversion remains outstanding.

### Native offline manual and screenshots — workflow 15 complete in prototype

`src/manual.rs` parses its supported Markdown subset into shared Rust blocks used by both committed HTML generation and the opt-in egui viewer. The Workspace menu opens the selected language's bundled manual; language changes reload the cached blocks. Headings, paragraphs, lists, tables, code, inline command copying, and six bundled screenshots render natively without WebView, JavaScript, subprocesses or network access. Screenshot decoding is lazy, restricted to bundled assets, and cached only while the manual is open.

Verification: manual parsing/rendering, native manual/workspace views, appearance and language suites cover exact generated HTML parity for both languages, escaped table pipes, safe local HTML images, native command tables, screenshot rendering, code blocks, menu exposure, and copy/close controls. The screenshot slice passed 3 native manual and 4 HTML renderer tests; native-feature Clippy, offline build, formatting, diff checks, and changed-file 300-line limits pass. Repository-wide source layout remains red on existing files over the new 300-line limit.

### Native release updates — workflow 15 complete in prototype

The native Settings menu opens a localized release-update panel through the existing Rust dispatcher. It shows current/latest versions, status and errors, release notes in a read-only, keyboard-focusable multiline control inside a bounded scroll view, opens a release-page link only for GitHub release URLs, and offers the existing check and install/restart actions. Release bodies remain literal plain text; the panel does not interpret remote Markdown or HTML. After the first workspace snapshot, the same background worker checks immediately and every 600 seconds; it polls status every 1.5 seconds only while an update operation is active, even when the panel is closed. A localized, accessible update button appears beside Settings only when a newer version is available. Repaint deadlines reuse egui's event loop, and the existing checksum-verified installer and all-workspaces idle gate remain authoritative. The production WebView remains unchanged.

Local verification: 16 focused update, appearance, and workspace tests; native-feature all-target Clippy (`-D warnings`); default and native-probe builds; formatting; diff checks; and changed-file 300-line limits pass. A first-frame response-order race is prevented by waiting for the initial snapshot before requesting update status. The repository-wide source-layout check is still pending the existing 300-line refactor. Graphify reports 6,102 nodes / 11,077 edges; CocoIndex indexed 566 files / 5,974 chunks with no errors. Native prototype workflow 15 is complete; overall native prototype parity is 24/25 (96%), while shipped/default conversion remains 0/25.

Issue #9 follow-up (2026-10-18): replaced the selectable release-note Label with a read-only multiline text control to restore the legacy `<pre tabindex="0">` keyboard-focus contract while retaining the same 260-point scroll viewport. Seven `native_ui_updates` tests pass, including AccessKit role/read-only/content and clipped geometry. Native-window keyboard traversal and exact-package acceptance remain open.

### Native task editor localization — workflow 11 partial

The native task editor receives the selected English or Korean language and uses the existing message catalogs for its heading, task fields, assignment and session-model controls, and plan-review/save actions. The English `task_controls` entry point remains available; the Workspace Review flow uses the language-aware entry point. Task IDs, instructions, paths, dependencies, commands, and agent/model IDs remain user data.

Verification passed 11 tests across the task editor, language, and message-catalog suites, plus native-feature all-target Clippy (`-D warnings`), formatting, and diff checks. Every changed Rust source/test file remains below 300 lines. Workflow 11 remains partial because other native Workspace and Agent Room controls still need bilingual coverage; prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### Native role assignments localization — workflow 11 partial

The native Team role editor now uses the selected language for its heading, predefined role names, add/remove actions, default-model text, and accessible names. Registered agent IDs and observed model IDs stay unchanged. The existing English `show_role_rules` entry point remains available; the Workspace uses the language-aware entry point.

Verification passed five role and catalog tests, native-feature all-target Clippy (`-D warnings`), formatting, and diff checks. Changed Rust files are 296, 86, 132, and 84 lines. Workflow 11 remains partial; prototype parity remains 24/25 and shipped/default conversion remains 0/25.

### Native execution consent localization — workflow 11 partial

The execution-consent panel now uses the Workspace's selected English or Korean language for its heading, warning, plan/repository labels, task details, and confirmation actions. User-provided goal, repository path, agent/model identifiers, arguments, permissions, writable paths, test commands, and plan digest are preserved as values. `execution::controls` remains an English compatibility entry point; the Plan Review screen calls the language-aware function.

Verification passed 15 tests across execution consent, launch compatibility, and message catalog contracts. Native-feature all-target Clippy (`-D warnings`), default `cargo check --all-targets`, formatting, and diff checks pass. Graphify refreshed the canonical graph to 6,363 nodes / 10,574 edges / 492 communities. CocoIndex still fails to import `mcp.server.fastmcp`, so no fresh index is claimed. The repository-wide 300-line guard still reports 29 existing files. The cache-cleanup panel still has hardcoded English copy; workflow 11 remains partial, native prototype parity 24/25, and shipped/default conversion 0/25.

Implementation: commit `a242d2a`, pushed on the issue #9 branch; [progress record](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218320478). AgentMemory records: `mem_mvb35m0t_ac1b8524bc43` (localization decision) and `mem_mvb3hgge_406665e57871` (default-test feature gate).

### Native cache-cleanup localization — workflow 11 partial

The native Workspace cache-cleanup panel now receives the selected interface language. Its title, preview and removal summaries, skipped-entry notice, loading and empty states, unavailable-path fallback, action labels, and clear-action accessible name use the existing English/Korean catalogs. The English `show` entry point remains available, and cache paths/counts remain data.

Verification passed 8 tests across native cache and language suites; default `cargo check --all-targets`, native-feature all-target Clippy (`-D warnings`), formatting, and the native probe build pass. The touched Rust files are 202 and 233 lines. Graphify refreshed to 6,373 nodes / 10,593 edges / 487 communities. CocoIndex was retried, but its CLI still fails to import `mcp.server.fastmcp`, so no fresh index is claimed. AgentMemory record `mem_mvb3rvo0_9541202c30e2` and Claude handoff `sig_mvb3sfe1_a8dff65a4949` track the slice; [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6218462009) is recorded. The cache-panel localization subunit is complete; the broader native bilingual audit remains open. The prototype remains 24/25 and shipped/default conversion remains 0/25.

### Default-build gates for native UI integration tests

Auto-discovered integration tests that import optional `eframe` or `native_ui_probe` are now gated by `native-ui-probe`, unless Cargo already declares that requirement for the target. The `native_ui_targets` regression scans root integration test files so new native-only tests cannot silently break the default all-target build.

The regression passes (2 tests); default `cargo check --all-targets` and native-feature all-target Clippy (`-D warnings`) pass. This is test-build wiring only; it does not convert the production WebView launcher. The source-layout gate remains red on 29 files above 300 lines.

### Native bilingual activity and default launch — issue #9 / workflow 11

The native Workspace header no longer displays its probe-only marker. Fixed CLI-generated activity text (session start, task-list update, token counts, changed-file counts, command exit codes, and clipped line counts) now uses the selected English/Korean message catalog; agent-produced text and identifiers remain unchanged. `ui-new` and `ui` use the shared Rust eframe launcher in the default Cargo build.

TDD added bilingual rendering and default-feature contract tests. The former failed on `NATIVE UI PROBE` and on English generated activity in Korean mode. Focused native localization suites passed 14 tests; `cargo build --bin agentconductor --locked` and the default-launch contract passed. The locked all-target run reached every target before the source-layout guard; all targets after that guard were then run separately and passed. The sole all-target failure is `source_layout`, which still reports 29 files above 300 lines. Native all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. Workflow 11 remains partial pending the bilingual copy audit. The WebView resources, fallback code, exact cross-platform package tests, and final release remain incomplete. Committed and pushed as `1c3413e`; [GitHub #9 progress](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219183508), AgentMemory `mem_mvb55br8_bd0c3051a682`, and Claude signal `sig_mvb55edm_6600327d067f` record this subunit.

## Remaining acceptance

- Port every shipped screen and interaction to the native Rust application while preserving existing workflow behavior.
- Audit all remaining visible Workspace and Agent Room text in both languages before closing workflow 11.
- Verify Korean IME composition, keyboard navigation, screen-reader accessibility, responsive layout, and separate-window lifecycle on macOS and Windows.
- Measure idle CPU and memory for the packaged native application against the existing resource budget.
- Remove project-owned JavaScript and Python runtime/build/verification paths only after equivalent Rust behavior and checks pass.
- Audit shipped dependencies and license notices, then build and smoke-test exact macOS and Windows release archives.

### Release package writer split — issue #9 / 300-line source bound

The package API and output contracts stay in `release_package::assemble`; the private `PackagePlan` and archive-writing implementation live in `src/release_package/archive.rs`. This separates package validation/license selection from archive output without changing exported functions or error handling.

The source-layout guard identified `src/release_package.rs` at 350 lines before extraction. The seven release-package and license tests passed before and after; native all-target Clippy (`-D warnings`), the locked application build, rustfmt, and diff checks pass. The source file is now 219 lines and its new archive module is 140 lines. The repository guard still reports 28 other files above 300 lines. Commit `6d763fa` is pushed; progress is in [GitHub #9](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219267552), AgentMemory `mem_mvb5aiww_40844f57405f`, and Claude signal `sig_mvb5b1pu_c9527e792667`. The full #9 migration remains open.

### Release smoke validation and workflow split — issue #9 / 300-line source bound

The public `release_smoke::validate_package` and `release_smoke::run_smoke` entry points now re-export code from focused modules: package manifest/checksum validation, subprocess setup/collection, and the four git/folder workflow scenarios. Command timeouts, error details, and evidence-file output retain their existing behavior.

The source-layout guard identified `src/release_smoke.rs` at 453 lines before extraction. All six release-smoke tests pass before and after; native all-target Clippy (`-D warnings`), the locked application build, rustfmt, and diff checks pass. The root is now a 6-line module; validation, runner, and workflow files are 74, 146, and 242 lines. The guard reports 27 other files above 300 lines. Commit `ecf2414` is pushed; progress is in [GitHub #9](https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219375990), AgentMemory `mem_mvb5ie1k_0362c7154e2b`, and Claude signal `sig_mvb5ig35_7102ed248986`. The full #9 migration remains open.


## Quota display module split — issue #9 / DD-41

The 404-line quota display module is split into a small facade, label/time helpers, and badge rendering. The `quota::{Badge, badge, badges, budget_rows, window_labels, window_reset_labels}` API remains stable, so the controller and integration tests need no caller changes. The existing quota display suite passed 8/8 before and after the move. The locked app build, native-feature all-target Clippy with warnings denied, rustfmt, and diff checks pass. The repository-wide 300-line guard now reports 26 remaining files; #9 is still open. Graphify: 6,413 nodes / 10,623 edges / 488 communities. CocoIndex CLI was not available. AgentMemory `mem_mvb5qjfg_74253af00fa0`; Claude signal `sig_mvb5qlrj_00a813ca9aa4`.

GitHub issue #9 progress: https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219503212.


## Bilingual native workflow 11 — completed in the source build

The final audit found two remaining English strings in the eframe UI: the visible `Run` prefix on status tags and the profile-avatar accessibility name. English/Korean catalog messages now render both while preserving profile IDs and status values as data. New render/accessibility regressions first failed against the original hardcoded text, then passed after localization. A source scan found no remaining direct English labels in these controls; the invariant `AgentConductor` product heading remains branded.

Verification passes 33 tests across 14 UI/i18n suites, covering language persistence, light/dark appearance, Workspace and Agent Room controls, navigation, profiles, task activity, and accesskit labels. The default application build, native-feature all-target Clippy (`-D warnings`), rustfmt, and diff checks pass. Native source-build parity is 25/25 (100%); published v0.1.24 remains WebView, so release conversion is still 0/25 pending exact macOS/Windows package, accessibility/IME, visual, resource, and license acceptance. Graphify: 6,416 nodes / 10,626 edges / 478 communities. AgentMemory `mem_mvb62w4q_5fedb050b69f`; Claude signal `sig_mvb62yin_0028abb68615`.

GitHub issue #9 workflow 11 verification: https://github.com/neodamul/AgentConductor/issues/9#issuecomment-6219714532.

### Default native shell and runtime requirements

The default Rust eframe launcher now owns the Workspace and Agent Room windows. The unused Cocoa/WebView launchers and direct `wry`/`tao` dependencies are removed; Windows no longer lists WebView2 as an application requirement. The native viewport loads the embedded AC icon. Focused release, i18n, branding, and Windows tests pass (9 total); default all-target compilation and native-feature Clippy pass. Full all-target tests have one known failure: the 300-line source-layout guard reports 25 files, including one remaining JavaScript file, above the limit. Exact macOS/Windows package acceptance, visual/IME/accessibility/resource checks, and the remaining file splits are still open. Published v0.1.24 is unchanged until a later release.

The unused `Room::document` and setup HTML assembly are removed from the compiled application. The Rust native UI remains responsible for Workspace/Agent Room rendering and the manual viewer; both native headers show the package version. A Rust source guard prevents the removed document modules from embedding browser assets. The old JS/HTML/CSS source files and tests inspecting them are not all deleted yet. The release binary scan has no WebView runtime or browser-script markers. Focused verification passes 39 tests across 10 UI suites, all-target check, native-feature Clippy, optimized release build, rustfmt, and diff checks. The strict layout test still reports 25 files over 300 lines; exact package/UI/platform acceptance remains open.

The 367-line `src/ui/room.js` and its markup-only tests are deleted. Rust quota-formatting and snapshot coverage remains; native eframe tests own task rows, quota presentation, routing, and the UI. The prior static JS assertion for Korean IME handling was removed because it only tested the deleted WebView page; native IME behavior still needs real macOS/Windows validation. Focused verification passes 21 tests with 1 environment-dependent icon test ignored across 8 suites; all-target check, native-feature Clippy, release build, rustfmt, and diff checks pass. The rebuilt app contains no legacy JavaScript asset names and does not link WebView. The strict source-layout test is now reduced from 25 to 24 remaining files over 300 lines. Other unused browser assets/tests and exact package acceptance remain open.

### Agent Room run-status banner — issue #9 / R-99 / DD-107

The native Agent Room now renders a state-colored banner above the run controls. It summarizes the localized run state and completed-task count, and displays the run error or first failed task error when available. Stop and Resume controls remain in their existing action row. A focused rendered-UI regression first failed without the banner, then passed after integration; the new banner test and all nine Agent Room render tests pass. Changed Rust files are below 300 lines. Actual packaged macOS/Windows placement and visual comparison are still required, so this does not change the 0/25 accepted-workflow count. Graphify refreshed to 6,427 nodes / 11,127 edges. AgentMemory records `mem_mvdqgjox_5d95f8ca3e7a` and `mem_mvdqhogq_3e475cb866fa`; CocoIndex was not needed for this Rust UI slice.

### Agent Room task-row status attention — issue #9 / R-99 / DD-107

Failed task rows now have a persistent red outline. Rows waiting for approval or a tool decision pulse their outline; setting egui's animation duration to zero leaves a static outline without animation repaint requests. The rendered regression first failed when no row outline existed, then passed for failure color, two pulse phases, and the static reduced-motion style. The related focused suites pass. The app does not currently verify or expose an OS-level reduced-motion preference, and exact packaged macOS/Windows appearance remains untested. Overall accepted workflow count remains 0/25. Graphify refreshed to 6,437 nodes / 11,144 edges. AgentMemory record: `mem_mvdqn2f4_256393044629`.

### Failed-run feedback action — issue #9 / UC-100 / DD-118

The native failure banner now exposes the localized “Fix with feedback” action when a failed integration test or worker maps to a planned task. Clicking it selects that task and drafts its bounded captured output into the existing editable feedback field, retaining any prior task-specific draft for the operator to review. The new rendered interaction regression failed before the action existed, then passed after wiring the banner to task-control state; it verifies button accessibility, task selection, and visible draft contents. The test and touched Rust modules remain below 300 lines. This widget-level behavior does not prove the packaged app or end-to-end retry path; macOS/Windows visual and send/reverify acceptance remain open, and #9 remains at 0/25 accepted workflows.

### Review Plan task disclosure — issue #9 / UC-106 / DD-124

Review Plan task editors now start as compact disclosures showing task ordinal, ID, and the first non-label instruction line; Remove remains available in the header. Expanding a card exposes the same editable agent, model, instruction, scope, dependency and test-command fields. The Red test reproduced all fields being expanded with no summary; Green verifies the one-line summary, hidden body by default, and body visibility after an actual disclosure click. The existing task-edit and width tests pass with their editor body explicitly exposed for those checks. Changed Rust files remain under 300 lines. Packaged macOS/Windows, keyboard, screen-reader and multi-task visual review remain open; overall issue #9 stays at 0/25 accepted workflows.

### Agent Room review evidence pane — issue #9 / UC-04 / DD-06

The legacy Agent Room keeps Activity, Diff & tests, Logs, and Lessons in separate tabs. The
native Room previously rendered diff/test evidence inline beneath Activity, making the central
human-approval evidence harder to find. The Rust Room now provides a dedicated Diff & tests tab;
it only renders evidence bound to the current run revision. If the run has changed, the pane
shows the localized stale-evidence warning and accessible Review latest action, which reuses the
existing revision-bound evidence request. Candidate approval/merge still depends on current
evidence. TDD covers selecting the tab, keeping diff/test output out of Activity, warning for
stale evidence, and emitting the evidence-refresh event. This is native widget evidence only:
packaged macOS/Windows, keyboard navigation, and live approval gating remain unaccepted, so the
verified operator workflow count stays 0/25.

The Workspace phase summary also lists multiple goals. It marks the active goal as current and
sends inactive goal selection through the existing revision-bound `select_goal` command; the
control is disabled while a request or job is active. On success, the Workspace reloads its
snapshot so the goal form and phase summary agree. The controller archives the previous goal's
last run and clears the old proposal when changing goals. A rendered interaction test covers
the accessible selector and serialized request; it does not prove packaged parity.

### Start feedback stays visible at short window heights — issue #16

During plan approval, pending/error feedback now renders in a bottom panel outside the scrollable
Workspace content. The approval-to-Agent-Room test reproduces the old failure at 700px height: the
scroll-contained status label was not rendered inside the viewport. With the panel, the test checks
that the pending status bounds remain inside the short viewport while the approved start is still
pending, then verifies the real Room dispatch opens the started run.
running, then verifies the real Room dispatch opens the started run. The panel reuses existing
status text and asynchronous updates. This branch-level regression does not establish packaged
macOS/Windows acceptance; issue #16 remains open until that runtime check follows #9 integration.

### Generated-plan review journey coverage — issue #9

The existing native approval journey now checks that all three tasks generated by the mock planner
appear in the Review Plan accessibility tree before exact-plan consent. It then continues through
pending-start feedback and the Agent Room transition. The focused offline journey and 300-line
source-layout checks pass. This verifies native-probe plan visibility, not exact packaged UI or any
of the 25 accepted operator workflows; issue #9 remains open at 0/25.

### Resume preserves bounded integration repair — R-123 / UC-113 / DD-132

The upstream failed-run guard blocked every `failed` report, including the repairable integration
failures that R-123 explicitly routes back through the coordinator. Native Resume now uses the
existing `workflow::repairable` predicate: only failures whose error begins with `integration ` and
whose tasks are all complete enter `resume-due` and the existing three-round repair loop. Other
failed runs remain blocked until the operator revises or reassigns work. TDD confirmed the new
repairable-integration regression fails under the blanket guard and passes with the predicate;
the non-repairable guard test remains green. Nine tests across `ui_failed_resume`, `repair`, and
`ui` pass; native-feature all-target Clippy, rustfmt, diff checks, and source-layout test pass.
This corrects the Resume entry point but does not change verified workflow acceptance (0/25).

### Latest-main full regression — 2026-10-19

After merging `main` commit `5993970`, the complete offline locked native-feature all-target
suite passed: 659 passed, 14 ignored, 0 failed across 230 suites (193.74 seconds). The first
sandboxed attempt reported four AgentMemory contract tests failing only because macOS denied
their `127.0.0.1` test-server binds; rerunning the same suite with local loopback permission
passed all four. Native-feature all-target Clippy (`-D warnings`), rustfmt, diff checks and
the 300-line source-layout test also pass. This is regression evidence, not end-to-end parity:
Windows package/UI, real IME/accessibility, and all 25 operator workflows remain unaccepted.
Workspace content. The approval-to-Agent-Room test reproduces the old failure at 700px height: the
scroll-contained status label was not rendered inside the viewport. With the panel, the test checks
that the pending status bounds remain inside the short viewport while the approved start is still

### Agent Room four-stage workflow context — 2026-10-19

The Rust Agent Room now shows the existing four-step workflow row (`01 Plan`, `02 Tasks`, `03 Verify`, `04 Complete`) after its run-status banner. The active step comes from the persisted report: prepared plans map to Plan, active work to Tasks, waiting approval/approved reports to Verify, and completed reports to Complete. Failed runs remain on Plan when planning produced no tasks, on Tasks when planned work is incomplete, or on Verify when all planned work completed and verification failed. The current step has a localized AccessKit name; the stage labels fit the 792×532 minimum test viewport. Red-first rendered tests cover state mapping and English/Korean labels. The final offline all-target suite passes 667 tests, 14 ignored and 0 failed across 235 test targets; all-target Clippy with `-D warnings` also passes. Exact isolated macOS review shows the waiting-approval candidate highlighting Verify. Graphify was refreshed to 6,728 nodes / 11,666 edges. AgentMemory record `mem_mvelvv92_2b278ee1838f`; Claude signal `sig_mvem4axx_855da1382df5`. CocoIndex is unavailable in this host environment. This does not advance accepted end-to-end workflows: issue #9 remains 0/25.

Temporary-space audit on 2026-10-19 found `/private/tmp` at 535 MiB, not the reported 41 GiB. The Rust `target` directory had grown to 9.0 GiB during regression; after removing its 2.8 GiB regenerable incremental cache it is 7.9 GiB. Temporary compiler wrappers and test logs were removed; the current isolated review app and its test evidence remain. Host Rust tools require temporary Z3 soname and invocation wrappers because Homebrew LLVM references a missing `libz3.5.1.dylib`; these are not product dependencies.

### Agent Room candidate review visual check — 2026-10-19

Using the Rust executable built from source commit `2ec925b`, a fresh isolated mock run reached
`waiting_approval`: all three tasks completed and all three recorded test commands exited with code
0. At 1140×772, the actual Rust Agent Room was inspected in light and dark themes. Its status,
four-stage progress, task/session list, candidate review rail, test evidence, and review controls
were visible. The read-only “Load change and test evidence” action displayed the current revision,
candidate commit, and passing test results; approval and merge were not invoked. Temporary run
data and the 77 MB app bundle were deleted after closing the app. The already-built source-layout
test binary also passes (1 passed). The regular Cargo test command was not rerun because this
host's rustc cannot load Homebrew LLVM's configured `libz3.5.1.dylib` soname. No screenshot files
were retained. This mock-run visual check does not count as a real-agent workflow or advance issue
#9's verified end-to-end acceptance from 0/25; Windows, real approval/merge, IME, accessibility,
and the remaining workflow gates stay open.

### Inline quota refresh spacing — 2026-10-19

R-64/UC-54 requires the compact per-agent quota value and refresh action to remain together
with a 5px gap. The rendered egui regression measured an 8px gap at the 248px rail width before
the change. The Rust sidebar now groups the value and refresh/spinner in a local horizontal row
with 5px spacing; provider-window details remain in the tooltip and accessible label. The new
geometry regression and related sidebar roster/density/icon, quota display, and source-layout
suites pass (16 tests total); focused Clippy, rustfmt, and diff checks pass. The source/test files
are 282/176 lines. Exact packaged macOS/Windows screenshots and full workflow acceptance remain
open; issue #9 stays at 0/25. Graphify refreshed to 6,754 nodes / 10,785 edges / 518 communities.
CocoIndex structural grep located the sidebar's row construction; semantic indexing is unavailable
because this repository has not been initialized for CocoIndex.

### Exact-source macOS package — 2026-10-19

The optimized Rust release build for `aarch64-apple-darwin` and the Rust `package-release`
command succeeded at source commit `7df4a93d5d347e7a80e523cd2826a170ba646c12`. Explicit
`validate-package` passed: 17,560,864-byte single executable, 214 dependency license records,
binary SHA-256 `9d9772bc2e27e23deb588e9e1a46bf2a88cced35c0182ae9a144494fc4500e83`; archive
SHA-256 `28ce1d33f465af229a955470cd56f4234673a4a58cfec80347ba87f42e20dff3`. The package command
also completed all four shared/worktree × Git/folder smoke cases, including approval-gated merge
checks. A GUI inspection attempt was not valid evidence: a separate CUA launch displayed the
existing default Workspace rather than the requested isolated workspace. No controls or Save were
activated; the window and CLI process were closed and task-owned temp files removed. Reuse a single
verified package-app process with explicit isolated arguments for future UI inspection. The local
macOS artifact is not a release or Windows validation and does not advance issue #9 from 0/25.

Correction (2026-10-19): the exact package UI was then launched once with explicit `ui-new` and an
isolated workspace argument. The process command line confirmed the packaged `.app`; CUA attached
to that same process and inspected its 1140×772 empty Workspace screenshot and accessibility tree.
The expected setup fields were present and Coordinator was disabled; no save or user data access
occurred. This verifies only the packaged empty Workspace, not quota cards or end-to-end parity.
The task-owned isolated HOME/workspace was removed afterward. `/private/tmp` measured 456 MiB;
unrelated work directories and the reusable Rust build cache were retained.

### Exact packaged availability-restriction spot-check — 2026-10-19

The exact macOS package was run with an isolated HOME, workspace, repository, planner fixture,
and Rust mock agent. A simulated `usage_limit_reached` JSON event put all three tasks into
`Waiting availability`; the task inspector showed the restriction reason, timezone-labelled
schedule field, and retry button. The focused `native_ui_task_scheduling` suite passes all three
tests, including a real Room-dispatch persistence check. The CUA keyboard inherited the active
Korean IME and could not enter the ISO timestamp, so the packaged click and due-time resume were
not manually exercised. This does not indicate a product defect and does not advance #9's strict
0/25 acceptance count. The isolated 41 MiB package/HOME/workspace fixture and compatibility link
were deleted after the package and watcher processes exited; unrelated temp data was preserved.

### Test fixture temp-storage correction — 2026-10-19

Investigation found 1,145 stale Rust mock CLI executables (774 MiB) in the macOS user temp
directory. The shared process and workflow test fixtures compiled those executables there and
created their run roots there as well. Removed only the known test-generated executable names
after confirming no corresponding test process remained. Added a failing-then-passing regression
and changed both shared fixtures to use Cargo's `target/debug/tmp`, keeping repeat test output in
the disposable build tree. The full offline locked all-target suite completed with all executed
tests passing; environment-dependent provider/login tests remained ignored. After the issue #36
worker finished, removed its disposable 3.4 GiB Cargo target cache while retaining its 22 MiB
source worktree and draft PR. `/private/tmp` returned to 466 MiB. The current user temp directory
measures 804 MiB; unrelated macOS temp entries and active lease files were left untouched.

### Exact packaged agent-card spot-check — 2026-10-19

Opened the exact package once in an isolated HOME/workspace and registered a non-running fixture
(`parser=claude`, executable `/usr/bin/true`). The 1140×772 package screenshot and accessibility
tree showed the provider icon, status, session count, model refresh, compact quota value/refresh,
edit/remove and reorder actions. The quota refresh displayed the trusted-project-folder requirement;
it did not invoke Claude or query account usage. Closed the one package process and removed its
isolated files. This is visual/component evidence only. Real provider quota/model contents, Windows,
saved public screenshots, and workflow parity remain unverified; #9 stays at 0/25.

### Exact current-HEAD package attempt — 2026-10-19

Built the optimized macOS binary offline from `a8e023b9e4f26a63cc6480d2db9374bd02193cf8` in a
temporary Cargo target directory. The exact package executable launched `--help`, and
`validate-package` accepted its manifest: SHA-256 `ab023bf255a7e0fb87c81dc0b3aac96c27403b0a41481f029be093180a55fb74`,
214 dependency license records, `aarch64-apple-darwin`. The package smoke proceeded to its first
run command, then process creation returned OS `Operation not permitted`, including after setting
`TMPDIR` to an allowed `/private/tmp` directory; direct invocation of the same packaged `run`
command hit the same boundary. This is not a passing workflow or evidence of a product defect.
No package archive or UI acceptance is claimed. The temporary package source checkout, package,
isolated HOME/workspace, 712 MiB build target and compatibility alias were removed; `/private/tmp`
returned to 466 MiB. The strict native UI acceptance count remains 0/25.

### Workspace sharing menu route — 2026-10-19

UC-42 settings import/export now starts from **Settings → Workspace settings sharing** rather than
the bottom of the Workspace form. The existing asynchronous folder picker, import preview,
permission re-verification and result state remain unchanged; the sharing flow opens in a native
window. A Probe-level pointer/accessibility regression checks that export/import controls are absent
from the closed Workspace and appear after selecting the Settings item. Focused native sharing,
appearance, workspace, style and source-layout tests pass (20/20), as do all-target native-feature
Clippy, formatting and diff checks. This fixes menu placement only; exact-package macOS/Windows
inspection is pending and issue #9 remains at 0/25 verified workflows.

### Issue #16 default-path compatibility and Rust counterpart — 2026-10-19

PR #38 merged the localized start-pending feedback into the shipped WebView compatibility path
(`65c77b7`); issue #16 closed after macOS and Windows validation and exact-package smoke passed.
The Rust branch already implements the corresponding post-approval progress state and exact-run
transition. Its `native_ui_start_journey` regression passes 1/1, so copying the JavaScript patch
would add a second implementation instead of improving the Rust app. This closes the compatibility
bug, not native UI parity: issue #9 remains 0/25 pending packaged workflow acceptance.

### Exact 0.1.30 package launch — 2026-10-19

Built and validated the exact `dbad31d` macOS package (214 dependency license records; binary SHA-256
`d2b66628f69d248e5dc92b1244c035afb783ab4280057831fa20174ae0e38e85`). In a single isolated app
window, the native Workspace rendered at 1140×772 and saved its goal and project folder to the
temporary workspace state. No authenticated CLI or external service was used. This proves launch
and one save action only; no legacy screen comparison or end-to-end parity is claimed. Issue #9
remains 0/25, with Windows and full workflow acceptance open.

### Responsive role selector width — 2026-10-20

Role agent/model selectors now use the legacy 130px minimum in desktop content areas at least
900px wide, while retaining 108px on compact layouts to keep the five-column table within the
viewport. The desktop AccessKit width regression first failed at 108px; five focused native UI
suites pass after the adjustment (28 tests), along with the source-layout guard and formatting.
The full offline all-target suite passes 745 tests with 14 ignored and no failures; native-feature
all-target Clippy passes with warnings denied. This visual component check does not advance issue
#9's end-to-end acceptance (1/25). Platform Validation run `39369276017` was in progress on
macOS and Windows for PR head `2ba31c4` at the last check. This documentation-only update
supersedes that head without changing Rust code; check the current PR #22 run before accepting
cross-platform verification.


### Active workspace archive guard — issue #9 / UC-79

The archive action is now available only for inactive saved workspaces. The exact v0.1.31 package previously allowed an operator to archive the active item and switched to a new default after confirmation. An AccessKit regression reproduced that defect before the UI condition changed. The focused native workspace suite passes 7/7; targeted native Clippy, formatting, and `git diff --check` pass. Both changed Rust files stay below the 300-line cap. Exact macOS source `4de454a3e605cc5f8f368eb264cec8fc601145e6` passed package creation, manifest validation, packaged help, ZIP integrity, and four approval-gated runtime smoke cases. In the exact package UI, the current item has no Archive action and an inactive saved item retains it. The existing user workspace was observed read-only. The full all-target suite was interrupted after 250 tests across 80 suites and remains unverified; Windows UI and exact-head CI remain open. This safety fix does not change the conservative packaged workflow acceptance count of 2/25.

### Packaged named-workspace lifecycle — issue #9 / UC-30, UC-31

Verified the exact v0.1.31 macOS package from `a8be85f` with an isolated `HOME`, an empty initial workspace, and a single process. Created three named workspaces; saved and reloaded repository/goal settings; inspected all five role rows and the unsaved-state gate; confirmed Cancel preserves an unsaved role draft; switched back to a saved workspace; removed a workspace from the recent list, reopened it from saved workspaces, and archived an inactive workspace through the explicit confirmation. A later saved-state plan check exposed that missing coordinator validation happened only after clicking. The initial menu contained no paths from the user's real home. No provider CLI or external service ran. Evidence: `docs/evidence/issue9-a8be85f-workspace-ui.json` (binary SHA-256 `74204cbc9492379a71769a6f679d19ee28012b823e280e8a6fdd03af89300057`). Counted as distinct Workflow 2; packaged acceptance advances to 3/25, while the shipped Rust replacement remains 0/25. Windows UI and exact screenshots remain open.

### Grouped plan and Settings gate corrections — issue #9

The packaged audit found the saved Agent Roles screen enabled Generate plan when the saved goal was empty or the coordinator did not resolve to a registered profile; clicking navigated and only then displayed validation. The native UI now disables that action until both conditions are met and shows the existing localized reason. Regression coverage checks empty goal, missing/unknown coordinator, a valid coordinator, and disabled AccessKit action state. The same Actions batch found timing assumptions in the settings and start/merge UI journeys; they now wait for the intended dialog content and prior UI action completion. Focused planning tests pass (2) and settings/start/docs/layout suites pass (10). The exact pushed macOS package from `4a114ec6a76141c48ebe8138694398572cae1acd` passed manifest validation and packaged `--help`; one isolated UI process displayed the disabled Generate plan control and missing-coordinator guidance. Evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`. A broader native-feature run was stopped after five minutes at 363 passed / 14 ignored across 120 suites without a reported failure; full verification and cross-platform Actions remain open. Packaged acceptance stays 3/25; the source correction does not add an accepted workflow.

Graphify re-extraction after the use-case/evidence update found 7,509 nodes and 12,943 links, with no source-graph changes. CocoIndex Code still cannot start because its environment is missing `mcp.server.fastmcp`.

### Grouped dialog continuation and full-suite checkpoint — 2026-10-20

Read Claude's latest AgentMemory handoff signal `sig_mvgo0qng_9ea40889a2f2`; it accurately contains the exact package, five-dialog UI evidence, current acceptance count, and open cross-platform gates. Audited native UI markers and found no `TODO`, `FIXME`, or unimplemented macro; the prior grouped Settings pass already covers all six custom dialogs. The current user workspace was observed read-only and left untouched. Full exact-head local verification at `27e307e`: 768 tests passed, 14 ignored across 260 suites; all-target Clippy with warnings denied, rustfmt, and diff checks pass. PR #22 run `39423261706` is pending for both macOS and Windows. This source-level pass adds no packaged workflow; acceptance stays 2/25.

### Grouped operator-surface review — issue #9 / 2026-10-20

Exact package `a1ea942` passed a grouped single-window Git/worktree UI workflow: retain the goal, switch to a clean Git repo, plan three isolated tasks, run and pass their tests, review the exact diff, approve, confirm merge, and verify the final clean `main` worktree. Evidence: `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Packaged acceptance is 5/25 (20%); shipped Rust replacement remains 0/25. Windows UI, accessibility, Korean IME, authenticated providers, resource measurement, exact-head CI and final license closure remain open.

Follow-on Git/worktree smoke found that switching repositories with an unchanged goal retained the previous active slug, leaving plan metadata untracked and blocking the clean-baseline gate. The red-first test `changing_repository_detaches_the_active_goal_before_planning_again` reproduces the failure and verifies the new repository stays clean after planning. The fix clears only `active_slug` when the repository changes and preserves historical slug records. Focused tests passed 15/15; full regression passed 771 tests with 14 ignored across 260 suites; all-target Clippy passed with warnings denied. Graphify now contains 7,517 nodes and 12,953 edges.

Reused one isolated v0.1.31 macOS app process to inspect Workspace settings and role assignments, MCP registration, settings sharing, Messenger notifications, cache cleanup, custom CLI model IDs, and completed-run evidence. The cache was empty with its clear action disabled. All panels were dismissed without saving, deleting, or invoking external services. Ten related native UI suites passed (48 tests). Evidence is `docs/evidence/issue9-4a114ec-grouped-native-ui-audit.json`; this component review does not add workflow acceptance. Exact-head macOS/Windows CI, Windows UI, authenticated providers, accessibility, Korean IME, resources, and final license closure remain open.

### Exact-package New Workspace Enter regression — 2026-10-20

TDD reproduced that pressing Enter in the saved-workspace name field returned no create action. The native menu now submits the same validated create action as its button. The exact v0.1.31 macOS package built from `60260761041ee53e0414276b47ff8d3cd0c56806` passed `validate-package` (17,942,384-byte binary, 214 dependency records, bundled SQLite). In one isolated app process, entered `enter-smoke`, pressed Enter, and confirmed the saved workspace appeared in the menu as current. Evidence: `docs/evidence/issue9-6026076-workspace-enter-ui-smoke.json`. Focused tests passed 17/17 across 7 suites; full native-feature verification then passed 772 tests, 14 ignored, 0 failed across 261 suites. All-target Clippy, rustfmt, diff and source-layout checks passed. This confirms the component in a package but does not complete a distinct full workflow, so packaged acceptance stays 5/25 (20%) and shipped Rust replacement stays 0/25. Exact-head macOS/Windows CI run `39442859013` remains in progress. CocoIndex Code still fails to start because `mcp.server.fastmcp` is missing; Graphify was re-extracted without clustering after the source/evidence update (7,520 nodes / 12,957 edges).

### Compact legacy step navigation — 2026-10-21

The isolated v0.1.31 screenshot showed a 48px equal-width card row where a checked-in same-size capture with a v0.1.24 label has compact content-sized steps. Provenance identifies that reference as an earlier Rust build, not the published WebView UI; it supports a Rust component layout comparison only. The red bounds test failed at 48px; the native row now uses compact localized buttons while preserving step order, accessible names, and wrapping on narrow content. Exact source `227854d3f7bbe01efd3953325b9acb890d77b903` packaged successfully: `validate-package`, packaged `--help`, and ZIP CRC passed; executable SHA-256 `8568714474af52d04e7cf2f678dd69d27a8f95c4149c59c51b59993fa34ea97a`, archive SHA-256 `c773cea387c4a86d19fa86ba2e145af5ef3a79ff4e538166db2a2b7c45c3cd72`. One exact-package app window showed the compact navigation and allowed switching through all three stages; no workspace or settings were saved. Header/start tests pass (5 tests / 3 suites). The exact-head macOS CI run failed the sequential Settings journey while waiting for the MCP label. That test now uses the language captured from the initial rendered UI and reports the full current AX labels on timeout; it passed three consecutive local runs and then passed in a grouped four-suite run (6 tests). Focused Clippy, rustfmt, source-layout, and diff checks pass. The all-target attempt was interrupted after 666 passed / 14 ignored across 213 suites (exit 130), so it is incomplete. No screenshot file was retained. README/manual screenshots, live Korean IME, exact Windows UI, remaining workflows, and a full suite remain open; packaged acceptance is still 5/25 and shipped replacement 0/25.

### Selected Workspace step heading — 2026-10-21

The bilingual header now identifies the selected Workspace step and removes the repeated step marker from the section cards; titles, descriptions, and all three accessible step controls remain. The first English-step regression failed against the generic `PREPARE A GOAL` eyebrow. Focused header and AccessKit tests pass 17/17. The exact current-source package from `2fd8e57f27d5f5c7b5d4a63c2eae7b20ea7b1a14` passes package validation, packaged `--help`, ZIP integrity, and its bundled-license manifest check (214 dependency records; binary SHA-256 `818f4d137127e1d650d520796ebe97d7a41be41d5f0f786eea529b9607cc5f8a`). The serial offline native-feature all-target suite passes 775 tests with 14 ignored across 262 suites; all-target native-feature Clippy, formatting, source layout, and diff checks also pass. The packaged smoke passes four shared/worktree × Git/folder modes, with 12/12 fixture task tests, parallel execution, approval denial, and approved merge; evidence is `docs/evidence/issue9-2fd8e57-macos-smoke/`. The exact package was not opened: a pre-existing 0.1.31 process had the user's workspace loaded, so no second app was launched or user state changed. Graphify update found no additional code-graph changes and its query linked the header test. This is a component correction only; packaged acceptance remains 5/25 (20%), shipped Rust replacement 0/25. Exact-package visual review, Windows UI, screenshots, Korean IME, and the remaining workflows remain open. CocoIndex Code remains unavailable because its environment lacks `mcp.server.fastmcp`.

### MCP Jira/Confluence source import — Workflow 12

Exact-package verification completed registration, explicit tool discovery, source preview, and a separately approved apply in the v0.1.31 macOS Rust prototype. Keep the coordinator read-only; the fixture's `permission_mode` is `read_only`. The local stdio fixture returned Jira `AC-1` and Confluence page `53553565`; the operator reviewed separate records and their hashes before applying them. The UI then reported `Applied sources: 2`. This raised exact-package workflow acceptance from 5/25 to 6/25 (24%); it does not change shipped/default Rust replacement, which remains 0/25. Evidence: `docs/evidence/issue9-2fd8e57-mcp-import-ui.json`.

### Active Workspace card step markers — 2026-10-21


### Custom profile quota JSON round-trip — 2026-10-21

When extending the advanced JSON editor for `AgentProfile`, audit both directions: `serialize` controls which existing fields are visible, while `apply` controls which edits survive saving. The UI omitted `quota_command` and `quota_rules` from both lists even though custom quota execution supported them, so an operator could not configure or preserve quota settings through profile editing. Add a save-path regression before updating the allowlist; add an AccessKit render test for the visible fields. Preserve unrelated typed profile fields and normalize nullable collection fields consistently. The fix passes 15 focused tests, the full native-feature suite (803 passed, 14 ignored), warning-denied Clippy, formatting, and source-layout checks. An exact package at `016451b` passed validation, packaged help, ZIP integrity, and the four-case shared/worktree × Git/folder smoke. One isolated package window confirmed quota settings remained visible after saving and reopening a profile; the test profile was removed and no CLI was invoked. No screenshot was retained, so this is accessibility-state verification rather than pixel-level visual review or a new accepted workflow. Evidence and hashes are recorded in `docs/evidence/issue9-016451b-macos-package.json` and `docs/evidence/issue9-016451b-macos-smoke.json`.

### Bound the coordinator analysis popup — issue #9 / 2026-10-21

The coordinator-analysis menu rendered an arbitrary analysis string in one label without a scroll bound. A deterministic egui regression with long content failed first: the text shape's clip rectangle covered the full 900×700 viewport. The menu now wraps that label in a vertical `ScrollArea` capped at 70% of viewport height. The passing test checks the painted clip rectangle; AccessKit reports the full off-screen label bounds even when the rendered shape is clipped, so accessibility bounds are not a valid height assertion here. Twenty-two tests across seven UI suites, all-target native-feature Clippy with warnings denied, rustfmt, diff checks, and `source_layout` pass. The exact-source macOS/Windows Actions run `39605800639` is pending. One isolated app window visually confirmed the current Workspace screen; this component fix does not advance packaged workflow acceptance beyond 8/25. Graphify refreshed to 8,101 nodes / 13,917 edges without clustering. CocoIndex is unavailable because the host has no `cocoindex` executable; no indexing result is claimed.

### Menus remain overlays

The Workspace and Settings menus are egui popups: they cover the existing page instead of changing its layout. Rendered position tests cover those menus, goal phase and coordinator-analysis popups, MCP source details, profile rules, task editors, and Agent Room briefing. The 2026-10-22 packaged UI spot check reopened Workspace and Settings and observed both overlays; this did not add a packaged workflow acceptance.

### Remove duplicate Agent Room branding — 2026-10-23

The shared Workspace navigation already renders the AC mark, application name, and version. The Agent Room context bar repeated those three labels, creating two identical product rows in a single window. The Room now retains only its workspace, phase, authentication, and action context. The two rendered regressions failed before the fix and pass after it. Focused Room/titlebar tests pass 3/3; warning-denied library and targeted-test Clippy, rustfmt, diff checks, and the release package build pass. Source commit: `6f1111c5ae61c91933e03a20fb04a4df513193de`; macOS package SHA-256: `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`. The packaged UI launch returned the main Workspace window through CUA rather than the Agent Room; therefore visual acceptance and issue #9 workflow count do not advance. Windows rendering remains unverified.
### Current-head docs and source-layout verification — issue #9 / 2026-10-23

At branch head `acf2c322f0087a684703263114902bb7e07a49f7`, the optimized offline
`public_docs` and `source_layout` integration suites pass 13 tests, and `cargo fmt --all -- --check`
passes. This verifies documentation assertions and the 300-line source/test layout guard at the
current docs head only; it does not establish product behavior or packaged acceptance. Evidence:
`docs/evidence/issue9-current-head-layout-doc-tests-20261023.json`.
### Current-head macOS package and license verification — issue #9 / 2026-10-23

Built the exact branch head `88e372e95a8c2507e15c1c8b24743bf7d1f9b38a` as the
macOS ARM64 package. The application binary hash remains `0c4e1fa888cc18d13da40f81683032686829a79ea5411250a2b4ab59c8397a23`; the new archive hash is
`6019a0ff883c5713e519c9932153c14bc1e793918265a80aa1c773efc5868894`. Package validation and
four deterministic package smoke cases pass. The SBOM has 214 components with license expressions
for all entries, and reviewed dependency license texts are included. `self_cell 1.3.0` declares
`Apache-2.0 OR GPL-2.0-only`; both texts are bundled, preserving the permissive Apache option.
Evidence: `docs/evidence/issue9-current-head-macos-package-license-20261023.json`.

This package is not release-ready: it still reports v0.1.36, its manual screenshots are historical,
and Windows package/runtime/license verification remains open. This verification adds no packaged UI
workflow acceptance.

### Windows UTM verification requires a guest transfer path

The existing Windows ARM64 VM can reach its signed-in desktop without stopping or resetting it. UTM's IP query returns OSStatus -2700 when QEMU Guest Agent is missing or stopped, and selecting a shared folder on the host does not prove that Windows can read it. Record Windows build/package/UI verification as **not run** until the guest agent or another authorized transfer path is available. Evidence: `docs/evidence/issue9-windows-utm-login-20261023.json`.

The follow-up desktop check after the user-approved reset found UTM 5.0.6 (125) still displaying `Display output is not active`. A read-only `utmctl exec` request for `cmd.exe /c ver` returned OSStatus -2700 and no guest output. No additional VM power, guest, or share changes were made. See `docs/evidence/issue9-utm-windows-state-20261024.json`.

### UTM Z: transfer checkpoint (2026-10-23)

The exact issue #9 PR head `d1e1b3746b3ea08cbaa09c1ac167e0c569c9810d` is available on the user's existing mapped Windows share as `Z:\AgentConductor-d1e1b374.tar.gz`. Host-side source and share-copy SHA-256 match: `6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd`. Windows still displays a restart-apps confirmation. Do not claim guest visibility or a Windows build until the archive is opened and verified from inside the guest. No restart/shutdown was selected. Evidence: `docs/evidence/issue9-utm-z-share-head-20261023.json`.

## UTM Z: mapping and guest-agent boundary — 2026-10-23

The exact source archive `AgentConductor-d1e1b374.tar.gz` is verified in the user's existing host share directory `/Users/neodamul/Downloads/UTM` (SHA-256 `6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd`). The Z: mapping was not changed. UTM `file pull` talks to the guest agent, while Windows mapped drive letters are logon-session-specific; a guest-agent path-not-found response does not mean the interactive user's Z: mapping is off. UTM remains started with input capture off. The current guest screen visibly shows the Guest Tools restart-apps prompt; no VM restart/shutdown or Windows build is verified. See the live screen recheck above. Sources: [UTM Windows guest support](https://docs.getutm.app/guest-support/windows/) and [Microsoft drive-mapping guidance](https://learn.microsoft.com/en-us/windows/win32/services/services-and-redirected-drives).

### Windows signal-settings save oracle — 2026-10-28

Exact-package run `40527176678` passed the Windows workspace save, Team Briefing, and scheduled retry journeys. Its signal-settings driver failed after Save because it treated any remaining `Local Rust` UIA element as an open dialog and did not report the preference file. The failure could not distinguish an unsaved preference from a stale/off-screen accessibility node. The follow-up driver first waits for `signal-transport.json` to contain `agentmemory` and port `7333`, then checks only visible in-window controls and reports the persisted value plus the bounded control tree. Local driver/layout tests pass; exact Windows package verification is still pending, so workflow acceptance remains unchanged.

The Windows package in PR #192 run `40529297038` exposed an earlier failure in the same journey: after real keyboard input, the port edit still read `3111` instead of `7333`, so Save was never reached. The log did not establish whether the field lacked focus or the key events were ignored. The test now verifies the target edit owns focus and uses checked `SendInput` events, with Ctrl+A sent in one ordered batch. The #9 run `40536122604` validated the superseded driver and was cancelled before package validation; do not count this workflow until a package containing the fix passes.

Corrected source commit `62876116` is under validation in run `40539263133`. macOS build, source layout, applicable tests, native UI launch, formatting, Clippy, and unsafe-block checks all passed. Windows build and source layout passed; its full test suite is still running. The exact Windows package has not yet verified port persistence or Settings closure, so packaged acceptance remains 22/25.

### Windows scheduled retry must prove controller dispatch — 2026-10-28

Exact Windows package run `40636799942` failed the schedule journey after the driver reported a foreground-checked pointer click. The accessible `Schedule retry` button remained enabled, the task stayed at `manual_required` with no `available_at`, and the run directory contained no `coordinator-ui-*` action owner record. This localizes the failure before the workflow schedule handler: the UI event was never dispatched. The native Room/controller integration test already proves that a structured schedule action persists the requested time, so this run does not justify changing schedule persistence logic.

The driver now sends checked Win32 `SendInput` down/up events only after foreground and target-process checks, and it must observe the action button enter its pending/disabled state before printing its submitted marker. The exact package test independently requires a controller `owner.json` record before polling persisted `manual_schedule`; this prevents a fake click or success marker from being mistaken for workflow acceptance. Focused local suites pass 13 tests, Clippy with warnings denied, rustfmt, diff checks, and the 300-line limit. Exact Windows package rerun is pending; candidate acceptance remains 22/25 and shipped Rust replacement 0/25. Evidence: `docs/evidence/issue9-windows-schedule-dispatch-40636799942-20261028.json`.
