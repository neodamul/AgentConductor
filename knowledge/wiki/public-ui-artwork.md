# Public UI artwork: design previews and runtime captures

Verified: 2026-11-11 · Source commit: `c908a65bc4185074fa18116b6d6648b3c273fc12`.

## Sources

- `README.md`, `docs/user-manual.md`, `docs/user-manual.ko.md`, generated offline HTML manuals.
- Approved design contract: `docs/design/approved-goal-ui/README.md` and its preview images.
- Public mockup copies: `docs/assets/ui-approved-workspace-design.png`, `ui-approved-tasks-light.png`, `ui-approved-tasks-dark.png`.
- Runtime screenshots: `docs/assets/ui-workspace-current.png`, `ui-workspace-korean-current.png`, `ui-workspace-dark-current.png`.
- About mark: `src/ui/about.html` and canonical `src/ui/assets/ac-64.png`.
- Regressions: `tests/about_runtime.cjs` and `tests/ui_live_bridge.rs`.

## Verified facts

- README and both manuals label the approved Goal UI mockups as design previews with sample data. They distinguish them from screenshots captured from the running application.
- The PNG embedded in the About dialog is byte-identical to `src/ui/assets/ac-64.png`. `about_runtime.cjs` checks this before exercising the dialog in light and dark themes at 840 px and 1280 px.
- The live documentation fixture configures safe read-only coordinator arguments and asserts that `chat_blockers` has no Claude entry. The refreshed 1280×892 workspace captures contain no coordinator-unavailable banner.
- The two Markdown manuals render to committed standalone HTML; `manual_renderer` and `public_docs` passed seven tests. `scripts/check_screenshots.py` and `cargo fmt --check` passed after commit.

## Asset fingerprints

SHA-256:

| Asset | SHA-256 |
|---|---|
| `src/ui/assets/ac-64.png` | `c76a4162ec4098bd3b71e7b2390ed67cc47f70f54aef3f51fabb2abe9a66880f` |
| `ui-approved-workspace-design.png` | `5ada909eda4825b9d973ddb0ff70a86be04c2505a038e69db46950d4a36f65b7` |
| `ui-approved-tasks-light.png` | `bdbeef2b2d19e00fd60e3e06359aa4bc7eae89763558a38070a5d1d149a3c244` |
| `ui-approved-tasks-dark.png` | `710bfb4a67b9d53e93ee8bc253ef059703482d5d6895a8650407bb55af1e84de` |
| `ui-workspace-current.png` | `7cfdb6d647e6f371b2286c4e6aeb1b164a507424cc0a4163c8578822a08491f4` |
| `ui-workspace-korean-current.png` | `b1b461e868155169177bbd39203b4b2d8cdefc8625fd57824e7b888d74a5e1d5` |
| `ui-workspace-dark-current.png` | `4fe23440896646e9698f0ea8f038643af5d186df1a5bd8b786fda4a635b0f0e5` |

## Known conflict and revalidation

PR #762 independently refreshes the workspace capture but its English image still contains the coordinator-unavailable banner. A PR comment points to the corrected fixture and clean assets in PR #761. Do not replace the clean captures with those images without rerunning the warning assertion.

Recheck these facts when the About image, coordinator policy, UI markup, design preview, or public screenshots change. A mockup is not evidence that the shipped app implements the depicted interaction.
