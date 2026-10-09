# AgentConductor

A local-first Rust orchestrator for coding CLI agents, designed to coordinate
Claude Code, Codex, OpenCode, and custom command-line agents.

Use your existing CLI authentication. AgentConductor does not require a separate
LLM API key or an OpenAI-compatible endpoint configuration.

- Assign scoped tasks to agents in separate Git worktrees.
- Collect structured output, logs, code changes, and test evidence locally.
- Review an integration candidate before approving its merge.
- Connect custom CLI agents through the same process I/O contract.

**Status:** Early implementation. The complete orchestration workflow is not yet available.

## Interface preview

The illustration below is a **design mockup**, not a screenshot of a working UI.
All task names, statuses, logs, and test results are illustrative.

![AgentConductor interface concept showing three agent tasks, logs, a diff, and merge review](docs/assets/ui-preview.png)

## Runtime and distribution

The distribution target is a single native `agentconductor` executable, with
SQLite included. Git and the selected agent CLIs must be installed and authenticated
on the host. Optional memory and indexing integrations are separate services.

Git worktrees separate code changes; execution permissions depend on the selected
CLI and sandbox configuration.
