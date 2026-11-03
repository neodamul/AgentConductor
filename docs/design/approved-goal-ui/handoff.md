# Claude collaboration handoff

Approved design date: **2026-11-04 Asia/Seoul**. Engineering documents are English;
conversation with the user remains Korean. The preview and reviewed images are Korean.

## Authority and branch

- Tracking issue: [#387](https://github.com/neodamul/AgentConductor/issues/387), enhancement.
- Documentation branch: `codex/issue-387-approved-ui-spec`, created through the issue's Develop association.
- Base observed for this package: main `3d332cf15bacc4b202195aed0e339a291f2949d3`.
- Earlier Claude design: PR #388, head `e1597727e29891622fa8fb2cdc341a33cfc3c653`.
- Canonical design: this directory's behavior/visual documents and paired images, not PR #388's stale draft.
- Rust work remains parked on `codex/issue-382-unified-tool-modal`; this task does not resume it.
- Do not edit or clean the other agent's worktree or rewrite its branch.

## Most important corrections to PR #388

1. Right details are a **docked rail**, initially closed, opened/closed only by the toolbar icon.
   No X, no slide animation, no dimmed background, no Details tab in the body.
2. Left cards use status borders/pulses and hover text. No permanent live sentence,
   printed executable path, or fixed "three sessions" label. All configuration functions remain.
3. Goal list, briefing, coordinator chat, and settings are slide-overs; results are centered dialogs.
4. Goals belong to a Workspace; slug stays internal. New Goal starts from Workspace step 1.
5. Result popup is detailed immediately and is shared by automatic completion and list Result.
   No result-details expander or New Goal shortcut inside the result popup.
6. Header/footer stay fixed while the card body scrolls. Header information row has fixed height.
7. Workspace/Goal section spacing, six-step compact tabs, hover disclosure, and both themes are required.
8. Titlebar order and divider alignment are fixed. Update is before chat/settings/right-panel icons.
9. Choice fields use one anchored-list standard across Workspace/roles/Fallback, inspector and Settings.
   Keep Agent/Model → Fallback/Model, immediate draft selection, model reset and unsupported/manual cases.
   Use Claude Code as display name, 1.5px status borders, muted matching quota-window colors, and neutral desktop tabs grouped by subdued backgrounds with integrated Prepare/Run/Finish icons.
10. Goal folder action is beside Result; red trash is separated at the far left of that same compact row.
   Real permanent deletion requires confirmation and native ownership/lifecycle checks.

## Existing functionality inventory to preserve

Inspect `src/ui/startup.html`, `startup.js`, `room.html`, `room.js`, `room.css`,
message catalogs, native folder/dialog handlers, and `docs/design-guide.md` on the implementation branch.
In particular, keep every existing task-inspector section and every field in `briefTeam()`.
This package's seven inspector sections and briefing cards are a visual inventory,
not a replacement backend schema.
Preserve MCP registration, profile executable discovery/editing, model capabilities, fallback agents,
manual/automatic quota resumption, non-steering checkpoints, runtime controls, and audit records.

## Implementation workflow

- Read `behavior.md`, `visual-spec.md`, `architecture.md`, and `acceptance.md` before touching UI code.
- Agree one owner for shared specification files; leave independent work in separate worktrees/issue branches.
- Register implementation deltas and architecture trace before TDD changes.
- Follow `plan → task → verify → completed`; keep source/tests separate and each file ≤300 lines.
- Reuse current backend state and events. Keep slow operations asynchronous and refresh independent.
- Validate locally before waiting on remote CI. Compare every component in both themes/languages.
- Commit/push completed units, record evidence and AgentMemory handoffs, remove only owned temporary artifacts.
- Do not close implementation issues or publish a release based on design-only evidence.
- Do not merge the older design unchanged: reconcile it against this approved contract first.

## Design-reference limits

The HTML and its JavaScript are local documentation tooling, not product code.
It does not launch CLIs, create worktrees, open a native file explorer, delete disk files, or perform updates.
Native quota/model/approval/delete/merge/schedule behavior still needs implementation tests.
Agent-card tool buttons, some subviews, and menu contents are intentionally incomplete examples;
they must not be treated as permission to omit existing features.

Captured `mock-*` sessions, percentages, tests, commits, and sample paths are fixtures.
The sidebar's fixed examples do not prove agent catalog discovery or authentication works.
Windows/macOS native rendering, screen-reader/IME behavior, and real provider limits are not validated here.

## Shared artifacts

The complete package can be reviewed without running AgentConductor: open `preview.html` or `gallery.html`.
Source/image hashes and local browser checks are in `verification.json`.
`verify-and-capture.py` regenerates evidence with local Chrome and its DevTools frame capture;
its output is documentation evidence only, and the script is not shipped.

Claude's 2026-11-04 review identified two retained capabilities needing explicit screens: profile
Edit (including executable/Browse) and R-118 active-run close confirmation. Both are now referenced
in the preview and image gallery. The actual R-118 backend remains the merged #383 implementation.
Keep the documentation on this branch/PR #389; do not move it onto or merge the superseded #388.
