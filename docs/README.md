# AgentConductor engineering documents

Entry point for the architecture and delivery documents. The project follows this
system-architecture method; every new requirement passes through each step before
design changes and TDD implementation:

```text
Requirements (R) → Business goals (BG) → System features (SF) → Use cases (UC)
→ Functional / non-functional analysis (FR, NFR, C) → Quality attribute scenarios (QAS)
→ Design drivers (DDR) → Architecture and design decisions (DD) → Plan / TDD → Release
```

## Document map

| Step | Document | Contents |
|---|---|---|
| 1 | [01-requirements.md](01-requirements.md) | R-01 … R-46 with acceptance criteria |
| 2 | [02-business-goals.md](02-business-goals.md) | BG-01 … BG-18 and success measures |
| 3 | [03-system-features.md](03-system-features.md) | SF-01 … SF-39 |
| 4 | [04-usecases.md](04-usecases.md) | UC-01 … UC-38 |
| 5 | [04b-functional-nonfunctional-analysis.md](04b-functional-nonfunctional-analysis.md) | FR-01 … FR-31, NFR-01 … NFR-17, C-01 … C-05 |
| 6 | [05-qas.md](05-qas.md) | QAS-01 … QAS-39 |
| 7 | [05b-design-drivers.md](05b-design-drivers.md) | DDR-01 … DDR-14 and the architecture view |
| 8 | [06-design-decisions.md](06-design-decisions.md) | DD-01 … DD-54, DD-UI-01 |
| 9 | [07-implementation-plan.md](07-implementation-plan.md) | Current status, Plan → TDD → Verify → Complete checklists |
| — | [user-manual.md](user-manual.md) | User manual: desktop guide and full command reference |
| — | [08-cli-mvp-release.md](08-cli-mvp-release.md) | CLI MVP release verification |
| — | [09-completion-audit.md](09-completion-audit.md) | Completion audit and remaining acceptance |
| — | [10-cross-platform-development.md](10-cross-platform-development.md) | macOS/Windows continuity and release acceptance |
| — | [11-cooperative-checkpoints.md](11-cooperative-checkpoints.md) | Checkpoint directive protocol for non-steerable CLIs |
| — | [interface-gallery.md](interface-gallery.md) | Interface evidence |
| — | [../knowledge/wiki/index.md](../knowledge/wiki/index.md) | Source-fingerprinted knowledge assets per verified slice |

## Traceability matrix

