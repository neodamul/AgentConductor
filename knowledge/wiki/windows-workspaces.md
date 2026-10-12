# Windows workspace lifecycle

Sources: [UC-77–UC-80](../../docs/04-usecases.md), [DD-95–DD-98](../../docs/06-design-decisions.md), [manual](../../docs/user-manual.md).

- Canonical Windows paths carry an extended prefix. Present project paths through the existing drive/UNC conversion, preserving filesystem identity and root separators.
- A metadata collection is independent work. Workspace creation waits asynchronously for admitted collection before switching; Enter explicitly submits Create, rather than the first dialog button.
- Deletion is a confirmed rename of app-owned workspace data to the local recovery directory. A stable navigation lease prevents opening during the rename; active/unknown work and project overlap are refused.
- MCP registration names the server capability. Document-source bindings are optional and remain separate from registration.
