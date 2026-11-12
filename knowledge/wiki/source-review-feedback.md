# Source review disclosure and checkbox geometry

Source: [fingerprinted regressions](../raw/source-review-feedback-916.json), #916 / #917. Trace: R-134 / UC-124 / QAS-130 / DD-143.

Disclosure belongs to the operator. Put the initial guide state in markup; progress/result renders must not assign details.open. No flag, persistence service or polling loop is needed because the guide DOM node is retained.

Source checkboxes inherited a generic full-width input rule, moving their glyph into the middle and pushing the header outside the card. Scope16px dimensions and theme-consistent checked presentation to source headers. Keep real checked/disabled semantics and keyboard focus; required Jira remains selected. Bound reference text and provide full tooltips rather than hiding overflow at the dialog edge. Confirm exact checked references in Apply, not only a displayed count.

Regenerate references with verify-and-capture.py so captures and manifest describe the same bytes; invoking only the Node renderer leaves the manifest stale.
