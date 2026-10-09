# Native Codex subscription and session evidence

Verified: 2026-10-10 KST with installed codex-cli 0.162.0. Scope: native text turns; no model tool call requested.

## Sources

| Local source | SHA-256 |
|---|---|
| `src/protocol.rs` | `1cbb1259b84fe3c1d9c8f994d8a596e48594556211277725051ec15220257d53` |
| `tests/protocol.rs` | `69d55551d9bf98815f584112c50436df7a537b2cd6a50f0ef6124eeb283bb461` |
| `tests/native_codex.rs` | `ec57d14cc163773c51c973abb374d145fc784b2e0783e9b8d5ef8323a5fb5676` |

## Retained raw evidence

| Local spool | SHA-256 |
|---|---|
| `.agentconductor/verification/codex-native-session-97711/initial/stdout.jsonl` | `f69fd476daf53d1e9ee8be49bdf38b8b5c9c1658499185f7c2fd970c5049851a` |
| `.agentconductor/verification/codex-native-session-97711/resume/stdout.jsonl` | `692195441e193e65971bc83db6c12bd12b2fe94e6735c6572a256e3adbbf1568` |
| `.agentconductor/verification/codex-native-concurrent-98059/one-initial/stdout.jsonl` | `d3048de71c0eb49d2e239bfcfd6434c53d9e23ed390098c20711d27862c64b38` |
| `.agentconductor/verification/codex-native-concurrent-98059/one-resume/stdout.jsonl` | `c4f0fe024ab0d94ae2ad3eeaacef826fc9d19ca7f8a15aa7d1b11d343566004b` |
| `.agentconductor/verification/codex-native-concurrent-98059/two-initial/stdout.jsonl` | `37bcf1898a2fc4a38eab177d08fe736105e3f1f6cde46c76ec84a52e0d518d57` |
| `.agentconductor/verification/codex-native-concurrent-98059/two-resume/stdout.jsonl` | `a6aa2d24837805653214a7910ce8ee0c0692746eefd117365e1dfa5f95dd0e60` |

Raw evidence is local and excluded from Git. Source changes require revalidation. See [session recovery](session-recovery.md), [CLI protocols](cli-protocols.md) and [full-product audit](../../docs/09-completion-audit.md).

## Verified contracts

Codex login status reported ChatGPT authentication. The test removes OPENAI_API_KEY and CODEX_API_KEY from the child environment, uses the existing auth store and sends prompts over stdin through codex exec with JSONL and read-only sandbox selection. Initial and exact-ID resumed turns return successful terminal events, expected text, retained ownership evidence and quiescent process groups. Resume recalls the previous user-turn marker; it does not select the latest session implicitly.

Two initial turns run concurrently in the same directory and return distinct native IDs. Their exact-ID resumes run concurrently and each recalls its own marker. Context/session test: 7.74 seconds; two-session concurrency/context test: 7.13 seconds. These are actual CLI generation turns, distinct from metadata-only initialization or mocks. No tool execution occurred in the observed streams.

The CLI emitted nonterminal item.completed error items about malformed host agent role definitions, then completed each turn successfully. The normalizer now retains these as warning events rather than ordinary progress. Terminal turn.failed/error remain failures. Host role files were not modified. Provider-reported usage is retained in raw terminal events; resumed values must not be blindly summed as independent per-turn costs. Subscription billing is not inferred from token counts.

## Remaining proof

This does not establish filesystem/network/ref containment, risky-tool human mediation, native code implementation/test/merge, model-change behavior, interrupted reassignment or other providers. Claude remains unauthenticated and OpenCode has no configured model catalog at this checkpoint. Their live contracts remain open; no API-key substitute is introduced.

Regression: 135 passed, four ignored, 37 suites in 52.91 seconds. New ignored generation tests ran explicitly as described above. Clippy/format/whitespace pass; Graphify 377 nodes/1,199 edges; CocoIndex located normalization. No runtime dependency added.
