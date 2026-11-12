# Observe lifecycle boundaries in UI regressions

Source: [fingerprinted #919 checks](../raw/ui-regression-boundaries-919.json). Trace: R-210 / QAS-209 / DD-222.

Use the CDP navigation helper after writing a page preset. Calling location.reload inside a Runtime.evaluate leaves subsequent evaluation attached to a disappearing context. Require the target Workspace snapshot and controls, not just a logged snapshot request.

A released busy flag is not an entire saved UI transaction: post-save polling and label refresh still run before completion feedback. A sequential regression must observe that success before replacing the next response with an error. Keep named timeout diagnostics and all functional assertions; blanket retries hide sequencing faults.

Capture every loaded reference script in provenance, including preview-workspace.js. Run verify-and-capture.py to regenerate captures and manifest together; the strict manifest test is evidence, not something to weaken.

The Rust node_runtime test discovers every tests/*.cjs script. Do not wrap those same browser scripts in Python discovery as well: duplicate wrappers added about53seconds per platform without extending coverage. Keep the JavaScript assertions and the existing Rust discovery runner.

The full suite exposed an older source_progress_903 assertion that expected automatic guide collapse. Its failed-Jira/retry/focus checks remain intact; the operator now explicitly collapses the guide before the test verifies persistence. Record aggregate failures and targeted reruns accurately rather than claiming an unexecuted green full run.
