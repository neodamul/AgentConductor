## Windows packaged Schedule action did not dispatch — issue #9 / 2026-10-25

Exact-head Platform Validation run `40112817061` on `a829db4f9b6c81be54674bad10d64c64f3bce9d4` passed macOS validation/package and Windows validation. The Windows native-package job built the release executable and passed `tests/windows_desktop.rs`, then failed `exact_windows_package_schedules_a_waiting_agent_room_task`: after UI Automation invoked Schedule retry, the run stayed `waiting_availability` with `manual_required` and no `available_at`; no `coordinator-ui-*` action job was created. The test proves the package opened and controls were present, but not that its click dispatched. This is a real unresolved packaged UI interaction. Acceptance remains **20/25**. Do not close #9 or count Workflow 20 until a later exact-package run proves persisted `manual_schedule` and the requested offset timestamp. Issue #149 is independent: all four agreed implementation steps are complete, and it was closed per its original completion rule. Run: https://github.com/neodamul/AgentConductor/actions/runs/40112817061.

## Workflow 20 Windows schedule-control regression — issue #9 / 2026-10-24

Windows validation run `40094613058` on source `db96a57158a4a8dddf028b88691ec9e83075694a` failed only in `tests/windows_schedule.rs`: UI Automation found the schedule editor but Windows marked it disabled, so `$time.SetFocus()` raised `ElementNotEnabledException`; the exact package job was skipped. A new AccessKit regression reproduces the cause when the workflow job is running but the selected task is `waiting_availability`. The schedule editor and button were incorrectly gated by global `editable`; they now remain enabled while that task waits and no action is pending. Red reproduced; local Green passes `native_ui_task_scheduling` (4), `native_ui_task_scheduling_interaction` (1), `source_layout` (2), warning-denied Clippy, rustfmt, and diff checks. Graphify refreshed to 9,404 nodes / 15,859 edges; CocoIndex CLI is unavailable. Source/test files are 296/260 lines. Exact Windows package UI recheck remains pending; this does not increase packaged acceptance beyond 20/25. Evidence: `docs/evidence/issue9-windows-schedule-regression-20261024.json`.

## Latest UTM Windows observation — issue #9 / 2026-10-24

After the user's Z: clarification, the stopped VM was started without reset or configuration changes. It reached the Windows `neodamul` sign-in screen. An empty Enter attempt was rejected; no credentials were entered or retried. The guest remains at the sign-in/error screen, so `Z:` and the Windows package are still unverified. This supersedes the earlier display-not-active observation but does not count as Windows package UI acceptance.

## Isolated Rust default startup — 2026-10-24

The source-built macOS binary at issue #9 head `99cb31aa` was bundled with a disposable unique bundle ID and launched with no arguments, an isolated `HOME`, and an empty workspace. The native default Workspace window appeared with an empty agent list and the registration prompt. The app process was stopped and the temporary bundle, HOME, and workspace were removed. No provider CLI or user profile was configured, and no UI controls were activated. This verifies the Rust default startup path but does not count as a packaged workflow or prove startup under the production bundle ID; packaged workflow acceptance remains 20/25 and shipped/default Rust UI acceptance remains 0/25. Evidence: `docs/evidence/issue9-default-ui-isolated-startup-20261024.json`.

The previous CUA attempt that bound to the shared production bundle ID selected the older installed app and displayed the user's profiles/workspace. No controls were clicked and no settings/workspace/approval/merge changed; automatic quota refresh remains unverified. Do not repeat that binding. UTM currently does not boot or display; leave the VM untouched. Windows runtime/UI verification remains blocked.

## macOS package UI launch crash triage — issue #9 / 2026-10-24

Two crash reports from separate v0.1.37 arm64 bundle IDs share the same signature: `SIGABRT` in `___RegisterApplication_block_invoke` while AppKit constructs `NSApplication`; each also has a background LaunchServices `_LSCopyLocalDatabase` frame. The crash occurs before the eframe app callback can render a window. The package bundle's plist is valid and its arm64 executable is present and mode 0755. This is evidence of a host AppKit/LaunchServices registration failure, not proof of a product rendering bug; the underlying OS failure remains unknown. No product code or host LaunchServices database was changed. UI acceptance remains **20/25** and shipped/default Rust UI **0/25**. Evidence: `docs/evidence/issue9-macos-ui-launch-crash-20261024.json`.

## Full locked/offline verification — issue #9 / 2026-10-24

At branch head `3ae08e2db6cbf0c02c74ac390fdbc9e3218ae889`, `cargo test --locked --offline` completed locally with **963 passed, 14 ignored, and 0 failed across 321 test binaries** (exit 0; 245.25 seconds). This is host/default-feature test evidence; it does not count as a packaged UI workflow, prove Windows runtime behavior, or establish release readiness. Packaged UI acceptance remains **20/25 (80%)** and shipped/default Rust UI remains **0/25**. Evidence: `docs/evidence/issue9-full-suite-3ae08e2-20261024.json`.

## Workflow 20 manual retry scheduling — package UI attempt stopped safely

The v0.1.37 macOS package CLI and a Rust `quota-once` fixture produced a disposable `waiting_availability` run with `manual_required`. Current-source `native_ui_task_scheduling` passes 3/3, including explicit timezone and persisted schedule dispatch. The attempted UI inspection did not verify the workflow: CUA selected an AgentConductor Workspace by display name, showing preconfigured profiles and saved workspace data instead of the requested Agent Room. No UI action or user data mutation occurred. The run and isolated fixture were stopped and removed. Because the displayed Workspace exposed quota-refresh timestamps, automatic profile quota queries may have run; this is unverified. Packaged UI acceptance remains **20/25 (80%)**, and shipped/default Rust UI remains **0/25**. A follow-up attempt using a temporary package copy with a unique bundle ID aborted with signal 6 (exit 134) before CUA exposed a window; the disposable run/root were then stopped and removed. Resolve reliable package launch and window binding before another UI attempt. Evidence: `docs/evidence/issue9-workflow20-package-ui-attempt-20261024.json`.

# Full-product completion audit



## Native manual AccessKit text regression — issue #9 / 2026-10-24

A source-level regression confirms the native manual's heading, paragraph, table cells, code and screenshot alternative text are present in AccessKit updates. Ordinary text is carried as `TextRun.value`, not `node.label`; the initial assertion was corrected after examining the tree, and the test passes without a product-source change. The exact v0.1.37 macOS package visually rendered tables and bundled Workspace/Agent Room screenshots in an isolated window. Manual-related tests passed 23/23; public-doc and source-layout tests passed 13/13; formatting passed. This adds no packaged workflow credit: acceptance remains **20/25 (80%)**, shipped/default Rust UI **0/25**. The interactive macOS CUA tree exposed only the manual container, so VoiceOver behavior remains unverified; Windows Narrator and Korean IME were not tested. Evidence: `docs/evidence/issue9-manual-accesskit-text-20261024.json`.

## Workflow 21 exact-package activity, logs, and lessons — issue #9 / 2026-10-24

The exact v0.1.37 macOS arm64 package from source `6ca19965` showed normalized task activity, a separate Raw logs tab with JSONL stdout/stderr, and the task-specific cause/action in Lessons. The deterministic mock run remained `waiting_approval`; no decision, feedback, reassignment, or merge was sent. Focused activity/lessons/layout tests passed 9/9. The package binary hash matches the workflow 19 evidence. This advances packaged UI acceptance to **20/25 (80%)**; shipped/default Rust UI remains **0/25**. The app window was closed and the copied fixture removed. No provider CLI ran and no screenshot file was retained. Windows guest boot/display, Windows package execution, Korean IME, and screen-reader acceptance remain unverified. Evidence: `docs/evidence/issue9-workflow21-task-activity-20261024.json`.

## QAS-29 exact-package plan-gate verification — issue #9 / 2026-10-24

An older package observation found that Generate plan could open Review without a coordinator. Product source `db568c3f` contains the gate from `4a114ec`; its rendered regression passes 1/1. The exact v0.1.37 package from Actions run `40011350882`, in an isolated HOME and disposable repository, kept Generate plan disabled without a coordinator. **Write plan manually** created three empty tasks without contacting an LLM or starting a process. Confirming the incomplete draft was rejected before invoking the registered `/usr/bin/true` profile. Evidence: `docs/evidence/issue9-qas29-plan-gate-source-20261024.json`. This is already represented in the packaged acceptance count and does not increase 17/25.

## QAS-158 live snapshot lock regression — issue #9 / 2026-10-24

An exact v0.1.37 package UI run with one live and one pending deterministic mock task showed a persistent pending spinner and disabled controls. The source regression reproduced the snapshot exceeding two seconds while the coordinator lock was held. Snapshot watcher restoration now uses the loaded report and SQLite snapshot connections are read-only. Package ea672184 exposed a separate disabled feedback editor; an AccessKit test failed before and passes after the editor was enabled only for a running task. The exact package from commit `6aa687a3` then showed live/pending status updating, allowed entering feedback for the running task, kept feedback disabled for the pending task, and enabled Assign after selecting the alternate agent. No feedback was sent and no reassignment was applied; the fixture run was stopped cleanly. Its ZIP SHA-256 is `c81ef8e1924fa7c22a9e4d6262a15060a6dca419ca6be61d02d15ebbe14eaff2`. Local checks pass: 25 `gates`/`ui`/`source_layout`/`public_docs` tests, availability (1), native task controls/probe (17), AccessKit editor (1), all-target Clippy, formatting, and source/test line limits. Graphify refreshed at 9,278 nodes / 15,677 edges. CocoIndex could not start because its CLI environment lacks `mcp.server.fastmcp`; no host dependency was changed. Packaged UI acceptance is now 18/25 (72%); shipped/default Rust UI remains 0/25 because PR #22 is still unmerged. Windows remains unbootable with no display; no UTM operation was performed. Evidence: `docs/evidence/issue9-qas158-live-snapshot-20261024.json`.

## Exact-package task session/model workflow — issue #9 / 2026-10-24

Built and validated the v0.1.37 macOS arm64 package from source commit `6ca19965`. In its exact Agent Room window, an isolated `/usr/bin/printf` JSONL fixture exposed `fixture-session-19` and the observed `fixture-small`/`fixture-large` catalog. Selecting and saving `fixture-large` updated the task through the Rust dispatcher; the read-only `sessions` command confirmed the saved model and session ID. The task's intentional failure evidence was invalidated, the run returned to ready, and then was stopped. The fixture made no external calls and touched only `/private/tmp/ac-issue9-workflow19-6ca19965`. Package manifest validation and ZIP integrity pass; `native_ui_task_controls` passes 4/4. Evidence: `docs/evidence/issue9-workflow19-session-model-20261024.json`. This advances distinct packaged UI acceptance to **19/25 (76%)**; shipped/default Rust UI remains **0/25**. The deterministic session/model values do not verify a live provider session or model catalog.

## Live checkpoint — issue #9 / 2026-10-24

PR #22 is open and draft at pushed head `6ca19965a324db55f24015fb84bdf91f2523501e`. Platform Validation run `40011350882` passed macOS and Windows validation and built both package artifacts from product source `db568c3f59fea29b15c8d60829e7fd41857fc62e`. Exact v0.1.37 package checks now include repository picker selection/cancel, QAS-29's missing-coordinator gate and incomplete-plan refusal, and workflow 19 session/model selection. Evidence: `docs/evidence/issue9-uc29-source-verification-20261024.json`, `docs/evidence/issue9-qas29-plan-gate-source-20261024.json`, `docs/evidence/issue9-workflow19-session-model-20261024.json`, and `docs/evidence/issue9-workflow21-task-activity-20261024.json`. Packaged UI acceptance is **20/25 (80%)**; shipped/default Rust UI remains **0/25**. The latest local all-target test attempt was interrupted (exit 130) and is not claimed as passing.

The user reports the Windows guest does not boot and has no display. UTM's current UI is the new-VM setup sheet, not an open Windows guest; host access to the named share folder was denied. Earlier post-reset `utmctl` IP/file-read observations are historical and do not establish that Windows currently boots. Windows UI, `Z:` visibility, and package execution remain unverified. Do not issue additional power, reset, or share-configuration actions. See `docs/evidence/issue9-utm-windows-state-20261024.json` and `knowledge/wiki/log.md`.

The workflow 19 package process and its disposable fixture are removed. Current `native_ui_task_controls`, `public_docs`, and `source_layout` pass 4, 11, and 2 tests respectively. Keep issue #9 open and PR #22 draft until the remaining UI, shipped-default, Windows, accessibility/IME, resource, license, and release gates are verified.

## Cross-platform package verification after Windows safety-gate fix — issue #9 / 2026-10-24

Platform Validation run `40011350882` on source `db568c3f59fea29b15c8d60829e7fd41857fc62e` passed every macOS and Windows validation check, including the previously failing Windows unsafe-comment gate. Both native-package jobs built release binaries, rendered the English and Korean manuals with Rust, executed the exact package smoke, and retained artifacts. macOS artifact `12527489015` is 21,164,379 bytes (artifact SHA-256 `faf6d2a0acea970638e4c59697d77175737c44e8dd79fc51ccc8971ca0703a79`; package archive SHA-256 `47075ce7a6908926d14d049e8613a02bce862170ff58bf3992b1c17f41ef7629`). Windows artifact `12528158860` is 15,301,355 bytes (artifact SHA-256 `02bc04f6cd86b4e9bdaa96c6d05bac58d46a94b5ee6b377dbaa1adf546b7e856`; package archive SHA-256 `9927f872a25505804c3f07ccff70ef33e318c49fdab83d2e15fe599c5a19e8d1`). Both artifacts expire 2026-11-06. These are Actions artifacts, not a GitHub Release. Exact packaged UI acceptance is 16/25 and shipped/default Rust UI remains 0/25; overall issue #9 and release readiness remain open. Evidence: `docs/evidence/issue9-platform-packages-40011350882-20261024.json`.

## Windows unsafe-comment gate repair — issue #9 / 2026-10-24

