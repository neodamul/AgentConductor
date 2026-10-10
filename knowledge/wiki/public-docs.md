# Public documentation: README vs user manual (DD-42)

Sources: `README.md`, `docs/user-manual.md`, `tests/public_docs.rs`.

- The README introduces a **desktop product** to a first-time user: what it does, how
  orchestration works (deterministic Rust engine vs optional LLM coordinator that only
  proposes), role assignment, usage-limit redistribution (quota group, fallback, reset
  schedule), prerequisites (agentmemory optional) and a 7-step desktop walkthrough with
  real screenshots. No shell blocks, under 120 lines.
- Everything else lives in `docs/user-manual.md`: desktop guide, every `--help` command,
  configuration, safety rules and troubleshooting.
- Packaging copies README-linked `docs/` files, so the manual ships; the manual may link
  only assets the README links (contract test).
- Lesson: a contributor-facing CLI reference is not a product introduction; validate the
  README against the questions a first-time user actually asks.
