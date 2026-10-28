# Native tool approval responsiveness

Verified: 2026-10-29. Status: local regression passes; exact Windows package rerun pending.

## Invariant

Do not start background task-detail or coordinator-judgment reads while a displayed
tool request is pending or approved. The native request worker serializes these
reads with human actions, and the UI disables its controls while a request is in
flight. Let the operator approve, deny, or refresh the exact displayed request
before loading optional details.

## Evidence

The Windows package job in run
[`40751085821`](https://github.com/neodamul/AgentConductor/actions/runs/40751085821)
passed preceding UI journeys, then timed out waiting for the pending request's
Allow button. Its UI Automation tree showed both `Loading task details…` and the
pending Bash request while the room remained in `Running`. The regression
`pending_tool_requests_skip_background_room_detail_loading` failed before the
guard and passes after it. The exact Windows package rerun is required before
claiming end-to-end acceptance.

## Revalidate when

The request worker becomes concurrent or the native room changes how background
reads and operator actions are scheduled.
