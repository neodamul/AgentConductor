# Changelog

Releases for macOS (Apple silicon) and Windows (x86_64). Stable versions start at
0.1.0 and continue with 0.1.1. Historical preview entries are retained. Each section is published as
the GitHub Release notes for its tag.

## v0.1.2

- Put MCP connection settings under the top-bar Settings menu; enlarge the gear and its pointer target.
- Fix Windows npm executable discovery and recover quota collection for existing extensionless profiles.
- Add a bounded, contained Windows ConPTY collector for Claude /usage; no model generation or automatic trust approval.
- Show original provider badges when desktop artwork is unavailable and repair legacy Windows icon alpha.

- Register OpenCode-shaped local and remote MCP servers in Workspace settings.
- Discover read-only tools and bind Jira/Confluence arguments to explicit goal references.
- Preview source text and provenance, approve selected evidence and retain immutable local snapshots for planning.
- Keep document bodies out of portable settings and shared memory; verify goal/settings/content digests before execution.
- Reuse one MCP session per server, bound responses/deadlines and close owned processes/sessions after each operation.
- Support authenticated local proxies and environment-backed remote headers; native OAuth browser authorization is not included.

## v0.1.1

### Added

- Goals become slugs with work folders; plans start with an analysis and run phase by phase (task → verification → completed); **Goals & progress** overview, **Plan next phase** and editable **Plan rules**.
- Slug folders live in `<repo>/.agentconductor/<slug>/` (or `~/.agentconductor/<workspace>/<slug>/` without a repository) and are kept out of `git status`.
- Repository published under the MIT license.
- Agent dialog: **Provider** comes first; switching provider replaces the Agent ID (unique suffix), executable path and options together and clears the old CLI's models. Renaming an agent keeps it as coordinator, in role rules and as fallback.
- **Settings** menu with interface language: English (default) or Korean; switches instantly and is remembered. UTF-8 is explicit everywhere (pages, release notes, Windows runners).
- Windows fixes found by the first real Windows CI run: release notes no longer fail under the cp1252 console encoding; executable lookup also searches `%APPDATA%\npm` and `%USERPROFILE%\.local\bin`.
- Agent dialog: choosing an alternate agent loads its models into **Alternate agent's model**, with an explanation of what the fallback does; the Settings button is a ⚙ icon.
- Exhausted quota shows when it returns, e.g. `0%/40% · 18:39`; the tooltip lists every window's reset time.

## v0.1.0

- Default quota refresh to five minutes, configurable per agent; update capacity silently in the background.
- Locate installed CLI executables automatically, provide native Browse, and standardize input control sizing.
- Use the project-owned AC mark for native application and executable identity.
- Clarify the three workspace steps and update actual native screenshots and the user manual.

### Added

- AgentConductor is published under the MIT license; `LICENSE` ships in every package.
- New workspaces start with an empty agent list; add the CLIs you use with **+ Add** (templates for Claude Code, Codex and OpenCode remain).
- Lower idle cost: windows refresh every 5 s when nothing is happening (1 s while work runs), pause while hidden, and no longer resend app icons every second (idle CPU ≈0.1 %).
- Workspace export now includes one `<slug>.zip` per goal with its progress; import verifies every archive before restoring.
- Quota collection in fresh workspaces, individual refresh icons and automatic two-minute
  refresh while idle; authenticated macOS Claude `/usage` metadata without generation.
- Compact Edit, Models reload and Remove icons in agent cards.

- Remaining quota beside paid agent names in the workspace and session lists, with
  window/source tooltips and explicit unavailable or stale states.

- Claude capacity in Agent Room: automatically collect provider-reported utilization and
  reset windows from owned task streams. **Check observed quota** preserves freshness
  without launching another prompt; absent utilization stays unavailable.

- When planning starts on a repository with uncommitted or untracked changes, a dialog now
  explains why a clean baseline is needed, lists the changed files and offers **Commit and
  continue** (one baseline commit on the current branch) or **Stop** (nothing changes).
- **Ignore and continue** in the same dialog for operating-system files (`.DS_Store`,
  `Thumbs.db`, `desktop.ini`, `._*`): excludes them locally without a commit, with a guide
  to ignore them in every repository.

- Planning progress: **Review the plan** shows the coordinator and elapsed time while a
  plan is generated, a *Plan ready* notice with scroll when done, and the exit status and
  log folder when planning fails.

