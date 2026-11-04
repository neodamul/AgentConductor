# Cached quota ring and popup contract

Source: GitHub #404, R-212 / DD-224. Both startup and run snapshots expose
`quota_indicators` by profile ID. Rendering does not start a CLI, timer or query.

`rings` selects up to two rows from `windows` or `budgets`. Sort provider windows by
interval; prefer Enterprise individual allocation before credit balance. Preserve all
popup rows. A single allocation is one ring, without an invented denominator ring.
Unknown, stale, expired and error capacities are null; unlimited is a separate flag.
Keep provider labels, reset text, source/time, errors and retry through the existing
per-agent asynchronous quota command. Unsupported custom agents remain first-class
profiles and get an explicit unsupported indicator, never invented quota support.

Native tests cover projection rules and the actual cached startup/run snapshots.
Ring geometry, status/avatar separation, pointer/popup interactions and native platform
package checks remain separate UI acceptance; this backend unit does not finish #404
or authorize a 0.2.0 release.
