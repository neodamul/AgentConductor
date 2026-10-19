# Rust native release packaging

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
