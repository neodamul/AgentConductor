# Rust native release packaging

## Windows tool-decision package journey timing — 2026-10-30

The Windows UI Automation driver observed `Confirm tool decision` in the final
accessibility dump after reporting that the expected state had not appeared within
500 ms. Its activation helper could then send repeated pointer clicks (up to three)
for one approval action, after UIA and keyboard attempts. The driver now verifies
the in-window process-owned target, sends one pointer click, and waits up to four
seconds for the resulting state. It no longer sends alternate approval inputs when
the outcome is ambiguous. A source-contract regression prevents retries and
alternate inputs from returning.
Local static Windows-driver tests pass on macOS, but Windows package acceptance is
still pending; this change does not claim a successful Windows runtime journey.

## Exact-current macOS package at issue #9 head — 2026-10-30

Built `codex/issue-9-git-head-fast-path` at source commit
`043d2ecb13022966d3ee8c09f2fd1ed8c9ae4510` with
`cargo run --release --locked --offline -j 2 -- package-release`. The Apple
Silicon archive is `dist/agentconductor-aarch64-apple-darwin-043d2ecb1302.zip`
with SHA-256
`e46c2e4973fe82ad494e66756f2a2802728fba8a6ebe99b8046de9dd93c40244`. Its
manifest records the exact source commit, a 19,523,904-byte executable, and
binary SHA-256
`c202e96cd51e21a030080e23e276e2c2b3aa2f37abae9c84ba9f5d065a567a71`.
`validate-package`, `SHA256SUMS`, ZIP integrity, and packaged `--help` pass. The
exact packaged executable also passed the four-case shared/worktree × Git/folder
mock workflow smoke, covering 12 parallel tasks and tests, refusal of merge
before approval, and verified merge after approval. This package contains
the bilingual offline manuals, license inventory, notices, SBOM, and README
assets. It does not prove exact GUI acceptance or Windows behavior and is not
a GitHub Release. Package evidence:
`docs/evidence/issue9-043d2ecb-macos-package.json`; workflow evidence:
`docs/evidence/issue9-043d2ecb-macos-smoke.json`.

## Exact macOS package for chat-send recovery — issue #9 / 2026-10-29

The optimized Rust release package was built locally from clean source commit
`f83c5fb15fb088eebd2d622357d719c074da4572` using
`cargo run --release --locked -- package-release`. The `aarch64-apple-darwin`
archive SHA-256 is
`ba37d5215b584b0a776c2942045796dc3918a5b6b6e82b82d487c268e8997423`.
Its manifest names the exact source commit; the embedded binary digest
`d7ab72c2a2523a080c790d3e9e1b4891af417717559e4423a415bcd82ead7c21` matches
the package `SHA256SUMS`. The ZIP includes both offline HTML manuals, license
inventory, notices, and SBOM. This verifies the local Apple Silicon package
only. Windows package recovery, the stopped full test suite, overall UI
acceptance, and PR #22 mergeability remain unresolved. Evidence and checklist:
`docs/07-implementation-plan.md`.

## Cross-platform package artifacts — issue #9 / 2026-10-24

Run `40011350882` on source `db568c3f59fea29b15c8d60829e7fd41857fc62e` passed all macOS and Windows validation gates, release executable builds, bilingual manual rendering, exact-package execution/smoke, and artifact retention. macOS artifact `12527489015` is 21,164,379 bytes (artifact SHA-256 `faf6d2a0acea970638e4c59697d77175737c44e8dd79fc51ccc8971ca0703a79`; ZIP SHA-256 `47075ce7a6908926d14d049e8613a02bce862170ff58bf3992b1c17f41ef7629`). Windows artifact `12528158860` is 15,301,355 bytes (artifact SHA-256 `02bc04f6cd86b4e9bdaa96c6d05bac58d46a94b5ee6b377dbaa1adf546b7e856`; ZIP SHA-256 `9927f872a25505804c3f07ccff70ef33e318c49fdab83d2e15fe599c5a19e8d1`). These are temporary GitHub Actions artifacts expiring 2026-11-06, not a published release. Issue #9 remains open at 16/25 exact-package UI workflows; shipped/default Rust UI remains 0/25. Evidence: `docs/evidence/issue9-platform-packages-40011350882-20261024.json`.

## Generated debug-cache cleanup — 2026-10-23

