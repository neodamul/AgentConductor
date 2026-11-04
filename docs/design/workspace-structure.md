# Workspace and Goal sections

Trace: R-210 / UC-202 / QAS-209 / DD-222, GitHub #427 and #435.

The approved mock is the authority. Keep its step eyebrow, heading/description, Workspace
section and Goal section distinct. Workspace owns the working directory, coordinator and
optional worktrees; Goal owns the editable objective, selection context, imports, optimize,
Goal list/count, Team briefing and New Goal actions. Keep every existing action and ID.

The same Goal list button stays reachable across preparation steps and the Room. Returning
to step one restores it to the Goal section. A Team briefing opens the existing detailed
slide-over without opening a run. On a cold Workspace the native read-only request resolves
only its saved run under the existing Workspace ownership boundary; no report disables the
button, and the server also rejects the request. Release the settings lock before reading
the report, never advance metadata/jobs from a read-only briefing. Reuse the existing run
briefing collector/renderer; no extra polling, LLM call or dependency.

Keep the approved input/section styles and natural control box model. Labels and notes use
the existing English/Korean catalogs and type scale. Variable fixture names are normalized
only in visual tests, never in product state. Tests cover section ownership, Goal actions,
step/Room restoration and complete briefing contents, including absent-run denial and an
unchanged workflow revision. Compare changed surfaces in light and dark without changing
the approved mock or widening tolerances. Document titlebar actions without backend support
as explicit remaining scope rather than inert controls.

## Titlebar and application menu

The brand column follows the rail width and contains icon, title, version and divider.
The action column starts with the hamburger menu, rail toggle and Workspace name.
The application menu reuses existing commands and adds existing Settings/manual/tool
guide/cache controls; the Workspace chooser retains recent entries. Escape restores
the actual opener, and opening Settings from the Room does not leave the run view.
Workspace-changing commands still use the existing home/unsaved-edit guards.

The update button remains conditional on an actual available update. The task inspector
is backed by a run controller and remains hidden before a run; no pre-run data or inert
action is invented. Native Windows package acceptance is tracked separately.
