# Build identity, disposable storage and planning recovery

Verified source scope: [source fingerprints and observations](../raw/2026-10-12-app-storage-and-recovery.json), 2026-10-12; platform/native acceptance is tracked in the [implementation checklist](../../docs/07-implementation-plan.md).
Trace: R-93–R-95 -> UC-83–UC-85 -> DD-101–DD-103. See [operator manual](../../docs/user-manual.md).

## Running build identity

Both embedded titlebars receive Cargo's compile-time package version through template replacement. Shared styling keeps it small and readable. No cloud lookup, hard-coded release number or new runtime dependency is needed. The generated-document test covers Workspace plus its embedded Room; the real Room document is checked through the existing native-room integration fixture.

## Disposable metadata cache

Cache eligibility is narrow: completed quota queries, metadata-controller output and catalogs whose producer marker identifies a completed controller. Local settings, plans, audit/diffs/tests, source folders and recovery archives are durable assets, not cleanup candidates. Live/unknown parents, current diagnostics, referenced paths, aliases and malformed provenance are retained. A query's output can still be parsed after its provider exits, so the query parent must also be absent before deletion.

Preview is read-only and binds paths, sizes and filesystem fingerprints. Manual cleanup verifies that digest again and rechecks each tree before removal. Deleted raw cache has no recovery copy. The current quota observations and imported model settings remain separate from raw files. Catalog launch failures remove newly created inputs only when no child was admitted; uncertain live children retain their files.

Idle startup and settled metadata reuse the same cleanup with an eight-eligible-folder retention budget plus protected references. The scanner bounds candidates and tree visits, does not read large output bodies, and does not run a resident daemon. Private snapshots use the existing referenced-history lifecycle. Old unverified metadata folders may remain until ownership can be established; broad deletion would risk evidence or active readers.

The native Settings menu exposes this policy as Preview and clear local cache. Preview lists exact paths and totals together with protected/uncertain counts; the UI clears only the reviewed digest through the existing background dispatcher. Exact v0.1.36 macOS package workflow 14 used an isolated HOME, showed one disposable fixture and zero protected entries, removed that fixture after review, and preserved `startup.json`. The package check adds no Windows runtime evidence. See `docs/evidence/issue9-9845db79-cache-cleanup-ui.json`.

## Blocked planning recovery

Planning now passes an exact per-proposal stop marker to the existing process supervisor. The UI can issue the stop before normal snapshot settlement, including when a proposal is malformed. Queued metadata is dropped, while an admitted collector finishes under its existing bound. The supervisor stops its owned process group/job and removes the disposable planning snapshot; no arbitrary PID is terminated.

Current-workspace deletion validates the exact name, app-owned location, source separation and inactive runs. It switches to a fresh workspace in the same window, releases both caller and method-held Startup references, then archives the old app data. Those references otherwise keep startup.lock held after navigation and cause a false lease error. Failure restores the prior workspace when possible and preserves its data. Unknown ownership or active execution still requires explicit recovery/Stop.

## Verification and lessons

Red: build labels absent, stop_planning/cache requests unknown, current deletion refused. Green fixtures cover an actual waiting planner, current deletion with project preservation, cache preview mismatch, post-preview modification, live/unknown/alias protection, source folders with cache-shaped names and retention after repeated observations. Python UI/package contracts and native/platform results are tracked separately; do not treat source fixtures as Windows desktop observations.

Resource lesson: full debug artifacts accumulated 22 GiB and exhausted local storage during regression compilation. Only this project's generated target output was cleaned; use low debug information and bounded build concurrency for local verification. Cargo artifacts are separate from application cache and workspace evidence.

Revalidate when process ownership, metadata producers, CLI cancellation, cache references or native navigation leases change.
