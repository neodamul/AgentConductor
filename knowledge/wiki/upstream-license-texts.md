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

Sources:
- [src/release_package/licenses.rs](../../src/release_package/licenses.rs)
- [tests/release_package.rs](../../tests/release_package.rs)
- [licenses/upstream/sources.json](../../licenses/upstream/sources.json)
- [.gitattributes](../../.gitattributes)
