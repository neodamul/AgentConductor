# Unix process outcomes

Trace: DD-24; R-06 normalized failure evidence and R-09 process lifecycle. Verified targeted contracts: 2026-10-10 KST. Full regression: 144 passed, zero failed, four explicit ignores, 41 result summaries in 70.88 seconds. Clippy/rustfmt/diff checks passed.

## Contract

A Unix signal termination has no numeric exit code. ProcessResult, durable Ownership and test evidence preserve `termination_signal` separately, with legacy records defaulting to unknown. Empty optional fields are omitted from serialized evidence so ordinary legacy evidence retains its shape. UI coordinator jobs track completion independently from exit code. A signal-killed coordinator is failed and releases the action slot; it is not left running and does not fabricate exit -1 or 128+signal. Audit results carry both outcome fields; task errors and UI notices expose recorded signals. Signals do not create quota restrictions or authorize retries.

## TDD evidence

Red: the new process/legacy tests failed to compile without signal fields. A real signalled UI coordinator test separately failed because the old controller reported synthetic exit -1. Green: a child self-terminating with SIGTERM reports null exit code/signal 15, while numeric exit 143 reports exit code 143/no signal. Terminal owner records retain the distinction. Two successive signalled UI jobs prove completion releases the action slot, with quiescence checked through kernel probes. Targeted process, async-control, ownership, UI, workflow and quota recovery suites passed 33 tests.

The full test attempt exposed ps execution denied by the local sandbox. The old tests accepted empty ps stdout without proving query success. Replaced all three descendant ps probes with bounded kill(pid, 0) checks: only ESRCH proves absence; EPERM and a still-present PID fail. This is stricter than treating an unqueryable process or zombie as absent. A separate full attempt reached memory tests and stopped because fixture TCP bind was denied. These failed attempts are retained; the full suite is rerun with localhost fixture socket access.

## Retained real fixture evidence

A fresh three-task execution of the older fixture in `signal-fixture-reproduction/run` succeeded, with all worker/test owner records reporting numeric exit 0 and all tests exit 0. It stops at waiting_approval, preserving the human merge gate. Therefore the historical empty-log/no-exit-code failure cannot currently be reproduced. Do not infer its signal or cause, erase the old runs, or claim this metadata change fixed that historical failure.

## Source fingerprints

- `src/process.rs`: `a0cc70068c53e7bd3c640b8f53e6ffe08773c3e0ad45d16792573ccd2535f0d6`
- `src/ui.rs`: `9c5a5e46c6a5b286da0ab577d15316949c293938bfc2332bf2220a63b82a59f0`
- `src/workflow.rs`: `a8f16de57053670bd9a4344177b39411b2bdcf771a3a1b27d2496a8991556bd3`
- `src/ui/room.js`: `46afc3e1cce45194d6cffc1a810cb04ab156b71d80eb5e717839e679107031e5`
- `tests/process.rs`: `78cac72075b7741ef50d6f0a9ef1499ee9e3d63b36958ab2f2943f63013b2ed0`
- `tests/ownership.rs`: `fd7c14dd20e755a69dae1f02a21d208b19d949f36087456c6cd38121b8c0156e`
- `tests/async_control.rs`: `fbc8794e3cff33d6422ce12213ccff8f7a43de09d9833caa1710cb4c001b2864`
- `tests/ui.rs`: `df596bc0e8d52b0a0fdd1ba4321d4ab128e8363b7abe9c69acbfe19614817628`
- `tests/support/liveness.rs`: `76e418192ca6035c1895cc8e76993e2c1e9f8a2dcf14c326288d02c1031ed471`
