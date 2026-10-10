# Exact packaged workflow acceptance

Trace: R-24 → BG-10 → SF-20 → UC-17 → QAS-14/QAS-18 → DDR-03/DDR-08 → DD-27.
The runner validates the package manifest, exact binary name, source commit and
SHA256SUMS before executing. Two integrity tests first failed on the missing
runner, then passed with the package contracts (four Python tests total).

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
- [scripts/build_release_artifact.py](../../scripts/build_release_artifact.py): SHA-256 `65449b33e5e71d5b57ae609f2ac9b168c6b6325ed5988f00b47ffd6e28d1527c`.
- [tests/test_release_smoke.py](../../tests/test_release_smoke.py): SHA-256 `1f0d44064fec9ab571a5395a2f6cdf8690cdb36d0bb99c85b24527f17b9b3eae`.
- [tests/support/release_worker.rs](../../tests/support/release_worker.rs): SHA-256 `408d4da8d03206d8b2043e0e7df437da63ced1736640d07a9b540a0519f98441`.
