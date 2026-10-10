# Interrupted cooperative recovery

Trace: R-27 → BG-11/BG-12 → SF-24 → UC-21 → QAS-22 → DD-29.

## Verified boundary

The compiled coordinator is killed after a real non-steerable worker records completed
pause/handoff acknowledgment but remains alive at its checkpoint. Kernel ownership is
still present; recovery and reassignment refuse. After releasing the worker and proving
quiescence, recovery preserves the owned session and partial file, leaves the run stopped,
creates no merge candidate and requires explicit resume. Resume starts attempt 2 with the
same custom session and invalidates the original checkpoint address.

The fixture covers both completed pause and completed handoff. It does not prove pending
or accepted instruction continuation, automatic target handoff, company LLM behavior or
Windows process ownership. Those scopes remain open. The existing ownership/recovery code
passed this new acceptance test without a runtime change. Test cleanup explicitly waits
on both killed and ordinary coordinator paths; Clippy exposed the missing explicit wait
in the branched harness and passed after the correction.

Targeted acceptance: **11 passed, 0 failed**, 3 suites (cooperative_worker, crash_recovery, stop_resume). Clippy with warnings denied, formatting and whitespace passed. Evidence: `.agentconductor/verification/cooperative-recovery/`. Graphify: 1,325 nodes / 3,108 edges without LLM; CocoIndex located recover. Ponytail has no exposed callable capability; no execution claimed.

No runtime dependency, GUI or daemon was added. The prior full regression remains
193 passed / 5 ignored / 0 failed; do not present this targeted acceptance run as a new
full regression. Full-product completion and releases remain unverified.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/cooperative.rs` | `f0affcf5c6ffc9ea057a14b882fd10bf9242073ba2320684d7ee1e6e6b193b76` |
| `src/workflow.rs` | `55823ffd3c64714deb5d08f294d5986848956177f0ae72a87ef677b7bb92cea8` |
| `tests/cooperative_worker.rs` | `007e68f1b1b4a220912d35132996232ac07030e05161e499a5a2ad907457e156` |
| `tests/support/cooperative_worker.py` | `2710c6d81443cbee454134df47545f28c5d0969b2b641e0699ed10adac939c59` |
| `tests/crash_recovery.rs` | `8748e045138fb4d466fb3a248f94fc1f099658af40a2852797aa040ff890557a` |
