# Rust coordinator chat revision fast path

## Decision

Read-only coordinator chat validates a digest of saved workspace context and the reviewed plan artifact without resolving Git `HEAD`. Mutating requests keep the full revision that includes `HEAD`.

## Request path

1. Workspace snapshots expose both `revision` and `chat_revision`.
2. The native chat panel sends `chat_revision`; a regression test ensures it does not accidentally send the full Git-bound revision.
3. Chat dispatch checks `chat_revision` before general startup-job dispatch. A stale saved goal or reviewed plan is rejected and requires refresh.
4. The accepted chat revision is passed to the supervised worker launch, avoiding a second slow Git lookup.
5. Job-running and live-job guards remain in force before launch.

## Trace

R-181 → BG-136 → SF-169 → UC-172 → FR-169 → QAS-179 → DD-192. GitHub main issue #234 is ported under Rust migration issue #9.

## Verification boundary

The startup regression verifies Git `HEAD` changes leave `chat_revision` unchanged while changing the full mutation revision, and that saved workspace changes reject a stale chat request. `startup_chat` passed 4 tests and `native_workspace_chat` passed 6. The UI request test first failed because the panel still sent the full revision, then passed after it was changed to use `chat_revision`. Exact macOS and Windows packaged chat workflows remain unverified; this change alone does not establish the cause of historical chat timeouts.

## Exact macOS package smoke at `f739b485`

Built the pushed source commit as an arm64 macOS release package. The archive passed ZIP integrity and the Rust package validator; its manifest names the exact source commit, and the embedded binary SHA-256 matched. The packaged binary's `--help` passed.

The packaged CLI smoke passed four isolated cases: shared and worktree execution, each against both a Git repository and a plain folder. Each case completed three parallel mock tasks and tests, refused merge before approval, and verified the approved merge. This validates the package CLI workflow only; no native GUI or Windows packaged chat acceptance is claimed.

Evidence: `docs/evidence/issue9-macos-package-f739b485-20261029.json`.
