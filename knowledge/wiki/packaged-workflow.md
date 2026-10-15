# Exact packaged workflow acceptance

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
- [scripts/build_release_artifact.py](../../scripts/build_release_artifact.py): SHA-256 `4bb83493073a871629f6c59333d04e9cc6e6800e1ebe50828e4c557344aae5c3`; it still orchestrates package assembly and invokes the Rust smoke/archive commands.
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
