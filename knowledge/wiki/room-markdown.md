# Room Markdown

Source: [GitHub #1099](https://github.com/neodamul/AgentConductor/issues/1099), DD-171.

Room prose uses the same offline renderer as coordinator chat. The plan editor preview reads the current draft. Historical plan analysis and task detail, activity narratives, lessons and result summaries render headings, lists, code, tables and links. Source values, raw logs, diffs and test output remain literal.

The page sends bounded batches to the native bridge and caches the results. The native renderer escapes embedded HTML, suppresses images and validates links. Keep the plain-text fallback readable if the bridge is unavailable.

Compact task labels use the first meaningful plain-text line in the plan row, task rail and inspector. Remove structural Markdown markers and inline delimiters, keeping identifiers and the full editable prompt.

## Raw log presentation (#1105)

Render plain log sections through the shared native Markdown renderer when the Raw
Logs tab is visible. Leave structured JSON records in preformatted blocks, preserving
decoded line breaks, metadata, and literal path escapes. A JSON example inside a
Markdown fence stays with its surrounding Markdown. Cache the rendered log view
until the underlying text changes.

The Room starts with the run status in the title row. Expanding restores the full
Goal and the run information row. The stepper, selected task, and Goal actions remain
available while switching between those states.
