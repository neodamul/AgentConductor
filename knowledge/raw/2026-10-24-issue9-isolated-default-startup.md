# Issue #9 isolated default startup — 2026-10-24

Launched the source-built macOS bundle from commit `99cb31aa` with no arguments, a unique temporary bundle ID, and a disposable `HOME`. The native Workspace window appeared. Its accessibility tree showed the three planning steps, workspace fields and controls, an empty CLI-agent list, and the agent-registration prompt.

The temporary `HOME` isolated recent-workspace data, profiles, preferences, and custom CLI registry. No provider CLI was configured or invoked. No UI controls were activated. The process was stopped and the whole temporary bundle/HOME/workspace directory removed. This is startup-path evidence only, not a packaged workflow or proof for the production bundle identity. No screenshot file was retained.

The prior CUA lookup by shared production identity selected the older installed app and showed the user's profiles/workspace. No control was clicked; possible automatic quota refresh is unverified. Avoid that lookup. The UTM Windows guest still does not boot or display; do not operate it.

Evidence: [issue9-default-ui-isolated-startup-20261024.json](../../docs/evidence/issue9-default-ui-isolated-startup-20261024.json).
