# Exact packaged workflow acceptance

## Windows test teardown and UI Automation input — 2026-10-23

Actions run `39962957846` passed macOS and Windows build/layout, but Windows tests found two test-harness issues. `charter_contract` tried to remove run directories while SQLite `Store` handles were still alive, which Windows rejects; explicitly drop both stores before cleanup. `windows_desktop` used UIA `ValuePattern.SetValue` on a multiline egui text editor, so the driver reported success without changing the app state. Focus the edit and send actual keyboard input, and make PowerShell errors fail the Rust test. Local verification passes `charter_contract` (5/5), source layout (2/2), warning-denied all-target Clippy, rustfmt, and diff checks. The Windows-only UI test remains unverified until the fix is pushed and Windows Actions reruns.

The user confirms Windows already maps host `/Users/neodamul/Downloads/UTM` as `Z:`. No share setup change or VM restart is required for that mapping. The exact `091aa9b4` source archive remains staged in that host folder; guest-side opening/build is still unverified because the restart-apps prompt remains visible.

## Windows manual HTML line-ending regression — 2026-10-23

On Windows, Git checkout produced CRLF in the committed offline HTML manual while the Rust renderer generated LF. The rendered HTML content was current; the test failed only because its full-string equality comparison included line endings. Normalize CRLF to LF on both rendered and checked-in HTML before comparison, preserving exact content validation. The regression is fixed at `091aa9b4576a863d05cdb400a8ace25de7b138c9`; local focused tests, all-target Clippy, rustfmt, and diff checks pass. Run `39962957846` passed the manual asset comparison on Windows and all macOS steps, but Windows failed in unrelated teardown and UIA input tests. The line-ending comparison fix is confirmed; see the test-harness repair section above. Do not close the platform gate until the updated Windows job passes.

The exact source archive is available through the user's existing UTM Z: host share at `/Users/neodamul/Downloads/UTM/AgentConductor-091aa9b4.tar.gz`; source and destination SHA-256 match (`7860127ec648972b2e108bd97fb66a13e71882499d1b4c5723abca58166144eb`). Guest-side Z: access and Windows build/tests remain unverified. The UTM VM remains started; input capture is off.

## Current checkpoint — 2026-10-23

Exact packaged native UI acceptance is **15/25 (60%)**; 10 distinct workflows remain. Workflow 15
now passes in a v0.1.36 macOS arm64 package from pushed source `b282fca47792890bbe85169652164f4ee56c7e42`;
its isolated-home confirmation is recorded in `docs/evidence/issue9-workflow15-ui-isolated-20261023.json`.
Workflows 13
and 14 use the exact v0.1.36 macOS arm64 package from product source
`9845db79af260b631bde7ade2104bae2400cc649`, archive SHA-256
`6bba2a52980147a01402949749789d6d49a615406a9399b5bdd9b449724b1570`. Workflow 14 verifies the
reviewed cache-preview-and-clear path; evidence is
`docs/evidence/issue9-9845db79-cache-cleanup-ui.json`. Workflow 13 verifies the first-use company
Messenger JSON template and its loopback-only fake transport; evidence is
`docs/evidence/issue9-messenger-default-template-20261023.json`. Earlier Workflow 12 is quota-limit
fallback through candidate review, human approval, and merge, with evidence at
`docs/evidence/issue9-ca9495b2-quota-fallback-ui.json`. Workflow 11 switches Agent Room language
while preserving task and approval status, with evidence at
`docs/evidence/issue9-6f1111c-language-status-workflow.json`. Workflow 10 is the coordinator-directed
operator request, with evidence at
`docs/evidence/issue9-b298ae7-packaged-ui-coordinator-request.json`. The exact package contains no
user repository changes. Current bilingual source-build manual screenshots are retained; exact-package
manual validation, Windows UI, accessibility/Korean IME, resource measurements,
authenticated-provider workflows, and final release/license review remain open. Keep issue #9 and
draft PR #22 open.

GitHub CLI rechecked PR #22 on 2026-10-23 at head
`bafa5335220bf1ea0330bc39662cf9d937d265cc`: it was `OPEN`, draft, based on `main`, and
`CONFLICTING`. This is a dated metadata snapshot; query again before describing later PR state.

