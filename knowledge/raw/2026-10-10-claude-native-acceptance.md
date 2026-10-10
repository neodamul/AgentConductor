# Claude CLI and native app acceptance observations

retrieved_at: 2026-10-10 (KST), host macOS 27.2, Claude Code CLI 2.1.287 (claude.ai Pro login), Codex CLI 0.162.1 (quota exhausted: 5h 99 %, weekly 94 %).
revision: main 9678d71 … 75b121d.

## Observations

1. `claude auth status` reported `loggedIn: false` inside the desktop-app session; the operator completed `claude auth login --claudeai` in a browser. Afterwards every live stream-json init event reported `apiKeySource: none`.
2. Live guarded workers issued `Read` on a not-yet-created target before `Write`; the file policy returned an error for `NotFound`, which terminated the control channel (`tool path cannot be inspected`). Source of DD fix in commit e178d3b.
3. Native app (ui-new) default Claude profile is `unverified`; the coordinator preflight refused Claude (`coordinator requires read-only capabilities`). Codex was unavailable because of quota.
4. The Claude coordinator spread tasks across claude/codex/opencode; the operator reassigned them in plan review (saved to `review.json`).
5. Workers requested read-only Bash (`ls -la; git status --short; cat <file>`). After stop-and-resume each attempt regenerated a different command or description, so exact approvals never matched (loop in `waiting_tool_approval`). Source of R-30/DD-32.
6. Accessibility automation: modal `<dialog>` removed the page from the AX tree; WebKit reported stale/duplicate values for dynamic task cards; Tab skipped buttons (full keyboard access off) while Option+Tab reached them; host input source Korean 390 Sebeolsik garbled synthetic keystrokes, so text entry used Edit → Paste. Source of R-32/DD-34 and open accessibility items.
7. Approved tool requests still offered "Review & allow" (fixed: buttons only for pending requests).

## Evidence locations (not committed)

Scratch workspace and run directories under the session scratchpad; screenshots captured via `screencapture -R`.