The Codex worktree's `target/` measured 9.7 GB (9.0 GB debug and 731 MB release). With no active Cargo/Rust build, the regenerable `target/debug` tree was removed to free about 9 GB while preserving `target/release`, the 42 MB extracted v0.1.36 app under `dist/`, and the release ZIP. The retained executable still exists and `--help` succeeds. A later debug test/build must recompile dependencies. This is storage hygiene only; it does not alter source or package evidence. Graphify refreshed the local code graph to 8,970 nodes, 14,140 edges, and 692 communities; the index remains ignored generated data. CocoIndex CLI is not installed in this runtime, so no CocoIndex result is claimed. Machine record: `docs/evidence/issue9-target-debug-cache-cleanup-20261023.json`.

## v0.1.36 CI dependency license audit — 2026-10-22

The paired v0.1.36 packages from PR-validation run `39740270740` record merge source `eff9f448` (head `cd9935de`, base `3fecd21d`). The macOS and Windows SBOMs contain 214 and 201 components, respectively, with no missing license expressions; manifest counts, archive checksums, and executable hashes match the embedded package records. `self_cell 1.3.0` is the only GPL-optional dependency (`Apache-2.0 OR GPL-2.0-only`); both packages bundle both license texts, so the Apache-2.0 option is available. This is package evidence, not a substitute for legal review. Repeat dependency closure against the final release source after issue #9 acceptance. Detailed evidence: `docs/evidence/issue9-eff9f448-license-audit.json`.

The native package path is implemented in Rust. CI builds the target executable,
renders the bilingual offline manuals, then runs
`target/release/agentconductor package-release`. The command reads locked Cargo
metadata offline, requires a clean tracked source tree, assembles the target package,
runs the packaged-workflow smoke suite, creates the ZIP, and writes its SHA-256 file.

The assembler refuses an existing destination and cleans partial staging on error.
It includes the exact platform binary, the macOS `.app` bundle where applicable,
README-linked public assets, README, product license, Cargo.lock, a dependency
license inventory, reviewed dependency license texts, third-party notices, a
CycloneDX SBOM, binary checksum, and source-commit manifest. License checks enforce
an explicit SPDX allowlist and validate upstream text against package name, version,
VCS revision, and SHA-256. The bundled SQLite copyright disclaimer is retained.

The Rust contract tests cover macOS and Windows package layouts, checksums and LF
text, overwrite refusal, missing/unsupported license rejection before output,
reviewed-license provenance and integrity, SQLite notice inclusion, staging cleanup,
and Windows-style `autocrlf` checkout preservation. These fixture tests are separate
from executing native packages on both operating systems. Exact-source GitHub CI is
still required for platform package evidence.

## Stable v0.1.36 release — 2026-10-22

