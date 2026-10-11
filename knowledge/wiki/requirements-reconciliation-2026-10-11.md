# Requirements and Claude work reconciliation (2026-10-11)

Reviewed source: `318f9303dbfd22a1c848a37159fb6646ef3f9255`.
The canonical Development checkout was clean at `36d3420` and 43 commits behind
origin/main. Fetch and fast-forward reconciled it without replaying prior Codex
changes. The old Documents checkout retains its older DD-41 edits as a backup.
No feature implementation or provider generation was performed in this review.

## Current requirements baseline

The trace now extends through R-59, BG-21, SF-49, UC-48, QAS-50 and DD-65.
Requirements, functional/nonfunctional analysis, QAS and design drivers were
reviewed together with implementation and tests. The original three-task limit
is superseded by R-53: one to twelve tasks; concurrency remains bounded by agent
session capacity. Manual planning makes the LLM coordinator optional.

| Scope | Current implementation / remaining acceptance |
| --- | --- |
| R-36, DD-42 | Desktop-first public README; separate English user manual shipped in packages. |
| R-37–40, DD-45–48 | Manual planning, native folder selection, named workspace creation/listing/switching/removal; native interaction acceptance remains explicit in the plan. |
| R-41, DD-49 | Version-specific CHANGELOG and release notes; release policy says publish only when requested, retain pending features in the next changelog section. |
| R-42/R-44, DD-50/DD-52 | Explicit baseline commit choice and local OS-file exclusion; no silent commit. |
| R-43, DD-51 | Slack/company messenger integration is planned, not implemented; transport and approval policy remain to be confirmed. |
| R-45–47, DD-53–55 | Visible planning progress, readable instructions, agent/model role rules and three numbered workspace steps. |
| R-48–51, DD-56–58 | Immediate agent persistence, observed model catalog reuse/discovery, plan retention on unrelated edits, compact agent cards. |
| R-52, DD-59 | Versioned settings export/import; no secrets/catalogs/goals/runs; imported local permission claims require reconfirmation. Two-machine acceptance remains open. |
| R-53/R-54, DD-60 | 1–12 tasks, add/remove and Markdown plan ingestion through a read-only coordinator snapshot. Native document/coordinator acceptance remains open. |
| R-55, DD-61 | Slug record/tree creation and Git exclusion implemented. Creation is an explicit request; automatic goal/UI integration remains part of the pending increments. |
| R-56–59, DD-62–65 | Sequential phases, tree-derived progress/verifier, slug ZIP exchange and editable effective plan rules are designed but not implemented yet. |

## Verification and delivery evidence

On the reconciled source, targeted local tests passed: plan_size 4, role_rules 6,
slugs 3, workspace_share 2 and source_layout 1 (16 total). This is local contract
evidence, not full all-provider or native Windows execution acceptance.
GitHub latest published release is v0.1.0-preview.6, with macOS arm64 and Windows
x86_64 ZIPs plus SHA256SUMS. Its package names identify source `890b1a657f1f`;
it does not contain all later changes through the reviewed source.

Latest platform run 38112177633 is terminal failure, but both validation jobs
have no execution steps. GitHub annotations explicitly say the jobs were not
started because recent account payments failed or the spending limit must be
increased. Therefore the latest source has no native CI result; this is not a
compiler/test failure. No automatic restart or new release was initiated.

The older completion audit and the beginning of the implementation plan retain
historical observations that are stale as current-status statements. In particular,
preview binaries are already published, earlier Claude/OpenCode live acceptance,
Korean IME and resource evidence are recorded in subsequent commits, and automatic
memory discovery/reviewed handoff now have implementations. Their specific evidence
must be inspected before closing broad acceptance; historical open rows must not
cause completed implementations to be duplicated.

Next implementation order in the current plan: DD-63 progress/tree verification,
DD-65 plan rules, DD-62 phased runs and stage movement, DD-64 archive exchange,
then native slug/progress/rules UI integration. Preserve the original human merge,
CLI authentication, local-first and TDD/source-layout constraints throughout.

## Sources

- [docs/01-requirements.md](../../docs/01-requirements.md): SHA-256 `9f2a7c41d9b60ea0f8d14f70af18d6cbb410d53e62d8ae00bd082014608a09e0`.
- [docs/05-qas.md](../../docs/05-qas.md): SHA-256 `aaf4bdb656d97b57886a759500fea9cb3ae8cbf542c1b1d0a37537b1b24ebf35`.
- [docs/05b-design-drivers.md](../../docs/05b-design-drivers.md): SHA-256 `5efbe2d452f0343b34619e76f73e9a8d237d1e5e3c0c02384aa8559f85a00fe4`.
- [docs/06-design-decisions.md](../../docs/06-design-decisions.md): SHA-256 `272fc2804d4bdede9cb4891efb9399650524e1e2cbcbc5a857c965f137ad8187`.
- [src/slugs.rs](../../src/slugs.rs): SHA-256 `3c99b21c69a8d50ed50b0f1c93b53dc1611051fe8e3f54538a9c4ba515f88843`.
- [src/roles.rs](../../src/roles.rs): SHA-256 `461f58d2410a0bf58d6ad40267bdd8f338ff9963da5767382148f2a9c89c9173`.
- [src/ui/setup/share.rs](../../src/ui/setup/share.rs): SHA-256 `470d737b333b9319ccfdd83c3e2c5eb2e7019ebbc9461986a964ef5992b23ded`.
- [tests/slugs.rs](../../tests/slugs.rs): SHA-256 `4221229cc485776f88138fa9613843003b0259ab3186dd36a5edb9e93af490b8`.
- [tests/plan_size.rs](../../tests/plan_size.rs): SHA-256 `89da130794af61d497801e7d7ae63c38fa53315db23a6bf0a4c114df90493119`.