Actions run `40008320185` passed Windows build/layout, all-target tests, native UI validation, formatting, and warning-denied Clippy; its final unsafe-comment gate failed at the Windows-only `SystemParametersInfoW` block in `src/native_ui_probe/motion.rs:54`. Added one `SAFETY` comment describing the valid writable BOOL result pointer in commit `c6c6052`; behavior is unchanged. Local rustfmt, host Clippy, the 2-test source-layout suite, and `git diff --check` pass. The follow-up run `40011350882` passed all Windows and macOS gates and both exact-package workflows; see the cross-platform package evidence above. Evidence: `docs/evidence/issue9-windows-unsafe-comment-fix-20261024.json`.

## Exact-package workspace create/archive workflow — issue #9 / 2026-10-24

The exact v0.1.37 macOS arm64 package from pushed source `278ad6de798e73e2a71502af03117c0004d61e92` saved a named workspace for a disposable plain folder with Git worktrees disabled, then archived it after a replacement workspace became current. The explicit confirmation completed; the old workspace disappeared from the registry, archive metadata records `confirmed_archive`, and the disposable project folder remained intact without `.git` during the check. The isolated app had no agents; no plan, task, merge, or user data was used. After verification, the app and the entire disposable temporary root, including its project fixture, were removed. This distinct operator flow advances packaged UI acceptance to **16/25 (64%)**; shipped/default Rust UI remains **0/25**. Evidence: `docs/evidence/issue9-workspace-create-archive-278ad6de-20261024.json`.

## Rust-only authored source inventory — issue #9 / 2026-10-24

The current tracked source inventory contains no authored JavaScript, Python, CSS, or HTML application UI under `src/`, `tests/`, `scripts/`, or `examples/`; `source_layout` passes both Rust-only and ≤300-line checks. The tracked `docs/user-manual.html` and Korean counterpart are generated offline documentation from the Rust manual renderer and remain release assets. Keep `tests/native_ui_assets.rs`: it is a negative regression preventing legacy browser UI assets from returning, not a dependency on those assets. This corrects a stale plan item and adds no packaged workflow or release acceptance. Evidence: `docs/evidence/issue9-rust-source-inventory-20261024.json`.

## Exact-package named-workspace lifecycle recheck — issue #9 / 2026-10-24

Exact macOS package source `f23381cf` was bound by its bundle path and run with a temporary HOME. The Workspace menu created two named workspaces; saving, switching, and reopening restored each name, repository folder, and goal. Removing the inactive first workspace from the recent list kept its folder and allowed it to be reopened from the retained saved-workspace entry. No CLI agent, MCP server, external service, plan, task, archive, or user workspace was touched. The single app process and all temporary files were removed after the check. This rechecks already accepted UC-30/UC-31 coverage and adds no workflow credit: packaged acceptance stays **15/25**, shipped/default Rust UI **0/25**. Evidence: `docs/evidence/issue9-f23381cf-named-workspaces-20261024.json`.

## Exact macOS package Settings spot-check — issue #9 / 2026-10-24

The exact `f23381cf` package previously displayed the gear-only Settings control in light and dark themes, and its Settings menu opened with the accessible name present. A follow-up launched the package binary with a temporary `HOME` and disposable `ui-new` root, but CUA lookup by display name resolved older package `91393d7b` and showed saved user workspace content. Testing stopped immediately; no field, setting, task, approval, or merge was touched, and the stale app was quit. No screenshot was retained and tooltip hover was not verified. This was not an isolated workflow test and earns no acceptance credit. Packaged UI acceptance remains **15/25** and shipped/default Rust UI remains **0/25**. Evidence: `docs/evidence/issue9-settings-gear-f23381cf-20261024.json`.

## UTM Windows guest state — issue #9 / 2026-10-24

The user confirmed that guest `Z:` already maps to `/Users/neodamul/Downloads/UTM`. After the user approved the explicit data-loss warning, UTM's Restart control reset the VM. UTM reports `started`; the display showed UEFI boot output, then `Display output is not active`. A later `utmctl ip-address` returned IPv4 `192.168.64.6` and IPv6 addresses, but the display remained inactive. One read-only `utmctl file pull` of `C:\Windows\win.ini` aborted with signal 6 (exit 134); an earlier staged-package pull did the same. macOS denied listing the host share folder. A temporary guest marker's cleanup could not be verified after guest I/O failed. Guest desktop access, Z: visibility, and Windows execution remain unverified. No share setting changed and no further power action was taken. Evidence: `docs/evidence/issue9-utm-windows-state-20261024.json`.

Read-only desktop recheck on UTM 5.0.6 (125) still shows `Display output is not active`. `utmctl exec` with `cmd.exe /c ver` returns OSStatus -2700 / child-process-not-found without guest output. No share, VM power, or guest changes were made during this recheck; Windows package and `Z:` verification remain open.

## Current PR #22 macOS lint checkpoint — issue #9 / 2026-10-23

At source head `6bf54f3d`, Actions run `39956497286` completed the macOS build, native tests, opt-in UI launch check, source layout, and formatting successfully; warning-denied Clippy failed on `manual_contains` in `tests/native_ui_task_activity.rs`. The test now uses `labels.contains(&result)`. On local Rust 1.99, the focused test (7), all-target Clippy, formatting, source-layout (2), and diff checks pass. The Windows job for the pre-fix synthetic merge was still executing at 13:11 UTC and is not evidence for the fix. The exact fix head still needs a fresh remote cross-platform run. Evidence: `docs/evidence/issue9-pr-clippy-fix-20261023.json`. AgentMemory: `mem_mvkzmzgs_5e9246d8aa0a`.

This lint correction does not change the product workflow count: packaged native UI acceptance remains **15/25 (60%)** and shipped/default Rust UI remains **0/25**. Issue #9 remains open and PR #22 remains a draft pending the wider UI, Windows, accessibility/IME, resource, integration, and release gates.

## Live UTM Windows screen recheck — issue #9 / 2026-10-23

UTM reports the Windows VM as `started`; the Windows restart-apps confirmation is visible and lists UTM Guest Tools Installer, Network File Explorer, and Task Manager. The Z: mapping and UTM share settings were not changed. Input capture was temporarily enabled for a Cancel attempt and restored off; the prompt did not dismiss. No restart, shutdown, or Windows build occurred. Guest Z: visibility and Windows verification remain unconfirmed. Evidence: `docs/evidence/issue9-utm-screen-recheck-20261023.json`.

## Existing UTM Z: share transfer checkpoint — issue #9 / 2026-10-23

The exact PR head `d1e1b3746b3ea08cbaa09c1ac167e0c569c9810d` was archived as `AgentConductor-d1e1b374.tar.gz` and copied to the user's existing Z: share folder, `/Users/neodamul/Downloads/UTM`. Source and destination SHA-256 both equal `6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd`. The Windows guest still shows the app-restart confirmation; guest-side Z: access, Windows build/package/UI, and tests are not verified. No VM restart or shutdown was selected. Packaged UI acceptance remains **15/25 (60%)**; shipped/default Rust UI remains **0/25**. Evidence: `docs/evidence/issue9-utm-z-share-head-20261023.json`.

The user's Windows UTM already maps `Z:` to the host share `/Users/neodamul/Downloads/UTM`; no drive or UTM share setting was changed. A source archive of branch head `c4cbd2a6c13b5ab59f50b1efa37842a54a9c7a9c` was copied to that host folder, and Finder showed `AgentConductor-c4cbd2a.tar.gz` there. Its source SHA-256 is `3d56f88a82e1cd83415a2aca66d140f0e5aedfe53b3a5ce549a6a3e2ef599433`; the destination checksum and guest-side `Z:` visibility are not verified. UTM remains started with input capture off. Windows displays a restart-apps confirmation; no restart or shutdown was selected. Windows build/package/UI checks remain unverified. Packaged UI acceptance remains **15/25** and shipped/default Rust UI remains **0/25**. Evidence: `docs/evidence/issue9-utm-z-share-transfer-20261023.json`.

## Latest main merge integration checkpoint — issue #9 / 2026-10-23

The #9 branch merged upstream `main` commit `602ec118` from pre-merge source `8b50d0bd` as `d1f5071fb5c47217b09c5283f1b0dd1731483d83` and pushed it to the issue-linked branch. PR #22 remains open/draft with two checks pending (`UNSTABLE`). The 39 conflicting paths were resolved while retaining the Rust-only shipped UI. Rust parity includes cumulative eight-path charter growth enforcement, full-plan charter reset after operator-approved scope recovery, and the revision-bound Agent Room amendment-undo route. The focused offline suite passes **69 tests across 15 suites**; rustfmt, diff checks, merge-marker scan, and the 300-line source/test limit pass.

The user reports that UTM Windows already has a Z: network drive mapped. The VM remains `started`; the QEMU Guest Agent is unavailable, and no Windows build, package, or UI check has run. Guest Tools installation is authorized but not needed unless transfer through Z: proves unavailable. Packaged native UI workflow acceptance remains **15/25 (60%)**; shipped/default Rust UI remains **0/25**. Evidence: `docs/evidence/issue9-main-merge-parity-20261023.json`.

## Previous Windows UTM transfer checkpoint — issue #9 / 2026-10-23

The latest product/test code checkpoint remains `72db2aa2d2f3bf9664536c1839047366c0d5decf`; current branch head is the documentation checkpoint `8b50d0bdddb04281e37b21c9177cc73f15beb1ff`. Current live checkpoint at branch head `8b50d0bd` reports PR #22 open/draft, `DIRTY`, and no checks. GitHub CLI confirms the PR targets `main`. `origin/main` is `602ec118`; v0.1.37 is the latest published release, while this branch is v0.1.36. Recheck and align the next stable version before packaging.

A fresh read-only `git merge-tree --write-tree --name-only HEAD origin/main` reports **39 conflict paths** across product code, tests, public/design documentation, generated manuals and Wiki files; no merge was attempted. The branch is 719 commits ahead and 17 behind its merge base. The earlier note that only the release changelog conflicted was incomplete.

The existing Windows ARM64 UTM VM is still started. `utmctl ip-address` fails with OSStatus -2700 because the QEMU Guest Agent is unavailable. A source-only archive for the exact branch head is prepared at `/private/tmp/agentconductor-windows-8b50d0bd.tar.gz` (SHA-256 `94a0621b4ee4d110931455763e6220e8767a7afc08d555aa0fdb316581ca7a90`), but it has not been transferred. Guest Tools installation, which enables host-to-guest transfer/control, remains gated on the previously requested action-time authorization. Windows showed a restart notification; no restart or shutdown occurred. No Windows build, package, or UI verification is claimed. Evidence: `docs/evidence/issue9-utm-build-transfer-20261023.json`.

Packaged native UI workflow acceptance remains **15/25 (60%)**; shipped/default Rust UI replacement remains **0/25**.

Focused checks for this audit checkpoint pass: `issue9_current_status` (1), `source_layout` (2), and `public_docs` (11); evidence JSON validation and `git diff --check` also pass. These documentation checks do not advance UI acceptance.

## Earlier exact-package/workflow checkpoint — issue #9 / 2026-10-23

The latest product/test commit on issue #9 is `72db2aa2d2f3bf9664536c1839047366c0d5decf`; subsequent branch commits update documentation only. A GitHub CLI query at branch head `575fef74d160d6050a878e89d3bafa7f831432f5` confirmed PR #22 is open/draft, targets `main`, and is `CONFLICTING`. Packaged native UI workflow acceptance remains **15/25 (60%)**, while the shipped/default Rust UI replacement remains **0/25**. Workflow 15's v0.1.36 macOS arm64 package UI was repeated with a verified disposable `HOME` and direct executable path: the menu showed the isolated workspace root and no agents, the bundled manual rendered its Agent Room image, and the release-notes panel reached its final item. No workspace save, task, approval, update, merge, or agent quota query occurred. The full locked/offline native-feature suite passed **943 tests, 14 ignored, 0 failed across 312 suites** at that earlier checkpoint. The latest screenshot/localization batch passes **32 focused tests across 8 suites**; the new release-package PNG-asset regression passes for macOS and Windows assembly fixtures. Warning-denied Clippy, formatting, and diff checks pass. These documentation and fixture changes do not advance packaged workflow acceptance. Windows and exact-package verification of the refreshed manual remain open. Evidence: `docs/evidence/issue9-workflow15-ui-isolated-20261023.json` and `docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.

## Current-source package check — issue #9 / 2026-10-23

The v0.1.36 macOS arm64 package built from source `575fef74d160d6050a878e89d3bafa7f831432f5` passes `validate-package` with 242 dependencies. Its bilingual HTML manuals and English/Korean Agent Room PNGs have the same SHA-256 hashes as the current repository assets. After discarding an initial wrong-window selection, the exact packaged app was launched with an isolated `HOME` and workspace; its native User manual dialog rendered the dark Korean Workspace screenshot and the English Agent Room screenshot showing three completed tasks, three passing tests, and pending human review. This confirms both screenshot asset paths render in the exact macOS package; it adds no workflow acceptance credit, which remains **15/25**. Windows exact-package verification remains open. Evidence: `docs/evidence/issue9-current-source-package-manual-20261023.json`.

## Current Workspace screenshot — issue #9 / 2026-10-23

The product/test checkpoint remains `72db2aa2d2f3bf9664536c1839047366c0d5decf`; subsequent branch commits update documentation and evidence only.

