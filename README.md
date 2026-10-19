# AgentConductor

**A desktop app that lets several coding agents — Claude Code, Codex, OpenCode or your
company's own CLI — work on one repository as a team, while you stay in control.**

You describe a goal. AgentConductor splits it into scoped tasks, runs agents in parallel,
tests the combined result and asks for your approval before merging. It runs on your computer and reuses signed-in agent CLIs — no API keys, no cloud service.

![Workspace settings and each agent’s account capacity](docs/assets/ui-workspace-current.png)

Choose **⚙ Settings → Interface language** for Korean. Open **Workspace ▾ → User manual** for offline English/Korean help.

![Workspace in Korean](docs/assets/ui-workspace-korean-current.png)
![Dark workspace in Korean](docs/assets/ui-workspace-dark-current.png)

> **Version:** 0.1.24 · macOS (Apple silicon) and Windows (x86_64).

## What it does

- **Workspaces** — create or switch while work continues in the background, and delete with local recovery; project files stay unchanged.
- **One goal, a team of agents** — scoped tasks have dependencies, tests and their own agent sessions.
- **Mix any agents** — Claude Code, Codex, OpenCode and company CLIs, with provider badges.
- **You approve every important step** — the plan and final merge; tool decisions are recorded.
- **Live team view** — reviewed task summaries and ID labels, provider icons, status lights, logs, diffs, tests and a briefing.
- **Light or dark** — choose a remembered theme in Settings; both show the same information.
- **Usage limits handled** — durable resume reservations survive app closure; waits beyond five hours use only a reviewed fallback, and Stop or reassignment cancels the old reservation.
- **Steer while it runs** — send feedback to one task, reassign it, change its model,
  stop and resume.
- **Learns from reviewed lessons** — lessons you accept are reused in later tasks and
  dropped automatically when the code they describe changes.
- **Import goal sources** — configure company MCP servers, preview Jira stories and
  Confluence pages, then apply selected evidence before planning. [MCP guide](docs/mcp-sources.md).
- **Responsive configuration** — register and edit agents while model or quota queries run in a bounded background queue; unrelated controls stay usable.
- **Model capacity** — temporary overload waits about a minute and retries up to three times.
- **Local and private** — one native executable with SQLite inside; data stays local.
- **Application updates** — check from the Workspace menu; install a checksum-verified release and restart when work is idle. Private-release access currently requires manual download.

## How orchestration works

AgentConductor has two parts with different jobs:

| | Who does it | What it decides |
|---|---|---|
| **Engine** | AgentConductor itself (built-in program, no LLM) | Starts and stops agent sessions, enforces file scopes and dependencies, runs tests, merges results into one candidate, handles usage limits, records every step. |
| **Coordinator** | An agent you pick (e.g. Codex or Claude) | Proposes the plan, reviews results and decides eligible tool requests outside the workspace policy. Final merge stays human-approved. |

The coordinator is optional: **Write plan manually** uses no LLM for planning. The engine runs, checks and merges work; final integration requires your approval.

CLI launchers expose execution sessions; Rust routes their context and results, keeps state in SQLite, and supports directions at cooperative checkpoints.
Reviewed knowledge stays in a local Markdown wiki; agentmemory is optional for sharing context and lessons.

## How roles are assigned

1. Set **Agent roles** per workspace: Claude for planning/verification, Codex for development/documentation/knowledge, and OpenCode or a company agent for testing.
2. The coordinator proposes tasks (one to twelve) following those rules — or you write them
   yourself, or let it convert an existing plan document with **Read plan…**.
   The CLI receives a structured output contract; each task has a role, agent, file scope, dependencies and test.
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
(Codex and Claude Code). Refresh is asynchronous, defaults to five minutes,
and can be changed independently in each agent’s Edit settings. Claude automatic refresh waits for a saved project folder and is retried after the saved folder changes. Queries do not block editing. Enterprise shows reported remaining/allocated capacity; an unavailable total stays unknown.

## What you need

- macOS 11+ (Apple silicon) or Windows (x86_64).
- Git for change history. Your project can be an ordinary folder; no repository or initial commit is required. Shared execution is default; worktrees are optional.
- At least one agent CLI installed and **already signed in** (Claude Code, Codex,
  OpenCode or a company CLI). Two or more give you a real team and a fallback.
- **agentmemory is optional.** Local history and lessons work without a memory server.

## Getting started

1. Download the macOS or Windows archive from [GitHub Releases](https://github.com/neodamul/AgentConductor/releases), check it
   against `SHA256SUMS` and extract it.
2. Open **AgentConductor**. The workspace window appears.
3. Press **+ Add** under *CLI agents* for each CLI you use. Templates fill in the details;
   standard Claude/Codex profiles use full access; custom CLIs handle their own tool approval unless explicitly restricted.
4. Name the workspace and write the goal. Under **Workspace settings** pick a coordinator
   and **Browse…** for an existing project folder, then **Save workspace**.
   Use **New goal** for the next feature in the same project; **Select goal** reviews earlier work. Switch between **Workspace settings**, **Agent roles** and **Review the plan** tabs.
   Choose **Generate plan** or **Write plan manually**. Ordinary folders use a snapshot for planning and work directly in the selected folder during shared execution, with history stored outside the project.
   Review diffs and tests, approve, then **Accept changes**; Git projects use **Merge**.
   The **Workspace** menu in the title bar creates and switches between named workspaces
   and offers adjacent **Export workspace** / **Import workspace** actions for reviewed sharing.

   ![Agent settings: automatically located executable and quota refresh minutes](docs/assets/ui-agent-settings-current.png)

5. Saving the workspace opens **Agent roles**. Review the tasks and press **Approve plan & run**.
6. Switch freely between Workspace and Agent Room; live progress continues. Answer tool requests, send feedback, or reassign tasks.
   ![Agent Room showing a completed demonstration run and passing verification](docs/assets/ui-agent-room-native.jpg)
7. Review **Diff & tests**; use the CLI for a full large patch, then approve and merge or accept the exact candidate.

## Learn more

![Offline HTML manual](docs/assets/ui-manual-current.png)

- Manuals: [English](docs/user-manual.md) / [한국어](docs/user-manual.ko.md); offline HTML: [English](docs/user-manual.html) / [한국어](docs/user-manual.ko.html).
- [Design documents](docs/README.md) / [UI design guide](docs/design-guide.md) — requirements, use cases and consistent visual rules.

**License:** MIT — see [LICENSE](LICENSE).
