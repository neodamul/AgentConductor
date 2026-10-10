# Native macOS end-to-end with Claude Code workers

retrieved_at: 2026-10-10 (KST), commits 9bba24f…b1e404e, Claude Code CLI 2.1.287 (claude.ai login).

1. Claude coordinator planned three tasks but assigned codex/opencode; reassigned to claude in plan review.
2. Workers requested read-only Bash; live requests waited while the run stayed `running`.
3. Gap A: the Agent Room listed no live requests (ledger-only). Fixed (9bba24f).
4. Gap B: identical commands in a later call or resumed attempt reused consumed/approved ledger ids and waited until timeout. Fixed by binding live requests to the CLI tool-call id (9bba24f).
5. Gap C: when the run was resumed from the Room, its coordinator occupied the Room job slot, disabling and refusing live approvals. Fixed by synchronous live decisions (dc48859).
6. Closing the launching terminal tab killed the coordinator (tool killed the process tree); Room Recover first refused while descendants were still exiting, then verified absence and recovered; explicit Resume continued with the same sessions.
7. Final: candidate 917074f, three tests exit 0, exact approval and merge from the Room; repository clean.
8. Automation notes: the `datetime-local` field can open a separate Date Picker window; modal dialogs are reachable via AXFocusedUIElement; Option+Tab reaches dialog buttons.
