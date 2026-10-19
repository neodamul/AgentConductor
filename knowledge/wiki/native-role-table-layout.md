# Native role table layout

The native Workspace role editor uses five columns: role, primary agent/model, and alternate agent/model. Keep the English coordinator label concise so its explanatory text does not widen the table. Long CLI and model identifiers should be shortened in selector rows and values; retain the complete value in hover help and in the saved role configuration.

The regression renders both the Korean empty-registry state and the English populated state in an 820px viewport. The current implementation uses a 108px selector width and eight visible identifier characters plus an ellipsis. The exact Rust preview was also inspected at 1140×772; all columns were visible with no registered agents.

This is component-level width evidence only. Populated profile rendering in the packaged macOS and Windows applications, accessibility review of the full identifier tooltips, and issue #9 end-to-end workflow acceptance remain open.
