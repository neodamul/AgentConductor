# Rust migration acceptance tracking

Updated: 2026-11-03

## Current state

- Parent issue [#9](https://github.com/neodamul/AgentConductor/issues/9) is closed as a tracking transfer only; migration acceptance is **not complete**.
- The score remains **22/50**: candidate package acceptance **22/25**, shipped/default Rust UI conversion **0/25**.
- Root PR [#22](https://github.com/neodamul/AgentConductor/pull/22) is Draft.
- Each of the 25 canonical operator workflows has exactly one score-bearing issue labeled `rust-ui-workflow`. Each records candidate-package and shipped-default status separately.
- Workflows 01–22 have candidate-package status completed as part of the historical 22/25 aggregate. Workflows 23–25 remain uncredited. All 25 shipped-default Rust gates remain pending.

## The 25 individually tracked workflows

| # | Issue | Workflow | Candidate package | Shipped default |
| ---: | --- | --- | --- | --- |
| 01 | [#351](https://github.com/neodamul/AgentConductor/issues/351) | Save workspace name, goal, repository, and isolation mode | Completed in aggregate | Pending |
| 02 | [#350](https://github.com/neodamul/AgentConductor/issues/350) | Browse for a repository folder | Completed in aggregate | Pending |
| 03 | [#352](https://github.com/neodamul/AgentConductor/issues/352) | Register, edit, and remove agent profiles | Completed in aggregate | Pending |
| 04 | [#354](https://github.com/neodamul/AgentConductor/issues/354) | Assign roles and choose a coordinator | Completed in aggregate | Pending |
| 05 | [#353](https://github.com/neodamul/AgentConductor/issues/353) | Discover models and select role models | Completed in aggregate | Pending |
| 06 | [#355](https://github.com/neodamul/AgentConductor/issues/355) | Generate a plan and inspect live progress/failure logs | Completed in aggregate | Pending |
| 07 | [#356](https://github.com/neodamul/AgentConductor/issues/356) | Edit a plan or write task instructions manually | Completed in aggregate | Pending |
| 08 | [#357](https://github.com/neodamul/AgentConductor/issues/357) | Review consent and start execution | Completed in aggregate | Pending |
| 09 | [#358](https://github.com/neodamul/AgentConductor/issues/358) | Create, open, switch, and delete workspaces | Completed in aggregate | Pending |
| 10 | [#359](https://github.com/neodamul/AgentConductor/issues/359) | Import and export workspace settings | Completed in aggregate | Pending |
| 11 | [#360](https://github.com/neodamul/AgentConductor/issues/360) | Change language and theme | Completed in aggregate | Pending |
| 12 | [#361](https://github.com/neodamul/AgentConductor/issues/361) | Configure MCP servers and import referenced sources | Completed in aggregate | Pending |
| 13 | [#362](https://github.com/neodamul/AgentConductor/issues/362) | Configure and test messenger delivery | Completed in aggregate | Pending |
| 14 | [#363](https://github.com/neodamul/AgentConductor/issues/363) | Preview and clear local cache | Completed in aggregate | Pending |
| 15 | [#364](https://github.com/neodamul/AgentConductor/issues/364) | Open manuals, installation guidance, and release updates | Completed in aggregate | Pending |
| 16 | [#365](https://github.com/neodamul/AgentConductor/issues/365) | Inspect run/task status and agent activity lights | Completed in aggregate | Pending |
| 17 | [#366](https://github.com/neodamul/AgentConductor/issues/366) | Inspect and refresh per-agent quota | Completed in aggregate | Pending |
| 18 | [#367](https://github.com/neodamul/AgentConductor/issues/367) | Send task feedback and reassign work | Completed in aggregate | Pending |
| 19 | [#368](https://github.com/neodamul/AgentConductor/issues/368) | Select a task session and model | Completed in aggregate | Pending |
| 20 | [#369](https://github.com/neodamul/AgentConductor/issues/369) | Schedule work after an availability restriction | Completed in aggregate | Pending |
| 21 | [#370](https://github.com/neodamul/AgentConductor/issues/370) | Inspect task activity, logs, and lessons | Completed in aggregate | Pending |
| 22 | [#371](https://github.com/neodamul/AgentConductor/issues/371) | Read a factual team briefing | Completed in aggregate | Pending |
| 23 | [#372](https://github.com/neodamul/AgentConductor/issues/372) | Allow, deny, or refresh a tool request | Pending | Pending |
| 24 | [#373](https://github.com/neodamul/AgentConductor/issues/373) | Inspect diff/tests and approve, reject, or merge a candidate | Pending | Pending |
| 25 | [#374](https://github.com/neodamul/AgentConductor/issues/374) | Stop, resume, or recover a run | Pending | Pending |

## Separate cross-cutting quality gates

These are real project gates, but they are not extra items in the 50-point workflow denominator:

- [#345](https://github.com/neodamul/AgentConductor/issues/345): idle resource measurement and authenticated provider behavior.
- [#346](https://github.com/neodamul/AgentConductor/issues/346): Windows/macOS runtime, accessibility, and Korean IME acceptance.
- [#348](https://github.com/neodamul/AgentConductor/issues/348): final dependency/license audit, packages, manuals, and release.
- Existing implementation/bug follow-ups: #189, #226, #257, #283, #310. Link these to the relevant workflow issue; do not count them as additional score rows.

The aggregate 22/25 ledger and definitions for uncredited workflows 23–25 are recorded in `docs/evidence/issue9-acceptance-ledger-audit-20261028.json`. Older dated notes in `docs/07-implementation-plan.md`, `docs/09-completion-audit.md`, and the wiki are historical; the GitHub workflow issue checkboxes are the live source of truth.

## Acceptance rules

- Do not infer workflow score from source coverage or component tests.
- Candidate-package completion requires evidence for the candidate package. Shipped-default completion requires the accepted stable macOS and Windows release behavior.
- Every issue must retain exact revision, package hash, deterministic setup, tested controls, persisted result, and outcome.
- Close the issue only when its applicable gates and cross-platform evidence are satisfied. The closure of #9 itself does not close any workflow issue.
