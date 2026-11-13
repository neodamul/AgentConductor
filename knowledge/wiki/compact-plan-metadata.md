# Compact Plan metadata

Issue #948: long verification commands must not set collapsed task row height. Reuse summary-only ellipsis and i18n-bound title attributes for command and owner/path text. Keep original values in the expanded editor. A real assembled-UI regression covers both themes and narrow/normal widths; the pre-fix summary reached 1400px.
