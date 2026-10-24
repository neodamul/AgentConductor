# Native Agent Room request routing

Agent Room and Workspace share one native `Probe` in the common workflow where approval starts a run. The window can transition from Workspace to Agent Room without creating another `RequestWorker`. Routing must therefore follow the active UI surface, not just the surface that existed when the worker was first constructed.

The worker adds `room:true` at the shared send boundary for all Agent Room requests. The OpenRun transition enables this mode, and ReturnToWorkspace disables it. `Room::dispatch` removes the marker before parsing the typed task/evidence/controller request. Workspace packets remain unchanged.

The regression test must exercise a controller-backed request through `RequestWorker` after the Workspace-to-Room transition. Testing only the JSON marker helper misses this lifecycle condition. The task and evidence controls use the same dispatch boundary, so a single routing fix covers both.

See [the source observation](../raw/2026-10-19-agent-room-routing.md) and [implementation checklist](../../docs/07-implementation-plan.md).
