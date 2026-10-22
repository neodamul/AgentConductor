# Asynchronous metadata

DD-86 uses one transient metadata collector and a bounded durable queue, separate from planning. Global busy flags for catalog/quota jobs previously disabled unrelated controls. Background scheduling and observed catalogs are excluded from editable execution revision; execution settings and the exact reviewed plan remain bound. Catalog profiles are snapshotted privately per query, compared before merge and never overwrite newer registrations or changes. The edit dialog opens before executable resolution; delayed results apply only if the same dialog/input remain unchanged. No new dependency or service is introduced.

Invalid catalogs and metadata bookkeeping failures become durable failed observations, preserving Snapshot and Save across restart. A new admitted query clears that failure; live child ownership is retained. Evidence: tests/metadata_failure.rs.

## Shared-sidebar completion polling — issue #9, 2026-10-22

An apparently hung Claude quota refresh can be a stale UI snapshot even when the
CLI has exited successfully. The v0.1.36 macOS package kept its busy indicator on
Workspace settings; switching to Agent roles replaced it with the refresh button
and the completed observation. `Probe::update_ui` had restricted metadata polling
to the Team step even though the sidebar is present on all three steps.

Poll active metadata on every workspace step through the existing async worker
and bounded `refresh_due` scheduler. Metadata replies update catalogs/snapshots
without reloading unsaved workspace inputs. Idle metadata does not start fast
polling. The rendered regression uses a real request worker and disposable local
startup state; no agent, quota request, or update check is launched.

Evidence: `tests/native_ui_probe/renderer.rs`,
`docs/evidence/issue9-sidebar-metadata-polling.json`.