Release workflow [39728295847](https://github.com/neodamul/AgentConductor/actions/runs/39728295847)
passed macOS and Windows validation, exact-package execution, and publication for source
`6d0093d3576e50642bad39fa885b9b5695b0281c`. The stable, non-prerelease release is
[v0.1.36](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.36). The release
contains macOS Apple Silicon archive
`agentconductor-aarch64-apple-darwin-6d0093d3576e.zip` (SHA-256
`7af9cdc0f3631f60a3e8f58a48923c04bca70a01cbaf6ca1cf73322341ecc415`), Windows x86_64
archive `agentconductor-x86_64-pc-windows-msvc-6d0093d3576e.zip` (SHA-256
`c6970e8ffec84a6c2b42a6490c2840130d7ba6aeadc6afaf6f594b17539216bb`), and
`SHA256SUMS`; the downloaded checksum file matches the uploaded archive asset digests.
Both platform jobs rendered the bilingual manuals from the Rust binary and executed the
exact package before the publish job. This ships the v0.1.36 main application; the separate
issue #9 native Rust UI migration remains at 8/25 packaged workflows and PR #22 remains
open.

The latest completed validation of the source before the current package changes was
run 38730449653 on commit `a4a5fe0`: macOS validation passed, Windows build, layout,
tests, package contracts, and formatting passed, then Windows Clippy failed on a
Unix-only ZIP permission reassignment. The exact package jobs were skipped. A cfg
shadow-binding fix is now in the working tree; this run does not validate it.

No JavaScript or Python runtime is introduced into the packaged executable by this
packaging path. UI JavaScript and other project-owned verification tools remain
outside this slice and must be migrated and tested before issue #9 is complete.

Sources:
- [src/release_package.rs](../../src/release_package.rs)
- [src/release_package/licenses.rs](../../src/release_package/licenses.rs)
- [src/release_package/driver.rs](../../src/release_package/driver.rs)
- [tests/release_package.rs](../../tests/release_package.rs)
- [src/release_archive.rs](../../src/release_archive.rs)
- [.github/workflows/platform-validation.yml](../../.github/workflows/platform-validation.yml)
- [licenses/upstream/sources.json](../../licenses/upstream/sources.json)
- [.gitattributes](../../.gitattributes)

## 2026-10-17 exact macOS package

Commit `c7e2dc9e640c78023cb52c9f598b7652c41f2d7c` produced the aarch64 macOS ZIP
`agentconductor-aarch64-apple-darwin-c7e2dc9e640c.zip` with SHA-256
`dbd608d6ae5d10b705f82bcbb5148e24b78c2a4178e43741d86ec278db808e01`. The archive
integrity check and packaged workflow smoke passed four cases: Git and folder
sources, each with shared and worktree execution. Each case verified three parallel
tasks, tests, approval refusal, and approved merge. The selected macOS dependency
graph contains 182 packages and the package includes its dependency licenses,
notices, and CycloneDX SBOM.

This local result does not replace exact-source Windows package CI or Windows runtime
acceptance. The latest platform run, 38943406077, was canceled after about four
minutes while both runners were in the initial all-target build; package jobs did
not run. The local Mac has no Windows target or linker. See [upstream license text evidence](upstream-license-texts.md)
for Windows license provenance and the remaining attribution review gate.

## Exact-source package checkpoint — 98d5357 (2026-10-17)

The local `aarch64-apple-darwin` package built from `98d53575c15d2c2772c40edbbc842efb176cbebe` has SHA-256 `9b5f9a76a177887865e6ffca585c64f0e341a87889811ff22fa7ed8169846190`. Archive integrity and all four packaged workflow smoke scenarios pass. The current 214-dependency host audit validates allowed SPDX expressions and exact upstream license provenance. Focused provenance, release-package, and smoke tests pass; Clippy is clean.

The exact-source cross-platform workflow is [38995553011](https://github.com/neodamul/AgentConductor/actions/runs/38995553011). Run `38995553011` passed macOS build and source-layout validation but failed in `release_license_spdx`; the macOS package stage was skipped. Windows build and layout passed, and Windows tests were still running at the last check. Local reproduction identified a test-fixture defect: it copied the provenance index but omitted the matching pinned license file, and asserted a generic filename. The fixture now copies matching reviewed source files and compares exact packaged bytes. Twelve focused tests across four suites, Clippy, formatting, and the source-layout guard pass locally. The obsolete run was canceled after the macOS failure. New exact-source cross-platform run [38996644132](https://github.com/neodamul/AgentConductor/actions/runs/38996644132) is pending for `afa2182`. The local machine has no Windows target standard library/linker. Issue #9 remains open for exact Windows package evidence and native UI acceptance, including public Agent Room captures and live Korean IME checks.

## 2026-10-19 exact 0.1.30 macOS package

Committed HEAD `dbad31da69cc5dde04559af1e4695686dc2ae53b` built offline for `aarch64-apple-darwin`. `package-release` completed its four shared/worktree × Git/folder smoke scenarios, and `validate-package` accepted the manifest with 214 dependency license records. Binary SHA-256: `d2b66628f69d248e5dc92b1244c035afb783ab4280057831fa20174ae0e38e85`; ZIP SHA-256: `07b8956372b0f10ddbbacbfd74e645d1ac345a1c1eeebf2e971acd57a740f9eb`.

The exact packaged executable also opened one isolated native macOS window; a disposable Workspace was saved and persisted. This verifies package launch and one settings action only. Windows, the legacy UI visual comparison, and issue #9's 25 end-to-end acceptance workflows remain open (0/25).

## Published stable v0.1.30 artifact verification

The published [v0.1.30 release](https://github.com/neodamul/AgentConductor/releases/tag/v0.1.30)
uses source commit `2da56d91d35f26448e49a789b1f1ab1e717f5698`. Downloaded both
platform ZIPs and verified GitHub's SHA-256 asset digests and the binary checksum
inside each extracted package. Both include README, English/Korean Markdown and
generated HTML manuals, screenshots, dependency licenses, notices, and SBOM. This
release predates `db68218` and does not include its Windows custom-registry fix.


## 2026-10-20 exact-source macOS package at `47a8b37`

The current `aarch64-apple-darwin` release binary was rebuilt offline at HEAD `47a8b37a4eb3beb085930061dc2ce341238175a8`. Rust `package-release` and `validate-package` passed. The manifest records 214 dependencies, binary SHA-256 `9313d3adfa62682d1b85930eaffe9956ca364dc352587c586204ae19282d8562`, and the archive SHA-256 is `81d2ee3b1057974100de057c6feb09be88e373658367f3c25abbdf9b16bf110c`.

All four packaged mock workflow cases passed: shared/worktree × Git/folder, three parallel tasks and tests per case, unapproved merge refused, and approved merge verified. README, bilingual Markdown and rendered HTML manuals, dependency-license inventory, notices, and CycloneDX SBOM are present. Evidence: [`docs/evidence/issue9-47a8b37-macos-package.json`](../../docs/evidence/issue9-47a8b37-macos-package.json). This does not verify Windows packaging, native UI visual parity, or the 25 end-to-end acceptance workflows.

## 2026-10-20 exact PR-head package at 4b0a928

The exact pushed issue #9 head 4b0a928f65c00fece86a42992602bd7e945ae51 produced an aarch64-apple-darwin package. The package-release command and validator passed. Archive SHA-256: 8db25928b98f8ccd0c858f9fc3c32f3789d7ec201579c2d247d16e976f4217df. The 17,644,416-byte binary SHA-256 is 150dfe6f8473ba38c28606c1b0d2e248761022ec5b7d7225199584a3d94666cb. The manifest contains 214 dependency license records.

All four smoke cases passed for shared/worktree × Git/folder execution, with three parallel tasks and tests per case, refusal of unapproved merges, and verification of approved merges. Binary help, ZIP integrity, and all six retained archive checksums passed. A temporary Z3 soname compatibility symlink was used for the host LLVM tool and removed after verification. Evidence: [issue9-4b0a928f-macos-package.json](../../docs/evidence/issue9-4b0a928f-macos-package.json).

This package does not establish Windows package acceptance, packaged native UI visual parity, or the 25 end-to-end workflows. Those gates remain open; verified workflow acceptance stays 0/25.

## 2026-10-20 cross-platform CI packages at validated merge `4f70114`

Platform Validation run [39341425774](https://github.com/neodamul/AgentConductor/actions/runs/39341425774) passed both platform validation and exact release-package execution. The PR head was `56b2f8043679d54edfc747d33359fd83e8f8f972`; GitHub checked out its merge with `b2b30635f564dc7ea6801fdbe8f96a09693ebfeb`, source commit `4f70114fb63e0c01ad3d9531c3c87f52a2be0320`.

- macOS `aarch64-apple-darwin`: archive SHA-256 `690260dd0406e9b60c232f2d5cdbb5d8b679174fa24e33f64177df24d5564089`; binary SHA-256 `c4e97af38bfab603a9ebb5028bb3fc7af42723bf705173cc0d55c755234a1b21`; 214 dependency records.
- Windows `x86_64-pc-windows-msvc`: archive SHA-256 `4050e36b58559f93661cde1180fda352f3f19be5b0673e51228d58ba4c95eb66`; binary SHA-256 `fc125386684c048addc4fcc32d3a396e4a8abfa8147befba9f01359e255c1c64`; 201 dependency records.

Both downloaded archives matched the external checksum file and the binary hash in their embedded manifest; the archives contain no project JavaScript/Python files. Evidence: [macOS package](../../docs/evidence/issue9-4f70114-ci-macos-package.json) and [Windows package](../../docs/evidence/issue9-4f70114-ci-windows-package.json). This proves release-binary package execution on both CI platforms, not UI visual parity or the 25 accepted end-to-end workflows. No GitHub Release has been published; acceptance remains 0/25.

## 2026-10-20 packaged native workflow acceptance 2/25

The exact v0.1.30 macOS package built from `35a5c095ef2d00871065233dc82be20d53db8abd` completed one full isolated shared-folder journey. A Rust mock coordinator planned three non-overlapping tasks (`one.txt`, `two.txt`, `three.txt`); three stdin-driven mock sessions ran concurrently, each `git diff --check` passed, the operator reviewed the candidate and approved merge, and the packaged UI reported Completed. The candidate commit was `a51e7aa43f1799562495945d1a60ef5d0bbad939`; the disposable repository ended clean. No real provider CLI, API, or external service was used. Evidence: [`issue9-35a5c09-native-workflow-01.json`](../../docs/evidence/issue9-35a5c09-native-workflow-01.json).

The exact v0.1.31 package from source `2fc7bc22264724219f36c0e9a1c3858ffdc38025` also completed packaged UI workflow 3 in an isolated HOME/workspace: register `/usr/bin/true`, edit the profile ID, confirm removal, and verify the empty-agent state. No agent task or provider CLI was run. Evidence: [`issue9-2fc7bc2-agent-profile-ui.json`](../../docs/evidence/issue9-2fc7bc2-agent-profile-ui.json). Verified packaged UI acceptance is **2/25 (8%)**; screenshot refresh, Windows package/UI, and 23 other workflows remain open.

## Release builder packages its own executable — issue #9 / 2026-10-20

`package-release` now passes `current_exe()` into the Rust package assembler, so a custom
Cargo target directory cannot cause it to silently package an older binary from the
repository's default `target/release` directory. The existing five-argument assembler
entry point remains for callers that intentionally package that default path. The
regression creates distinct stale and current binaries, then verifies the packaged macOS
binary and manifest digest correspond to the explicitly selected current executable.
The locked all-target fixture suite passed (775 passed, 14 ignored), all-target Clippy,
rustfmt, and diff checks passed, and touched Rust files remain below 300 lines. This is
local package-contract evidence only; exact-head macOS/Windows CI and issue #9's overall
acceptance remain open at 2/25 packaged UI workflows (the shipped Rust replacement is 0/25).

## 2026-10-20 exact-source package at `7289562`

The committed issue #9 head `7289562e5de394555b196da75b0ecf3904f3c53c` builds an `aarch64-apple-darwin` package with the Rust `package-release` command. `validate-package`, packaged `--help`, ZIP CRC, and SHA-256 checks pass. The package contains a 17,925,856-byte executable, bundled SQLite, and 214 dependency records. All four shared/worktree × Git/folder mock smoke cases pass with three parallel tasks/tests each; unapproved merge is refused and approved merge verified.

Archive SHA-256: `abf7a55d3d189a060f3e5434f0af0de5f25fd1f1ebbae16710a124c6a894668d`. Binary SHA-256: `ae6a24792248f973b61a80dfe41805e29581f3bb720378124bb6321df71068db`. Evidence: [package](../../docs/evidence/issue9-7289562-macos-package.json), [smoke](../../docs/evidence/issue9-7289562-macos-smoke.json).

This is exact-source macOS CLI/package evidence, not a GUI workflow. It does not advance accepted UI workflows beyond 2/25, verify Windows, or authorize a public release.

## Remove smoke workspaces after failed checks — issue #9 / 2026-10-21

`run_smoke` retained its `ac-release-smoke-*` temporary workspace when a package failed
before the normal success-path cleanup. A Rust integration regression compiled a failing
package fixture, observed the leaked directory (Red), and verified that a drop guard now
removes it on all exits while preserving the separate evidence directory. The focused
`release_smoke` suite passes 7/7; targeted Clippy, rustfmt, and diff checks pass. Source and
test files are 161 and 269 lines. This build-tool cleanup fix does not add packaged UI
workflow acceptance; the current count remains 6/25. Windows exact-head validation is
pending.

The pushed commit `b4ad2f59f99ae32b2d1b1967e09509dd5933516a` also built a 18,008,720-byte
macOS executable with SHA-256 `66e55fe712ec33ca29d9064adc0aa4e67ce8b530b97a0c6e90f73289e6b96790`
and a package archive with SHA-256 `ab63c95f71ecc0754294f156c43fbe0649f4cbe8b22e6be3cd2d7374446a126e`.
Manifest validation, packaged `--help`, and all four approval-gated smoke cases pass. The
machine-readable package and smoke results are [here](../../docs/evidence/issue9-b4ad2f5-macos-package.json)
and [here](../../docs/evidence/issue9-b4ad2f5-macos-smoke.json); this is not a native UI
workflow and does not increase the 6/25 acceptance count.

## 2026-10-23 exact-source macOS package at `ca9495b2`

The committed Rust source `ca9495b21c77886df2f75b1672f965dfe75d53fc` built a native
`aarch64-apple-darwin` executable and package offline. `package-release`, `validate-package`,
packaged `--help`, ZIP integrity, and the archive checksum passed. The binary is 18,606,512
bytes with SHA-256 `c057aab0ea15b9a486ee05dbcdd88e541d4f8743f1ef9001eb642969739fbe27`;
the archive SHA-256 is `1f9b49bfcae1be68d6d816fcb87ca30af573092c809c53f04addb37ef6347eac`.
The manifest records 214 dependency licenses. All four packaged CLI smoke cases passed for
shared/worktree × Git/folder, each with three parallel tasks and tests, refusal of an
unapproved merge, and verification of the approved merge. Evidence:
[issue9-ca9495b2-macos-package.json](../../docs/evidence/issue9-ca9495b2-macos-package.json).

This did not launch the GUI or verify Windows, and does not add a packaged UI workflow
(acceptance remains 11/25). The packaged manuals still contain historical screenshots and
must be refreshed before any release.

## Exact issue-branch head package — 2026-10-24

The current branch head `8ef672bd2d94ae38ed6caeab4f1d93d8df52d47c` produced a local offline v0.1.37 macOS arm64 package using the Rust `package-release` command. Package validation and ZIP integrity passed. Its built-in smoke passed all four isolation/source combinations (shared or worktree × Git or folder), with three parallel tasks and tests per case, expected approved merge, and unapproved-merge refusal. The 18,858,304-byte executable SHA-256 is `95746fd5052c7c4fde4472ecce7afa7db2b01ec38a94c612bc9f15b4e0a78e4e`; the archive SHA-256 is `605ee5e9b65e26e41a9a2c5b9a9a4151fe41655bc5ee1d3cd5dc12e4e1bfa519`. The archive contains 242 dependency license records, a 242-component CycloneDX SBOM, project notices, and English/Korean HTML and Markdown manuals. This does not verify package UI behavior, Windows, or a GitHub Release. Evidence: `../../docs/evidence/issue9-local-macos-package-8ef672bd-20261024.json`.

## Current v0.1.37 macOS arm64 candidate — 2026-10-24

Source commit `08a1783476756a7b79b4823a91a370fcfd943135` builds a local v0.1.37 macOS arm64 package. `validate-package`, ZIP integrity, and four built-in smoke combinations pass: shared/worktree × Git/folder, with 3 parallel tasks and 3 tests per case, approved merge verified, unapproved merge refused. Binary SHA-256: `5e3fd1dfe84aa0cb7bec6e50918849b3b56bfc676ebda034dfe5df536fe9fd42`; archive SHA-256: `7cb20690d605ecbf79425d06ec885a08e39c78df58a43ed2d4bb722ac1f4272d`. The manifest reports 242 dependencies; license texts, inventory, CycloneDX SBOM and current bilingual manuals/screenshots are included. Exact package UI inspection used one isolated HOME and verified Workspace, six roles, disabled empty-plan actions, Settings, and Workspace menu without saving or starting work. This is a local candidate, not a GitHub release or UI workflow acceptance. Preserve the package as `dist/agentconductor-aarch64-apple-darwin-08a178347675.zip` until superseded by the next current-source candidate. Evidence: `docs/evidence/issue9-local-macos-package-ui-08a1783-20261024.json`.

On 2026-10-24, the user approved a UTM reset. Force-stop/start restored VM state and IP `192.168.64.6`, but not guest execution: `utmctl exec ... cmd.exe /c ver` returns OSStatus `-2700` and UTM displays “Display output is not active.” Do not treat Windows runtime as verified.

## Exact pushed-head package after Windows-only Clippy repair — 2026-10-24

Commit `278ad6de798e73e2a71502af03117c0004d61e92` builds as a local v0.1.37 macOS arm64 package. Manifest validation and ZIP integrity pass; the embedded smoke passes shared/worktree × Git/folder with three tasks/tests per case and approval-gated merge behavior. Archive SHA-256: `6b3771dbb6343a2874251da678880a2c5df33b1a3ec79be0b0fb44246c4f1c0e`; binary SHA-256 is unchanged from the inspected 08a1783 candidate. The previous generated package and smoke run were removed. Evidence: `docs/evidence/issue9-local-macos-package-278ad6de-20261024.json`. This does not add operator workflow credit or prove Windows runtime/release.
