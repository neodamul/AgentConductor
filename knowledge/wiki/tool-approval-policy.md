# Tool approval policy evidence

Verified: 2026-10-10 KST. Status: component contract, not native worker enforcement.

## Sources

| Local source | SHA-256 |
|---|---|
| `src/guard.rs` | `ff5cdedb4e1ca4a7493896cc2c393276bf34daabf439ce498fb6e6ab81cca8d9` |
| `src/claude_control.rs` | `c98653b92b45fa53f9f686347f05f8bf91f7e138784e878bbbad61c99741b1f3` |
| `tests/guard_policy.rs` | `f63762c2e9279a1704ec301dd4ee9905aa95643ea7f58b5f76233d988934c2d7` |
| `tests/guarded_control.rs` | `3c58ef135bd99b718107387dc8a886f24fbe879b71cc537b8320affff4d9b210` |

See [DD-18](../../docs/06-design-decisions.md),
[implementation checklist](../../docs/07-implementation-plan.md) and
[native protocol observations](cli-protocols.md) for scope and related evidence.
Changing any source hash makes this page's verification claims stale until rechecked.

## Facts

The classifier examines bounded JSON and filesystem metadata; it never reads file
contents, walks a whole directory, invokes commands or makes network requests.
Ordinary regular-file reads may pass. Edits and writes additionally require owned
task paths. Commands, searches and recognized sensitive paths require the human
evaluator. Unsupported tools/fields, traversal, settings, metadata, symlinks,
hardlinks, special files and changed workspace directory identities are refused.
The original bounded input reaches human review without rewriting.

Mock bidirectional subprocess verification proves: an out-of-scope write leaves
the original file intact; an owned write proceeds; a pending command is terminated
without a marker; its exact approval permits one marker; replay leaves retained
marker content unchanged. All attempts preserve raw stdout and terminal, quiescent
process ownership. Full regression: 116 tests passed, two ignored, 32 suites.

## Lesson: validate callback identity before evaluating approval

- Context: native pre-tool and permission phases share a one-shot decision.
- Failure: a duplicate tool-use ID was rejected after the evaluator already ran.
- Cause: identity validation followed decision consumption.
- Response: validate identity before evaluation; reject late requests after input closure.
- Evidence: both added controller tests first failed, then passed after the fix.
- Confidence: high for the tested component; native runtime integration is unverified.
- Revalidate when: callback protocol, permission cache or lifecycle semantics change.

## Unknowns and remaining work

Filename recognition cannot discover credentials embedded in arbitrary text.
Inspection does not prevent concurrent filesystem replacement after evaluation.
Mock bindings use fixed test contexts, not authoritative production fingerprints.
Native worker integration, durable operator commands and actual CLI containment
remain open in [the full-product audit](../../docs/09-completion-audit.md).
Merge approval remains a separate exact-candidate decision.