The README and English offline manual use a current-source 1140×740 empty Workspace screenshot.
The Korean manual uses separate light and dark Workspace images. The English and Korean manuals
now also use matching 1140×960 Agent Room images from a source-built Rust probe, captured after a
deterministic three-task run reached human approval. Both HTML manuals and the native viewer embed
the language-matched screenshots; regression tests compare the static HTML with Rust-rendered output.
These source-build assets add no packaged workflow credit, and exact-package/manual verification
remains open. Evidence:
`docs/evidence/issue9-readme-workspace-screenshot-20261023.json` and
`docs/evidence/issue9-korean-workspace-screenshots-20261023.json` and
`docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.

## Native manual link-handler fix — issue #9 / 2026-10-23

Exact-package UI follow-up at docs head `6b9641a1f6e47c9601618747a2b842b6e1b96da9` confirmed the bundled v0.1.36 macOS arm64 manual rendered and its visible official configuration-reference HTTPS link opened the expected Safari page. Archive SHA-256: `7572e511897ee73f01bcbfb53c5c19fc9ff423697611099b1a36bcb89206cb72`. The app was closed without saving workspace changes and the test-created Safari tab was closed; startup may have refreshed quota metadata. This proves one manual link interaction only and does not add separate workflow credit. Unsafe-scheme UI behavior remains source-test-only.

At source commit `d539e86f1d099279e7a80fe8f9753b9065621631`, safe links in the native manual now reach eframe's OS-browser handler: TDD reproduced that egui-winit drops `OpenUrl` when eframe's `links` feature is disabled, then verified the feature-on configuration and the existing HTTP(S)-only renderer policy. Focused release tests pass 14/14; warning-denied Clippy, rustfmt, diff check, and the macOS arm64 release build pass. `package-release` built v0.1.36 from this exact source, passed all four deterministic smoke modes, and `validate-package` confirmed 242 licensed dependencies, including bundled MIT/Apache texts for `webbrowser 1.2.5`. The binary grows by 235,136 bytes relative to `ec1fc3c1ac06`. At this source checkpoint package UI had not yet been checked; the later exact-package follow-up confirms the safe link opened in Safari (see `docs/evidence/issue9-package-manual-link-ui-20261023.json`). Packaged workflow acceptance remains **14/25**, and shipped/default Rust UI replacement remains **0/25**. PR #22 remains open/draft and was non-mergeable against `main` at the last remote check; Windows, accessibility/IME, remaining screens and workflows remain open. Graphify's raw refresh had 1,271 dangling edges, so the reviewed 9,035-node/14,220-edge/zero-dangling snapshot was restored. CocoIndex CLI is unavailable on this host. Evidence: `docs/evidence/issue9-native-manual-link-backend-20261023.json`.

The exact v0.1.36 macOS arm64 package at source `ec1fc3c1ac06f73d813911e7cc01f2779d9887cf` rendered the built-in manual's Agent Room screenshot and showed an automatic v0.1.37 update notice with scrollable release notes. Package validation confirmed binary SHA-256 `1409c4154a1cdf2c56283ee0a88f1572aca7bd197ad83e99f8ab35738b7e5aeb` and 214 dependencies; the embedded screenshot matches the repository asset SHA-256 `f65b84b8dd37090d2ead0ce44428f72ac4f791fb8fac2945653e940a1f53053b`. The test began with a temporary HOME, but saved user workspace/agent state later appeared and the effective app profile root was not proven. No setting, task, update, or merge was applied; a startup quota refresh cannot be ruled out. Exclude this attempt from acceptance. Packaged workflow acceptance remains **14/25**. Evidence: `docs/evidence/issue9-workflow15-ui-isolation-20261023.json`; AgentMemory `mem_mvkngork_57cb5ccae19a`.

Exact-package follow-up at source `c92e2b49658b26da2153cc743fb99b23e0aa7e28`: the v0.1.36 macOS arm64 UI completed a disposable three-task worktree run. All tasks and tests passed (3/3, exit 0, no timeout); candidate `cb180bde36e446ab70ad41ecab242cb74c2d3464` remained `waiting_approval`. The UI loaded review evidence with the candidate hash, test receipts/stdout and diff; no approval or merge was performed. Evidence: `docs/evidence/issue9-c92-package-approval-review-20261023.json`; AgentMemory `mem_mvkkhvmh_c8831ccc7e52`. The screenshot was displayed during inspection but was not retained as a repository file. This repeat verification does not increase packaged workflow acceptance.

The native manual table layout defect is fixed at `d051c8d4`: a column-width regression failed at 48 px, and a wrapped-row regression exposed a 77 px overlap. The exact v0.1.36 macOS package (binary SHA-256 `7efb41e3c20001b60282772fff4f7ccc372a44eea4b7e62451030cc70df7a36d`, archive SHA-256 `678b9fa141bc8a63fbbbf6e182fcb1c6d99f40ffe6b904fe443556b16ed79f6d`) passed validation and all four smoke modes; visual inspection confirmed the table uses the window width and rows do not overlap. Focused manual/layout tests pass 13/13. This does not change packaged workflow acceptance (14/25). The manuals now use a retained source-build Agent Room screenshot; it is not exact-package evidence.

Packaged native UI operator acceptance is **14/25 (56%)**; shipped/default Rust UI replacement remains **0/25**. Issue #9 is open and PR #22 remains draft. Current branch and PR #22 head: `codex/issue-9-native-ui-parity` at `ec1fc3c1ac06f73d813911e7cc01f2779d9887cf` (documentation checkpoint); latest product source commit: `25556157` (opt-in screenshot callback); native manual table fix: `d051c8d4d7cdcb583b6591a8aec0f37db214b246`; native manual safe-link source fix: `20377bdc07a842735e24eacbeeb940aabee4fd9f`; exact package source: `ec1fc3c1ac06f73d813911e7cc01f2779d9887cf`; exact package: v0.1.36 macOS arm64.

- **Native manual safe links — source-level fix:** commit `20377bdc07a842735e24eacbeeb940aabee4fd9f` reuses the standalone manual renderer's URL allowlist so links in native paragraphs, lists, table headers, and cells open safely; unsupported schemes remain plain text. The focused native manual, renderer, and source-layout suites pass 11 tests; warning-denied focused Clippy, rustfmt, and diff checks pass. Evidence: `docs/evidence/issue9-native-manual-safe-links-20261023.json`. Exact-package UI follow-up confirms the safe external link opens the expected Safari page, but this does not add packaged workflow acceptance; unsafe-scheme UI behavior remains unverified. Current bilingual manual screenshots are retained from the source build in `docs/evidence/issue9-native-agent-room-manual-screenshot-20261023.json`.
- **Native manual safe-link package build:** current source head `8c8a08a8` builds a checksum-valid macOS arm64 v0.1.36 package (binary SHA-256 `60c1295a7cae1d4341ceb0370e6654a64ff7144eef12533c74e394d5e56b111d`, archive SHA-256 `0b05c48941b7e5de6910b8506932d1c459046ec8136a140dbb1cc61716b4bea2`). Its SBOM has 214 components with zero missing license expressions; the package includes `licenses/`, dependency-license, and third-party-notice files. All four deterministic smoke cases pass, including 3/3 parallel tasks/tests and refusal of unapproved merges. Evidence: `docs/evidence/issue9-native-manual-links-package-20261023.json`. The app was not launched because the only visible AgentConductor window had switched to the user's saved workspace; the native manual interaction remains source-test-only, not exact-package UI evidence. No workflow credit.

- The full locked/offline native-feature suite at product commit `f4388209` passed **938 tests, 14 ignored, 0 failed across 310 test binaries**. Evidence: `docs/evidence/issue9-full-regression-f438820-release-20261023.json`. Latest docs/source-layout tests at `8f560038` passed 13/13; `git diff --check` passed.
- The exact packaged Claude quota refresh UI returned fresh metadata. Account values were not recorded. The package binary SHA-256 is `f804e8c8406408f901c7103fed4f341fc0ceedfb1a1e44506d2e9ba5457fc3ec`; archive SHA-256 is `8355d7bf0a86c7497ec9cc9775e095e5294133607defca07a1a5820a88887972`.
- Initial 15-second NFR-05 visible-idle sample: one AgentConductor process, no children; four five-second samples show 71–75 MiB physical/resident memory and 0.0–0.4% CPU (0.275% average). `footprint` reports 71 MB, within QAS-58's 100 MB target. Since-launch peak was 178 MB; startup/active peak, sustained idle and Windows remain unverified. Evidence: `docs/evidence/issue9-f438820-idle-resources-20261023.json`.
- 30-second follow-up on the same exact v0.1.36 visible idle Agent Room: seven five-second samples show CPU 0.0–0.4% (0.3286% average); `footprint` reports 73 MB, below the QAS-58 100 MB target. `top`'s first rsize reference was 175 MiB, then 73–75 MiB; shared-page RSS is not the QAS-58 metric. Since-launch peak is 180 MB. Evidence: `docs/evidence/issue9-f438820-idle-resources-30s-20261023.json`. This passes the 30-second idle target only; startup/active peak, longer idle and Windows remain open.
- A separate exact v0.1.36 package at source `c92e2b49` was observed in a visible idle Workspace settings window for 31 seconds: seven samples at 0.0% CPU, 57 MB physical footprint, 165 MB since-launch peak, one app process and no children. Evidence: `docs/evidence/issue9-c92-idle-resources-30s-20261023.json`. The isolated app and temporary data were removed. The current checkout's `target/` is 2.2 GB of retained release artifacts; this sample does not prove the state that previously occupied 35 GB. Startup/active, longer-idle, Windows and final-release measurements remain open.
- A user-authorized disposable mock run produced three disjoint tasks and three passing `git diff --check` results. Candidate `94fe3ada21bddf9086c1bd12679bf066b3fd22e6` remains waiting for approval and was not merged. It is a duplicate workflow and does not change 14/25 acceptance.
- The bilingual manuals now embed the retained 1140×960 Rust Agent Room screenshot from source commit `25556157`; the opt-in app framebuffer capture is documented in `knowledge/wiki/native-screenshot-provenance.md`. It is a development-build image, not an exact release-package capture, and adds no workflow credit.
- Local Windows toolchain is unavailable (`rustup`, Zig and MinGW are absent); the existing UTM Windows VM is stopped. Windows package/runtime/UI/license acceptance remains open.
- Latest-main reconciliation remains open: `main` has documented merge conflicts with the Rust branch. Preserve the Rust-only modular source and keep the PR draft. No release is authorized.
- Graphify refreshed the local code/Wiki graph after the screenshot and evidence updates: 9,035 nodes, 14,220 edges and 686 communities. The graph was rebuilt locally; no labeling LLM was available. CocoIndex is unavailable on this host.

The latest source and documentation checkpoints are pushed through `25556157` and `d6ecd3aa`; the current screenshot, bilingual Markdown/HTML manuals, evidence, Wiki provenance and checklist are committed. Remaining gates: Windows package/runtime/UI/licenses, screen-reader and Korean IME hardware checks, startup/active/sustained resource measurements, merge reconciliation, exact final-source package verification and final-source license review.

### Previous checkpoint — issue #9 / 2026-10-23 (superseded)

Packaged native UI operator acceptance is **14/25 (56%)**; shipped/default Rust replacement remains **0/25**. The latest product-code commit is `eb0bfce73ba25db817436c591daea0c6bd90630b`, which fixes packaging of manual-only image links. The complete locked/offline native-feature suite passed **938 tests, 14 ignored, 0 failed across 310 test binaries** at `12d6d808`; warning-denied all-target Clippy, formatting, and diff checks passed there. Evidence: `docs/evidence/issue9-full-regression-20261023.json`. At exact branch head `88e372e95a8c2507e15c1c8b24743bf7d1f9b38a`, optimized `public_docs` and `source_layout` pass 13 tests, rustfmt and diff checks pass, and the new current-head macOS package validates. The package remains v0.1.36; its 214-component SBOM has no missing license expressions, dependency license texts are bundled, and package smoke passes four isolated cases with three parallel tasks and tests each. Binary SHA-256: `0c4e1fa888cc18d13da40f81683032686829a79ea5411250a2b4ab59c8397a23`; archive SHA-256: `6019a0ff883c5713e519c9932153c14bc1e793918265a80aa1c773efc5868894`. Evidence: `docs/evidence/issue9-current-head-macos-package-license-20261023.json`. A subsequent pushed source commit `f43882095aa0308dc2870bd34269e0ea240bab8d` fixes Claude quota CLI resolution and builds a v0.1.36 macOS arm64 candidate. Package validation and all four isolated package smoke cases pass; binary SHA-256 is `f804e8c8406408f901c7103fed4f341fc0ceedfb1a1e44506d2e9ba5457fc3ec`, archive SHA-256 is `8355d7bf0a86c7497ec9cc9775e095e5294133607defca07a1a5820a88887972`. This build still contains historical manual screenshots and was not opened in the GUI. Its matching-hash executable successfully queried the signed-in Claude CLI with `PATH=/usr/bin:/bin` and persisted two quota windows without displaying values; the packaged UI refresh button remains unverified. Windows and all other listed platform/release gates remain open. Evidence: `docs/evidence/issue9-claude-quota-f438820-macos-package-20261023.json`. Workflow 13 and 14 remain accepted at the earlier exact v0.1.36 package source `9845db79`; their UI acceptance does not advance. The manual screenshot refresh and Windows package/UI/license gates remain open; no release is authorized.

Correction to the package UI status above: the exact f438820 package was opened in a disposable workspace after that checkpoint. Its Claude quota refresh button showed progress, updated the observation timestamp and summary, and returned to the enabled state. The screenshot was not retained because it exposed account usage and a local project path. This verifies one component, not a distinct workflow; acceptance stays 14/25.

The latest checkout (`8a46c8a0181a5d27440a309d9f68e9420037c858`, product code `f43882095aa0308dc2870bd34269e0ea240bab8d`) also passes the full offline, locked, release-profile native-feature regression: **938 passed, 14 ignored, 0 failed across 310 test binaries** in 351.22 seconds. This is source-level regression evidence and does not increase packaged workflow acceptance. Evidence: `docs/evidence/issue9-full-regression-f438820-release-20261023.json`.

Windows remains unverified. The host has no Windows Rust `core`/`std` target. A hidden disposable UTM Windows boot reached `started`, but its QEMU guest agent was unavailable, so IP discovery and `utmctl exec` failed; the VM was stopped and the probe-created window closed. Keep #9 open and PR #22 in draft until current screenshots, Windows package/UI, accessibility/IME, resource measurements, and final-source license review are complete. No release is authorized by this package checkpoint.

The earlier single-window, read-only inspection used the older `cff95dd` bundle; it is historical and does not establish exact-package workflow acceptance. The exact `ca9495b2` package UI journey is recorded above. Before rebuilding from `9845db79`, the Codex worktree `target/` measured 11 GB; idle `debug/incremental` and `debug/tmp` were removed, then the exact-source release build and package were produced. On 2026-10-23, `target/` measured 9.7 GB: 9.0 GB under `debug/` (8.7 GB dependencies), 731 MB under `release/`, and 184 KB for the unavailable Windows target. With no Cargo/Rust build process active, the regenerable `target/debug` tree was then removed; `target/` is now 731 MB and retains the release artifacts. The extracted 42 MB package under `dist/` remains intact; the 21 MB ZIP archive is separate. The packaged executable still exists and its `--help` command succeeds. Evidence: `docs/evidence/issue9-target-debug-cache-cleanup-20261023.json`.

The exact `9845db79` macOS package was measured in the already-running single app window, with no task active. Four `top` samples five seconds apart reported 79 MiB RSS and 0.0% CPU, and the app had no child processes. The executable SHA-256 matches the package evidence (`0c4e1fa…8397a23`). This idle sample is within NFR-05's limits, but does not close startup, active-work, sustained-idle, Windows, or final-source resource validation. Evidence: `docs/evidence/issue9-9845db79-idle-resources-20261023.json`.

### Previously verified v0.1.36 package visuals and acceptance details

The following source and UI observations are historical component evidence from `f1602e7d`/v0.1.36. They do not describe the latest packaged binary or change the current 11/25 acceptance count.

Packaged native UI operator acceptance is **11/25 (44%)**; the shipped Rust replacement remains **0/25**. Workflow 10, coordinator-directed operator feedback, was verified in the exact v0.1.36 macOS package at `b298ae79774e`. Workflow 11 switches the same package's Agent Room from Korean to English and back while preserving the pending candidate, task statuses, and test evidence; evidence is `docs/evidence/issue9-6f1111c-language-status-workflow.json`. The package archive SHA-256 is `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`. The current Rust source `f1602e7d` passes 929 all-target tests (14 ignored across 310 suites), warning-denied Clippy, rustfmt, diff check, and the 300-line source/test guard. Its exact macOS arm64 package validates and passes package smoke; manifest lists 214 dependencies, bundled SQLite, and Git plus selected authenticated agent CLIs as runtime requirements. The package was opened in one window and visually checked: the goal appears once in the wide Agent Room header, while the left column retains task status and verification. The same package's Korean Agent Room was also inspected in light theme: three task rows, candidate SHA, and three passing test results were visible; candidate `3db8840b` remained `waiting_approval`, with no approval, rejection, or merge. Dark theme and the saved user workspace were restored without changing workspace settings. Evidence: `docs/evidence/issue9-f1602e7-agent-room-light-theme.json`. Graphify's retained curated graph has 8,916 nodes, 14,081 edges, and zero dangling endpoints; the no-cluster extraction was not retained because it had 1,271 dangling edges. CocoIndex CLI is unavailable. These component checks do not add workflows. Fourteen packaged UI workflows remain. Windows UI, accessibility/IME, resource measurements, and final-source license review remain open. Keep issue #9 open and PR #22 in draft.

The native MCP registration dialog now follows the existing Settings → MCP servers eyebrow + registration-title hierarchy, with distinct English and Korean window titles. The exact v0.1.36 macOS package from `cff95dd02677519ebbb7c359f5ca9e3da0c504ed` was visually checked in Korean/dark, English/dark, and English/light; archive SHA-256 is `dafc056361c6e68a1165f30db1f3bd8a0cf99d24b59876d7d6eb7d2aa1ad44d8`. The saved workspace was left unchanged, and the candidate was not approved or merged. Evidence: `docs/evidence/issue9-cff95dd-mcp-dialog.json`. The screenshot was not retained. This component correction adds no workflow; acceptance remains 11/25. Windows verification remains open.
Packaged native UI operator acceptance is **11/25 (44%)**; the shipped Rust replacement remains **0/25**. Workflow 10, coordinator-directed operator feedback, was verified in the exact v0.1.36 macOS package at `b298ae79774e`. Workflow 11 switches the same package's Agent Room from Korean to English and back while preserving the pending candidate, task statuses, and test evidence; evidence is `docs/evidence/issue9-6f1111c-language-status-workflow.json`. The package archive SHA-256 is `6ae1d923a7af0485d47314325197bef0d96f9a4cd724c5d07b9df851be4d9be4`. The current Rust source `f1602e7d` passes 929 all-target tests (14 ignored across 310 suites), warning-denied Clippy, rustfmt, diff check, and the 300-line source/test guard. Its exact macOS arm64 package validates and passes package smoke; manifest lists 214 dependencies, bundled SQLite, and Git plus selected authenticated agent CLIs as runtime requirements. The package was opened in one window and visually checked: the goal appears once in the wide Agent Room header, while the left column retains task status and verification. The same package's Korean Agent Room was also inspected in light theme: three task rows, candidate SHA, and three passing test results were visible; candidate `3db8840b` remained `waiting_approval`, with no approval, rejection, or merge. Dark theme and the saved user workspace were restored without changing workspace settings. Evidence: `docs/evidence/issue9-f1602e7-agent-room-light-theme.json`. Graphify's retained curated graph has 8,916 nodes, 14,081 edges, and zero dangling endpoints; the no-cluster extraction was not retained because it had 1,271 dangling edges. CocoIndex CLI is unavailable. README image embeds were removed after confirming the assets showed v0.1.21/v0.1.24 or a prepared demo. Release packaging now discovers local image assets directly from both manuals instead of relying on README links; the retained manual screenshots are still historical and remain due for exact-package refresh. These component/documentation checks do not add workflows. Fourteen packaged UI workflows remain. Windows UI, accessibility/IME, resource measurements, and final-source license review remain open. Keep issue #9 open and PR #22 in draft.

## Exact v0.1.36 post-approval drift refusal — issue #9 / 2026-10-23

The exact macOS package from `aa8090405239` approved the exact candidate in a disposable copy
of the pending mock run. A matching baseline charter existed only in that copy; changing the
stored plan goal then caused the packaged merge command to exit 1 with `[goal_drift]`. The
package persisted one coordinator judgment and did not change the fixture repository HEAD. In
the single Agent Room window, the localized Korean blocked judgment appeared with the completed
3/3 task and test evidence, and Merge was disabled. The original app-managed candidate remained
pending approval with zero drift records and was restored; the temporary run copy was removed.
No provider CLI or external service was invoked. This is a distinct post-approval failure
workflow, advancing packaged UI acceptance from 8/25 to **9/25 (36%)**. Windows package/UI,
accessibility, Korean IME, resource measurements, and final cross-platform release gates remain
open. Evidence: `docs/evidence/issue9-aa80904-goal-drift-workflow.json`.


## v0.1.36 CI package-license audit — issue #9 / 2026-10-22

The paired v0.1.36 macOS and Windows packages from exact PR-validation run `39740270740` both record GitHub merge source `eff9f44858ada378ac230a71dbf040f4c47455bc` (PR head `cd9935de`, base `3fecd21d`). Their manifest dependency counts match the SBOMs: 214 macOS components and 201 Windows components, with no missing license expressions. Each archive checksum and executable SHA-256 matches its embedded checksum file; both packages include dependency notices and license texts. The sole GPL-optional expression is `self_cell 1.3.0`'s `Apache-2.0 OR GPL-2.0-only`; both license texts are bundled, and the permissive Apache-2.0 option is available. Evidence: `docs/evidence/issue9-eff9f448-license-audit.json`.

This package review does not replace legal review or close issue #9. Repeat dependency-closure and license review against the final release source after issue #9 acceptance; the Windows executable was hash-checked but not run on Windows in this audit.

## Full native-feature regression after merging #135 — issue #9 / 2026-10-22

At code commit `d4338e5b` (the later `21f5e6b` changes only the plan checklist),
`cargo test --locked --all-targets --features native-ui-probe` passes **866 tests, 14 ignored,
0 failed across 295 suites** (228.37 seconds). The additional focused repair, public-doc, and
source-layout batch passes 18 tests; repair Clippy with warnings denied, formatting, diff checks,
and the 300-line source/test guard pass. Graphify reports 8,408 nodes / 13,377 edges / 630
communities. CocoIndex is unavailable because its command is not installed. The exact-head
Platform Validation run `39722700547` was cancelled after a documentation-only follow-up push;
there is no current-head platform result. This is source-level verification only and adds no
packaged workflow acceptance: issue #9 remains 8/25, and the Rust UI remains unshipped.
After testing, `cargo clean` removed 494,075 generated files totaling 80.2 GiB from the Codex
worktree. The 750 MB `dist` package directory was preserved.

## Exact v0.1.34 packaged native plan-to-merge UI revalidation — issue #9 / 2026-10-22

The exact `aarch64-apple-darwin` v0.1.34 package from source `76d8082b02a722ab18cc4e2f0e8383b3c1a3fdd5`
was opened in one AgentConductor window. Its manifest records an 18,356,880-byte executable,
214 dependency notices, and binary SHA-256
`6076725213cde19f24b29cc5111da9ac6e885cb70c78e377a122376e290c0e05`; the archive SHA-256 is
`8b33b1378b4e3e145b90518609ecc1b70a9c5a67440e53c44d3df098dfa5ccab`.

In a disposable Git project, the UI reviewed a manual three-task plan using worktree isolation.
Three deterministic custom CLI processes completed in separate worktrees; each owned only its
fixture file, and all three test commands exited 0. The UI loaded the exact candidate diff, showed
all test output, required candidate approval and merge confirmation, then reached Completed.
The final disposable project's `main` points to candidate `2c071b8fbc887f139f0804206c417023a653a312`
and has no uncommitted changes. No provider CLI, external service, or product-repository path was
used. Evidence: `docs/evidence/issue9-76d8082-macos-package-ui-workflow.json`.

This repeats the already accepted packaged plan-to-merge journey patterns and adds no distinct
workflow: acceptance remains **8/25**. Windows GUI, authenticated provider workflows, full
accessibility parity, Korean IME, and idle-resource measurements remain unverified. During fixture
setup, one direct-plan editor draft showed three summary tasks but two editable rows; adding the
missing row restored three. A newly generated manual plan then showed all three. Reproduce this UI
observation before treating it as a confirmed product defect.

## Native scope-recovery button interaction evidence — issue #9 / 2026-10-22

Added a focused rendered interaction test for the recovery dialog's Preview changes and Approve and
apply buttons. It dispatches pointer movement before press and release, matching egui's hit-testing
sequence, then checks the exact task, owned paths, test argv, and reviewed digest in the emitted
events. The first test draft omitted the hover frame and failed to activate the button; correcting
the test harness made both button paths pass. `native_scope_recovery_actions` passes (1), existing
`native_scope_recovery` passes (2), with source-layout and public-doc checks passing. This closes a
component interaction-test gap only: no packaged end-to-end workflow is added; acceptance remains
8/25 and exact-head Windows/macOS package validation is still open.

## Previous local native-feature suite — issue #9 / 2026-10-22

At branch head `696b2f8` (product code `214e359` plus documentation-only commits),
`cargo test --locked --all-targets --features native-ui-probe` passed 845 tests, 14 ignored,
0 failed across 289 suites in 224.71 seconds. This covers the stabilized Settings dismissal test
locally but does not provide Windows runtime/package evidence or increase packaged UI acceptance
beyond 8/25. Graphify's local no-cluster code refresh reports 8,170 nodes and 14,029 edges.
CocoIndex Code remains unavailable: its installed launcher fails to import
`mcp.server.fastmcp`; no CocoIndex results are claimed.

## Current native-feature suite — issue #9 / 2026-10-22

At issue-9 branch head `630b73d` plus the current scope-recovery interaction-test unit,
`cargo test --locked --all-targets --features native-ui-probe` passes **855 tests, 14 ignored,
0 failed across 292 suites** (223.63 seconds). The new rendered pointer test activates both preview
and approval buttons and checks the dispatched task, paths, test argv, and digest; it is listed in
the macOS/Windows platform validation and Clippy commands. Focused source-layout (2) and public-doc
(11) tests, formatting, warning-denied all-target Clippy, and the 300-line guard pass. Graphify's
local no-cluster code refresh reports **8,301 nodes and 14,302 edges**. CocoIndex remains unavailable
in this runtime and was not installed. This is component evidence only: packaged workflow
acceptance remains 8/25; exact-head macOS/Windows package and complete native workflow gates remain
open.

## Native UI test feature-gate repair — issue #9 / 2026-10-22

Actions run `39634545852` failed on macOS and Windows because the newly added
`tests/native_room_briefing_layout.rs` integration test was auto-discovered without requiring
`native-ui-probe`. The test now has a crate-level feature gate. The no-default-feature target audit,
focused Team Briefing popup test, source layout, public-doc tests, formatting, and diff checks pass
locally. The workflow-equivalent all-target run completed; one unrelated `cli_workflow` case failed
once with a `DirectoryNotEmpty` cleanup race and passed on isolated rerun. All other targets passed.
PR merge-result run `39687975008` passed Windows but failed macOS in
`settings_dialogs_remain_accessible_across_sequential_open_close`: the Cache cleanup dialog was
still visible after the one-frame close check. Commit `214e359` replaced that fragile check with
bounded dismissal polling; the focused journey test passes on the current local head. The failing
run predates that fix, and no exact-current-head macOS/Windows result is available. No issue #9
workflow acceptance changes.

## Exact-head native UI package and menu review — issue #9 / 2026-10-22

Actions run `39613489337` passed all four jobs at `dbf893a18c4d73bc611c984fcc031684346ef67a`:
Windows/macOS platform validation and Windows/macOS native-package builds, manual generation,
package validation, and execution. In the exact-source v0.1.33 macOS app at
`/private/tmp/AgentConductor-1f75758.app`, the Workspace menu, Settings menu, and coordinator
selection popup were opened and visually confirmed to overlay the page without shifting its panels.
The earlier screenshot showing a vertical shift came from the stale 16:29 package under `dist/`;
the current packaged executable was built at 22:06 after the popup implementation. Focused local
layout verification passed 14 tests across five suites; formatting and diff checks pass. No source
change or new packaged workflow acceptance is claimed. Workflow acceptance remains 8/25 (32%), and
the stable shipped UI remains WebView.

The follow-up audit also opened a populated Team Briefing disclosure in the Rust room view. The
session details paint in the popup and the following Stop button remains at the same y-coordinate.
`tests/native_room_briefing_layout.rs` now guards this path; the focused popup audit passes 21 tests
across six suites, the layout/doc batch passes 19 tests across five suites, and warning-denied
Clippy passes for the new test. This is component evidence only; packaged workflow acceptance remains 8/25. Graphify refreshed locally to 8,136 nodes
and 13,973 edges. CocoIndex has no available CLI or MCP tool in this runtime, so no CocoIndex result
is claimed.

The follow-up menu audit also covers the Profile Rules JSON editor and task-detail editor: opening
either popup leaves the Save action or next task at its original y-coordinate. Both regressions pass
locally and are registered in the macOS/Windows native UI validation commands. They confirm popup
behavior only; no complete packaged workflow is added to the 8/25 acceptance count.

## Current package, exact-window review, and disk cleanup — issue #9 / 2026-10-21

The exact source at `de032e016323dda18ca369a08ab3207f38118f82` built as a macOS release package.
Package validation and the built-in four-case deterministic smoke passed. Manifest SHA-256 is
`36dd3e934b75fe278307815e9555ab78f72f65a01e270bc3cdf67ce7b6662695` with 214 dependency records.
The exact bundle was selected by full path and inspected in one isolated window: Workspace, Agent
Roles, and Review empty states rendered; no setting was saved, and this spot check does not add a
packaged workflow. The test window and isolated HOME/workspace were closed and removed.

The worktree contained 61 GB of Rust build output. With no Cargo/Rust compiler process active, only
the reproducible `target/debug/deps` and `target/debug/incremental` caches were removed. Existing
executables, source, release dependencies, and package artifacts were preserved; `target/` is now
2.2 GB. Issue #9 now reports 8/25 packaged UI workflows (17 remaining). Actions run `39543488348` passed macOS validation, Windows build/layout, and applicable Windows tests,
then failed only `public_docs`: a multiline assertion assumed LF while the Windows checkout used CRLF.
The assertion now normalizes CRLF before matching; exact-head Windows rerun is pending. No new workflow
acceptance is claimed. AgentMemory checkpoint: `mem_mvhmis60_5fa5b73d16bd`.

## OpenCode MCP server registration and discovery — issue #9 / 2026-10-21

The exact v0.1.32 macOS package registered a local server through Settings → MCP servers using
the OpenCode JSON/JSONC shape, saved it, reopened the editor, and discovered `get_issue` from a
deterministic Rust stdio fixture. Its request trace contained initialization and `tools/list`
only; no tool call, provider CLI, external service, or user workspace was involved. The isolated
HOME was removed. Evidence: `docs/evidence/issue9-256db1a-mcp-server-registration-ui.json`.
This is distinct from importing Jira/Confluence records, so acceptance advances to
**8/25 (32%)**, with 17 workflows open. Actions run `39541731070` exposed a second Windows-only
picker compile error; the intent is now passed into the dialog and a new exact-head run is pending.

## Windows folder-picker compile regression — issue #9 / 2026-10-21

Actions run `39541153826` exposed a Windows-only compile failure: the localized picker change
removed the `executable` parameter but the Windows file filter still used it. Actions run
`39541731070` then found the restored value was not passed from `pick` into the inner `dialog`.
Run `39542757398` compiled that production path successfully, then found `tests/windows_console.rs`
did not re-export `ui::language` into its isolated test shim. The shim now exports the existing
module. The Windows CI build remains the platform compile regression gate; local macOS tests do
not compile this `cfg(windows)` test. Focused local tests and macOS target check pass; rerun is pending.

## UC-79 recoverable workspace archive recheck — issue #9 / 2026-10-21

The exact v0.1.32 package from source `256db1a` was tested under a disposable HOME. It archived
an incomplete workspace only after switching to another workspace, displayed the exact saved
path and project-preservation warning, removed the entry from the active registry, and wrote a
recovery record under `workspace-trash`. The disposable project folder remained present. No
provider, external service, or user workspace was involved; the package process was closed.
Evidence: `docs/evidence/issue9-256db1a-uc79-archive-ui.json`. This revalidates Workflow 2 and
does not change acceptance from **7/25 (28%)**. Actions run `39541153826` was in progress for
the source checkpoint at the time of this validation.

## Workspace settings export/import package UI — issue #9 / 2026-10-21

The exact macOS package built from source `256db1a5b536650edf4bbe791f573cac716638ff`
passed package validation and deterministic release smoke. In an isolated HOME, the UI exported
`agentconductor.workspace.json`, previewed the import, required explicit “Import and replace
settings” confirmation, and restored the exported workspace name. No provider CLI or external
service ran and the user's workspace was untouched. Evidence:
`docs/evidence/issue9-256db1a-settings-share-ui.json`. This is a distinct packaged workflow, so
acceptance advances to **7/25 (28%)**, with 18 workflows open. No screenshot was retained. The
focused title/share tests, formatting, macOS-target compilation, and diff checks pass; exact-head
Actions run `39540102836` remains pending.

## Workspace settings picker prompts — issue #9 / R-52 / UC-42 / 2026-10-21

The Settings → Workspace settings export and import actions now pass distinct localized purposes
through the existing async picker worker. The regression covers project/export/import titles in
English and export/import titles in Korean. `native_folder_picker_purpose`, `native_ui_probe`, and
`workspace_share` pass 15 tests across three suites; formatting, a macOS-target `cargo check`, and
`git diff --check` pass. A whole-repository all-target test run reported 30 tests across four suites
before stalling and was interrupted, so it is incomplete. Windows cross-compilation and exact-package UI
round-trip remain unverified. The UI attach selected the installed app with the user's saved
workspace; it was quit without interacting with or saving that workspace. Only the two disposable
test directories created for this check were removed. Packaged workflow acceptance remains
**6/25 (24%)**; this component fix does not add an accepted workflow.
Commit `7c668a0` is pushed to the issue branch; issue #9 comment `6307473584` and AgentMemory
`mem_mvhkfr8u_67de9db17f0b` preserve the checkpoint.

## Exact PR-package revalidation and cross-platform packages — issue #9 / 2026-10-21

Actions run `39533269033` passed PR-head validation and release-package jobs for head
`d799ad6fe4199516c6344f9f63676eb2708d9157` and synthetic merge result
`d386e6a3cdd0bb3f7484b0578d61ebdb867b67d0`. macOS and Windows packages built, rendered both
offline manuals, executed, and were retained. Artifact `12317625213` is the macOS package
(21,193,358 bytes; artifact digest `ed43e0e0a37f71e00ddd91337b88a2f69f7479f88d2090f240f4bd709220c522`);
artifact `12318045860` is Windows (15,422,921 bytes; digest
`05a89173b6fc69655d34d42b46a722adbac31e47648708b140b42321888cd91a`). Per-platform executable
and archive checksums are in `docs/evidence/issue9-d386e6a-pr-merge-cross-platform-packages.json`.

The exact merge-result macOS app revalidated the previously accepted shared-folder plan-to-merge
flow: three independent Rust fixture tasks wrote disjoint files, all tests passed, the operator
inspected the exact diff and approved the candidate, and the app merged it and displayed Completed.
The disposable Git repository ended at candidate `517e82d46cfe047589fd0164f5d42f6c1933549a` with
only the three expected files. No real provider CLI, API, external service, or user workspace was
involved. This repeats Workflow 1 and does not add a distinct acceptance workflow. Evidence:
`docs/evidence/issue9-d386e6a-native-ui-plan-to-merge.json`.

Packaged native UI workflow acceptance remains **6/25 (24%)**, with 19 workflows open. The Rust UI
has not shipped publicly. Screenshots, Windows GUI execution, Korean IME/accessibility, and full
visual parity remain unverified.

## Pull-request merge-result macOS and Windows packages — issue #9 / 2026-10-21

Actions run `39524574228` completed successfully for PR head
`806bf3204e63fa829b13d8e4bf09168da6b0e0c1`, checking out and building its synthetic merge result
`9e3617545e6f0c6ad8357bda7a15c541302ce8dc` (the uploaded artifact names carry this merge SHA).
Both platform-validation jobs passed native builds, source/test layout enforcement, applicable
tests, the native UI launch path, formatting, and warning-denied Clippy. Both package jobs passed
optimized release builds, Rust-rendered English and Korean manuals, exact package execution, and
artifact retention. macOS artifact `12312859028` is 21,190,192 bytes with artifact digest
`7c66e34aa06e7ccf441f0a7fdd9a41ab734ba87b9b53b22f300376df027c47ed`; Windows artifact
`12313174827` is 15,417,423 bytes with digest
`e650695d4dfcd56adad9778ca5b79ce4b922b4d889de16b9436ef0358e6c49e3`. Evidence:
`docs/evidence/issue9-9e36175-pr-merge-cross-platform-packages.json`.

This verifies packaged CLI orchestration on both platforms, not another native UI journey. Verified
packaged UI acceptance remains **6/25 (24%)**, 19 workflows remain, and the public release still
does not ship the Rust UI. Exact packaged GUI parity and the remaining user journeys are open.
AgentMemory records: `mem_mvhg8qd7_f7ff48237803`; lesson `lsn_215b3738fa9a9cdd`. Graphify
re-extraction: 7,743 nodes / 13,278 edges. CocoIndex Code could not start because its environment
lacks `mcp.server.fastmcp`.

## Exact-package quota fallback and platform validation — issue #9 / 2026-10-21

The exact v0.1.32 macOS release binary from product source `b4ad2f5` was exercised against a
disposable Git repository and deterministic Rust CLI fixtures. Three independent tasks each
received a quota-limit event with a reset time, transferred to the configured company fallback on
attempt 2, and passed their tests. The run reached `waiting_approval`; an unapproved merge was
refused, approval of the exact candidate allowed merge, and the packaged CLI then reported
`completed` with the three expected files and a clean repository. No external provider or service
ran. Evidence: `docs/evidence/issue9-b4ad2f5-quota-fallback-cli.json`.

Exact-head Actions run `39520326287` passes both macOS and Windows platform-validation jobs,
including native build, source-layout enforcement, applicable tests, native UI launch-path,
formatting and warning-denied Clippy. The macOS native-package job also passed the release build,
bilingual manual rendering, package execution and artifact upload; artifact `12312373026` is
retained (21,189,925 bytes; SHA-256 `c5f8dce7a06e04ff2f59f7208b8fbaf1b4e094e6e2aa94d5937791c72ae61624`).
The Windows native-package job also passed release build, bilingual manual rendering, package
execution and artifact upload; artifact `12311274352` is retained (15,417,457 bytes; SHA-256
`8415762398af24ebf21d1830a4027fcff6d6b11647292235eae12b83456ab363`). Both validation and both
package jobs are green. Evidence: `docs/evidence/issue9-b714887-cross-platform-packages.json`.
Local scheduling/start and fallback/availability/profile regression batches pass (4 tests and
21 tests respectively; the overlapping scheduling suite is not double-counted). This is packaged
CLI and component evidence, not a new native UI journey: acceptance remains **6/25 (24%)**, 19
workflows remain, and the public Rust UI is not yet shipped. AgentMemory records:
`mem_mvhe2xb3_6ad4618a9535`, `mem_mvhe77xu_5497db2b04a4`.

## Latest local verification — issue #9 / 2026-10-21

At branch head `7a6256d`, the complete local `rtk cargo test --all-targets --no-fail-fast`
run passes **805 tests, 14 ignored, 0 failed across 272 suites**; Cargo reports 216.14 seconds
of test execution. All-target Clippy with `-D warnings`, `cargo fmt --all -- --check`, and
`git diff --check` pass. The earlier macOS start-journey timeout was not reproduced in this
complete run. Exact-head GitHub Actions run `39517629909` remains in progress; its predecessor
`39517056024` was cancelled after a later push. Packaged UI workflow acceptance remains **6/25
(24%)**; this verification does not count as another workflow or as cross-platform UI acceptance.

## Current UI parity checkpoint — 2026-10-21 / macOS navigation CI triage

Exact-head Actions run `39507118252` is complete: Windows passed and macOS failed only
`review_approval_shows_start_feedback_then_opens_the_started_run`, timing out while waiting for
the enabled `Open Agent Room` button after returning from a completed run to Workspace. The test
passes ten consecutive local executions on the current source; a focused run also passes. This
does not establish the failure as harmless or fixed. The UI wait now reports accessible button
states and labels on timeout so the next exact-head result identifies the last visible surface.
Formatting, diff checks, focused native UI tests, and warning-denied Clippy pass. This is test
diagnostics only; packaged acceptance remains **6/25 (24%)**, with 19 workflows open.

An optimized exact-source macOS package was built from `0935f481fba1b89b329bca604ebb1c0131035992`.
`package-release`, manifest validation, packaged `--help`, ZIP integrity, and the packaged smoke
passed. The binary is 18,008,720 bytes (SHA-256
`a5887a30c8475f9700c1e7e853a5e9b739ff07de219cc630fbfb8fac44e86ba4`); the archive SHA-256 is
`07d41c8297da13cbe4067b53a024caf812c913aa53cd0507a8c69e05d4591ab1`, with 214 dependency
records. Evidence: `docs/evidence/issue9-0935f48-macos-package.json`. Visual review is still open:
the existing AgentConductor window is on the user's saved workspace, so no second window was
opened. This package build does not add a packaged UI workflow.

## Historical UI parity checkpoint — 2026-10-21 / Agent Room header context

The stable v0.1.32 WebView titlebar shows app/version, repository, linked phase, and local CLI-auth context. Rust redundantly rendered “Agent Room” under its native window title and omitted those context fields. Red-first English/Korean rendered assertions reproduced both defects; the Rust header now shows the app/version, localized workspace/phase/auth context, and repository name while removing the repeated title. The 21 focused room/layout/source-layout tests pass across five suites; all-target Clippy (`-D warnings`), formatting, diff checks, and the 300-line source/test guard pass. Graphify refreshed to 7,719 nodes / 12,210 edges. The exact-head Actions run `39507118252` failed macOS native tests at prior HEAD `4ba268e`; Windows was still running when recorded. This is a component correction, not an accepted end-to-end workflow; packaged workflow acceptance remains 6/25 (24%). Current-package visual parity and cross-platform screenshot checks remain open.

## Current verification checkpoint — 2026-10-21 / issue #9 profile quota editor

Commit `dbf249f` on the issue #9 branch fixes the custom CLI profile editor's loss of `quota_command` and `quota_rules`. Red-first profile-save coverage reproduced the dropped fields; the editor now serializes and applies both, labels the advanced JSON section accordingly, and renders existing quota configuration accessibly. Focused profile/quota tests pass 15/15 across six suites. Full native-feature tests pass 803, with 14 ignored across 272 suites; warning-denied all-target Clippy, `cargo fmt --check`, `git diff --check`, and `source_layout` (including the 300-line guard) pass. The macOS package from source `016451b` passes `package-release`, `validate-package`, packaged `--help`, ZIP integrity, and all four shared/worktree × Git/folder smoke cases; binary SHA-256 is `2b0a4b03aac1bc87ef75a97039187ec60179cb84a834e6a8a39e04dbc0d34b92`, archive SHA-256 is `5e69ac1306d77eee5e6a7ac91897e7774ccaf8c003adfbb50a137476dffacea9`. In one isolated package window, AccessKit confirmed the quota editor fields survived profile save and reopen; the test profile was removed. No screenshot was retained and no quota command or external service ran. The exact packaged Workspace and Agent Roles screens were also rechecked in one isolated window without saving data. Evidence: `docs/evidence/issue9-016451b-macos-package.json` and `docs/evidence/issue9-016451b-macos-smoke.json`. Actions run `39505888033` is in progress on both platforms: macOS passed compilation and is checking source layout; Windows is compiling. This component fix is not a packaged end-to-end workflow: acceptance remains **6/25 (24%)**. The native Rust/eframe UI is already the default package UI; full visual parity is not yet accepted.

## Current verification checkpoint — 2026-10-21 / PR #22

Product source commit `cf92f39315c12922915109073e6e1a43365f5a14` is the Rust integration of main v0.1.32, merged from `3123ece`; its package evidence is retained below. The latest local branch includes a cross-platform Workspace action-row fix and a stable MCP settings journey check. Its native-feature all-target run passes **801 tests, 14 ignored, 0 failed across 271 suites**. Warning-denied all-target Clippy, `cargo fmt --check`, diff checks, the authored source/test ≤300-line guard, offline English/Korean manual rendering, focused public-doc tests, and the CI-feature focused regressions pass. The new UI fix has not yet been rebuilt into a package.

The optimized `aarch64-apple-darwin` release package for `cf92f39` passes package smoke in all four shared/worktree × Git/folder combinations (three tasks and tests each), verifies approved merges and refuses unapproved merges, and passes ZIP CRC. The package contains one 18,008,720-byte Rust executable and 214 dependency license records. Binary SHA-256: `1a7831dc567c5fc93859d5949dcb6cb9d7643fb26e0fe65ea65d81894805f741`; archive SHA-256: `fd17557d0be3a27470fccbbf75c372c279ed4d93bde905eb447d54f65755fc51`. Evidence: `docs/evidence/issue9-cf92f39-packaged-smoke.json`.

Packaged workflow acceptance remains **6/25 (24%)**; shipped/default Rust UI replacement remains **0/25**. GitHub issue #9 is open and PR #22 remains draft/mergeable. Platform validation run `39488170157` (#600) finished with native-test failures on macOS and Windows; both jobs passed build/layout, and the native-package job was skipped. The failing tests were the Agent Roles action-row coordinate assertion and the sequential MCP-settings journey's missing duplicate heading. The action group now fills the available width before right alignment, and its regression checks the rightmost action against the card edge; the settings journey now anchors to the dialog title and Save control. Both focused regressions pass locally, including the test-fixture feature set; the full post-fix local suite and Clippy also pass. The exact `cf92f39` v0.1.32 package remains open in one window; its Workspace step row, active underline, and right-aligned save control were visually checked without changing or saving fields. No screenshot artifact was retained, so this does not prove current-main screenshot parity. A new exact-head macOS/Windows Actions run, a package rebuilt with the UI fix, Windows package/UI, published-WebView parity, Korean IME/accessibility, authenticated provider workflows, idle-resource measurements, and final license/release review remain open. Run #598 was cancelled before either OS build completed; #597 was also cancelled before Windows validation completed.

## Stage card marker parity — 2026-10-21

The optimized v0.1.31 macOS package from `ca5c3f4dd8e48d5bef89c533153a4d01e92f1c02` (binary SHA-256 `0163e285900b005b6b9a3cc241cad253be8a5ca22ff0ffe9eea3aea7dd10f0fd`) was run directly with a disposable `HOME`. In one 1140×772 window, Workspace, Agent Roles and Review each showed the selected step marker in both the page eyebrow and active card. This matches a checked-in same-size capture carrying a v0.1.24 label, but `knowledge/wiki/native-screenshot-provenance.md` identifies that capture as an earlier Rust build; it does not establish parity with the published WebView UI. The check supersedes the earlier “show the step marker only once” rule. The inspection did not save a workspace, change settings, or run an agent. Accessibility-tree observations are retained in `docs/evidence/issue9-ca5c3f4-stage-card-markers.json`; current screenshots were inspected in-session but not saved as repository files. The exact package contains no JavaScript, TypeScript or Python source files; the package inventory is one native executable, bundled SQLite, and 214 dependency license records. No end-to-end workflow was added: packaged acceptance remains **6/25 (24%)**, while the shipped/default Rust UI replacement remains **0/25**. Windows UI, Korean IME/accessibility, comparison against a verified WebView capture, and the remaining packaged workflows are still open.

## Native MCP source import — 2026-10-21

The exact v0.1.31 macOS Rust package (`2fd8e57`, binary SHA-256 `818f4d137127e1d650d520796ebe97d7a41be41d5f0f786eea529b9607cc5f8a`) completed the full Workflow 12 path in one window under a disposable `HOME`: register an OpenCode-shaped local stdio MCP fixture, discover its Jira and Confluence tools, preview `jira:AC-1` and `confluence:53553565`, and separately approve applying both source records. The UI showed each record's content, URL, and hash, followed by `Applied sources: 2`. The coordinator and MCP server were deterministic local Rust fixtures; no vendor CLI, external MCP server, or credentials were used. Focused offline verification passed 17 tests across three suites. Evidence: `docs/evidence/issue9-2fd8e57-mcp-import-ui.json`. Packaged workflow acceptance is now **6/25 (24%)**; the shipped/default Rust replacement remains **0/25**. Windows UI, Korean IME/accessibility, full parity, and 19 packaged workflows remain open.

## Native Workspace heading correction — 2026-10-21

The selected step now supplies the bilingual page eyebrow, while each section card keeps its title and description without repeating the step marker. Red/Green header and AccessKit checks pass 17 tests across two suites; source-layout, rustfmt, diff checks, all-target native-feature Clippy, and an optimized release build pass. The exact `2fd8e57` macOS package validates with 214 dependency records and bundled licenses; packaged `--help` and ZIP CRC pass. The serial offline native-feature all-target suite passes **775 tests, 14 ignored, 0 failed across 262 suites** (300.95s). The exact package smoke passes all four shared/worktree × Git/folder modes: 12 fixture tasks and tests, verified concurrency, refusal of unapproved merges, and successful approved merges; retained evidence is `docs/evidence/issue9-2fd8e57-macos-smoke/`. A subsequent exact-package visual audit inspected Workspace, Agent Roles, empty Review, MCP, sharing, Messenger, and cache in Korean dark and English light themes at 1140×772; fourteen screenshots are retained under `docs/evidence/issue9-2fd8e57-visual-audit/`. It used one disposable profile (`fixture`, `/usr/bin/true`) and did not save settings, send messages, clear cache, or run an agent. This component inspection adds no workflow: packaged acceptance remains **5/25 (20%)**; shipped Rust replacement remains **0/25**. Windows rendering, Korean IME, full UI parity, and the remaining workflows are open.

## Exact-package visual batch — 2026-10-21

The v0.1.31 package built from `2fd8e57f27d5f5c7b5d4a63c2eae7b20ea7b1a14` was inspected in one window at 1140×772 using a disposable profile. Fourteen retained captures cover Workspace, Agent Roles, empty Review, MCP registration, workspace sharing, Messenger notifications, and cache cleanup in Korean dark and English light themes; each file's SHA-256 is recorded in `docs/evidence/issue9-2fd8e57-visual-audit.json`. The screenshots show the saved fixture profile only. No real workspace or credentials were loaded; no settings were saved, no message was sent, no cache was deleted, and no agent was run. The inspected forms and dialogs fit the viewport without visible overlap; this is a component visual check and not acceptance of an end-to-end workflow. The metric remains **5/25 (20%)**, with shipped Rust replacement **0/25**. Windows, IME, accessibility on hardware, and the remaining workflows remain unverified.

## Historical branch checkpoint — 2026-10-21 (before the 2fd8e57 package)

PR #22 is open and remains draft; the native-session test-fixture correction is `6e6ef4d`. Packaged native UI acceptance is **5/25 (20%)**; the shipped Rust replacement remains **0/25**. The compact step-navigation product change is `227854d`; its exact macOS package passed package validation, packaged `--help`, and ZIP CRC, and Workspace → Agent Roles → Review was checked at 1140×772. Test-only commit `b204eb5` stabilizes the Settings journey's selected-language wait. On that source, the grouped Settings/header/start-journey tests pass 6 tests across 4 suites. The all-target run first found one `native_session` fixture failure because it wrote a live owner record before initial lease acquisition; after correcting the fixture, `native_session` passed 3/3 and the complete suite passed **794 tests, 14 ignored, 0 failed across 263 suites** in 258.50 seconds. Source layout, public-doc, rustfmt, all-target Clippy with `-D warnings`, and diff checks also pass. Actions run `39452437163` tested merge commit `11588965` from PR head `c5dac47` into base `fdfd6d3`, not the current head; both OS jobs failed only `native_ui_settings_journey`. No exact-head Actions result has been observed. The older 794-test result below is from product code `1b1a920` and does not verify this branch head. Keep the migration open and do not merge or release until current-head platform validation and all remaining acceptance gates pass.

## Historical checkpoint — 2026-10-21 (before the current issue #9 progress)

The full user-requested product remains in development. Issue #9 is open and PR #22 remains draft. The current packaged workflow evidence is from the v0.1.31 macOS build at product commit `a1ea942`: **5/25 (20%)** distinct packaged end-to-end UI workflows accepted; the shipped Rust replacement remains **0/25** until an accepted release ships. The exact package and Git/worktree workflow evidence are in `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Source `6026076` builds and validates as a v0.1.31 macOS package; its one-window UI check confirms Enter creates and selects a saved workspace (component evidence only: `docs/evidence/issue9-6026076-workspace-enter-ui-smoke.json`). Current product code `1b1a920` passed the complete local native-feature all-target suite: **794 passed, 14 ignored, 0 failed across 263 suites** (222.61s); all-target Clippy (`-D warnings`), formatting, diff, public-doc and source-layout checks pass. The current exact-source macOS package at docs-only commit `0d09b88` also passes manifest validation, packaged `--help`, ZIP CRC, and package-content scans; its 17,942,384-byte binary SHA-256 is `1425a6384fd7b24382acc91226de007fe8646c6b579e5d9d632d4b3f427b51db`, and evidence is in `docs/evidence/issue9-0d09b88-macos-package.json`. The checked-in Workspace, dark Workspace, Agent Settings, and Agent Room screenshots still show v0.1.24; no replacement was saved from the current exact Rust package, so the screenshot refresh remains an explicit release gate. No exact-head Actions run is listed for current head `0d09b88`; earlier run `39448772842` targets stale commit `275faf5`.

