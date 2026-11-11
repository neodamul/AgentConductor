# Truncated text tooltips (#749)

A delegated pointer listener checks the hovered element and its ancestors for
actual ellipsis or line-clamp overflow. It appends the full visible text to any
existing description and restores the original title when the text fits. Ordinary
input values are supported; password values are excluded. The listener runs only
on pointer events and creates no polling loop. Tests cover dynamic elements,
resizing, existing titles, inputs and password fields, plus a browser walk of
Workspace, roles, plan and running-task screens. Other dialogs, verification,
review, dark mode and native app acceptance still need explicit coverage.
