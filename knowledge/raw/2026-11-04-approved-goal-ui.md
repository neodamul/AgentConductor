# Source record: approved Goal UI

- Observed/approved date: 2026-11-04 (Asia/Seoul; user conversation date).
- Source: user-reviewed local HTML design and subsequent explicit user revisions in this Codex chat.
- Upstream draft: https://github.com/neodamul/AgentConductor/pull/388 at e1597727e29891622fa8fb2cdc341a33cfc3c653.
- Current issue: https://github.com/neodamul/AgentConductor/issues/387.
- Canonical artifact: docs/design/approved-goal-ui/preview.html and its local resources.
- Source fingerprints: docs/design/approved-goal-ui/manifest.json and verification.json.
- Evidence: both-theme captures and browser interaction reports; application runtime is not verified.
- External assets: two pinned MIT Lobe Icons SVGs with source/hash/license records in brand-icons/.

The user explicitly distinguished a default-closed docked inspector from slide-overs, required
shared popup results, then standardized choices as anchored lists. Role Fallback/model controls
must remain. Latest visual revisions use 1.5px status borders, matched muted quota-window colors,
fixed logical sizes in fullscreen, grouped backgrounds with icons, and balanced red trash glyphs.
Claude Code is a display name; its internal ID/command remains claude.

These observations record requirements, not a claim that product features or deletion are implemented.
Rust development remains paused. The documentation tooling must not ship as the application UI.
