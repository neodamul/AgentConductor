# AgentConductor

**A desktop app that lets several coding agents — Claude Code, Codex, OpenCode or your
company's own CLI — work on one repository as a team, while you stay in control.**

You describe a goal. AgentConductor splits it into scoped tasks, gives each task to an
agent, runs them in parallel, tests the combined result and asks for your approval
before anything is merged. It runs entirely on your computer and uses the agent CLIs you
are already signed in to — no API keys, no cloud service.

![Workspace settings and each agent’s account capacity](docs/assets/ui-workspace-current.png)

The interface is in English by default; choose **⚙ Settings → Interface language** for Korean.

![Workspace in Korean](docs/assets/workspace-korean.jpg)

> **Version:** 0.1.2 · macOS (Apple silicon) and Windows (x86_64).

## What it does

- **One goal, a team of agents** — work is split into tasks with their own files,
  dependencies and tests, and each task runs in its own agent session.
- **Mix any agents** — Claude Code, Codex, OpenCode and custom/company CLIs side by side,
  shown with their installed app icons.
- **You approve every important step** — the plan, risky tool calls and the final merge.
- **Live team view** — status lights, activity, logs, diffs, test results and a one-click
  team briefing.
- **Usage limits handled** — when an account hits its hourly or weekly limit, its work
  waits for the reset or moves to a fallback agent you chose.
- **Steer while it runs** — send feedback to one task, reassign it, change its model,
  stop and resume.
- **Learns from reviewed lessons** — lessons you accept are reused in later tasks and
  dropped automatically when the code they describe changes.
- **Import goal sources** — configure company MCP servers, preview Jira stories and
  Confluence pages, then apply selected evidence before planning. [MCP guide](docs/mcp-sources.md).
- **Local and private** — one native executable with SQLite inside; data stays local.

## How orchestration works

AgentConductor has two parts with different jobs:

| | Who does it | What it decides |
|---|---|---|
| **Engine** | AgentConductor itself (built-in program, no LLM) | Starts and stops agent sessions, enforces file scopes and dependencies, runs tests, merges results into one candidate, handles usage limits, records every step. |
| **Coordinator** | An agent you pick (e.g. Codex or Claude) | Only *proposes*: reads a read-only copy of the repository and suggests the task split. It can also review results. It never edits code or approves anything. |

The coordinator is optional: choose **Write plan manually** and split the work yourself —
no LLM is involved in planning at all. Everything that actually runs, checks and merges is
done by the engine, and every decision with consequences is yours.

## How roles are assigned

1. You set **Agent roles** once per workspace — for example Claude for planning and
   verification, Codex for development, documentation and knowledge, OpenCode or a company
   agent for testing.
2. The coordinator proposes tasks (one to twelve) following those rules — or you write them
   yourself, or let it convert an existing plan document with **Read plan…**.
   Each has a role, an allowed agent, the files it may touch, its dependencies and its test.
3. You edit the proposal: change which agent does which task, the files, or the tests.
4. You approve the exact plan. The engine starts each task in a separate session of its
   agent (up to three sessions per agent). Files never overlap between tasks, and a task
   waits until the tasks it depends on are done.
5. Later you can reassign any task to another agent; it continues from the work so far.

## When an account hits its usage limit

Agents that share one subscription are put in the same **quota group**, so a limit
pauses all of them together. When a CLI reports a limit (hourly, weekly or any other):

- **If you set a fallback agent** (for example a company CLI on a different account),
  the work moves there automatically, keeping partial files and logs.
- **Otherwise** the session waits and resumes at the reset time the provider reported,
  or at a time you set if the reset time is unknown.
- You can always reassign the task by hand. Results are tested and need your approval
  again after any move.

**Account capacity** shows remaining usage and reset time where the CLI reports it
(Codex and Claude Code on macOS). Refresh is asynchronous, defaults to five minutes,
and can be changed independently in each agent’s Edit settings.

## What you need

- macOS 11+ (Apple silicon) or Windows (x86_64; WebView2 Runtime, included in current
  Windows).
- Git, and a Git repository with no uncommitted changes.
- At least one agent CLI installed and **already signed in** (Claude Code, Codex,
  OpenCode or a company CLI). Two or more give you a real team and a fallback.
- **agentmemory is optional.** AgentConductor keeps its own local history and lessons;
  it can share reviewed lessons with a local agentmemory server if you ask it to.

## Getting started

1. Download the macOS or Windows archive from
   [GitHub Releases](https://github.com/neodamul/AgentConductor/releases), check it
   against `SHA256SUMS` and extract it.
2. Open **AgentConductor**. The workspace window appears.
3. Press **+ Add** under *CLI agents* for each CLI you use. Templates fill in the details;
   choose how each one is sandboxed.
4. Name the workspace and write the goal. Under **Workspace settings** pick a coordinator
   and **Browse…** for the repository, then **Generate plan** or **Write plan manually**.
   The **Workspace** menu in the title bar creates and switches between named workspaces
   and exports/imports settings you can commit to Git for your team.

   ![Agent settings: automatically located executable and quota refresh minutes](docs/assets/ui-agent-settings-current.png)

5. Review the tasks, adjust them and press **Approve plan & run**.
6. Watch the Agent Room. Answer tool requests, send feedback, or reassign tasks.
7. Open **Diff & tests**, **Approve** the candidate, then **Merge approved candidate**.

## Learn more

- [User manual](docs/user-manual.md) — every screen, setting and safety rule, plus the
  command-line reference for automation.
- [Design documents](docs/README.md) — requirements, use cases, quality attributes and
  design decisions.

## License

MIT — see [LICENSE](LICENSE).
