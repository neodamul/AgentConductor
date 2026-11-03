# Exact-source macOS package audit

Date: 2026-10-18
Source commit: `2996dc6af777dfe2b511d207e02e83ffdabf81c6`
Target: `aarch64-apple-darwin`

## Package evidence

- Archive: `dist/agentconductor-aarch64-apple-darwin-2996dc6af777.zip`
- Archive SHA-256: `c97f3e174336abce9de48e4607800b16c4205c318ae1972998312942915be91d`
- Binary size: 17,411,584 bytes
- Binary SHA-256: `c4618e60f777de0f96e21c69864fbe34267686a70f771eebf435eb921cc36d7f`
- Manifest records the exact source commit, target, 214 dependencies, bundled SQLite, Git, and selected authenticated CLIs.
- The archive contains the app bundle, README, English and Korean HTML manuals, `DEPENDENCY-LICENSES.md`, all collected license texts, `SBOM.cdx.json`, `manifest.json`, and `SHA256SUMS`.
- `validate-package` passed. Archive digest matched `ARCHIVE-SHA256SUMS`.
- Packaged smoke passed four combinations: Git/shared, folder/shared, Git/worktree, and folder/worktree. Each ran three tasks and tests; every case verified approved merge and refused unapproved merge.

## Host toolchain note

Homebrew LLVM 23.1.3 expects `libz3.5.1.dylib`, while the host has Z3 5.3.1.
Rust commands therefore needed a temporary `DYLD_LIBRARY_PATH` alias in
`/private/tmp`; no Homebrew files were changed and the alias is not part of the
package. The package binary itself passed validation and the complete packaged
smoke. Windows packaging and UI acceptance are still open. This local package was
not published as a GitHub release.