The grouped UI batch (`cb49fa1`) fixed four audited defects: return from a running Agent Room without stopping it, preserve the selected Review step, safely stop-and-delete a workspace with an active plan only after plan/metadata work settles, and fit the agent editor at 640px. The subsequent Rust scheduler batch (`fdc3528`) restores timeout accounting, bounded automatic retry, and Stop-aware backoff. Current exact issue head `27e307e` passes the complete offline native-feature all-target suite: 768 passed, 14 ignored, 0 failed across 260 suites (211.51s); all-target Clippy with warnings denied, rustfmt, and diff checks pass. The grouped exact-package Settings dialog audit also passes. This full suite validates source behavior, not additional packaged UI acceptance.

The exact macOS package from code source `7419557` was opened in one isolated app window for a grouped screen/control audit: Workspace Settings, Settings/MCP registration, profile registration, and Agent Roles. Two temporary workspaces were created and one inactive workspace archived; a delayed Rust mock plan showed Running/Stop planning and then returned the coordinator to Ready after stop-and-delete. Evidence: `docs/evidence/issue9-7419557-packaged-ui-audit.json`. No provider CLI or external service ran. This spot-check does not constitute a complete packaged workflow. On 2026-10-20, exact source `7289562e5de394555b196da75b0ecf3904f3c53c` built a 17,925,856-byte macOS binary and passed package and smoke validation; evidence is in `docs/evidence/issue9-7289562-macos-*.json`. A newer local package from source `a8be85fc2392821fab345d3e1b879a6aa9bc6714` built a 17,942,384-byte binary, included 214 dependency notices, passed package validation, packaged `--help`, ZIP integrity, and four mock workflow cases (12/12 tasks and tests; rejected unapproved merges; verified approved merges). Evidence: `docs/evidence/issue9-a8be85f-macos-package.json`. The exact package's dialogs were checked and dismissed in a disposable workspace; evidence: `docs/evidence/issue9-a8be85f-packaged-dialog-ui.json`. A separate exact-package workflow exercised UC-30/31 with an isolated `HOME`: create and save named workspaces, save role-stage state, switch and verify persisted settings, remove from recent and reopen, and archive an inactive workspace after confirmation. Evidence: `docs/evidence/issue9-a8be85f-workspace-ui.json`. This added Workflow 2 and advanced packaged UI acceptance to **3/25** at that checkpoint; shipped Rust replacement remained **0/25**. At previously tested code head `27e307e`, the full native-feature suite passed 768 tests / 14 ignored across 260 suites; all-target Clippy, rustfmt and diff checks passed. Exact PR head `604579c` Actions run `39424348558` completed with test failures on both OSes: the settings journey sampled before MCP registration became visible, and Windows also attempted a merge before the previous UI action settled. Local fixes are in this batch. A later full local run stopped after five minutes at 363 passed / 14 ignored across 120 suites without a reported test failure. Focused planning, settings/start, documentation and source-layout checks passed (10 tests across 4 suites); full-suite verification remains open. Exact pushed source `4a114ec6a76141c48ebe8138694398572cae1acd` built a 17,942,384-byte macOS package (binary SHA-256 `f288ff81542870c49a40a73824a5fe70b0d10fe917370dfa66c3f13251d2ec11`); `validate-package` and packaged `--help` passed. In one isolated app window, a saved disposable goal without an agent coordinator showed Generate plan disabled with coordinator guidance; evidence: `docs/evidence/issue9-4a114ec-plan-gate.json`. A follow-on grouped audit inspected Agent Roles, empty Review, MCP, workspace sharing, Messenger, and cache preview together without saving or deleting data; evidence: `docs/evidence/issue9-4a114ec-settings-batch.json`. The native UI suite batch passed 122 tests across 23 suites, and feature Clippy, rustfmt, and diff checks pass. These remain component checks and do not add a workflow. Actions run `39429494121` was cancelled on both Windows and macOS during the native build/test step; it provides no platform validation. The subsequent positive-path package workflow adds Workflow 4, bringing current acceptance to **4/25 (16%)** with **21 packaged workflows** still open. Exact screenshots, Windows UI/package, accessibility/IME, provider integration, resources, and final license closure remain open.

