# Update indicator visibility

Verified: 2026-11-12. Scope: shared Workspace/Room titlebar, issue #857.

## Contract and lesson

The titlebar update indicator is hidden before an update response is known and
whenever the updater reports no newer version. It becomes visible only when
`newer` is true, including installation or a failed installation of a known update.
Manual checks remain available in the application menu even when the indicator is hidden.
The existing updater supplies version comparison and background checks; no additional
poll, worker, process, or request is needed to control presentation.

Commit `29f85beb` (#673) removed visibility control while matching the mock's
always-visible sample icon. A visual example of an available update must not change
the product's conditional behavior. The parity fixture must explicitly provide
`newer: true` when measuring that icon; the normal titlebar excludes it.

Regression coverage exercises initial/unknown, current, newer, checking, failure,
installation, manual checks, and navigation into the Room. The initial-state test
failed before the fix; all 14 checks passed in each of four theme/width cells.
Revalidate when the updater state schema or shared titlebar ownership changes.

Sources: [observation](../raw/update-indicator-857.json),
[issue](https://github.com/neodamul/AgentConductor/issues/857),
[renderer](../../src/ui/startup.js),
[regression](../../tests/update_indicator_runtime.cjs),
[release-note safety](update-notes-safety.md).
