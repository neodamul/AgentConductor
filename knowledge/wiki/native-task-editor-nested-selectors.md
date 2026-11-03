# Native task editor nested selectors

The task editor contains agent, model, and execution-option ComboBoxes. egui tracks one
open popup at a time, so opening a child selector replaced the parent editor popup. The
editor disappeared before the user could select an option.

The editor now stores its open state separately in egui temporary data and renders its
fields in a foreground `Area`. The Area remains in the AccessKit tree and keeps the same
overlay layout. Its selection stays open after a child ComboBox opens or closes. Escape
and an outside click close the editor; when a child popup is open, its click is allowed
to close the child first.

The rendered regression uses two agent profiles, opens the editor and agent selector,
selects the second profile, and verifies the selected task plus the still-visible editor.
It separately verifies Escape and outside-click dismissal. `cargo test --locked --test
native_ui_tasks --test source_layout` passes all 14 tests across two suites, including
the 300-line source-layout check. Focused Clippy with warnings denied and rustfmt pass.
The broad `cargo test --locked` run was interrupted after 818 passed / 14 ignored across
267 suites without a reported failure; it is incomplete and is not represented as a full
pass. This component-level result does not count as a packaged end-to-end workflow for
issue #9; the accepted workflow count remains 8/25.

Graphify code extraction reports 8,336 nodes and 14,385 edges for this source state.
CocoIndex Code was rechecked, but its installed launcher fails because the host Python
environment lacks `mcp.server.fastmcp`.
