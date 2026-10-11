# Provider-driven agent identity and quota reset time

Trace: R-68, R-69 → BG-26 → SF-56/57 → UC-56/57 → FR-52/53, NFR-26 → QAS-61/62 → DD-74/75.

## What
- The agent dialog lists **Provider** before **Agent ID**. A provider change replaces every template-derived field (ID via `uniqueId(base, others)`, executable resolved with `resolve_executable`, args, input, permission, sessions, refresh minutes) and drops `catalog`/`quota_group` on save.
- `save_profiles` takes `renames {old: new}`; coordinator, role entries and fallbacks follow the new ID. Fallbacks to removed agents are cleared before validation (previously the save failed with `unknown fallback agent`).
- `quotaBadge` (startup.js and room.js, kept identical) appends ` · HH:MM` (or `M/D HH:MM`) for the latest reset among exhausted windows, ` · reset unknown` without one; tooltip shows `resets …` per window.

## Lessons
- Changing an ID is a reference migration, not a field edit: carry an explicit rename map instead of guessing from list order.
- Reset times were already in cached observations — showing them cost no extra polling (resource budget, see [resource-budget](resource-budget.md)).
- The two `quotaBadge` copies are tested by extracting the one-line function and running it in `node`; keep the function on one line.