| Requirements | BG | SF | UC | FR / NFR | QAS | DDR | DD |
|---|---|---|---|---|---|---|---|
| R-01–03 subscriptions, I/O, CLI registry | BG-01 | SF-01 | UC-01 | FR-01, FR-06, NFR-06, NFR-07, C-01 | QAS-05, QAS-12 | DDR-04 | DD-03, DD-04 |
| R-04–06 planning, parallel work, evidence | BG-02 | SF-02–04 | UC-02 | FR-03–06, NFR-05 | QAS-01, QAS-06, QAS-08 | DDR-02, DDR-06 | DD-01, DD-05 |
| R-07–09 approval, dangerous actions, stop/resume | BG-03 | SF-05, SF-06 | UC-03, UC-04 | FR-07–09, NFR-01, NFR-03 | QAS-02–04 | DDR-01, DDR-03 | DD-02, DD-06, DD-07, DD-14, DD-17–19 |
| R-10–11 memory, wiki | BG-04 | SF-07 | UC-05 | FR-11, NFR-04 | QAS-07, QAS-13 | DDR-05 | DD-08, DD-21 |
| R-12–16 Rust, TDD, tools, licenses, docs | BG-05 | SF-12 | UC-17 | FR-15, NFR-12, C-02–C-04 | QAS-14 | DDR-06, DDR-08 | DD-01, DD-09 |
| R-17 intervention UI | BG-03 | SF-08 | UC-03, UC-04 | FR-14, NFR-10 | QAS-21 | DDR-07 | DD-10, DD-22, DD-23 |
| R-18 restriction recovery | BG-07 | SF-09 | UC-09 | FR-08, NFR-03 | QAS-09 | DDR-03 | DD-11 |
| R-19 sessions, role transfer | BG-08 | SF-10 | UC-10 | FR-10, NFR-08 | QAS-10 | DDR-03 | DD-12 |
| R-20 model catalogs | BG-09 | SF-11 | UC-11 | FR-02 | QAS-11 | DDR-04 | DD-13 |
| R-21 native startup workspace | BG-09 | SF-18 | UC-15 | FR-03, FR-04, FR-14 | QAS-16 | DDR-01, DDR-06 | DD-22, DD-23, DD-25 |
| R-22 quota | BG-10 | SF-19 | UC-16 | FR-13, NFR-09 | QAS-17 | DDR-07 | DD-26 |
| R-23–24 cross-platform, releases | BG-10 | SF-20 | UC-17 | FR-15, NFR-11, NFR-12 | QAS-14, QAS-18 | DDR-08 | DD-27, DD-35–DD-38 |
| R-25–26 supervision, fallback | BG-11 | SF-21 | UC-18 | FR-10, NFR-04 | QAS-19 | DDR-05 | DD-28 |
| R-27 cooperative checkpoints | BG-11, BG-12 | SF-24 | UC-21 | FR-11, FR-12, C-05 | QAS-22 | DDR-05 | DD-29 |
| R-28 team briefing | BG-12 | SF-22 | UC-19 | FR-13, NFR-09 | QAS-20 | DDR-07 | DD-30 |
| R-29 status lights | BG-12 | SF-23 | UC-20 | FR-13, NFR-10 | QAS-21 | DDR-07 | DD-31 |
| R-30 live tool decisions | BG-13 | SF-25 | UC-22 | FR-07, FR-17, NFR-01, NFR-13 | QAS-23 | DDR-01, DDR-09 | DD-32 |
| R-31 isolated planning | BG-13 | SF-26 | UC-23 | FR-16, NFR-02, C-05 | QAS-24 | DDR-02 | DD-33 |
| R-32 Korean input | BG-13 | SF-27 | UC-24 | FR-18, NFR-10 | QAS-25 | DDR-07 | DD-34 |
| R-33 leave workspace without saving | BG-09 | SF-28 | UC-25 | FR-19, NFR-10 | QAS-26 | DDR-07 | DD-39 |
| R-34 application icons | BG-14 | SF-29 | UC-26 | FR-20, NFR-10, NFR-12 | QAS-27 | DDR-10 | DD-40 |
| R-36 product README and user manual | BG-15 | SF-30 | UC-27 | FR-21, NFR-14 | QAS-28 | DDR-11 | DD-42 |
| R-37 plan without an LLM coordinator | BG-16 | SF-31 | UC-28 | FR-22, NFR-15 | QAS-29 | DDR-12 | DD-45 |
| R-38 discoverable workspace settings | BG-17 | SF-32 | UC-29 | FR-23, NFR-16 | QAS-30 | DDR-13 | DD-46 |
| R-39 multiple workspaces | BG-17 | SF-33 | UC-30 | FR-24, NFR-16 | QAS-31 | DDR-13 | DD-47 |
| R-40 named, registered workspaces | BG-17 | SF-34 | UC-31 | FR-25, NFR-16 | QAS-32 | DDR-13 | DD-48 |
| R-41 release notes summarize changes | BG-10 | SF-35 | UC-32 | FR-26 | QAS-33 | DDR-08 | DD-49 |
| R-42 explain and resolve an unclean repository | BG-13 | SF-36 | UC-33 | FR-27 | QAS-34 | DDR-01 | DD-50 |
| R-44 ignore OS files and continue | BG-13 | SF-36 | UC-36 | FR-29 | QAS-37 | DDR-01 | DD-52 |
| R-45 visible planning progress | BG-12 | SF-38 | UC-37 | FR-30 | QAS-38 | DDR-07 | DD-53 |
| R-46 human-readable task instructions | BG-13 | SF-39 | UC-38 | FR-31 | QAS-39 | DDR-07 | DD-54 |
| R-43 messenger integration (planned) | BG-18 | SF-37 | UC-34, UC-35 | FR-28, NFR-17 | QAS-35, QAS-36 | DDR-14 | DD-51 (proposed) |

Known gaps: historical design traces cite UC-06/07/08 and UC-12–14 and features
SF-13–17 that are not defined in the use-case/feature documents; their scope is covered
by the rows above. New entries must use the next free identifier at every step.
