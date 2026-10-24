# Exact-source macOS package verification

Date: 2026-10-20
Issue: #9
Source commit: `7419557168ddc3ffe76c446cf292152cc9fc1932`
Target: `aarch64-apple-darwin`

The Rust `package-release` command built a 21,580,718-byte ZIP for the exact issue head. `validate-package`, the packaged `--help`, archive checksum, and ZIP CRC checks passed. The package manifest records one 17,925,856-byte native executable, 214 dependencies, bundled SQLite, and the exact source commit. Binary SHA-256: `6988a2dd852b23665be6f43f4d2d7e20092dbef5e20db49574cd6d1982aa6836`. Archive SHA-256: `610f3987658acd5e8c319d827539ab0fe2745d45ee28d2dced754c50365becfb`.

The packaged smoke runner completed all four shared/worktree × Git/folder cases. Each case ran three concurrent mock tasks and three tests, refused an unapproved merge, and verified the approved merge. No real provider CLI or API was invoked. Machine-readable [package evidence](../../docs/evidence/issue9-7419557-macos-package.json) and [smoke results](../../docs/evidence/issue9-7419557-macos-smoke.json) are retained in the repository.

This is exact-package runtime evidence only. The GUI was not opened, Windows was not verified, and packaged native UI acceptance remains **2/25**. No release was published.