The completion audit distinguishes the latest product/test commit from the last successful GitHub
PR metadata response. If a later refresh fails, record the failure and leave the live PR state
unconfirmed instead of copying the older open/draft or merge-conflict state forward as current.
The audit guard checks both commit references and the explicit caveat.

## Workflow 15 exact-package observation excluded from acceptance — 2026-10-23

The v0.1.36 macOS arm64 package from source `ec1fc3c1ac06f73d813911e7cc01f2779d9887cf`
rendered the built-in manual's Agent Room screenshot, showed the automatic v0.1.37 update notice,
and displayed scrollable release notes through their final entries. The UI test requested a
temporary HOME and did not save settings, start a task, install an update, or merge. During the
run, however, saved user workspace and agent state appeared; the effective profile/data root and
CUA-bound process were not proven. A startup quota refresh cannot be ruled out. The app exited,
the temporary HOME was removed, and the source worktree remained clean. Do not count this run:
acceptance stays **14/25** pending a repeat with verified process identity and isolated roots.
Evidence: `docs/evidence/issue9-workflow15-ui-isolation-20261023.json`; AgentMemory
`mem_mvkngork_57cb5ccae19a`.

## Workflow 15 isolated-package confirmation — 2026-10-23

Repeated on the v0.1.36 macOS arm64 package from source `b282fca47792890bbe85169652164f4ee56c7e42`
using a direct executable launch, disposable `HOME`, and explicit `ui-new` root. The Workspace menu
reported only `/private/tmp/agentconductor-issue9-isolation-b282fca-20261023/.agentconductor/desktop`;
the initial screen had no registered agents. The bundled manual rendered its embedded Agent Room
image. The update panel showed installed v0.1.36/latest v0.1.37 and scrolled through the final
release-note entry. No task, approval, settings save, update, merge, or agent quota query occurred.
The app, temporary HOME, and package artifacts were removed. This resolves the earlier isolation
gate and advances packaged acceptance to **15/25 (60%)**. The stale manual screenshot was later
replaced with bilingual source-build captures; exact-package verification, Windows behavior, and
other workflow gaps remain open. See
`docs/evidence/issue9-workflow15-ui-isolated-20261023.json`.

