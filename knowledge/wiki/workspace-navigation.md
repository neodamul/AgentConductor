# Workspace back navigation

## Sidebar orientation and agent registration — issue #9 / R-17 / DD-137

Keep the agent rail's localized `YOUR TEAM` eyebrow above `CLI agents`, with the localized hint
that registration uses an executable and authentication remains with that CLI directly below.
Reuse the existing English/Korean message keys. These labels orient the user before the agent
cards; they do not change registration behavior, card controls, or agent state. The Rust rendered
regression checks both labels in both languages, and the source/test files remain within the
300-line limit. This focused evidence does not establish full application parity.

## Three-step Workspace navigation and action placement — issue #9 / R-17 / DD-137

The current Rust Workspace uses three equal-width, 50px numbered step buttons. The selected step
has an accent underline and a tinted surface. Keep the selected stage in the page eyebrow and its
card eyebrow, and keep each step button keyboard/screen-reader accessible. The Workspace save and
Agent Roles save actions align to the right edge; the primary action uses the shared light/dark
accent pair. The implementation uses native egui controls and painting without a web view or new
dependency. Rendered tests cover English/Korean layout at 480px and 836px, action positions, and
4.5:1 button-text contrast in both themes. These component tests do not establish packaged legacy
UI parity or add to the 6/25 packaged end-to-end acceptance count.

Trace: R-33 → BG-09 → SF-28 → UC-25 → FR-19/NFR-10 → QAS-26 → DDR-07 → DD-39.

## Problem (operator report, 2026-10-10)

The workspace screen had no way to leave without saving. "Open run" sat at the bottom of
the plan review, edits could not be dropped, and saving the workspace cleared the proposal
that also linked the started run, so the Agent Room became unreachable.

## Contract

The titlebar shows "Back to run" (when a run is available) and "Discard changes" (while
edits are unsaved). Leaving with edits opens a confirmation listing the unsaved areas; focus
starts on "Keep editing". Discard rebuilds the view from the saved configuration and stored
proposal without any native request. The workspace stores `last_run` separately from the
editable proposal; it is offered only inside the workspace directory with a run report.

## Evidence

Red: titlebar/dialog contract and post-save `run_available` assertions failed. Green after
the controls, `last_run` and sticky legible titlebar. Native acceptance discarded a Korean
goal edit and left `startup.json` byte-identical. Full regression 208 passed, 0 failed.

## Source fingerprints

| Source | SHA-256 |
| --- | --- |
| `src/ui/startup.html` | `29f34dd637ba2b288163635dcd2c0612b46cc9e8c89f1768622847c5158167f9` |
| `src/ui/startup.js` | `99a69c1a70e188c12b25fa5d447bb42412bb9a0b7cd252a696d9e2345315b4ba` |
| `src/ui/startup.css` | `2846acee652b621ef2962121c10f18d3acddd01ce6e0339cfd92f15345aaf3b3` |
| `src/ui/setup.rs` | `ebb0addb4d178c3a6009d621e1dffdee82b16916e1eb685fedf436aa9734b3dd` |
| `tests/workspace_navigation.rs` | `9ee7ee2d21563f211130bf68484758299868f7a3bfaa7884fb14be0773ae1a79` |
| `tests/startup.rs` | `3b420b89294c1403691e0e28f1b7c7bf11ad5cf5124150051ab863f156d1b39e` |
