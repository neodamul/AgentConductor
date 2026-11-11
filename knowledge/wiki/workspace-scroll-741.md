# Workspace scrolling (#741)

The Workspace step keeps its heading and action footer stationary. Its content
scrolls independently so long Goal input cannot cover the Jira, Confluence or
optimizer controls. The body has a zero minimum height so a flex layout can shrink
it, and the footer stays outside that scroll area. The focused browser regression
uses a tall Goal field at 840 and 1280 pixels and checks that the tools have a
visible rectangle above the footer after scrolling. Native app and dark-theme
acceptance are still required.

Windows rounded the final tools row onto the footer boundary (0.125 px overlap).
The regression now asserts at least 1 px of clearance, and the scroll body adds
2 px of bottom padding so the last row stays clear of the fixed footer.