This checkpoint is not completion evidence for Windows, the other workflows, authenticated providers, accessibility/IME, idle resources, or final license closure. Do not merge PR #22 or publish a release until the remaining criteria below are verified. See the current issue #9 checklist in [the implementation plan](07-implementation-plan.md).

The subsequent positive-path exact-package UI workflow registered a local Rust fixture CLI, generated and reviewed a plan, approved execution, passed its 1/1 test, reviewed and approved the diff, confirmed merge, and reached Completed in one isolated process. This adds **Workflow 4**, advancing packaged acceptance to **4/25 (16%)**; 21 packaged workflows and the shipped Rust replacement remain open. Evidence: `docs/evidence/issue9-4a114ec-native-ui-plan-to-merge.json`. The first fixture attempt referenced `/usr/bin/test`, which does not exist on macOS; the fixture was corrected to `/bin/test` and the complete flow passed in a fresh disposable folder.

A subsequent same-build recheck used a three-session local Rust fixture and a fresh non-Git shared folder. The exact reviewed plan had three independent tasks with distinct paths and overlapping process intervals; all three task tests passed, the combined diff was reviewed, candidate approval and app-level workspace integration completed, and the run reached Completed. The enabled approval action and green Completed panel remained readable in dark mode; light mode was restored. Activity, Raw logs, Lessons, and the read-only team briefing were inspected together; each task's result and clean stderr were visible, while provider-native session/model metadata remained unavailable for the mock CLI. This strengthens the already-counted parallel workflow evidence and does not add a fifth workflow. The target folder was not a Git repository, so no Git commit or branch merge is claimed. Evidence: `docs/evidence/issue9-4a114ec-parallel-ui-plan-to-merge.json`.

