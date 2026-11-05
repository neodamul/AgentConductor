# Step tab overflow

Step tabs use their natural caption widths while there is room. When their row
is full, groups and tabs shrink and captions use ellipsis. Each app tab has a
localized title with the full caption. The reference uses the same shrink rules.

For a like-for-like layout comparison, the parity harness removes the mock
window frame's effect on the available content width for step comparisons.
This is a measurement adjustment; the displayed mock keeps its frame. Windows
fonts may consume more width than macOS fonts, so a fixed viewport cannot imply
that every caption fits without ellipsis.
