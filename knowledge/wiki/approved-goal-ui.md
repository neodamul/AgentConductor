# Approved Goal UI: preserving meaning across a redesign

Status: maintained design fact, verified 2026-11-04. Scope: AgentConductor #387.

## Sources

- [Source record](../raw/2026-11-04-approved-goal-ui.md).
- [Normative behavior and visuals](../../docs/design/approved-goal-ui/README.md).
- [Evidence and limitations](../../docs/design/approved-goal-ui/acceptance.md).
- [Claude handoff](../../docs/design/approved-goal-ui/handoff.md).
- [Resource budget](resource-budget.md).

## Decisions and reasons

A Workspace links one working directory and owns Goals. Internal slug folders are storage identity,
not a second user-facing lifecycle. Commit/merge applies approved results to the linked directory.
The six-step shell removes duplicate navigation without removing existing backend capabilities.

Task details are a third docked column, initially closed, toggled only by the toolbar icon.
Goal list/briefing/chat/settings are slide-overs; result/delete confirmation is a centered dialog.
This distinction prevents accidental modal blocking and preserves limited screen space.

Choices use a common anchored list at the clicked field, including coordinator/model and role
Agent/Model → Fallback/Model. Primary and fallback model capabilities stay separate. Unsupported
catalogs allow CLI default or explicit ID input. Claude Code branding does not rename the CLI ID.

The two themes share geometry, icon sizes, 1.5px status borders and information. Fullscreen expands
content rather than scaling controls. Quota-window colors match the ring, popup legend and meter;
errors/unsupported data never become fabricated percentages.

## Lessons and verification scope

- Do not replace the full task inspector or briefing with a summary-only mock: inventory fields first.
- Do not treat a slide-over and a docked panel as interchangeable.
- Do not remove group meaning to simplify styling; use grouped backgrounds and accessible icon names.
- Preserve model/Fallback controls while making selection more compact.
- Native popup close events need real browser event-loop verification, not virtual-time assumptions.
- Captures need source hashes and state checks; screenshot filename alone does not identify the state.

The reference checks interactions and tokens with fabricated data. It does not verify native Windows/
macOS widgets, CLI quota, actual deletion, updates, model discovery, approval enforcement or Rust UI parity.
Revalidate when the behavior contract, CSS/JS hashes, palette, backend schema, or rendering platform changes.
Implementation acceptance remains open; the paused Rust goal is not resumed by design approval.
