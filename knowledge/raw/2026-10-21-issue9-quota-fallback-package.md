# Issue #9 exact package quota-fallback checkpoint — 2026-10-21

The exact v0.1.32 macOS release executable from product source `b4ad2f59f99ae32b2d1b1967e09509dd5933516a` was run in a disposable Git repository with deterministic Rust CLI fixtures. Three parallel tasks each received a structured `usage_limit_reached` event carrying `resetsAt=4102444800`. Every task transferred from the paid profile to the configured company fallback on attempt 2 and passed its integration test.

The run reached `waiting_approval`. The packaged binary refused an unapproved merge, accepted approval for the exact candidate, merged the three expected files, and reported `completed` with a clean repository. No provider CLI or external service ran. This is packaged CLI verification, not native UI workflow acceptance; the count remains 6/25.

Evidence: `docs/evidence/issue9-b4ad2f5-quota-fallback-cli.json`. AgentMemory: `mem_mvhe77xu_5497db2b04a4`.
