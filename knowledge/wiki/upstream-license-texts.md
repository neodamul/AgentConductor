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
third-party notice now include both permissive licenses. This is license support
for evaluation only; eframe is not a production dependency, and native Windows
build/UI validation remains pending.

Sources:
- [clipboard-win BSL-1.0 at the crate source commit](https://github.com/DoumanAsh/clipboard-win/blob/3b27cf2bfd1adcfa6e0264eb51c1025ddaf0f342/LICENSE)
- [error-code BSL-1.0 at the crate source commit](https://github.com/DoumanAsh/error-code/blob/e4615e514db3ff64f5f1b328b4865b20a3dcdbc3/LICENSE)
- [libloading ISC at the crate source commit](https://github.com/nagisa/rust_libloading/blob/f4ec9e702de2d0778bccff8525dc44e4cacac2d1/LICENSE)

Sources:
- [src/release_package/licenses.rs](../../src/release_package/licenses.rs)
- [tests/release_package.rs](../../tests/release_package.rs)
- [licenses/upstream/sources.json](../../licenses/upstream/sources.json)
- [.gitattributes](../../.gitattributes)