A follow-on exact-package component batch reused the same app process to inspect Workspace phase analysis, Agent Room candidate evidence, and Updates release notes. The accessible analysis disclosure expanded; candidate revision, passing test, and exact diff were visible; the release-notes link and scrollable full notes were present. The release metadata check ran, but the link was not opened and no screenshot was retained. Five grouped UI suites pass (23 tests) and all-target native-feature Clippy is clean. This adds no workflow acceptance; the total remains **4/25**. Evidence: `docs/evidence/issue9-4a114ec-review-updates-batch.json`.

The same packaged process was also checked in dark mode at 1140×772, then restored to light mode. Completed status, passing-test count, selected diff tab, diff text, and inspector remained readable; no defect reproduced. That run had no enabled primary action; a later three-task run in the same package verified the enabled approval action and green Completed panel (see the follow-up evidence below). Five related theme/layout suites pass (15 tests), as do rustfmt and diff checks. No screenshot artifact was captured. Evidence: `docs/evidence/issue9-4a114ec-dark-theme-audit.json`. Component evidence does not change the 4/25 acceptance count. Actions run `39431587615` was cancelled on Windows and macOS during native build/test; current docs head `459ba84` has no reported Actions run.

At the exact product source `4a114ec`, the complete offline native-feature all-target suite now passes: 770 passed, 14 ignored, 0 failed across 260 suites (216.85s); all-target Clippy with warnings denied also passes. These source checks do not add packaged workflow acceptance. Exact macOS/Windows Actions results, Windows UI, and the remaining acceptance gates are still open.

