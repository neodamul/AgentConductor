# Asynchronous metadata

DD-86 uses one transient metadata collector and a bounded durable queue, separate from planning. Global busy flags for catalog/quota jobs previously disabled unrelated controls. Background scheduling and observed catalogs are excluded from editable execution revision; execution settings and the exact reviewed plan remain bound. Catalog profiles are snapshotted privately per query, compared before merge and never overwrite newer registrations or changes. The edit dialog opens before executable resolution; delayed results apply only if the same dialog/input remain unchanged. No new dependency or service is introduced.

Invalid catalogs and metadata bookkeeping failures become durable failed observations, preserving Snapshot and Save across restart. A new admitted query clears that failure; live child ownership is retained. Evidence: tests/metadata_failure.rs.
