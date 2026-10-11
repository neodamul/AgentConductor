# Changelog

Pre-releases for macOS (Apple silicon) and Windows (x86_64). Each section is published as
the GitHub Release notes for its tag.

## v0.1.0-preview.7

### Added

- When planning starts on a repository with uncommitted or untracked changes, a dialog now
  explains why a clean baseline is needed, lists the changed files and offers **Commit and
  continue** (one baseline commit on the current branch) or **Stop** (nothing changes).
- **Ignore and continue** in the same dialog for operating-system files (`.DS_Store`,
  `Thumbs.db`, `desktop.ini`, `._*`): excludes them locally without a commit, with a guide
  to ignore them in every repository.

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
