# Session cleanup concurrency and focus

Issue: #962.

Guard the shared mutation entry point, not only its button. Disable dialog actions and keep pending feedback focusable; restore focus after repaint to the surviving cleanup/Close button. A generation invalidates inventory reads started before a mutation; refreshes requested while acting are coalesced and resumed afterwards. Never remove backend ownership/deletion validation.

Real DOM checks cover double click/direct handler invocation, pending dismissal, last-row cleanup, failed retry and stale read resurrection. Approved danger styles are scoped to the confirmation controls.
