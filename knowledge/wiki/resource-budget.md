# Resource budget as a product constraint

AgentConductor exists because running three AI desktop applications simultaneously
exceeds the operator's host resources. Minimal memory/CPU and fast interaction are
primary design criteria for every feature, not an optional cleanup stage.

Use one lightweight native shell and existing authenticated CLI processes only while
needed. Avoid resident provider CLIs, duplicate app windows, unnecessary daemons,
heavy orchestration frameworks and speculative parallel background probes. Dispatch
blocking I/O off the UI thread; bound subprocess output/deadlines, deduplicate requests,
cap concurrency, and clean owned process groups. Async work still consumes resources
while active; do not equate nonblocking behavior with zero CPU/memory.

Measure idle and active app/process-tree memory and CPU before claiming efficiency.
Account for the embedded native webview helpers as well as Rust and temporary CLI
children. No whole-process-tree resource budget or comparative benchmark has been
established in this increment. The current quota probes are short-lived and serialized;
a delayed-provider integration verifies nonblocking request/snapshot handling.

User correction persisted as shared agentmemory lesson `lsn_40bb8ef530c1f55e` and sent
to Claude as request signal `sig_mv3e7zoe_00ebb85f0bd8` on 2026-10-11. Transmission
success is not evidence that the recipient read or applied the message.

DD-73 increases the default quota interval to five minutes and permits per-agent
1–60 minute settings. Filesystem executable lookup and embedded AC artwork add no
resident process. Quota refresh messages are suppressed; values and tooltip provenance remain.

## 2026-10-11 re-measure after the translation layer (DD-76)

Release build, idle workspace window, 60 s sampling: Korean 106.9 MiB main RSS / 0.00 % CPU; English 112.4 MiB / 0.00 % CPU. The translation layer adds no measurable cost (the observer is idle without DOM changes). Main RSS at window start is still above the 100 MiB NFR-05 target — open follow-up.
