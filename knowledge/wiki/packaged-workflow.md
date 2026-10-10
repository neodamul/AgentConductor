# Exact packaged workflow acceptance

Trace: R-24 → BG-10 → SF-20 → UC-17 → QAS-14/QAS-18 → DDR-03/DDR-08 → DD-27.
The runner validates the package manifest, exact binary name, source commit and
SHA256SUMS before executing. Two integrity tests first failed on the missing
runner, then passed with the package contracts (five Python tests including the archive regression).

The exact macOS da746b9 package completed shared and worktree fixtures. In each:
three CLI workers overlapped, completed their owned file, emitted JSONL session
and result records; three integration tests passed; diff included all owned files;
target HEAD stayed at base before approval; unapproved merge was denied for missing
approval; explicit exact candidate approval followed by merge produced the candidate
HEAD, correct scoped files and a clean Git state. No provider generation or network
was used. This is compiled packaged orchestrator evidence with deterministic workers,
not authenticated Claude/Codex/OpenCode or company LLM acceptance.

Raw evidence: `.agentconductor/verification/packaged-workflow/smoke.json` and command
stdout/stderr; scoped temporary workspaces are retained for inspection. All child
commands are terminal, and no GUI or daemon was launched.

CI now builds native release binaries only after both platform validation jobs pass,
packages and executes them with this same runner, then creates archives and archive
checksums. Archives and smoke evidence are retained as CI artifacts; this workflow
never creates a GitHub Release. Native Windows packaged evidence remains pending.
The upload action is actions/upload-artifact v7.0.2, pinned to
`cf430e030ddbb5b0abf93d22962f4752f3646cd9`; its repository LICENSE was reviewed as MIT
and it is a CI dependency, not bundled runtime code.

Sources:
- [scripts/smoke_release.py](../../scripts/smoke_release.py): SHA-256 `f3eab794ad25782d2786df08c7f62861f4b7d8220854150a37fc3039f459f1e4`.
- [scripts/build_release_artifact.py](../../scripts/build_release_artifact.py): SHA-256 `c61511b5acc4b36d6732f255b05b5e3cd80dd91a7055507df0fb2ea66e8b0b81`.
- [tests/test_release_smoke.py](../../tests/test_release_smoke.py): SHA-256 `1f0d44064fec9ab571a5395a2f6cdf8690cdb36d0bb99c85b24527f17b9b3eae`.
- [tests/support/release_worker.rs](../../tests/support/release_worker.rs): SHA-256 `408d4da8d03206d8b2043e0e7df437da63ced1736640d07a9b540a0519f98441`.

## Registry timestamp regression and actual archive

The first actual archive attempt failed because cached license files had timestamps
before 1980. A new Red/Green contract covers ZIP date clamping while preserving
file bytes and executable permission bits, and refuses overwriting an archive.
The fixed writer uses the standard library with strict_timestamps=False, retains a
pending archive on failure, and refuses symlinks. A failed partial local ZIP was
preserved with a .failed suffix before verification.

Actual d765cd2 macOS package archive: 2,978,551 bytes, SHA-256
`5a104600fa0e446adb82d351bed0d65508973567e749cec5fe476af9c6c77ee7`.
All ZIP entry CRCs passed and the archived binary matched its package manifest.
Evidence: `.agentconductor/verification/packaged-workflow/archive.json`.
Graphify structural extraction: 1,742 nodes / 3,475 edges (before archive follow-up).
CocoIndex located the binary-validation definition and call without an index/LLM.
Ponytail has no callable capability here; no execution is claimed.
- [scripts/release_archive.py](../../scripts/release_archive.py): SHA-256 `ff1e68e41ff1934298dc0024bec2e124ef5d383b040070418e47bf143e9353ef`.
