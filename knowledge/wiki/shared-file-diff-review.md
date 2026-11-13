# Shared per-file diff review

Issue: #957.

Step 6 and the finished-result dialog share acResultView.fileRow and the existing bounded diff parser. Step 6 uses a div row with the approved addition/deletion badges; result lists retain their li rows and original labels. All diff text is inserted as text, including binary notices.

Interactive rows must not be recreated on unchanged polling snapshots. Cache the review object identity plus run/state/error/candidate key; changed evidence still rebuilds the summary. Native buttons provide keyboard operation and aria-expanded. Existing candidate approval validation is unchanged.

Actual DOM tests cover both themes and viewport widths, expand/collapse, line counts, safe text, binary notices, and open/focused row retention through polling. Approved element parity covers closed controls, open diff and line colors.
