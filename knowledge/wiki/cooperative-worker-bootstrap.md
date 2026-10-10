# Cooperative worker bootstrap

Trace: R-27 → BG-11/BG-12 → SF-24 → UC-21 → QAS-22 → DD-29.

## Contract and decision

Checkpoint cooperation is explicit and disabled by default. A custom CLI declares
`checkpoint_v1` and accepts one `{session_id}` argv binding. Rust persists its session
identity before launch and holds a session lease; returned foreign identities fail.
The run namespace survives restart and is allocated transactionally before any directive
is needed. Bootstrap only routes existing checkpoint commands and creates no directive,
task transition, tool permission or merge approval.

The worker receives the local run, address file and current executable in reserved
environment variables. The prompt instructs it to poll before work and between safe
units, acknowledge exact addressed directives and publish monotonic reports. This is
voluntary cooperation; a registration flag alone does not prove compliance. Existing
profiles omit the disabled field during serialization to preserve approval-bound digests.
Inherited checkpoint routing is cleared before applying explicit child environment.

## TDD evidence

Red: both initial tests failed on the unknown cooperative profile field. Green: add
explicit capability validation and owned-session bootstrap. A fixture-path mismatch
was corrected using actual task artifact paths, retaining failed-run evidence locally.

Three new tests cover invalid capability/session/environment registrations; three real
parallel non-steerable CLI processes polling before work and at an intermediate safe
checkpoint in shared and worktree modes; addressed rework, accepted/completed
acknowledgments and increasing reports; foreign returned session rejection; and absence
of merge authorization after successful checkpoint completion.

Final full regression: 191 passed, 5 ignored, 0 failed in 48 suites; 66.29 seconds summed
suite time, not wall time. Clippy with warnings denied, formatting and whitespace passed.
Private evidence: `.agentconductor/verification/cooperative-worker/`. The fixture uses
system Python for a controlled CLI, not a company LLM. No GUI, daemon or new dependency
was introduced. Graphify: 1,304 nodes / 3,070 edges without LLM. CocoIndex located
worker_address. Ponytail has no exposed callable capability in this session; no execution claimed.

## Remaining acceptance

Automatic agentmemory discovery/import, individual worker authentication, native capability
registration interaction, actual company CLI compliance, checkpoint resume/recovery and
handoff-driven scheduler transitions with proven old-process quiescence remain open.
Provider-generated initial session contracts require separate verified binding. Native
Windows and verified macOS/Windows Releases remain unfinished. Local routing context is
not worker authentication. This slice does not complete the product.

## Source fingerprints

Historical fingerprints at bootstrap checkpoint `c4f6291`. Subsequent exit reconciliation
and resume evidence are recorded in [cooperative exit reconciliation](cooperative-exit.md).

| Source | SHA-256 |
| --- | --- |
| `src/plan.rs` | `74fa9e41ed92e47f22e91c3a9f14d8822761247a2edb107ed86339d04e254d24` |
| `src/cooperative.rs` | `676335ce12692baf6a53877b40fa07f28f8f4bb444d10277af6eb60440bc3eca` |
| `src/workflow.rs` | `d63a2d6d59fef870a8b94c44a9fca57e615ccfb142dbad4daf84f86bbfe123a7` |
| `src/process.rs` | `6c7105cc8ee7256c69a7105015ca26b477fb28e7b7ae0a32c20945a81d78fd81` |
| `tests/cooperative_worker.rs` | `199248a509400aeee160d682b523ea7190c155c2aa71751231cd45a7a753d90d` |
| `tests/support/cooperative_worker.py` | `2a3d2a6fcba38c2b664882bf0661064991896f236b337fde8cd08581c89644b4` |
| `docs/11-cooperative-checkpoints.md` | `c6a5af6bfbd524992d41dc01ef18bf37d138c4bde3f876b53e571edd9e18ce6b` |
