# Idle polling and suspended-page timeouts

## Decision

Scheduled snapshot polling is background observation. Its request timeout must not create an error toast by itself. The workspace and Agent Room keep showing the current connection state, while explicit refreshes and operator actions continue to report errors.

Page request deadlines count visible active time. They pause while the page is hidden, resume with the remaining budget when it becomes visible, and restart rather than fail immediately when a callback arrives materially late after suspension. Native requests are never retried automatically; this preserves single-shot mutation semantics.

## Implementation locations

- `src/ui/startup.js`: workspace request bridge, background poll flag, scheduler and visibility resume.
- `src/ui/room.js`: Agent Room request bridge, background poll flag, scheduler and visibility resume.
- `tests/idle_poll_timeout_runtime.cjs`: deterministic timeout, poll intent and feedback regression.
- `tests/test_idle_poll_timeout.py`: repository test runner entry.

## Trace

R-184 → BG-139 → SF-172 → UC-175 → FR-176/NFR-75 → QAS-182 → DDR-39 → DD-196. GitHub issue #221.
