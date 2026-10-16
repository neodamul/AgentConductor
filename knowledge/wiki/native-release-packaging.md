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
