# Reviewed upstream license texts

Trace: R-24 → BG-10 → SF-20 → UC-17 → QAS-18 → DDR-03/DDR-08 → DD-27.

Three Windows dependencies omit their license text from the published crate:
`webview2-com` 0.39.1, `webview2-com-sys` 0.39.1, and `webview2-com-macros` 0.8.1.
The retained MIT texts came from the exact upstream commits recorded in each
crate's `.cargo_vcs_info.json`:

- [webview2-rs license at edc2caf](https://github.com/wravery/webview2-rs/blob/edc2caf886175ccaebe86078c9cfe1ae2a187328/LICENSE)
- [webview2-rs license at dffa41a](https://github.com/wravery/webview2-rs/blob/dffa41a8a46d3f5565eefbff2de57d38d399f158/LICENSE)

`licenses/upstream/sources.json` pins each package name, version, SPDX declaration,
VCS commit, source path, and SHA-256. During offline package assembly,
`src/release_package/licenses.rs` uses a retained copy only if the crate omits its
own text and every provenance field and content digest matches. Missing, altered,
or mismatched text fails before package output. The assembler also includes the
bundled SQLite source copyright disclaimer and rejects licenses outside its reviewed
allowlist.

Rust package tests cover valid provenance and rejection for a wrong revision,
version, and modified bytes. The Windows-style `autocrlf` checkout test confirms the
retained official license bytes stay unchanged. This contract test does not replace
exact-source Windows package CI; that platform run is pending for the current Rust
assembler.

Native UI prototype audit (2026-10-16): the isolated eframe target dependency
graphs add ISC (`libloading` 0.8.9) and, on Windows, BSL-1.0 (`clipboard-win`
5.4.1 and `error-code` 3.4.0). `clipboard-win` declares BSL-1.0 but its crate
archive omits the license file. The exact ISC and BSL-1.0 texts were retrieved
from the packages' `.cargo_vcs_info.json` commits and byte-checked against the
published source. The omitted clipboard notice and the libloading ISC text are
pinned under `licenses/upstream/` with SHA-256 provenance; `error-code` ships its
own BSL-1.0 text.
`tests/native_ui_license_provenance.rs` verifies the clipboard record and digest;
`tests/release_package.rs` verifies ISC and BSL-1.0 notices are included when
assembling the candidate Windows dependency set. The release allowlist and
third-party notice now include both permissive licenses. This was evaluation-only
license support at the 2026-10-16 prototype checkpoint; the current Rust eframe UI
is part of the production application. Native Windows build/UI validation remains
pending. ISC and Boost Software License 1.0 permit
internal/commercial use and do not require publishing the application's source;
redistribution must preserve the license/copyright notices. Here BSL-1.0 means
Boost Software License, not the Business Source License.

Sources:
- [clipboard-win BSL-1.0 at the crate source commit](https://github.com/DoumanAsh/clipboard-win/blob/3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342/LICENSE)
- [error-code BSL-1.0 at the crate source commit](https://github.com/DoumanAsh/error-code/blob/e4615e514db3ff64f5f1b328b4865b20a3dcdbc3/LICENSE)
- [libloading ISC at the crate source commit](https://github.com/nagisa/rust_libloading/blob/f4ec9e702de2d0778bccff8525dc44e4cacac2d1/LICENSE)
- [ISC text at Open Source Initiative](https://opensource.org/license/isc)
- [Boost Software License 1.0 text](https://www.boost.org/LICENSE_1_0.txt)

## Exact macOS package (2026-10-17)

The default native application now uses eframe. The exact `aarch64-apple-darwin`
dependency graph contains 182 packages; 34 crate archives omit root license text.
Their selected MIT license choice is retained from SPDX and pinned in
`licenses/upstream/sources.json` by crate name, version, Cargo VCS revision, and
SHA-256. For dual or triple SPDX expressions the selected MIT option appears in the
crate's declared expression. The package also contains the dependency inventory,
third-party notices, and CycloneDX SBOM.

The source commit `c7e2dc9e640c78023cb52c9f598b7652c41f2d7c` produced
`agentconductor-aarch64-apple-darwin-c7e2dc9e640c.zip` with SHA-256
`dbd608d6ae5d10b705f82bcbb5148e24b78c2a4178e43741d86ec278db808e01`. The packaged
workflow smoke passed shared and worktree modes with both Git and ordinary-folder
sources. The ZIP integrity check and binary/package checksums passed.

The Windows target adds `gl_generator` 0.14.0 and `khronos_api` 3.1.0, both declared
Apache-2.0 and missing license text in their crate archives. Their exact Cargo VCS
revisions and the retained, SHA-256-pinned Apache license text are recorded in
`licenses/upstream/sources.json`; `tests/release_license_gl.rs` checks both records.
The focused license and package test suites pass at `c7e2dc9`. This validates the
license assembly contract, not an actual Windows archive: Actions run 38943406077
was canceled after about four minutes during the initial build on both runners, and
no Windows package job ran.

This evidence covers the exact macOS package only. The local machine has no Windows
Rust target or linker, so exact Windows packaging and runtime checks remain pending.
The first archive exposed SPDX MIT template placeholders where native crate archives
omitted license text. A TDD regression now rejects those unresolved fields. Exact
source-version notices are retained for the 28 affected native dependency records,
including egui, objc2, and profiling. `dispatch` 0.2.0 has no upstream LICENSE file;
its MIT notice is reconstructed from the crate's MIT metadata, Cargo author, and
initial source commit date. That attribution is an inference and still requires
company legal review. This check covers the macOS native graph only; the full
Windows package remains pending.

Source: [SPDX MIT text](https://spdx.org/licenses/MIT.txt).

Sources:
- [src/release_package/licenses.rs](../../src/release_package/licenses.rs)
- [tests/release_package.rs](../../tests/release_package.rs)
- [licenses/upstream/sources.json](../../licenses/upstream/sources.json)
- [.gitattributes](../../.gitattributes)

## Current native package provenance — 2026-10-17

Exact upstream license files and source hashes are now pinned for `egui-wgpu 0.37.0`, `objc2-metal 0.2.2`, `objc2-metal 0.3.2`, `objc2-quartz-core 0.3.2`, and `spirv 0.4.0+sdk-1.4.341.0`. The provenance records in `licenses/upstream/sources.json` identify package version, Cargo VCS revision, license path, and SHA-256; the focused test rejects altered or mismatched provenance. Cargo expressions such as `Apache-2.0/MIT` are normalized to SPDX `Apache-2.0 OR MIT` for the package SBOM.

The package test now rejects the SPDX template fields `<year>` and `<copyright holders>` in native dependency notices. Before the fix it failed on `block2 0.5.1`; after pinning exact versioned text and SHA-256 provenance for 28 affected records, 12 tests across the release-license, native provenance, package, and smoke suites pass. The upstream repositories and exact source revisions are recorded in `licenses/upstream/sources.json`.

`dispatch 0.2.0` is the one exception to copied upstream license files: the pinned repository has no LICENSE file. Its bundled MIT notice uses the author in Cargo metadata and the year of the initial source commit, with both evidence links recorded alongside the notice. This attribution is inferred, not upstream-issued, and requires company legal review before distribution. The exact-source CI run `38996644132` passed macOS validation; Windows is still checking the native UI launch path, and neither platform job includes this unpushed notice update.

The 214-dependency current-host audit and local package/license tests pass at source `98d5357`; the exact macOS ZIP SHA-256 is `9b5f9a76a177887865e6ffca585c64f0e341a87889811ff22fa7ed8169846190`. Cross-platform run [38995553011](https://github.com/neodamul/AgentConductor/actions/runs/38995553011) passed macOS build/layout but failed a test-fixture assertion before packaging; Windows build/layout passed; the obsolete run was then canceled after the local fixture fix. The fixture now copies and compares exact pinned upstream text. Run [38996644132](https://github.com/neodamul/AgentConductor/actions/runs/38996644132) has now passed macOS and Windows validation, with both exact-package jobs in progress; it does not contain the current unpushed change. The 28 native MIT records no longer use SPDX placeholders. Final company legal approval remains a gate for the inferred `dispatch 0.2.0` attribution described above.
