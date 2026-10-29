# Rust coordinator chat revision fast path

## Decision

Read-only coordinator chat validates a digest of saved workspace context and the reviewed plan artifact without resolving Git `HEAD`. Mutating requests keep the full revision that includes `HEAD`.

## Request path

1. Workspace snapshots expose both `revision` and `chat_revision`.
2. Chat dispatch checks `chat_revision` before general startup-job dispatch. A stale saved goal or reviewed plan is rejected and requires refresh.
3. The accepted chat revision is passed to the supervised worker launch, avoiding a second slow Git lookup.
4. Job-running and live-job guards remain in force before launch.

## Trace

R-181 → BG-136 → SF-169 → UC-172 → FR-169 → QAS-179 → DD-192. GitHub main issue #234 is ported under Rust migration issue #9.

## Verification boundary

The startup regression verifies Git `HEAD` changes leave `chat_revision` unchanged while changing the full mutation revision, and that saved workspace changes reject a stale chat request. `startup_chat` passed 4 tests. Exact macOS and Windows packaged chat workflows remain unverified; this change alone does not establish the cause of historical chat timeouts.
