# Windows workspace lifecycle

Sources: [UC-77–UC-80](../../docs/04-usecases.md), [DD-95–DD-98](../../docs/06-design-decisions.md), [manual](../../docs/user-manual.md).

- Canonical Windows paths carry an extended prefix. Present project paths through the existing drive/UNC conversion, preserving filesystem identity and root separators.
- A metadata collection is independent work. Workspace creation waits asynchronously for admitted collection before switching; Enter explicitly submits Create, rather than the first dialog button.
- Deletion is a confirmed rename of app-owned workspace data to the local recovery directory. A stable navigation lease prevents opening during the rename; active/unknown work and project overlap are refused.
- MCP registration names the server capability. Document-source bindings are optional and remain separate from registration.

Verified lesson: a historical successfully reaped worker PID can be reused by an unrelated system service. Startup navigation must distinguish terminal-success evidence from live/unknown ownership rather than block a new workspace or kill the unrelated PID. The current-PID regression and native creation/deletion flow cover this case.

Profile-save follow-up: await native persistence inside the handler, route errors into the open profile modal, and close only after success. Native Claude-to-Codex save and rejected cyclic fallback were verified in an isolated workspace. Reference: [DD-99](../../docs/06-design-decisions.md).

Quota follow-up ([PR #1](https://github.com/neodamul/AgentConductor/pull/1)): reject an empty project path before launching Claude, retain all-model and named-model weekly windows separately, and explain unsupported Codex API-key quota observations. The existing CLI remains the authentication boundary.
