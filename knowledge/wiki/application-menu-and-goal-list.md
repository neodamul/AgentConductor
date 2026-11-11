# Application menu and Goal-list behavior (#750, #751, #753)

The approved menu groups commands into Workspace, Agents, View, and Help. Workspace contains recent workspaces, Goal actions, workspace creation/opening/settings, plan rules, coordinator messaging, and settings import/export. Agents contains registration; View toggles the left rail and inspector; Help contains updates, manuals, tools guidance, and About. The direct Settings item opens the Settings panel, and Close window remains a top-level action. The separate Workspace chooser is unchanged.

Menu groups expose `aria-haspopup` and `aria-expanded`. Click or hover opens a group; ArrowRight, Enter, or Space opens it and focuses the first item. Up/Down moves within the current menu level. ArrowLeft or Escape closes a submenu and returns focus to its parent; Escape at the root closes the menu and restores focus to its opener.

Goal-list text preserves line breaks, collapses to two lines, and expands or collapses with a keyboard-accessible button that updates `aria-expanded`. The agent-registration dialog uses a 6 px label gap, 12 px field rhythm, and 34 px single-line controls. The plan-source row for reading or hand-writing plans is hidden to match the reviewed plan step; the underlying handlers remain available to other paths.

Local verification covers the menu in light and dark themes at 840 and 1280 px, Goal text expansion, and plan-source visibility. The consolidated source branch is tracked in PR #756. The shared design-parity specification and Room renderer still require the separately tracked 300-line refactor before final delivery.