Historical audit baseline: 2026-10-10 KST. The detailed table below began as an audit of requirements through R-59/DD-65; rows are updated where current evidence is now authoritative, while remaining work is explicit. See [requirements reconciliation](../knowledge/wiki/requirements-reconciliation-2026-10-11.md) for the reviewed historical baseline.

| Requirement | Current authoritative evidence | Remaining acceptance work |
|---|---|---|
| R-01 Native authentication | Native Claude/Codex metadata; installed Codex 0.162.0 ChatGPT-authenticated stdin/JSONL generation and exact-ID context resume with API-key environment variables removed | Claude login and OpenCode configuration; native implementation and all-provider task execution |
| R-02 Subprocess control | Process groups/raw spools, exit/timeout/recovery tests, live bounded normalized SQLite attempt events and fail-fast malformed-stream tests | Native all-provider progress fixtures and sustained resource bounds |
| R-03 Registered providers | Templates, custom contract, parser/argv tests | Real provider contract fixtures and end-to-end registration/execution |
| R-04 Planning | Planner mock tests and plan validation | Goal -> native planner -> registered agents -> verified result end-to-end |
| R-05 Isolation/parallelism | Shared/worktree mock E2E; leases/checkpoints and transitive dependency refresh | Native concurrency evidence |
| R-06 Evidence | Git/test integration, snapshots, reviewed rework, retained workspaces, bounded typed coordinator proposals; distinct persisted Unix signal/exit outcomes | Normalized summaries/failure inventory and native review contract evidence |
| R-07 Approval | Exact candidate binding and mock E2E | Full negative tests for target/index/diff/test changes; native execution review |
| R-08 Dangerous actions | Typed SQLite approval ledger, opt-in Claude worker binding and human CLI controls, compiled shared/worktree approval/expiry/rejection/stop/staleness tests, owned verifier and async cancellation/alias/replay/closed-channel negatives, native metadata initialization | Actual native denial/approval/failure containment for every provider; ignored/external command dependencies and OS containment; profile labels alone do not prove containment |
| R-09 Lifecycle | Quota, CLI stop/resume/revise/review, leases, checkpoints, crash recovery and rework tests | Native contract evidence and interrupted native review recovery |
| R-10 Memory | Outbox/offline recall tests, live intermediate summaries, reviewed task-scoped runtime lesson handoff with current source hashes, development saves | Native lesson service integration, confidence-aware retrieval, cross-run/project context and native session proof |
| R-11 Wiki | English raw/schema/wiki files, reviewed runtime assets with source hashes/staleness and derived pages/index/log; mirror alias/failure tests | Native ingestion/review flow, source revalidation UI and all asset integration evidence |
| R-12 Rust/lightweight | Exact source package `9845db79af260b631bde7ade2104bae2400cc649`: one 18,606,512-byte macOS executable, bundled SQLite, 214 dependency records; four approval-gated package smoke cases, packaged `--help`, ZIP CRC, and checksum pass | Windows exact-head package/UI and cross-platform idle-resource measurements; retained exact-current screenshots |
| R-13 Workflow/TDD | At test head `12d6d808` (product code `9845db79`), locked/offline native-feature all-target suite passes **938 tests, 14 ignored, 0 failed across 310 test binaries**; warning-denied all-target Clippy, rustfmt, diff check, public-doc/source-layout/package tests (16), and the authored-file ≤300-line guard pass | Complete the requirement-by-requirement audit and final macOS/Windows release verification |
| R-14 Development tools | RTK and Ponytail used; Graphify queried for quota-fallback evidence on its curated snapshot (8,916 nodes / 14,081 edges) | CocoIndex executable is unavailable in this environment; no CocoIndex result is claimed. Use it when available and retain evidence |
| R-15 Distribution licenses | Exact macOS package manifest records 214 dependencies; package validation and bundled-notice checks pass | Recheck final dependency closure and every bundled/optional license boundary for both final release packages |
| R-16 English/public docs | English requirements, design, plan, and user manuals; public README remains free of internal decisions | Replace historical manual screenshots only after exact-current UI captures pass review; revalidate both manuals in the release package |
| R-17 UI | Eframe-native source coverage is 25/25; distinct packaged end-to-end workflows accepted are **20/25 (80%)**; source-level interaction tests do not add to package acceptance; direct native transport has no UI HTTP listener | 5 packaged workflows; exact current GUI parity and retained screenshots; Windows rendering; full accessibility and Korean IME; idle-resource measurements; public Rust replacement remains unreleased |
| R-18 Availability | Parsing, schedule, bounded retries, watch tests | Native restriction fixtures, crash-safe schedule ownership and runtime UI |
| R-19 Multi-session transfer | Mock concurrency/reassignment, native session exclusion and refreshed dependency context; installed Codex two concurrent independent contexts with exact-ID resumes | Claude/OpenCode/custom native concurrency, reassignment and interrupted task evidence |
| R-20 Session models | Native Claude/Codex catalog parsing, revision-bound model edits, and exact-package fixture UI controls | OpenCode/self-hosted catalog discovery; authenticated provider session/model validation; Windows UI verification |

| R-21 Native startup | Private revision-bound registry, read-only planner, three-task review, exact consent/baseline lease tests; actual native fixture setup through merge | Provider-authenticated planning, dialog accessibility and final packaging |

## Execution order

- [x] Restore full completion criteria and record the user's correction in agentmemory.
- [x] Finish T-03 explicit stop/resume, including integration-test interruption and CLI verification.
- [x] Finish T-02 crash/ownership recovery and cross-run session/working-file ownership.
- [x] Complete human feedback/rework and dependency propagation.
- [x] Implement and verify coordinator-generated review feedback from bounded diff/test evidence using mock subprocess contracts; native contracts remain open above.
- [ ] Enforce dangerous-action capabilities and human gates before native execution.
- [ ] Complete runtime memory/wiki/lessons and source revalidation.
- [ ] Implement the lightweight human-intervention UI using the approved visual direction.
- [ ] Perform real native CLI end-to-end execution/model/session verification.
- [ ] Verify final licenses, resource usage, artifact checksum and complete-product acceptance.

Proceed autonomously under existing authorization while resources permit. Preserve the
user's original scope. Keep the goal active until this audit is proven, rather than
presenting the checkpoint as a finished product.

## New requirement acceptance (R-22–29)

| Requirement | Current evidence | Acceptance |
| --- | --- | --- |
| R-22 quota | Codex authenticated stdio/native refresh; normalized window tests | Partial: Claude capture/resources remain |
| R-23 Windows continuity | Historical CI `38039813985` on `47eb2d7` passed native Windows checks. At this checkpoint the local Windows Rust target is absent (`cargo check --target x86_64-pc-windows-gnu` reports missing `core`/`std`), and the UTM Windows VM is stopped | Verify exact current source on Windows, including package/runtime/UI, containment, and user-visible behavior; no current Windows pass is claimed |
| R-24 platform releases | Exact macOS issue-head package at `ca9495b21c77886df2f75b1672f965dfe75d53fc`: manifest, 214 dependencies, checksum, packaged `--help`, archive integrity and four approval-gated smoke combinations pass | Windows exact-head package/UI, 25/25 packaged UI acceptance, final-source license closure; do not release before every gate passes |
| R-25 subscription-independent supervision | Deterministic execution loop; optional structured review | Partial: idle policies/designated AI lifecycle |
| R-26 per-agent fallback | Eight tests; registered policy/model UI implemented | Partial: native policy interaction/chain restart acceptance |
| R-27 cooperative memory steering | UC-21/DD-29; ledger/REST and daemon fixture; six worker tests cover bootstrap, polling, quiescent exits, unresolved directives, killed-coordinator pause/handoff recovery and explicit resume | Partial: automatic memory discovery/import, worker authentication, company LLM acceptance, interrupted pending/accepted instruction continuation and automatic reviewed-target handoff |
| R-28 status briefing | Six tests; CLI/native factual query; actual three-session read-only UI; exact-ID agentmemory progress import verified against the configured daemon | Partial: automatic live worker report collection, active-provider and expired approval acceptance |
| R-29 status lights | Actual three-state native pulse and Stop/cleanup; mapping tests | Partial: live reduced-motion/resource acceptance |


The 2026-10-21 grouped settings inspection found the workspace-sharing dialog opening at the viewport's upper-left with egui's compact default width, unlike the centered MCP, Messenger, and cache dialogs. A rendered regression failed at x=30 before the fix. The share dialog now anchors to the center with a 480px initial width; 18 focused tests across six Settings/MCP/Messenger/cache/theme suites pass. The exact `97a16bb` macOS package was then opened in one isolated temporary workspace: share was visually verified centered/readable in light and dark themes, MCP, Messenger, and cache dialogs were inspected, profile registration and all five role rows rendered, and the empty Review stage correctly disabled approval. No settings, workspace, or cache data was changed, and no screenshot artifact was retained. This component correction does not increment packaged workflow acceptance: it remains 5/25, with 0/25 shipped Rust replacements. Public screenshots, Windows UI, and the remaining 20 packaged workflows remain open.

A grouped Git/worktree smoke exposed and fixed a repository-switch bug: when the saved goal text was unchanged, saving a different repository retained the old active slug, leaving new plan metadata untracked and blocking the clean-baseline gate. The red test `changing_repository_detaches_the_active_goal_before_planning_again` reproduces this and verifies that creating the next slug leaves the new repository clean. The fix clears only the active slug on repository changes and retains the historical goal record. Focused validation passed 15 tests across five suites; all-target regression passed 771 tests with 14 ignored across 260 suites; all-target Clippy passed with warnings denied. The exact a1ea942 macOS package passed its runtime smoke. One isolated UI run confirmed goal-preserving repository switch, three parallel worktree tasks, 3/3 tests, exact diff review, human approval, merge, and a clean final main worktree. Evidence: `docs/evidence/issue9-a1ea942-git-worktree-ui-smoke.json`. Packaged acceptance is now 5/25; shipped Rust replacement remains 0/25. Windows UI, complete visual parity, accessibility, Korean IME, authenticated provider workflows, idle-resource measurement, final license closure, and exact-head CI remain open.

