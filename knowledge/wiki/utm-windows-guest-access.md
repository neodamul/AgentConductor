# UTM Windows guest access and shared drives

## Verified state — 2026-10-24

After explicit user approval, the existing Windows VM was reset and returned to `started`. `utmctl ip-address` reported `192.168.64.6` plus IPv6 addresses. One read-only guest-agent pull of `C:\Windows\win.ini` succeeded. The UTM display later showed `Display output is not active`.

A guest-agent command wrote the output of `dir Z:\` to a temporary guest file; the retrieved localized bytes were undecodable and consistent with a missing-drive error. This does not establish whether the user's interactive Windows session still has its mapped `Z:` drive. A later `query user` / `net use` probe and file pull did not return and were interrupted. The share configuration was not changed, and no subsequent VM power action was taken. No Windows build, package, UI or workflow acceptance is claimed.

## Interpretation and next verification

UTM documents guest tools as the route for Windows guest integration and says SPICE WebDAV directory sharing appears as a network drive; if that WebDAV drive is absent, UTM documents `C:\Program Files\SPICE webdavd\map-drive.bat`. This documentation does not prove that the user's existing `Z:` mapping uses SPICE WebDAV. Preserve that mapping unless the user authorizes a change.

The guest-agent path and the user's interactive logon are distinct verification surfaces. First restore a usable VM display or user-session control; then inspect `Z:` in the signed-in desktop and run the exact PR-head Windows validation. Do not infer user-session drive state from a service-context lookup.

Evidence: `../../docs/evidence/issue9-utm-windows-recheck-20261024.json`. The unresolved gate is listed in `../../docs/07-implementation-plan.md` and `../../docs/09-completion-audit.md`.

Reference: [UTM Windows guest support](https://docs.getutm.app/guest-support/windows/).


## Approved reset follow-up — 2026-10-24

The user approved another UTM restart after the reset warning was shown. The restart control was confirmed; no share settings changed. The VM remains started with `Display output is not active`; Ctrl+Alt+Delete did not wake the guest display. `utmctl` status/IP calls failed to return in a short bounded check and were interrupted. No further power action was taken. The interactive `Z:` mapping remains unknown. Do not infer its state from service-context guest probes.

The package evidence commit `868863a45539bce528088a3c109176fe3547d12e` is pushed. Windows Actions run `40001524380` remains stalled in build; new head run `40003663262` is queued. Windows UI/package acceptance remains open. Evidence: `../../docs/evidence/issue9-utm-approved-reset-followup-20261024.json`.
