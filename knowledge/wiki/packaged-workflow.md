# Exact packaged workflow acceptance

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


## Current checkpoint — 2026-10-22

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
