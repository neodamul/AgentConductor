# Interrupted rework continuation

Trace: R-27 → BG-11/BG-12 → SF-24 → UC-21 → QAS-22 → DD-29.

## Contract

Explicit resume preserves unfinished local rework from the exact prior plan/task/agent/
session/attempt binding. Unexpired pending and accepted requests retain their instruction,
prior state and local source in a bounded continuation. The original directive becomes
superseded; its immutable continuation receipt is committed in the same SQLite transaction.
Preparation can be retried if the coordinator fails before saving the resumed report.

The next prompt labels uncertain prior application, requires inspection of partial files
and forbids duplicating applied changes. Current human feedback, paths and permissions
control. This is context, not a new directive, acknowledgment or permission. Completed,
rejected and expired rework is excluded; explicit resume supersedes unfinished pause/
handoff without replaying stop instructions. The context budget is 16 KiB.

## TDD and acceptance

Red: missing preparation API failed compilation. A second Red check showed accepted
pause/handoff remained accepted after resume rather than being superseded. Green adds
provenance-bound rework preservation, transactional receipt/supersession and non-rework
exclusion. Four local tests cover retention/idempotency, expiration/foreign binding,
pause/handoff supersession and completed/rejected exclusion.

The compiled fixture kills the coordinator after accepted rework while a partial file
exists. Live ownership blocks recovery; quiescence permits stopped recovery. Explicit
resume provides the original instruction/source to attempt 2; the real fixture checks
the partial file before producing revised content. Old addresses fail and merge approval
remains separate. This is a controlled CLI, not a company LLM acceptance test.

Full regression: **199 passed, 0 failed, 5 ignored**, 49 suites. Clippy with warnings denied, formatting and whitespace passed. Evidence: `.agentconductor/verification/cooperative-continuation/`. The existing-agentmemory-daemon test was not repeated in this slice.

## Remaining scope

Automatic memory collection and reviewed-target handoff, worker authentication, context
across reassignment/arbitrary repeated interruptions, actual provider acceptance and
native Windows/releases remain open. No GUI, daemon or Cargo dependency was added.
No Ponytail execution is claimed because callable capabilities are unavailable.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/cooperative.rs` | `e833f9a1019474d39864ffe39d29e3640dbe29572f0470286865b78b5decb982` |
| `src/workflow.rs` | `4d9f61377460759dc76a3762c7c25450aea56ab82400ebd6dd0302d44df241a2` |
| `tests/cooperative_continuation.rs` | `c216abb9aec5c42ec73ef793d915cee434dbf61360759b9aa1f4d7a5b1ae6c3f` |
| `tests/cooperative_worker.rs` | `58c4cf99e8a993da19d10ee1ecc28b40ee91317b524d58ca57a1f133c73d9b4c` |
| `tests/support/cooperative_worker.py` | `f6fd0be02319683738c88b2050678789e657b4641367aca20f01ab30d17bc09c` |
