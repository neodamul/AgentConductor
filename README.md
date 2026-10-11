# AgentConductor

**A desktop app that lets several coding agents — Claude Code, Codex, OpenCode or your
company's own CLI — work on one repository as a team, while you stay in control.**

You describe a goal. AgentConductor splits it into scoped tasks, gives each task to an
agent, runs them in parallel, tests the combined result and asks for your approval
before anything is merged. It runs entirely on your computer and uses the agent CLIs you
are already signed in to — no API keys, no cloud service.

![Agent Room: three agents finished their tasks; tests passed; waiting for your approval](docs/assets/ui-agent-room-native.jpg)

> **Status:** pre-release preview for macOS (Apple silicon) and Windows (x86_64).

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

1. The coordinator proposes three tasks — or you write them yourself. Each names an agent
   from the ones you registered, the files it may touch, what it depends on and how it is
   tested.
2. You edit the proposal: change which agent does which task, the files, or the tests.
3. You approve the exact plan. The engine starts each task in a separate session of its
   agent (up to three sessions per agent). Files never overlap between tasks, and a task
   waits until the tasks it depends on are done.
4. Later you can reassign any task to another agent; it continues from the work so far.

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
(currently Codex).

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
3. **Register agent** for each CLI you use. Templates fill in the details; choose how
   each one is sandboxed.
4. Choose the repository and write the goal, then **Generate plan** with a coordinator or
   **Write plan manually**.

   ![Workspace: goal, planning coordinator and the proposed plan](docs/assets/ui-startup-native.jpg)

5. Review the tasks, adjust them and press **Approve plan & run**.
6. Watch the Agent Room. Answer tool requests, send feedback, or reassign tasks.

   ![Session lights: waiting, failed and running sessions](docs/assets/ui-status-lights-native.jpg)

7. Open **Diff & tests**, **Approve** the candidate, then **Merge approved candidate**.

## Learn more

- [User manual](docs/user-manual.md) — every screen, setting and safety rule, plus the
  command-line reference for automation.
- [Design documents](docs/README.md) — requirements, use cases, quality attributes and
  design decisions.