## Exact-head UI regression and package checkpoint — 2026-10-21


## MCP source-review popup layout — 2026-10-21

The remaining native inline disclosure was the full-source control on MCP import-review cards. A rendered two-record regression exposed an egui widget-ID collision: clicking one source could open the other source's body. The control now uses a per-reference ID and a bounded menu popup; the test verifies the selected source body is visible and the next record stays at the same screen position. The exact earlier Actions run also exposed stale task-popup tests that expected inline content; those now click the popup and verify the bounded editor. Focused MCP/UI/workflow/document/layout coverage passed 33 tests, warning-denied all-target Clippy, formatting, and diff checks. Graphify refreshed to 8,051 nodes / 12,715 edges / 613 communities.

The pushed source `733852e` builds a validated macOS ARM package with SHA-256 `156d40a118ab990fc699b7f0358026ceac2032a33b5952d2666b9cff6d0e4ddd`; ZIP SHA-256 is `aac823d80d230c9435ef3b9ff3025e35edc15936e035314c97530204f7eaa63a`. Package validation, help, archive CRC, Info.plist, and all four approval-gated smoke modes pass. The exact package launched in one process and loaded the Workspace and three configured agent cards. macOS screenshot capture failed, and the MCP-source popup was not opened in that native package window; rendered egui interaction coverage passes. Exact-head cross-platform run `39592532512` remains active. Packaged workflow acceptance remains 8/25; issue #9 stays open.

A further scan found five page-reflow panels: menu-launched User manual, Agent tools guide, workspace-action confirmation, update details, and execution consent. They now use centered egui windows with stable IDs and visible dismissal; the execution-review button remains in place. Rendered regressions confirm surrounding controls retain their coordinates and existing actions remain covered. The full locked all-target suite with `test-fixtures` and `native-ui-probe` passes (508 tests, 14 ignored across 165 suites); all-target Clippy with warnings denied, rustfmt, and diff checks pass. An exact current-source `aarch64-apple-darwin` package from `9cea0a4` passes package validation, four approval-gated smoke cases, packaged help, Info.plist lint, ZIP CRC, and manifest hash checks; evidence is `docs/evidence/issue9-9cea0a4-macos-package.json`. Graphify refreshed locally to 8,095 nodes, 12,822 edges, and 607 communities; CocoIndex is unavailable in this runtime. Actions run `39597791275` for `920d426` remains in progress; its macOS all-target tests and Windows native build are running. CUA timed out when resolving the existing app by name and exact bundle path, so native visual inspection remains incomplete. Packaged workflow acceptance remains 8/25; issue #9 stays open.

## Exact-head package smoke and menu-layout recheck — 2026-10-22

The exact `d23c3c2` macOS package passes manifest validation, packaged `--help`, archive integrity, and all four deterministic shared/worktree × Git/folder smoke cases. Each case runs three overlapping mock tasks and tests, refuses a merge before approval, and verifies the exact candidate after approval. Evidence: `docs/evidence/issue9-d23c3c2-macos-package.json`. In the one-window Rust preview (`78cc5f0`), Workspace and Settings menus were opened and both overlay page content; the production UI is unchanged between that preview and `d23c3c2`. Twenty-two rendered menu/popup layout tests pass across six suites. This remains component evidence; packaged acceptance is 8/25. Actions run `39687975008` is still in progress: macOS native tests are running and Windows is building.

## Current UTM Z: guest-agent boundary — issue #9 / 2026-10-23

The archive `AgentConductor-d1e1b374.tar.gz` is present in the user-provided host share folder `/Users/neodamul/Downloads/UTM`; SHA-256 is `6ead3cd60551fa1a9682a3eeb3420ca034005614680db0a63aa2ab3e06e272cd`. The user's existing Z: mapping was not changed. `utmctl file pull` uses the UTM guest-agent API and returned path-not-found for Z:. Windows maps drive letters per logon session, so this guest-agent result does not prove the interactive user's Z: mapping is disabled. UTM remains started; input capture is off; the current guest screen visibly shows the Guest Tools restart-apps prompt. No VM restart/shutdown or Windows build occurred. See the live screen recheck above; Windows Z: access/build/package/runtime/UI remain unverified. Packaged UI acceptance remains 15/25 (60%); shipped/default Rust UI remains 0/25. Sources: [UTM Windows guest support](https://docs.getutm.app/guest-support/windows/) and [Microsoft drive-mapping guidance](https://learn.microsoft.com/en-us/windows/win32/services/services-and-redirected-drives).

## UTM guest channel recheck after approved reset — issue #9 / 2026-10-24

This follow-up supersedes only the earlier claim that every post-reset file pull failed. The first read-only pull of `C:\Windows\win.ini` succeeded. `utmctl` reports the VM started and returned `192.168.64.6` plus IPv6 addresses. The display remains inactive. A guest `dir Z:\` result was written and retrieved as undecodable localized text consistent with an unavailable drive, which does not prove the logged-in user's drive mapping state. A later `query user` / `net use` command and file pull stopped returning; the host-side clients were interrupted. Do not infer Windows UI, share, build, or package acceptance from guest-agent access. The existing share setting was not changed and no further power action was taken. Evidence: `docs/evidence/issue9-utm-windows-recheck-20261024.json`.

## Exact current-head local macOS Rust package — issue #9 / 2026-10-24

Rust package source `8ef672bd2d94ae38ed6caeab4f1d93d8df52d47c` built offline as v0.1.37 for `aarch64-apple-darwin`. `validate-package`, ZIP integrity, and the package's four shared/worktree × Git/folder smoke cases passed; each case completed three parallel tasks/tests, allowed the expected approved merge, and refused an unapproved merge. The package includes 242 dependency license records, a 242-component CycloneDX SBOM, bilingual offline manuals, and project notices. Binary SHA-256: `95746fd5052c7c4fde4472ecce7afa7db2b01ec38a94c612bc9f15b4e0a78e4e`; archive SHA-256: `605ee5e9b65e26e41a9a2c5b9a9a4151fe41655bc5ee1d3cd5dc12e4e1bfa519`. This is local package evidence only, not a GitHub Release or visual acceptance. Windows package/runtime and default Rust UI acceptance remain open. Evidence: `docs/evidence/issue9-local-macos-package-8ef672bd-20261024.json`.


## User-approved UTM reset follow-up — issue #9 / 2026-10-24

The user approved the UTM VM reset. The restart control was used and its data-loss warning confirmed; the share configuration was preserved. The VM is started, but the display remains `Display output is not active`; Ctrl+Alt+Delete did not restore it, and guest-control commands did not return. No additional power action was taken. The documentation-only package evidence commit `868863a45539bce528088a3c109176fe3547d12e` is pushed. Actions run `40001524380` remains on the Windows build step since 18:17 UTC with its macOS job cancelled; current-head run `40003663262` is queued. Windows desktop, mapped `Z:`, and package behavior are unverified. Acceptance remains **15/25** packaged workflows and **0/25** shipped/default Rust UI. Evidence: `docs/evidence/issue9-utm-approved-reset-followup-20261024.json`.



## Settings gear existing-package spot-check — issue #9 / 2026-10-24

Bound the already-running full-path package `f23381cf` (v0.1.37) without starting another process. The screenshot and accessibility tree showed the gear-only Settings button named `설정`; its menu exposed language, theme, MCP, sharing, messenger, cache, updates, and update-repository actions. Escape dismissed the menu. The Settings renderer and regression test are unchanged from this package source to the current branch. The app showed saved workspace/profile data, so no isolated workflow was attempted; no fields or settings changed and no screenshot was saved to the repository. Current-source focused appearance/roles/plan/workspace tests pass 23/23 across six suites; this adds implementation regression evidence, not packaged workflow acceptance. This spot-check earns no acceptance credit: 15/25 packaged UI workflows, 0/25 shipped/default Rust UI. Evidence: `docs/evidence/issue9-settings-gear-existing-package-20261024.json`.



## Read-only Agent Roles and Plan Review spot-check — issue #9 / 2026-10-24

On the already-running v0.1.37 package, inspected the six-row role matrix and its primary/fallback agent/model columns. Plan Review showed no reviewed plan and disabled save/run controls. Returned to Workspace without generating a plan or changing saved data. This was not an isolated workflow and earns no acceptance credit. No screenshot asset is retained because private workspace context was visible. Evidence: `docs/evidence/issue9-settings-gear-existing-package-20261024.json`.

## Current 0.1.37 README and Workspace manual captures — issue #9 / 2026-10-24

README and both Markdown manuals now identify v0.1.37. The current-source Rust probe replaced the English Workspace image and refreshed the Korean light/dark Workspace images in isolated temporary homes; no provider CLI, MCP server, user workspace, or user profile data was used. The opt-in probe's startup update check requests only the public GitHub latest-release metadata. Both offline HTML manuals were regenerated by the Rust renderer. Four focused documentation suites pass 14 tests. This is documentation evidence only: no packaged workflow acceptance credit is added, Windows remains unverified, and the Agent Room screenshots were not refreshed. Evidence: `docs/evidence/issue9-current-workspace-docs-20261024.json`.

## Current-head native journey and Windows checkpoint — issue #9 / 2026-10-24

The prior macOS Step 3 CI failure is no longer reproducible at the current code: the focused `review_approval_shows_start_feedback_then_opens_the_started_run` test passed locally in 5.31 s, and Platform Validation run `40003966004` passed its full macOS job (native tests, opt-in UI launch, formatting, Clippy and unsafe-comment gate). Local formatting and `source_layout` + `public_docs` also pass. The same run's Windows job passed build/layout and remains in its all-target native test step; later Windows UI-launch, formatting and lint results are pending. UTM is started at `192.168.64.6`, but read-only `utmctl exec ... cmd.exe /c ver` returns OSStatus `-2700`, so guest execution and the mapped Z: drive remain unverified. No code change was made for a failure that is not reproduced. Packaged UI acceptance remains 15/25; shipped/default Rust UI remains 0/25. Evidence: `docs/evidence/issue9-current-head-platform-recheck-20261024.json`.

## Exact current-source macOS package and UI inspection — issue #9 / 2026-10-24

Built local v0.1.37 macOS arm64 package from `08a1783476756a7b79b4823a91a370fcfd943135`. `validate-package`, ZIP integrity and embedded smoke passed. Smoke covered shared/worktree × Git/folder; every case ran three parallel tasks/tests, verified an approved merge and refused an unapproved merge. The binary is 18,858,304 bytes (SHA-256 `5e3fd1dfe84aa0cb7bec6e50918849b3b56bfc676ebda034dfe5df536fe9fd42`); archive SHA-256 is `7cb20690d605ecbf79425d06ec885a08e39c78df58a43ed2d4bb722ac1f4272d`. Manifest reports 242 dependencies; license inventory, SBOM and latest bilingual manuals/screenshots are bundled.

The exact packaged app was opened once with a fresh temporary HOME. CUA inspected Workspace, the six-role matrix, the empty Review state (approval disabled), Settings menu, and Workspace menu. No user configuration or agent service was present; no configuration was saved, and no plan/task/approval/merge was run. The process and temporary HOME were removed afterward. This confirms package startup/rendering but is not a distinct operator workflow; acceptance remains **15/25 packaged** and **0/25 shipped/default Rust UI**. Four older generated package/archive pairs were deleted; only the latest local candidate remains in `dist/`. Windows, final UI acceptance and GitHub release remain open. Evidence: `docs/evidence/issue9-local-macos-package-ui-08a1783-20261024.json`.

## Exact pushed-head macOS candidate and replacement CI — issue #9 / 2026-10-24

The exact pushed source `278ad6de798e73e2a71502af03117c0004d61e92` builds as v0.1.37 macOS arm64. Package manifest validation and ZIP integrity pass; all four embedded shared/worktree × Git/folder smoke cases pass with three tasks/tests each and correct approval-gated merge behavior. Archive SHA-256: `6b3771dbb6343a2874251da678880a2c5df33b1a3ec79be0b0fb44246c4f1c0e`; binary SHA-256 matches the previously visually inspected package. The prior candidate and smoke run were removed. No UI workflow credit is added. Exact-head Actions run `40008320185` passed all macOS checks and Windows build, source/test layout, and all-target tests; Windows native UI launch is in progress, with formatting, Clippy, unsafe-comment gate, and job conclusion pending. UTM guest process execution remains unverified despite the approved reset. Evidence: `docs/evidence/issue9-local-macos-package-278ad6de-20261024.json`.

## Windows validation step advanced — Actions run 40003966004 — issue #9 / 2026-10-24

Updated observation after the earlier checkpoint: the Windows job has now passed both build/layout and all-target native tests. The opt-in native Rust UI launch step is in progress; formatting, warning-denied Clippy and the unsafe-comment gate have not started. macOS remains fully green. This run tests remote source `bc9b7d3`; its product/test code matches the local branch (later commits only update docs). Do not claim Windows validation complete yet. Run: https://github.com/neodamul/AgentConductor/actions/runs/40003966004.

## Approved UTM Windows reset follow-up — issue #9 / 2026-10-24

After the user approved the reset warning, force-stopped and restarted the confirmed `Windows` VM. UTM reports `started` and the guest again has `192.168.64.6`, but `utmctl exec ... cmd.exe /c ver` still returns OSStatus `-2700`; the console says “Display output is not active.” Guest execution and the mapped `Z:` share remain unverified. No further power or share changes were made. Actions run `40003966004` passed Windows build/layout, all-target native tests, opt-in UI launch and formatting, then failed warning-denied Clippy because `tests/windows_console.rs` re-exported unused `language`; the unsafe-comment gate was skipped. Removed that unused item only. Local formatting and warning-denied all-target native Clippy pass. Full local all-target tests are running; a clean Windows run is still required because macOS excludes the `cfg(windows)` test module. Evidence: `docs/evidence/issue9-windows-ci-progress-40003966004-20261024.json`.
