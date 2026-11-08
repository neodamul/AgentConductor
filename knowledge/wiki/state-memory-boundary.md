# State memory boundary

`state/memory.rs` owns queries for pending outbox items, delivery marking and
local recall. `state/records.rs` retains the atomic transaction that writes a
document, its audit entry, outbox item and search entry together. Splitting the
access methods must not split or weaken that transaction or alter the Store API.
