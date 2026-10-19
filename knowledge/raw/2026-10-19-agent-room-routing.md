# Agent Room routing investigation

## Observation

During local macOS runtime acceptance, an approved run was started from the Workspace screen. The same native UI worker then transitioned into Agent Room. Task detail and candidate evidence requests were sent without the `room:true` discriminator and were routed into workspace setup parsing, producing an unknown `task` request error.

## Change and evidence

The shared RequestWorker now tags every packet while in Agent Room. The OpenRun response and ReturnToWorkspace transition synchronize the worker routing mode with the visible surface. A controller-backed test creates a completed three-task run, transitions a Workspace worker into Room mode, requests task 0, and checks the returned task identity.

The first TDD run failed because the transition setter did not exist. After implementation, all three request-routing tests passed. A previous UI fixture had already run three independent file-scoped tasks and reached human review, but runtime task-detail/evidence inspection exposed the lifecycle bug. No candidate was merged. Full tests, Clippy, final runtime verification, and platform parity remain tracked separately in `docs/07-implementation-plan.md`.

## Local environment notes

At inspection time `/private/tmp` was about 546 MiB and the Data volume had 48 GiB free; the earlier reported 41 GiB temporary-directory use was not present at that path. Only task-owned build wrappers, the isolated app bundle, and fixture repositories are candidates for cleanup. Other Issue #9 logs, screenshots, package evidence, and fixture directories were left untouched.

Graphify rebuilt the source graph locally without LLM clustering. The installed `cocoindex-code` command could not start because its Python environment lacks `mcp.server.fastmcp`; no dependency was installed during this run.
