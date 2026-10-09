# AgentConductor

A local-first Rust orchestrator for coding CLI agents, designed to coordinate
Claude Code, Codex, OpenCode, and custom command-line agents.

Use your existing CLI authentication. AgentConductor does not require a separate
LLM API key or an OpenAI-compatible endpoint configuration.

- Assign scoped tasks in a shared workspace or optional Git worktrees.
- Collect structured output, logs, code changes, and test evidence locally.
- Review an integration candidate before approving its merge.
- Connect custom CLI agents through the same process I/O contract.

**Status:** Early implementation. The complete orchestration workflow is not yet available.

## CLI commands

Build the current CLI with `cargo build --release`. These commands do not start agents:

```sh
./target/release/agentconductor validate plan.json
./target/release/agentconductor normalize codex stdout.jsonl
```

The command checks three tasks, registered agents, dependencies, and non-overlapping path
scopes. The workspace mode is `shared` by default; `worktree` is optional.

## Interface preview

The illustration below is a **design mockup**, not a screenshot of a working UI.
All task names, statuses, logs, and test results are illustrative.
The console concept includes task feedback, pause/resume, and explicit approval controls.

![AgentConductor interface concept showing three agent tasks, logs, a diff, and merge review](docs/assets/ui-preview.png)

### Alternative views

All previews below are illustrative design mockups with planned interaction controls.
Open an image for the full-size view, or [browse the interface gallery](docs/interface-gallery.md).

| Agent room | Dispatch desk | Review cockpit |
|---|---|---|
| [![Agent room with targeted feedback and an inspector](docs/assets/ui-agent-room.png)](docs/assets/ui-agent-room.png) | [![Dispatch desk with task editing and stage checkpoints](docs/assets/ui-dispatch-desk.png)](docs/assets/ui-dispatch-desk.png) | [![Review cockpit with inline feedback and candidate approval](docs/assets/ui-review-cockpit.png)](docs/assets/ui-review-cockpit.png) |
| Direct task feedback and progress reports | Goal, assignments, and checkpoints | Code evidence and approval controls |

## Runtime and distribution

The distribution target is a single native `agentconductor` executable, with
SQLite included. Git and the selected agent CLIs must be installed and authenticated
on the host. Optional memory and indexing integrations are separate services.

Choose a shared workspace or enable Git worktrees for separate code changes.
Execution permissions depend on the selected CLI and sandbox configuration.
