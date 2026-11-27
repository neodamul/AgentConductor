# State memory boundary

`state/memory.rs` owns queries for pending outbox items, delivery marking and
local recall. `state/records.rs` retains the atomic transaction that writes a
document, its audit entry, outbox item and search entry together. Splitting the
access methods must not split or weaken that transaction or alter the Store API.

## Repository and coordinator contention (#1107)

The Workspace uses the kernel repository lease as a cross-surface fact for planning.
The Room also checks the run's coordinator lock before approval, merge, or quota.
These probes do not write PID or ownership metadata. Gate mutations before launching a
child, keep the actual worker's locking and candidate/revision validation, and translate
a concurrent lock conflict to an actionable retry message. Read and live stop paths stay
independent of the busy flags.
