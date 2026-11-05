# Native session identifiers

`native_guard::session_id` keeps its public path while its implementation lives in
`src/native_guard/session.rs`. Identifiers use operating-system random bytes and
UUID version 4 and variant bits. The same private random source also protects
the native guard orchestrator. The split must preserve both consumers and the
Windows and Unix error handling; a deterministic or process-local generator
would not be an equivalent replacement.

The tests in `tests/native_guard_session_id.rs` check the identifier shape and
uniqueness, as well as the native guard facade's 300-line area limit. Windows
execution remains necessary to validate its system RNG path.
