# Task and integration review scope

Issue: #963.

The task renderer must clear integration-only surfaces, not only verification rows. Hide review-summary when drawing a task. The integration renderer restores visibility before checking its cached unchanged evidence, so switching back does not require reloading or discarding reviewed evidence.

The real-DOM footer regression checks both directions and retains revise, approve, merge and completed-result coverage.
