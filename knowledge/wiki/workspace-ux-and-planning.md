# Workspace UX, role rules and planning inputs (DD-45 … DD-60)

Sources: `src/ui/setup/*.rs`, `src/ui/workspaces.rs`, `src/ui/picker.rs`, `src/roles.rs`,
`src/planner.rs`, `src/ui/startup.{html,js,css}`, tests `manual_plan`, `workspace_*`,
`baseline_commit`, `planning_progress`, `plan_readability`, `role_rules`,
`profile_persistence`, `model_catalogs`, `workspace_share`, `plan_size`.

- **Flow:** numbered cards — 1 Workspace settings (name, goal, coordinator, repository with
  Browse, worktrees, Save), 2 Agent roles (role → agent + model, Suggest defaults; Read plan,
  Write plan manually, Generate plan), 3 Review the plan (1–12 tasks, add/remove).
- **Authority stays human:** role rules constrain the coordinator (`roles::apply` rejects
  violations and pins models); approval and merge remain human; imported sandbox claims are
  downgraded unless re-confirmed.
- **Persistence rules:** agents save immediately (`save_profiles`); catalogs survive saves
  (`carry_over`); a plan survives changes that do not alter its execution view
  (`execution_view` = profiles without catalogs) — learned after a plan vanished when
  models loaded.
- **Planning inputs:** goal, role rules (`roles.json`), optional plan document
  (`--from plan.md`, untrusted, ≤64 KiB). Progress shows coordinator and elapsed time from
  the job's `started_ms`.
- **Lessons:** never restart the operator's window without asking (unsaved edits were
  lost); UI tests that pin layout belong next to each user correction; re-render only on
  structural change to avoid focus jumps.
