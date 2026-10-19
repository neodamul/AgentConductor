# Planning retry timeout and recovery

Verified: 2026-10-19. Scope: request-timeout handling in the shipped planning startup screen.

## Behavior

Plan startup can spend time saving workspace settings, checking whether a Git baseline is needed, and preparing the repository before the coordinator job is visible. These steps now use the existing five-minute planning bound; periodic snapshots use that bound while startup is active instead of the 15-second interactive default. Once the preparation chain ends, ordinary requests keep their shorter default.

Timeout guidance says that progress is checked automatically and advises reopening the current workspace or run if the visible state stays stale. It no longer asks operators to find a refresh control that the screen does not expose.

## Evidence and limits

The source regression asserts the planning bound reaches save, baseline checks/actions, and snapshots. Local tests cover timeout copy in both supported locales, previous-failure clearing, and new-job progress ownership. This is code-path evidence; no live slow filesystem or Windows native package was exercised for this issue.

Trace: R-152 → BG-107 → SF-139 → UC-142 → FR-137 → QAS-148 → DD-162. GitHub #36.
