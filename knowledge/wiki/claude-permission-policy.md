# Claude effective permission policy

Issue: #931. Trace: R-122 / BG-77 / SF-109 / UC-112 / FR-107 / QAS-118 / DD-131.

A configured full-access argv expresses intent, not effective access. Managed organization settings can disable bypass. A worker first sends stream-json control initialization in its existing child process and forwards the user prompt only when `current_permission_mode` equals `bypassPermissions`. Unknown or different modes block without a generation request. System init/status metadata continues checking the running session.

The asynchronous Models initialization reuses that no-generation metadata channel, storing effective mode, timestamp and an execution-profile fingerprint. Cached evidence is a configuration hint, never runtime permission. A user can explicitly select native callbacks in the advanced profile editor. Unsupported callback arguments reject saving rather than silently changing to automatic approval.

A nonempty or malformed `permission_denials` result is an error even with exit code zero and `is_error:false`. The task blocks, retains partial files/logs, and does not enter automatic repair or usage-limit fallback. Human reassignment remains possible. Coordinator calls use restricted argv rather than the worker's full-access request: their decoder must reject denials without requiring worker bypass mode.

## Evidence and lessons

- [Ad-hoc matrix](../../docs/evidence/931-permission-ad-hoc.md) separates subprocess/UI fixtures from installed-CLI and physical-platform evidence.
- [Claude SDK types](https://github.com/anthropics/claude-agent-sdk-python/blob/main/src/claude_agent_sdk/types.py) expose system permission mode and result permission denials.
- [Managed settings](https://code.claude.com/docs/en/settings) govern organization restrictions.
- A local initialize-only CLI probe confirmed `current_permission_mode`; no user prompt was sent.
- Test actual invocation context: a worker permission profile is not the coordinator's restricted execution profile.


## Read-only coordinator correction (#942)

The strict worker rule is not a blanket coordinator rule. A planner may first probe an original absolute project path, be denied, then read its allowed exported snapshot and emit valid structured output. Applying the worker rule to that recovered proposal was a release-blocking regression in 0.2.8.

Coordinator decoding now bounds and matches read tool calls/results, requires observed failures matching denial IDs and later successful reads of the denied tool kinds, rejects late/unresolved/write/malformed or incomplete-history denials, and still validates the requested final JSON. Worker decoding remains strict in every permission mode. Raw events retain the denial evidence. Planning prompts explicitly require snapshot-relative paths. The real local failure replay recovered four phases and ten tasks with role/plan/check/phase validation, without another generation request or user workspace write.