The README and English offline manual use a visually checked blank Workspace capture from the
current source-built Rust UI. The Korean manual embeds newly inspected light/dark Workspace
captures. Both manuals now also embed language-matched Agent Room images showing an isolated
three-task, three-test run waiting for approval; the native manual viewer embeds both PNGs. Both
HTML manuals were regenerated and covered by a render-equality regression. These source-build
images add no packaged workflow credit. Exact-package/manual verification remains open. Evidence:
`docs/evidence/issue9-readme-workspace-screenshot-20261023.json` and
`docs/evidence/issue9-korean-workspace-screenshots-20261023.json` and
`docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.

The release-package assembly regression fixture now uses the exact English and Korean Agent Room
PNG asset names and checks both are copied into macOS and Windows package layouts. The test guards
asset collection, but it does not replace an exact current-source package UI inspection or add
workflow acceptance.

## Agent-tools guide component review — 2026-10-23

In the already-running exact v0.1.36 macOS arm64 package from source `9845db79`, the workspace menu opened the separate Agent tools guide. Its accessible content exposed the AgentMemory, Ponytail, RTK, Graphify, and CocoIndex install commands with individual copy buttons. At 1140×768, all commands and both Close controls were visible without clipping or overlap. The operator closed the guide without copying a command, changing settings, or switching the saved workspace. The screenshot was inspected in-session but not retained. This is component evidence only: packaged UI acceptance remains **14/25 (56%)**. Evidence record: `docs/evidence/issue9-agent-tools-guide-component-20261023.json`.

## Stale built-in manual screenshot — found and resolved 2026-10-23

At discovery, the v0.1.36 package rendered `docs/assets/ui-agent-room-native.jpg`, visibly labeled v0.1.24, in both language manuals. This historical stale image was later replaced with separate current-source English and Korean PNGs; both HTML manuals and the native viewer now embed the matching image. The replacement uses a deterministic disposable three-task/three-test fixture and leaves its candidate waiting for approval; no approval or merge occurred. This documentation fix does not change packaged acceptance (15/25). Exact-package verification of the new assets remains open. Discovery evidence: `docs/evidence/issue9-manual-stale-screenshot-20261023.json` and `docs/evidence/issue9-manual-screenshot-run-20261023.json`. Replacement evidence: `docs/evidence/issue9-bilingual-agent-room-screenshots-20261023.json`.

## Exact-package macOS idle resource sample — 2026-10-23

The already-running v0.1.36 package from source `9845db79` was observed without starting a second app window or invoking an agent. Four `top` samples at five-second intervals reported 79 MiB RSS and 0.0% CPU each; one process was present and it had no child process. The executable SHA-256 matched the package manifest. This 15-second idle sample meets the NFR-05 limits for this state only; startup, active work, longer idle periods, Windows, and final-source measurements remain open. Evidence: `docs/evidence/issue9-9845db79-idle-resources-20261023.json`.

## Workflow 14 — preview and clear local cache — 2026-10-23

The exact package ran with an isolated HOME and no registered agents. A disposable completed quota
query was created after startup, then Settings → Preview and clear local cache showed its exact path,
one eligible entry, and zero protected or uncertain entries. Clearing the reviewed inventory removed
only that fixture and preserved the 938-byte `startup.json`. No provider CLI or MCP server ran. The
package's background update check remained enabled; a GitHub release read may have occurred. The
screenshot was inspected but not retained. See the machine-readable
`docs/evidence/issue9-9845db79-cache-cleanup-ui.json`.

## Workflow 13 — first-use company Messenger payload — 2026-10-23

In the exact package, selecting company webhook showed the documented JSON template and Send test
message displayed “Test message sent.” An injected fake curl captured a loopback HTTPS URL and JSON
body; no request was sent and settings were not saved. See
`docs/evidence/issue9-messenger-default-template-20261023.json`.

## Workflow 12 — quota fallback through reviewed merge — 2026-10-23

The exact v0.1.36 macOS arm64 package from product source
`ca9495b21c77886df2f75b1672f965dfe75d53fc` ran three independent tasks in a disposable shared-Git
fixture. Each primary `quota-agent` attempt emitted a structured `usage_limit_reached` event with a
reset timestamp. All three tasks transferred to the configured `mock` fallback on attempt 2,
created only their owned files, and passed `git diff --check` (3/3). The operator reviewed the exact
three-file diff, approved it, refreshed approval-bound evidence, and merged through the native UI.
The run reached Completed and the fixture repository was clean at candidate commit
`9ca548243128d007e8678d64cf4df6654d1a9230`. No real provider CLI, external service, or user
repository was used. This distinct workflow advances acceptance from 11/25 to **12/25 (48%)**;
13 remain. Source/package hashes, run ID, approval revision, and test records:
`docs/evidence/issue9-ca9495b2-quota-fallback-ui.json`.

The exact v0.1.36 macOS package's Korean Agent Room was also inspected in light theme in the
existing single window. Three completed task rows, candidate `3db8840b02ce28558701698c327a5820b7dc7311`,
and all three passing test exit codes were visible. The candidate remained `waiting_approval`;
no approval, rejection, or merge occurred. Dark theme and the saved user workspace were restored.
This screen check does not add a workflow. Evidence: `docs/evidence/issue9-f1602e7-agent-room-light-theme.json`.
AgentMemory: `mem_mvk6fhw8_8eff3cddc6fc`. Graphify's no-cluster extraction had 1,271 dangling
edges, so the curated snapshot was restored. The retained graph has 8,916 nodes / 14,081 edges
and zero dangling endpoints (AgentMemory `mem_mvk6ikgc_355d84efc620`). CocoIndex CLI unavailable,
so no index result is claimed.

## Agent Room goal de-duplication — 2026-10-23

Exact source `f1602e7d32022fac377a168bd2bfb6686593db02` removes the repeated workspace goal from
the narrow task-list column while retaining the single prominent Agent Room header, task rows,
verification summary, and human-review controls. The red AccessKit regression observed the same
goal at both header and sidebar positions; after the fix, all goal bounds occupy the header row.
The exact v0.1.36 macOS arm64 package passed manifest validation and smoke, and a single-window
visual check confirmed the goal appears once. The disposable candidate remained
`waiting_approval` with 3/3 tasks and tests; no approval or merge occurred. Evidence:
`docs/evidence/issue9-f1602e7-agent-room-goal-layout.json`. This component fix adds no workflow;
packaged UI acceptance stays 11/25. Windows UI remains unverified.

The related selected-language Agent Room regression verifies that the app name is absent from the
page because the shared native shell owns the branding. Its initial failure was an obsolete test
expectation; the production page already avoided duplicate branding. The corrected focused tests
pass. The distinct language-and-status journey is recorded as Workflow 11 below.

## Workflow 11 — Agent Room language and status — 2026-10-23

In the exact v0.1.36 macOS package built from `6f1111c5ae61c91933e03a20fb04a4df513193de`,
switch Settings from Korean to English and back while checking Agent Room phase, task, test,
review-evidence, and candidate-action labels. The pending candidate remained `waiting_approval`
with 3/3 tasks and 3/3 tests; no approval, rejection, or merge occurred. Evidence:
`docs/evidence/issue9-6f1111c-language-status-workflow.json`. This distinct workflow advances
acceptance from 10/25 to **11/25 (44%)**; 14 workflows remain.

## Workflow 10 — coordinator-directed operator request — 2026-10-23

In the exact v0.1.36 macOS package at `b298ae79774e`, a coordinator request sent from Agent Room
through a disposable copied run completed the visible queued → processing → completed lifecycle
and returned an evidence-backed read-only response. The candidate stayed `waiting_approval`; all
three tasks and tests remained complete, and no approval or merge was created. The original run
was untouched. Evidence: `docs/evidence/issue9-b298ae7-packaged-ui-coordinator-request.json`.
This distinct workflow advances packaged UI acceptance from 9/25 to **10/25 (40%)**.

## Exact v0.1.34 package UI revalidation — 2026-10-22

The exact v0.1.34 macOS ARM package built from `76d8082b02a722ab18cc4e2f0e8383b3c1a3fdd5`
was opened in one app window. The binary SHA-256 is
`6076725213cde19f24b29cc5111da9ac6e885cb70c78e377a122376e290c0e05`; its archive SHA-256 is
`8b33b1378b4e3e145b90518609ecc1b70a9c5a67440e53c44d3df098dfa5ccab`. The UI completed a
manual, reviewed three-task plan in Git worktrees using only a local deterministic custom CLI
fixture. Each worktree produced one disjoint file and passed its local test. The exact diff and
test output were reviewed in the UI; candidate and merge each required explicit confirmation, and
the run reached Completed with a clean disposable project. No provider CLI or external service ran.

The first fixture output used Codex-shaped `turn.completed` events while the registered mock
profile used the Custom parser; that is not recognized as a successful Custom result. The fixture
was corrected to emit `{"type":"result"}` and protocol tests pass (9/9). This is a harness format
mismatch, not a product parser change. One initial manual-plan draft showed three summary tasks but
only two editable task rows; adding the third row allowed the run, and a later fresh manual plan
rendered all three. Reproduce before calling it a defect.

This revalidates existing packaged plan-to-merge workflows 4 and 5, so acceptance stays **8/25**;
no new distinct workflow is counted. Machine evidence: `docs/evidence/issue9-76d8082-macos-package-ui-workflow.json`.

## Three-task Review row visibility follow-up — 2026-10-22

The v0.1.34 package note above described a manual plan whose summary showed three tasks while the
editor seemed to show only two. A rendered test now exercises the complete current Rust Workspace
screen at 1140×772 with three manual tasks. All three localized task disclosure controls are present
and their accessibility bounds are inside the viewport. The first test assertion used English text
while the host defaulted to Korean; changing the assertion to use the active locale fixed the test
harness, and the observation was not reproduced as a current-source defect.

This is component evidence, not exact-package runtime verification; acceptance remains **8/25**.
Evidence: `docs/evidence/issue9-review-three-task-rows.json`.


## Historical checkpoint — 2026-10-22

Exact packaged end-to-end UI acceptance for issue #9 is **8/25 (32%)**; native Rust source
coverage is 25/25, while the shipped/default Rust UI remains **0/25** until an accepted release
ships. Exact PR #22 head `287e9f2` passed macOS and Windows validation and native-package execution
in Actions run [39735141070](https://github.com/neodamul/AgentConductor/actions/runs/39735141070).
Artifacts: macOS `12407514908` (21,600,253 bytes, SHA-256
`1151a58d91fafa17bb3900bd34edf24418ae98857a45e64ac642cbc271adcd08`); Windows `12408746101`
(15,718,557 bytes, SHA-256 `efda584bf92e48290e478ab2ccb820c55cf6bf9dbd0da28b62f806e04b3eabb8`).
This verifies builds, Rust-generated bilingual manuals, and exact-package smoke execution, not UI parity
or a distinct workflow. Evidence: `docs/evidence/issue9-287e9f2-exact-head-ci.json`.

Workflow 8 covers OpenCode-format MCP server registration, save/reload, and local tool discovery. The
exact `256db1a` macOS package registered the local server and discovered `get_issue` without calling
it. Evidence: `docs/evidence/issue9-256db1a-mcp-server-registration-ui.json`.
Workflow 7 covers workspace settings export and import: the exact `256db1a` macOS package
exported settings, previewed them, required explicit confirmation, and restored the original name.
Evidence: `docs/evidence/issue9-256db1a-settings-share-ui.json`.

Windows GUI behavior, screen-by-screen parity, retained current screenshots,
authenticated-provider workflows, accessibility/Korean IME, idle-resource measurement, and final
license/release audit remain open. Seventeen packaged workflows remain. Smoke tests, source tests,
repeated journeys, and screen-only audits do not increment this count. Keep issue #9 and draft PR
#22 open.

## Exact-source macOS package for issue #9 — 2026-10-20

Source `7419557168ddc3ffe76c446cf292152cc9fc1932` produced a validated `aarch64-apple-darwin` package. The 17,925,856-byte executable SHA-256 is `6988a2dd852b23665be6f43f4d2d7e20092dbef5e20db49574cd6d1982aa6836`; the 21,580,718-byte archive SHA-256 is `610f3987658acd5e8c319d827539ab0fe2745d45ee28d2dced754c50365becfb`. The manifest records 214 dependencies and bundled SQLite. Package validation, packaged `--help`, ZIP CRC, and archive checksum pass.

The Rust package smoke completed shared/worktree × Git/folder with three parallel mock tasks and tests in each case, refused unapproved merges, and verified approved merges. No provider CLI/API was invoked. Machine-readable [package](../../docs/evidence/issue9-7419557-macos-package.json) and [smoke](../../docs/evidence/issue9-7419557-macos-smoke.json) evidence are retained. This does not verify the GUI, Windows package, or a release; packaged native UI acceptance remains **2/25**.

## Historical checkpoint — 2026-10-20

The exact v0.1.30 macOS package built from source `35a5c095ef2d00871065233dc82be20d53db8abd`
completed shared-folder Git workflow 1 in the packaged native UI. A Rust mock coordinator
created three disjoint tasks; three local Rust mock workers ran concurrently; all three
`git diff --check` tests passed; the operator reviewed and approved the candidate and merge;
the UI reached Completed and the disposable repository was clean. No provider CLI, API,
credential, or external service was used.

Evidence: [`docs/evidence/issue9-35a5c09-native-workflow-01.json`](../../docs/evidence/issue9-35a5c09-native-workflow-01.json).

The exact v0.1.31 macOS package from source `2fc7bc22264724219f36c0e9a1c3858ffdc38025`
completed packaged UI workflow 3: register a local CLI profile, edit its ID, confirm removal,
and verify the empty-agent state in one isolated window. It did not invoke the configured CLI.
Evidence: [`docs/evidence/issue9-2fc7bc2-agent-profile-ui.json`](../../docs/evidence/issue9-2fc7bc2-agent-profile-ui.json).

The current verified packaged native UI count is **2/25 (8%)**. Source coverage remains 25/25;
the shipped Rust replacement remains 0/25 until a release ships. Windows acceptance and visual
parity remain open.

## Exact v0.1.31 macOS package runtime verification — 2026-10-20

The clean source commit `d884d7c769b51c8478fa3919777628b328621d4f` built a validated
`aarch64-apple-darwin` package with 214 dependency notices. The 17,909,232-byte binary
SHA-256 is `1d9823bd9fd1bb355596e522b75515fb8947450e5360144265af39c1f2e38ac1`; the ZIP
SHA-256 is `8d1bd8c17e18120b3b9ee020ff7b6aaedc3b60552cc2f5da88febeab8c849ff8`.
The ZIP integrity test, package manifest validation, packaged `--help`, and all four
shared/worktree × Git/folder deterministic smoke cases passed. Each case ran three
parallel tasks/tests, denied unapproved merge, and verified approved merge. The package
archive contains no project-owned `.js` or `.py` files. No desktop window or external
agent CLI/service was launched. This verifies the exact packaged runtime only; it does
not advance native UI acceptance beyond 1/25 or verify Windows, accessibility, or IME.

Evidence: [`docs/evidence/issue9-d884d7c-macos-package.json`](../../docs/evidence/issue9-d884d7c-macos-package.json).

## Exact issue-head package smoke — 2026-10-20

The optimized macOS package built from exact issue #9 HEAD
`1ff6814fa0f6bce185e3f03fa1e5ce7b894a2cdd` passed package validation and its packaged
help command. Its deterministic smoke runner passed shared/worktree × Git/folder cases, each
with three parallel tasks and tests, rejection of an unapproved merge, and verification of an
approved merge. Evidence: [`docs/evidence/issue9-1ff6814-macos-package.json`](../../docs/evidence/issue9-1ff6814-macos-package.json).
This package/runtime check does not exercise the native UI and does not change the accepted
native UI workflow count of 1/25. Exact-head macOS/Windows Platform Validation run
`39365586146` remains pending.

Trace: R-24 → BG-10 → SF-20 → UC-17 → QAS-14/QAS-18 → DDR-03/DDR-08 → DD-27.
The Rust `smoke-package` command validates the package manifest, exact binary
name, source commit and SHA256SUMS before executing the candidate. The Rust
acceptance suite exercises shared and worktree modes on Git repositories and plain
folders, including parallel scoped tasks, tests, diffs and approval-gated merge.

The exact macOS da746b9 package completed shared and worktree fixtures. In each:
three CLI workers overlapped, completed their owned file, emitted JSONL session
and result records; three integration tests passed; diff included all owned files;
target HEAD stayed at base before approval; unapproved merge was denied for missing
approval; explicit exact candidate approval followed by merge produced the candidate
HEAD, correct scoped files and a clean Git state. No provider generation or network
was used. This is compiled packaged orchestrator evidence with deterministic workers,
not authenticated Claude/Codex/OpenCode or company LLM acceptance.

Current source verification (2026-10-15): the new command passed in debug and
release-mode integration tests; the full Rust suite passed 478 tests with 14 ignored.
Exact-source macOS and Windows package CI for this runner is still pending.
The prior 90df88e Windows run also exposed a ZIP timestamp test-fixture defect: it set the epoch timestamp only on Unix but asserted the 1980 clamp on all systems. The fixture now sets and verifies the epoch cross-platform; focused macOS archive tests pass, and Windows confirmation is pending.

Raw evidence: `.agentconductor/verification/packaged-workflow/smoke.json` and command
stdout/stderr. Successful temporary workspaces are removed; failed workspaces remain
available for diagnosis. All child commands are terminal, and no GUI or daemon was
launched.

CI now builds native release binaries only after both platform validation jobs pass,
packages and executes them with this same runner, then creates archives and archive
checksums. Archives and smoke evidence are retained as CI artifacts; this workflow
never creates a GitHub Release. Native Windows packaged evidence remains pending.
The upload action is actions/upload-artifact v7.0.2, pinned to
`cf430e030ddbb5b0abf93d22962f4752f3646cd9`; its repository LICENSE was reviewed as MIT
and it is a CI dependency, not bundled runtime code.

Sources:
- [src/release_smoke.rs](../../src/release_smoke.rs): SHA-256 `134c8d748ffa6e5e3ac556c3c1fd95721e86d74bb2c0215f836414851772e037`.
- [tests/release_smoke.rs](../../tests/release_smoke.rs): SHA-256 `e6675307ebb535dcaf27f4eb6b14d196f4c45241791795486a0cfc72731da9cb`.
- [src/release_archive.rs](../../src/release_archive.rs): SHA-256 `e16d69b2d8752773d633710eac81a48fe022e97a8125834b3bf7c1fd6cfba4f5`.
- [src/macos_bundle.rs](../../src/macos_bundle.rs): SHA-256 `94192a13d69a8f1a74294f6d4a87313c954eb0a08a94080d89698d98da2c307f`.
- [src/release_package/driver.rs](../../src/release_package/driver.rs): Rust `package-release` orchestration reads locked metadata offline, assembles the package, runs packaged-workflow smoke checks, creates the ZIP, and writes its checksum.
- [src/release_smoke.rs](../../src/release_smoke.rs): validates package provenance/checksums before running the exact packaged binary and exercising approval-gated merge behavior.
- [src/release_archive.rs](../../src/release_archive.rs): creates the deterministic platform archive and refuses unsafe package entries.
- [tests/support/release_worker.rs](../../tests/support/release_worker.rs): SHA-256 `1e48fa4b43dc6be3686c29e9b0e5f1e7e2143f7b64b895dbe886593dd7e60554`.

## Registry timestamp regression and actual archive

The first actual archive attempt failed because cached license files had timestamps
before 1980. A new Red/Green contract covers ZIP date clamping while preserving
file bytes and executable permission bits, and refuses overwriting an archive.
The former Python archive writer was replaced with the Rust archive module, which
clamps DOS timestamps, retains a pending archive on failure, and refuses symlinks.
A failed partial local ZIP was preserved with a .failed suffix before verification.

Actual d765cd2 macOS package archive: 2,978,551 bytes, SHA-256
`5a104600fa0e446adb82d351bed0d65508973567e749cec5fe476af9c6c77ee7`.
All ZIP entry CRCs passed and the archived binary matched its package manifest.
Evidence: `.agentconductor/verification/packaged-workflow/archive.json`.
Graphify structural extraction: 1,742 nodes / 3,475 edges (before archive follow-up).
CocoIndex located the binary-validation definition and call without an index/LLM.
Ponytail has no callable capability here; no execution is claimed.

## Exact-source checkpoint — 2026-10-17

The macOS release archive built from source commit `98d53575c15d2c2772c40edbbc842efb176cbebe` has SHA-256 `9b5f9a76a177887865e6ffca585c64f0e341a87889811ff22fa7ed8169846190`; archive integrity and all four packaged workflow smoke scenarios pass locally. The Rust smoke runner now creates a unique evidence directory for each invocation, so repeated runs preserve their own command logs and cannot collide. A regression test runs the same package twice at one evidence root.

Run [38995553011](https://github.com/neodamul/AgentConductor/actions/runs/38995553011) is validating the exact source on macOS and Windows and will build and execute both native packages after validation. Run `38995553011` failed the macOS `release_license_spdx` fixture after macOS build/layout passed; its package stage was skipped. Windows build/layout passed and tests were in progress. The local failure was reproduced and fixed in the fixture by copying the exact pinned upstream license text and checking the emitted file bytes instead of assuming a `LICENSE-MIT` filename. The focused package/provenance/smoke suites pass locally; The obsolete run was canceled after the macOS failure. New exact-source cross-platform run [38996644132](https://github.com/neodamul/AgentConductor/actions/runs/38996644132) is pending for `afa2182`.

## Initial exact 0.1.30 package GUI plan — 2026-10-19 (superseded by current checkpoint)

The packaged native UI generated a three-task plan using the Rust mock CLI in a disposable Git
repository. The plan is pinned to base `0c17ea0`, digest
`a44381e87c6a2fe6d6d00b6dadd1382b21a42e8dd290ee1528d31f058ed0e3b9`, writable files
`one.txt`, `two.txt`, `three.txt`, and `git diff --check` tests. The app displayed the exact-plan
consent gate. At that time no worker had run and the count did not change. The setup was later
regenerated from a clean fixture and completed as described in the 2026-10-20 checkpoint above;
the earlier digest, base commit, and pending status are historical only, not the accepted workflow
evidence.


## Current Windows test-fix archive staging — 2026-10-23

The pushed issue #9 head `82c1b309eb21212a88fbb765659772d16fd03a66` was archived as
`AgentConductor-82c1b309.tar.gz` and copied to the user's existing host folder
`/Users/neodamul/Downloads/UTM`. The source and host-copy SHA-256 values match:
`23726a374a9bb4aac42cd5382c012ea20f68c6edc4fc815999c0272cef7e9e68`.

The user confirmed the Windows guest already maps this folder as `Z:`. Do not alter
the mapping or VM power state. UTM reports Windows started. A PowerShell probe
started by `utmctl exec` did not create a host-visible marker. This only says the
guest-agent process did not write through the user's host mapping; it does not test
the interactive Windows logon session. The Windows build/package/UI check remains
open. `utmctl exec --debug` returned two OSStatus `-10004` errors and no guest
stdout; `utmctl file push` aborted with signal 6. These guest-agent failures are
separate from the user's interactive mapped-drive session. The first post-fix Actions run
[`39967301046`](https://github.com/neodamul/AgentConductor/actions/runs/39967301046)
was superseded by the documentation commit: macOS validation steps passed, while
Windows was cancelled during build. Replacement run
[`39968582271`](https://github.com/neodamul/AgentConductor/actions/runs/39968582271)
has passed both validation jobs, including Windows build, source/test layout, native
tests, opt-in UI launch, formatting, and Clippy. The Windows and macOS release-package
jobs are now running; no release-package pass is claimed yet.

Read-only GitHub metadata currently reports PR #22 OPEN/DRAFT at head `d7b8989d`
against base `602ec118`; it is `MERGEABLE` while checks are still `UNSTABLE`. The
issue body still contains the older `e7ec233` head.

Evidence: `docs/evidence/issue9-utm-z-share-82c1b309-20261023.json`.


## Exact Rust macOS package checkpoint — 2026-10-23

`package-release` built current Rust branch commit `91393d7b82c92b0c3e4891b4730fc8475aa250fd`
for `aarch64-apple-darwin`. `validate-package` passed, the package smoke run succeeded,
and ZIP integrity passed. The archive is 21,672,395 bytes with SHA-256
`9c068e85d2610f3736b935f3bcdc49dfeb7194a34c73141c568b8ae831c369c6`; the bundled
native executable is 18,858,304 bytes with SHA-256
`3fa44f63193cdb47ec6d06db450a090804b1241eddb4938e3c2da4451beeaa65`. The manifest
records 242 dependencies and the package includes dependency license notices and the
SBOM. The tracked source inventory has no JavaScript, TypeScript, or Python files.

This local package is not a GitHub release. No Windows package or native UI check has
passed, and this package run adds no UI-workflow acceptance. The exact package UI, the
Windows target, shipped-main integration, and remaining release gates are open.
Old generated archives/packages were removed from `dist`; only this package and its
checksum were retained. Evidence: `docs/evidence/issue9-rust-macos-package-91393d7-20261023.json`.


## 2026-10-23 Workflow 9 — post-approval charter drift refusal

Exact package `aa8090405239` approved a known candidate in a disposable copied run. The copy's
baseline charter matched its plan; changing only the saved plan goal caused the exact package's
merge command to exit 1 with `[goal_drift]`. The audit contains one judgment and the fixture
repository HEAD is unchanged. The native Agent Room displayed the localized Korean blocked
judgment, retained 3/3 task and test results, and disabled Merge. The original app-managed
approval candidate remained untouched and was restored to the single window. The synthetic
charter baseline and plan mutation existed only in the disposable copy; no provider or external
service ran. Evidence: `docs/evidence/issue9-aa80904-goal-drift-workflow.json`.

This is a distinct failure-branch workflow after exact human candidate approval, not a repeat of
the happy-path merge. Packaged native UI acceptance advances from 8/25 to **9/25 (36%)**; 16
workflows remain. Windows packaged UI acceptance remains open.
