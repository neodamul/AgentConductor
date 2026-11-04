# Workspace and Goal model audit

Source: GitHub #411, R-213 / DD-225. Audit existing behavior before replacing it:

- English internal folder IDs use semantic coordinator output for non-English goals,
  with a date placeholder beforehand, never phonetic transliteration.
- Existing folder IDs/paths survive Workspace and Goal display edits.
- New Goal clears Goal-only context while retaining folder/coordinator/profiles/roles.
- Existing next_phase keeps the Goal; ordinary user projects need no `.git`, but the
  private external history implementation still requires a Git executable.

The native audit found that explicit create_slug could claim an unrelated existing
folder or follow a link and write its Goal marker. This defeats later deletion checks.
Reserve a new direct-child root with exclusive create_dir before adding any marker or
Git exclusion. Do not call the legacy migrating root resolver while allocating a new
Goal. Treat dangling links as occupied during automatic naming. Imported/legacy registered
Goals retain their existing migration paths; this correction only changes new allocation.

Tests: workspace_goal_rules, startup_folder_plan, folder_workflow, slug_flow, slug_folder,
slugs, architecture_trace (24 tests, seven local suites). UI-only modal geometry/focus,
step-6 continuity and Windows package/reparse checks remain separate acceptance.
