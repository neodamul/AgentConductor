# Agent Room routing investigation

## Observation

During local macOS runtime acceptance, an approved run was started from the Workspace screen. The same native UI worker then transitioned into Agent Room. Task detail and candidate evidence requests were sent without the `room:true` discriminator and were routed into workspace setup parsing, producing an unknown `task` request error.

## Change and evidence

The shared RequestWorker now tags every packet while in Agent Room. The OpenRun response and ReturnToWorkspace transition synchronize the worker routing mode with the visible surface. A controller-backed test creates a completed three-task run, transitions a Workspace worker into Room mode, requests task 0, and checks the returned task identity.

The first TDD run failed because the transition setter did not exist. After implementation, all three request-routing tests passed. The full native-feature suite, doctests, all-target Clippy with warnings denied, Rust formatting, and the native app build also passed. A fresh UI fixture then completed three independent file-scoped tasks, loaded task activity, and displayed the candidate diff with three passing `git diff --check` results. The candidate remains unmerged. This slice does not change full parity acceptance: it remains 0/25; Windows, IME/accessibility, packaged binaries, and the other workflows stay open in `docs/07-implementation-plan.md`.

## Local environment notes

At inspection time `/private/tmp` was about 546 MiB and the Data volume had 48 GiB free; the earlier reported 41 GiB temporary-directory use was not present at that path. Only task-owned build wrappers, the isolated app bundle, and fixture repositories are candidates for cleanup. Other Issue #9 logs, screenshots, package evidence, and fixture directories were left untouched.

Graphify rebuilt the source graph locally without LLM clustering (6,716 nodes, 11,651 edges). The installed `cocoindex-code` command could not start because its Python environment lacks `mcp.server.fastmcp`; no dependency was installed during this run.
