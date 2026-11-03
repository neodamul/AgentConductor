# Signed-out Claude and Codex quota queries

## User-visible contract

When an interactive Claude usage query encounters a first-run screen indicating no login, a login-method chooser, or an expired OAuth token, it stops immediately and tells the user to sign in with `claude` and `/login`. It must not wait for the normal quota timeout. Folder-trust prompts remain handled by the existing trusted-folder path.

When Codex's app-server rate-limit request reports authentication required, HTTP 401, unauthorized access, or a token-related error, the quota query returns a sign-in message and suggests `codex login`. A ChatGPT authentication-required error retains the API-key-specific message because usage windows require ChatGPT sign-in. Other errors retain the diagnostic message.

## Rust implementation and evidence

The Rust terminal collector applies Claude screen classification before issuing `/usage` and again while collecting output. The Codex app-server collector classifies the server error before returning the quota result. Neither path changes CLI credentials, calls an OpenAI-compatible API, nor starts a generation request.

Regression coverage is in `tests/claude_usage.rs`: signed-out, login-selection and expired-token Claude screens; Codex authentication, 401 and expired-token errors; and preserved API-key guidance. The tests were first run against the unmodified implementation and failed to compile because the Codex classifier was missing. After the implementation, all focused tests pass. Exact Windows/macOS package evidence belongs to the current PR/commit and must not be inferred from source tests alone.

## Provenance

- Main bug: GitHub #317, fixed by PR #318, squash-merged at `db0da0b`.
- Rust port: issue #9 root branch, tracked in `docs/07-implementation-plan.md`.
