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

The exact `6a1f69e` macOS package passed manifest validation and its four-case
parallel workflow smoke. In its single app window, remaining on Workspace
settings during a real Claude refresh replaced the spinner with `92%/69%`.
Codex also refreshed to `80%/18%`; no generation or merge was requested.
Windows runtime verification of this fix remains open.

## Test worker lifetime on Windows

Dropping `RequestWorker` disconnects its queue; its thread still owns the Room
until it observes that disconnect. A fixed 20ms sleep before removing the test
directory races that release and fails on Windows with OS error 32. Integration
fixtures now wait for the worker's actual Arc reference to disappear, then drop
their last Room before removing its directory. The wait is bounded and test-only;
closing the product window gains no blocking join or new shutdown policy.

Run `39748528606`, job `119723469899` also exceeded a five-second cache-clear
test reply wait. That integration guard is now 30 seconds under loaded CI;
application I/O deadlines are unchanged. Local validation passes 51 tests across
four suites. Actual Windows confirmation remains required.

## Native periodic quota deadlines

Polling an active request is different from scheduling the next request. The
native sidebar previously implemented only the first half: manual refresh
completed, but an idle workspace never started its next automatic collection.

Keep last attempts per profile in the window and use the core Rust interval
policy (five minutes by default, configurable independently). The native event
loop requests a repaint at the earliest due deadline, then sends one request
through the existing worker. Minimized windows, running jobs, pending requests,
and active metadata defer automatic collection. Only saved profiles qualify;
draft fields are never reloaded by metadata replies. Failed admission is also
rate-limited so it cannot spin, and a manual request updates only its own time.
Metadata snapshots omit icons and retain already-loaded icon data locally.

Tests use simulated timestamps and a real worker rejecting an unregistered
agent. They establish scheduling, failure throttling, independent deadlines,
manual refresh timing and draft preservation without launching a provider.
Authenticated collection and a full real-time cadence remain separate package
acceptance checks; these tests do not establish those outcomes.

Exact package `017b51c68af3` started collection without a refresh click. Codex
returned a new authenticated observation. Claude 2.1.292 emitted only its
screen-reader-mode line before the bounded ready-prompt deadline; no `/usage`
or generation request was sent. Treat this as a failed provider observation,
not proof of successful Claude collection or a scheduler failure.

The package also exposed a brief global metadata loading banner during worker
roundtrips. Background metadata should use only its agent-row indicator; retain
foreground save progress and actual errors. Rendered regression coverage belongs
in `tests/native_ui_probe/metadata_feedback.rs`.
