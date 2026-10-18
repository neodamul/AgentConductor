# Native registration transaction audit — 2026-10-18

## Scope and trace

R-48 → BG-09 → SF-41 → UC-40 → FR-33 → QAS-41 → DD-56/DDR-13, native Rust adaptation. Preserve the shipped v0.1.24 custom `aipro` starting template, profile-only save, and unsaved workspace fields.

## Red evidence

- A real AccessKit pointer click on sidebar **Register CLI agent** changed to Team without opening the profile editor.
- The existing Save UI regression failed after setting `/tools/claude`: `Save must stage edits until persistence succeeds`; the registered profile had already changed before any response.
- The first native run showed the editor title absent and the fallback label literally `startup.no_fallback`.
- The original v0.1.24 `src/ui/startup.js` initializes a new profile with custom `aipro`. A regression failed when the native Add route selected the first (Claude) template.
- The first whole-target fixture run found the two new integration targets missing the repository's required `native-ui-probe` feature gate. Both target files now declare it; a complete rerun is in progress.

## Green implementation

Sidebar Add opens the same modal used for Edit, starts from the shared custom CLI template with a unique ID, and resolves its executable asynchronously. The Team page no longer contains a second, pre-saving add menu.

Editor owns unsaved registration and edit drafts. Saving builds a separate proposed registry and sends it with the observed revision. The live list and local coordinator/role references change only after successful persistence and refresh. Rejections keep the modal open and refresh the authoritative profiles/revision for retry. Cancel and Escape discard the draft. The profile-only request never includes workspace name or goal.

The editor now has a visible, localized registration heading; the fallback empty choice is localized in English (`None`) and Korean (`없음`).

## Verification

- Five egui UI-to-Room regressions cover Add, Cancel, persisted save/reopen, stale revision followed by retry, and coordinator/role renaming after persistence.
- Three editor regressions cover duplicate/capacity limits, provider switch then cancel, and concurrent removal of the edited profile.
- The existing Save/Cancel test now asserts the live profile remains byte-for-byte unchanged while persistence is pending.
- Registration-heading/empty-fallback render test covers English/Korean and Light/Dark.
- The focused follow-up log `/private/tmp/ac-registration-followup-tests.log` reports 21 passed across 8 targets; source-layout's 300-line guard passes. This run preceded the feature-gate-only fix.
- An isolated optimized macOS `ui-new` window at `/private/tmp/AgentConductor-Registration-QA.app` verified the real title, `custom` provider, `aipro` executable, English None label, Cancel, successful registration notice, first-agent coordinator, persisted second profile, and preserved unsaved workspace name/goal. Reading its disposable `startup.json` showed only saved name/goal (empty) and both intended profiles. Escape on the final binary closed the modal without adding a card. The QA window then exited; its temporary home was not the user's home.
- Optimized binary SHA-256: `7e50676aa023ff805e057f3c50564774ed7062d3120f7ab96312ed1e63e40bc9`. Its `--help` smoke exits successfully. It is an unpublished issue-branch binary, not a release artifact.
- All-target/all-feature Clippy and rustfmt passed before adding the two required target gate attributes; final full-suite rerun currently owns session 74683 and writes `/private/tmp/ac-registration-all-tests-final.log`.

## Limits

This corrects part of registry workflow 3. It does not demonstrate full replacement behavior or visual parity. Accepted Rust replacement progress remains 0/25 operator workflows; issue #9 stays open. Exact Windows UI/package behavior, Korean IME, full keyboard/screen-reader pass, complete screen comparison and final screenshots remain outstanding.