- Task instructions are requested as readable *Goal / Inputs / Steps / Expected output /
  Done when* sections in the goal's language; the instructions box is twice as tall and
  grows with its content.

- **Agent roles** (step 2): decide per workspace which agent does verification,
  development, documentation, knowledge assetization and testing; the coordinator must
  follow these rules, and each planned task shows its role. Each role agent can be pinned
  to a model from its observed catalog; the engine applies that model to the task.
- The workspace screen is now numbered: **1. Workspace settings**, **2. Agent roles**,
  **3. Review the plan**.

- Agents added, edited or removed are saved immediately (other unsaved edits are kept
  as typed); removing an agent also updates the coordinator and role rules.

- Model lists load automatically once when a built-in agent is added; **Models** refreshes
  them. Lists update quietly in the role table without moving focus.
- Compact agent cards: provider, sessions, model count, permission capability and fallback
  at a glance.

- **Export settings… / Import settings…** in the Workspace menu: share name, coordinator,
  workspace mode, agents and role rules through `agentconductor.workspace.json` in the
  repository. No secrets, model lists, local paths, goals or plans are exported; imported
  sandbox claims must be re-confirmed on each machine.

### Fixed

- New workspaces showed no model lists; they now reuse lists already observed on this
  machine and fetch missing ones once in the background.
- Saving plan edits dropped each task's role.
- A generated plan was discarded when models were loaded or the workspace was saved with
  unrelated changes; it is now kept unless the goal, repository, workspace mode or an
  agent's execution settings change.

- Plans can have one to twelve tasks; **+ Add task** and **Remove task** in review.
- **Read plan…**: convert an existing Markdown plan document into tasks with the
  coordinator (read-only, role rules applied, completed items skipped).
- **Save workspace** moved into step 1.

### Changed

- **Open existing workspace…** now lists the workspaces saved in `~/.agentconductor`
  (including ones removed from the menu) instead of opening a folder dialog, so an
  arbitrary folder can no longer become a workspace by mistake.

## v0.1.0-preview.6

### Added

- **Write plan manually** — plan the three tasks yourself without any LLM coordinator; the
  draft uses the same validation and exact-digest approval as a generated plan.
- **Workspace name** above the goal, and a **Workspace** menu in the title bar: see every
  registered workspace by name, create one with **New workspace…** (name only), add an
  existing folder, switch in place, or remove an entry from the list (files are kept).
- **Workspace settings** section: goal first, then planning coordinator, then repository
  with a **Browse…** folder dialog next to the Git-worktree option.
- A workspace can be saved without a repository; only planning needs one.
- Release notes are now taken from this changelog.

### Fixed

- README and manual named the agent button "Register agent"; the button is **+ Add**.

### Known limitations

- Codex live re-verification waits for the account usage reset (2026-10-16).
- Claude account-capacity refresh is unavailable (no provider channel).

## v0.1.0-preview.5

### Added

- `memory-discover RUN PORT` finds and imports cooperative checkpoint reports from a local
  agentmemory server without memory IDs, scoped to each task's current address.

### Fixed

- Windows containment test no longer times out on a cold PowerShell start (test only).

## v0.1.0-preview.4

### Added

- A completed cooperative **handoff** now continues automatically on the agent's reviewed
  fallback once the relinquishing process has exited; without a fallback it stops as before.

### Verified

- Live Claude workers: an out-of-scope write is blocked before it happens; a rejected
  command never runs and that worker continues.
- Merge is refused when the target branch moved after approval.

## v0.1.0-preview.3

### Changed

- README rewritten as a short desktop-first introduction: what it does, how orchestration
  works, how roles are assigned, usage-limit handling and what you need.
- New user manual (`docs/user-manual.md`) with the desktop guide and every command; shipped
  in the package.

### Added

- Windows: provider icons are also found through Start Menu shortcuts.

## v0.1.0-preview.2

### Fixed

- Checksums and license notices are written with LF line endings on every platform, so
  `SHA256SUMS` verifies on macOS and Windows.

## v0.1.0-preview.1

### Added

- First pre-release: native desktop workspace and Agent Room for Claude Code, Codex,
  OpenCode and custom CLIs; three-task plans with exact approval; parallel sessions in a
  shared directory or Git worktrees; live tool approval; usage-limit waiting and reviewed
  fallback; reviewed lessons; macOS and Windows packages with licenses, SBOM and checksums.

- Compact agent quota groups show slash-separated percentages on one line beside their refresh icon; tooltips explain Enterprise Current/Total or provider duration labels.
