# Native release package contract

R-24/DD-27 requires tested macOS and Windows binaries with license notices and
checksums. The previous host-only packager omitted Windows .exe selection and
retained obsolete mockup metadata. TDD Red: missing isolated package assembly API.
Green: two Python contract tests cover both binary names, byte-exact checksums,
SBOM/notices, referenced public assets, overwrite refusal and unknown/missing
license rejection before creating output. The assembler does not modify the wiki.
CI runs these contracts on native macOS and Windows runners.

Local target-filtered locked metadata review: 42 macOS dependencies and 108 Windows
dependencies recorded in `.agentconductor/verification/native-packaging/` with declared
SPDX expressions and license text fingerprints. Additional Windows declarations
include Apache-2.0, Apache-2.0 AND MIT, Unicode-3.0 and permissive alternatives.
The package retains all available license/NOTICE files and the SQLite source
copyright disclaimer. Provider apps/artwork, agentmemory, RTK, Ponytail, Graphify,
CocoIndex and system WebView runtimes are not bundled. Their redistribution would
require its own review. The root project provides no public license grant.

This contract verification is not artifact execution evidence. Actual native
release builds, packaged-binary smoke tests, archive checksums and GitHub Release
publication remain required. Assembly refuses existing output and uses an exact
source commit; the CLI refuses dirty tracked source. A clean source check does
not itself prove that a preexisting binary was rebuilt, so the release workflow
must build and test the binary before assembly.

Sources:
- [scripts/package_release.py](../../scripts/package_release.py): SHA-256 `96cdc69baadb088a3ba279eaec2481488165a27e280a43b12663b3844e0b8e6e`.
- [scripts/release_package.py](../../scripts/release_package.py): SHA-256 `a47050b88d998fbbee79112ef88db6d2a47717ce00e707f53b0d2fb6d5eb9502`.
- [tests/test_release_package.py](../../tests/test_release_package.py): SHA-256 `981e794d320b38fbe9161d537453b1d10aa85e7193034f04002f750b837d4080`.

## Actual macOS package checkpoint

Commit da746b9 built an aarch64-apple-darwin release and packaged the same binary.
Binary size: 5,115,648 bytes. SHA-256:
`cb3549215f61e822c2a21e9d6039e74d0a378ff0a11069bd59d39409614cb12c`.
Exact packaged --help exited 0 and its bytes matched the manifest/checksum file.
Evidence: `.agentconductor/verification/native-packaging/macos-package.json` and
`packaged-help.log`. This establishes package assembly and startup only; packaged
workflow execution and a native Windows package remain pending before publication.

## Native desktop prerequisites

README previously claimed Windows execution was unavailable after the native port
passed its validation CI. The public platform notes now describe system WebKit on
macOS and installed Microsoft Edge WebView2 Runtime on Windows, alongside Git and
authenticated selected CLIs. No local UI HTTP server or Node runtime is required.
Package manifests also expose the target's desktop prerequisite. Both target
contract cases failed on the omission (Red), then all seven Python package/smoke
tests passed after the correction (Green). This metadata correction does not prove
the pending Windows packaged execution or full-product acceptance.

Updated source fingerprints after prerequisite and notice-provenance changes:
- [scripts/release_package.py](../../scripts/release_package.py): SHA-256 `fa2fe805b65d5d57fdd6ad6e33092dce49a5ced3c86235ffb36a5d53dab17b54`.
- [tests/test_release_package.py](../../tests/test_release_package.py): SHA-256 `d364a35e87c81decca1df205d183c69e355fe6dfa92661c6291d0c762ef83402`.
- [README.md](../../README.md): SHA-256 `5c45cdf9f0b7c23096712391cbe0b829d913b5bf384d399e024ba06d52973d05`.
