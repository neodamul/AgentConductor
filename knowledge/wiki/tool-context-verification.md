# Tool-context verifier evidence

Verified: 2026-10-10 KST. Status: compiled worker/operator contracts; live native enforcement pending.

## Sources

| Local source | SHA-256 |
|---|---|
| `src/process.rs` | `b4ca248496d432d31f94f56a43d70d6db4ca99050a4fbb77e699e7fa2dce28e0` |
| `src/workflow.rs` | `b8e6f14580c106105de2b2492c07d83af4c00a18bab41e804922b48512b7e5ef` |
| `tests/async_control.rs` | `67afc3bff3f6b7061a67ec6dacd91179e212e094b874330ea0d27a99e51aabfb` |
| `tests/tool_context.rs` | `823c44e89f6e9ba00348e0e03c31b7fb47a94a6634f9cd4942edeeba9043d7e5` |

Changing source hashes makes the relevant verification claims stale. See
[DD-19](../../docs/06-design-decisions.md), [file policy](tool-approval-policy.md)
and the [full-product audit](../../docs/09-completion-audit.md).

## Facts and boundaries

The internal `tool-context` command accepts a bounded typed request and derives task,
native session, canonical workspace and plan/Git fingerprints from SQLite run state.
It checks repository membership, target branch/baseline, task baseline and policy.
It cannot approve or consume a decision. The supervising API gives it ten seconds,
owned process-group containment, raw spools and recovery-inventoried attempt paths.
The caller supplies the trusted engine executable; a registered agent must never
select or replace that executable in production integration.

Ignored sensitive file targets contribute Unix identity, size, permissions, ownership,
mtime and ctime metadata. The verifier does not read their contents or add them to Git.
Metadata is evidence of the current target, not a content hash or prevention of a
concurrent replacement. Native containment and approval consumption need live tests.

Compiled command tests pass in shared and optional worktree modes. Changing a working
file changes the binding; foreign writes and unknown tasks are refused. Ignored
synthetic `.env` changes alter the binding without appearing in reachable Git objects.

## Lesson: cancellation and ordinary exit have different contracts

- Context: asynchronous callbacks may await an owned verifier subprocess.
- Failure: detached callbacks could outlive stop; normal exit cancelled a queued final frame.
- Cause: JoinHandle drop does not abort work, and one exit flag conflated two lifecycles.
- Response: abort-on-drop evaluations, cancellation before signal escalation, separate exit/cancellation state and refusal of late permission replies.
- Evidence: Red → Green tests for pending timeout, an uncooperative verifier, owned descendants and final-frame draining.
- Confidence: high for local transport contracts; native generation/tool behavior is not yet proven.
- Revalidate when: callback protocol, supervisor termination or provider idle semantics change.

Regression: 121 passed, two ignored, 34 suites in 43.06 seconds; Clippy/format checks pass.
Native Claude metadata-only initialization also passes (0.90 seconds), with no user turn.
It provides compatibility evidence for the control wire, not authenticated task execution.
Worker binding and typed operator controls have compiled CLI evidence; all-provider native enforcement remains open. See [worker decisions](native-worker-decisions.md).
