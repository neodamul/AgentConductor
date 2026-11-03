# Acceptance and verification

This is **design-reference evidence**, not application or release acceptance.
Approval date: 2026-11-04 (Asia/Seoul). The exact source/image hashes are recorded in
[verification.json](verification.json); browser interaction evidence is in [interactions.json](interactions.json).

## Reproduce locally

No application build, external service, API key, or CLI agent is needed.
The artifact uses only local assets and system fonts.

```sh
python3 docs/design/approved-goal-ui/verify-and-capture.py
# Repeat interactions alone without replacing image evidence:
node docs/design/approved-goal-ui/verify-interactions.mjs --check-only
```

Capture prerequisites: local Chrome at the script's documented path, Python standard library, and modern Node.
Interaction checks use Node's built-in WebSocket/fetch and one temporary headless Chrome process.
Change executable locations for another host; these tools are not bundled with the application.
The first command runs the whole pipeline in one temporary browser with explicit viewport sizes.
The browser driver uses real asynchronous close events, not virtual-time assumptions about native dialogs.
Both runners remove their own temporary browser profiles; do not kill or clean unrelated processes.

The image capture viewport is 1280 × 960. Default PNGs are clipped to the verified app rectangle
(x=50, y=16, width=1180, height=800), excluding preview-only captions/arrows.
The full-window case uses a 1920 × 1080 viewport and records its own frame crop.
Screenshots are static samples; inspect the running pulse and reduced-motion behavior in the live reference.

## Image coverage

Twenty-six states per theme, **52 images total**. See [paired gallery](gallery.html).

| Group | Images in each theme |
|---|---|
| Main flow | Workspace/Goal, roles, plan, tasks, tool request, permission denial, verification, review/merge, result, new Goal |
| Docked layouts | Inspector upper, inspector lower, collapsed left rail, wide/full-window |
| Modal/slide-over | Goal list, permanent-delete warning, briefing, coordinator chat, settings, coordinator/model list, role/model list, fallback-agent list, fallback-model list |
| Quota | Healthy details, error/retry, unsupported |

The pipeline records the actual scenario title, theme, explicit viewport, measured app frame, and
source/image hashes. Every capture checks frame bounds and absence of script errors or horizontal
scroll displacement; quota/choice captures additionally require their popup to remain open.
The interaction suite checks default/closed inspector state, docking, fixed headers/footers, popup
behavior, factual fields, choice/Fallback rules, button cursors, icon shapes, and both themes.
Completed Goal-list fixtures use the preview completion transition, not a status-label-only change.

## Local interaction coverage

Each interaction run covers both themes at widths 840 and 1280:

- [x] Inspector starts closed; no body Details tab and no internal close button.
- [x] Icon opens/closes the dock; center width changes/restores; no modal backdrop or overlap.
- [x] Left/right toggles are independent; selected task refreshes the open inspector.
- [x] All eight card variants keep header/footer stationary while a long body scrolls.
- [x] Slide-over focus does not scroll the frame; rapid reopen survives pending close timers.
- [x] Closing a slide-over releases background interaction.
- [x] Settings changes the local theme without changing layout.
- [x] Briefing retains factual fields across plan/run/attention/verification/review states.
- [x] Goal-list Result reuses the same centered popup and leaves the list stable.
- [x] Trash is separate from result/folder; Cancel is the default and preserves the Goal.
- [x] Confirmed mock deletion can remove the last in-memory Goal; a replacement Goal can be saved.
- [x] Preview arrows keep the same coordinates across all ten scenarios and sit below the image.
- [x] Ellipsized labels keep complete tooltip text.
- [x] Quota windows have distinct muted colors shared by rings, popup legends, and meters.
- [x] Full-window layout preserves icon and border sizes.
- [x] Anchored coordinator/role choices update immediately; dismissing without selection preserves values.
- [x] Fallback choice/model remain available, reset correctly, and are separated by the right arrow.
- [x] Settings and inspector choices use the same anchored list style.
- [x] Prepare/Run/Finish remain visible as inline group icons; buttons use a pointer cursor.
- [x] Agent and Goal removal share the same red trash SVG.
- [x] Unsupported model catalogs allow CLI default or validated manual model IDs.

Critical text contrast is checked for seven text pairs per theme (14 checks): primary/secondary text on
page/card surfaces, primary action text, warning text/fill, and error text/fill. All must be ≥4.5:1.
Eight additional non-text quota/surface pairs must meet 3:1. Six group-background/secondary-text
pairs must meet 4.5:1, for 28 checked pairs total.
A passed token-pair check is not a blanket accessibility audit of every browser/native widget state.

## Product implementation acceptance remains open

The following must be verified in the real application before completion or release:

- [ ] Existing CLI subprocess authentication, sessions, capability-based model discovery and fallbacks.
- [ ] Goal/phase state mapping and restoration across restart; no independent UI lifecycle.
- [ ] Real quota windows, Enterprise semantics, error/unsupported display, asynchronous five-minute refresh.
- [ ] Actual worktree/branch/folder identity, including non-Git and worktree-disabled Workspaces.
- [ ] Real native folder opening on macOS and Windows.
- [ ] Permanent deletion: exact owned folder, active-job cleanup, cancellation, partial failure, and unrelated-data protection.
- [ ] Native updater, real release notes, settings persistence, MCP, import/export/share and install-help menu entries.
- [ ] Exact diff/test evidence, stale-candidate protection, human approve/reject/merge and audit records.
- [ ] Full task activity, changes/tests, raw logs, lessons, thinking fold and interruption history.
- [ ] Non-steering instructions delivered via the established checkpoint protocol.
- [ ] macOS/Windows native widgets, keyboard/screen-reader behavior, Korean IME and English/Korean labels.
- [ ] Every component in both themes; unchanged information and clear enabled/disabled/hover/focus/error states.
- [ ] Both panels open at minimum supported native size, long names, scaling, and real data volumes.
- [ ] No UI-thread process/HTTP/MCP/Git/filesystem blocking; independent bounded refresh/event delivery.
- [ ] Source/tests separated, per-file ≤300 lines, local TDD and required regression tests.

Preview buttons and sample metrics must not be cited as evidence for those unchecked items.
The reference intentionally does not execute real deletion, agents, commands, or repository mutations.

## Visual review rules

Review images side by side at the same state and dimensions, then inspect the live artifact.
Do not compare a JS screenshot to a claimed Rust implementation or use a stale binary.
Do not omit fields because a simplified mock does not exercise them.
Any approved behavioral change needs an explicit design update, linked issue, and new evidence.
