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
| 1 | [01-requirements.md](01-requirements.md) | R-01 … R-210 with acceptance criteria |
| 2 | [02-business-goals.md](02-business-goals.md) | BG-01 … BG-165 and success measures |
| 3 | [03-system-features.md](03-system-features.md) | SF-01 … SF-198 |
| 4 | [04-usecases.md](04-usecases.md) | UC-01 … UC-202 |
| 5 | [04b-functional-nonfunctional-analysis.md](04b-functional-nonfunctional-analysis.md) | FR-01 … FR-204, NFR-01 … NFR-75, C-01 … C-06 |
| 6 | [05-qas.md](05-qas.md) | QAS-01 … QAS-209 |
| 7 | [05b-design-drivers.md](05b-design-drivers.md) | DDR-01 … DDR-39 and the architecture view |
| 8 | [06-design-decisions.md](06-design-decisions.md) | DD-01 … DD-222, DD-UI-01 |
| 9 | [07-implementation-plan.md](07-implementation-plan.md) | Current status, Plan → TDD → Verify → Complete checklists |
| 10 | [14-architecture-verification.md](14-architecture-verification.md) | Requirement → test evidence for the consolidated architecture, and the open gaps |
| — | [user-manual.md](user-manual.md) | User manual: desktop guide and full command reference |
| — | [08-cli-mvp-release.md](08-cli-mvp-release.md) | CLI MVP release verification |
| — | [09-completion-audit.md](09-completion-audit.md) | Completion audit and remaining acceptance |
| — | [10-cross-platform-development.md](10-cross-platform-development.md) | macOS/Windows continuity and release acceptance |
| — | [11-cooperative-checkpoints.md](11-cooperative-checkpoints.md) | Checkpoint directive protocol for non-steerable CLIs |
| — | [13-rust-review-policy.md](13-rust-review-policy.md) | Rust review rules adopted from the Google Rust Style Guide, exclusions and measured baseline |
| — | [interface-messages.md](interface-messages.md) | Stable UI message IDs, language catalogs and localization checks |
| — | [interface-gallery.md](interface-gallery.md) | Interface evidence |
| — | [Approved unified Goal UI](design/approved-goal-ui/README.md) | 2026-11-04 behavior, visual contract, both themes, images and Claude handoff (#387) |
| — | [../knowledge/wiki/index.md](../knowledge/wiki/index.md) | Source-fingerprinted knowledge assets per verified slice |

## Identifier and consolidation rules

The documents grew one request at a time, so later requirements often refine or replace earlier
ones. They are consolidated without renumbering, because code, tests and the implementation plan
cite the identifiers:

- Every identifier is defined once, in its own document: a heading that starts with it
  (`### UC-15 …`, `### FR-46 / NFR-21 …`), a table row that starts with it, or a paragraph or
  list item that starts with `ID:`. A second meaning gets the next free number.
- An entry that a later one changes keeps its text and gets a `Status:` line directly under its
  heading: `Amended by X` (still in force except the stated part), `Superseded by X` (no longer
  in force; read X instead) or `Merged into X` (the same need, now stated in X). The use case and
  QAS of a changed requirement carry the same status, pointing to the replacing UC and QAS.
- A change that has no requirement of its own (a GitHub issue fix) amends the requirement it
  refines: its section in 07 starts with `Trace: amends R-n`, and when it changes the agreed
  behavior the requirement gets a `Status:` line naming the issue.
- `tests/architecture_trace.rs` enforces these rules: no identifier is defined twice, every
  cited identifier is defined, every requirement appears exactly once in the matrix below, and
  every `Status:` line names identifiers that exist.

## Traceability matrix

Grouped by area. The chain is read from each requirement's `Trace:` line (or, for the early
requirements without one, from the earlier matrix rows and the plan); "current" means no later
requirement changes it.

### Agent CLIs and profiles

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-01 Reuse existing CLI authentication and subscriptions | BG-01 | SF-01 | UC-01 | FR-01, FR-06, NFR-06, NFR-07, C-01 | QAS-05, QAS-12 | DDR-04 | DD-03, DD-04 | current |
| R-02 Control subprocess I/O | BG-01 | SF-01 | UC-01 | FR-01, FR-06, NFR-06, NFR-07, C-01 | QAS-05, QAS-12 | DDR-04 | DD-03, DD-04 | current |
| R-03 Register three native CLIs and arbitrary CLIs | BG-01 | SF-01 | UC-01 | FR-01, FR-06, NFR-06, NFR-07, C-01 | QAS-05, QAS-12 | DDR-04 | DD-03, DD-04 | current |
| R-20 Query model catalogs through existing authenticated CLI subprocesses d | BG-09 | SF-11 | UC-11 | FR-02 | QAS-11 | DDR-04 | DD-13 | current |
| R-31 Planning with unverifiable company CLIs | BG-13 | SF-26 | UC-23 | FR-16, NFR-02, C-05 | QAS-24 | DDR-02 | DD-33 | current |
| R-34 Recognizable agent icons | BG-14 | SF-29 | UC-26 | FR-20, NFR-10, NFR-12 | QAS-27 | DDR-10 | DD-40 | current |
| R-48 Agent registrations are saved immediately | BG-09 | SF-41 | UC-40 | FR-33 | QAS-41 | DDR-13 | DD-56 | current |
| R-49 Model lists prepared per agent | BG-10 | SF-42 | UC-41 | FR-34 | QAS-42 | DDR-13 | DD-57 | current |
| R-51 Compact agent list | BG-14 | SF-41 | UC-40 | FR-36 | QAS-41 | DDR-13 | DD-58 | current |
| R-66 New workspaces start without agents | BG-09 | SF-41 | — | — | QAS-59 | DDR-13 | DD-72 | superseded by R-101 |
| R-67 Agent preferences and consistent native identity | BG-25 | SF-55 | UC-55 | FR-51, NFR-25 | QAS-60 | — | DD-73 | current |
| R-68 Provider choice defines the whole agent profile | BG-26 | SF-56 | UC-56 | FR-52, NFR-26 | QAS-61 | — | DD-74 | current |
| R-71 Fallback model choices are always offered | — | SF-59 | UC-59 | FR-55 | QAS-64 | — | DD-77 | current |
| R-72 Windows CLI metadata and provider visibility | BG-28 | SF-60 | UC-60 | FR-56, NFR-28 | QAS-65 | — | DD-78 | current |
| R-73 Self-hosted CLI model catalogs without discovery | BG-29 | SF-61 | UC-62 | FR-57, NFR-29 | QAS-67 | — | DD-80 | current |
| R-78 Nonblocking agent registration and metadata | BG-33 | SF-65 | UC-68 | FR-63, NFR-34 | QAS-73 | — | DD-86 | current |
| R-91 Save an edited agent profile visibly | BG-46 | SF-78 | UC-81 | FR-76, NFR-47 | QAS-87 | — | DD-99 | current |
| R-101 Agent CLI list is a master list | BG-56 | SF-88 | UC-91 | FR-86 | QAS-97 | — | DD-109 | current |
| R-102 Reorder agent cards | BG-57 | SF-89 | UC-92 | FR-87 | QAS-98 | — | DD-110 | current |
| R-132 Custom CLIs that approve their own tools can run | BG-87 | SF-119 | UC-122 | FR-117 | QAS-128 | — | DD-141 | current |
| R-154 Custom CLI behavior belongs to its agent profile | BG-109 | SF-141 | UC-144 | FR-139 | QAS-150 | DDR-17 | DD-164 | current |
| R-158 Configure custom CLI models, execution options, and result decoding | BG-113 | SF-145 | UC-148 | FR-143, NFR-53 | QAS-154 | DDR-19 | DD-168 | current |
| R-183 A standard, schema-based CLI interface | BG-138 | SF-171 | UC-174 | FR-171, FR-172, FR-173, NFR-74 | QAS-181 | DDR-38 | DD-194, DD-195 | amended by GitHub #341 |
| R-186 Every agent prompt has one standard form and its reply is checked | BG-141 | SF-174 | UC-177 | FR-178 | QAS-184 | — | DD-198 | current |
| R-189 A valid JSON block is found even after an unfinished one | BG-144 | SF-177 | UC-180 | FR-181 | QAS-187 | — | DD-201 | current |

### Account capacity and availability

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-18 Persist subscription/quota/transient restrictions, source evidence, ob | BG-07 | SF-09 | UC-09 | FR-08, NFR-03 | QAS-09 | DDR-03 | DD-11 | current |
| R-19 Support multiple independent logical/provider sessions per profile, di | BG-08 | SF-10 | UC-10 | FR-10, NFR-08 | QAS-10 | DDR-03 | DD-12 | current |
| R-22 Remaining subscription quota | BG-10 | SF-19 | UC-16 | FR-13, NFR-09 | QAS-17 | DDR-07 | DD-26 | current |
| R-26 Per-agent availability fallback | BG-11 | SF-21 | UC-18 | FR-10, NFR-04 | QAS-19 | DDR-05 | DD-28 | current |
| R-60 Collect Claude subscription quota without extra generation | BG-22 | SF-50 | UC-49 | — | QAS-51 | — | DD-66 | amended by R-97, R-148, R-201 |
| R-61 Inline agent quota — Show paid CLI remaining capacity beside the agent | BG-22 | SF-51 | UC-50 | FR-46, NFR-21 | QAS-52 | — | DD-67 | amended by R-64, R-108, R-130 |
| R-63 Agent-specific refresh and action icons | BG-22 | SF-53 | UC-53 | FR-49, NFR-23 | QAS-56 | — | DD-69 | current |
| R-64 Compact quota windows | BG-24 | SF-54 | UC-54 | FR-50, NFR-24 | QAS-57 | — | DD-70 | amended by R-130 |
| R-69 Show when exhausted quota returns | BG-26 | SF-56, SF-57 | UC-56, UC-57 | FR-52, FR-53, NFR-26 | QAS-61, QAS-62 | — | DD-74, DD-75 | superseded by R-130 |
| R-75 Enterprise quota budgets | BG-30 | SF-62 | UC-65 | FR-60, NFR-31 | QAS-70 | — | DD-83 | current |
| R-85 Enterprise usage as a percentage | BG-40 | SF-72 | UC-75 | FR-70, NFR-41 | QAS-81 | — | DD-93 | current |
| R-86 Claude capacity on Windows reports why it is unavailable | BG-41 | SF-73 | UC-76 | FR-71, NFR-42 | QAS-82 | — | DD-94 | current |
| R-97 Usage query accepts the Claude folder-trust prompt | BG-52 | SF-84 | UC-87 | FR-82 | QAS-93 | — | DD-105 | current |
| R-127 Durable quota resume and five-hour fallback | BG-82 | SF-114 | UC-117 | FR-112 | QAS-123 | — | DD-136 | current |
| R-130 Compact quota cards and readable step explanations | BG-85 | SF-117 | UC-120 | FR-115 | QAS-126 | — | DD-139 | current |
| R-136 Approval readiness and Claude API usage limits | BG-91 | SF-123 | UC-126 | FR-121 | QAS-132 | — | DD-145 | current |
| R-145 Saved-project Claude quota retry | BG-100 | SF-132 | UC-135 | FR-130 | QAS-141 | — | DD-154 | current |
| R-148 Claude prompt-compatible metadata collection | BG-103 | SF-135 | UC-138 | FR-133 | QAS-144 | — | DD-157 | current |
| R-201 A slow Claude usage query is waited for and shown | BG-156 | SF-189 | UC-192 | FR-193 | QAS-199 | — | DD-213 | amended by GitHub #321 |

### Workspaces, settings and interface

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-21 Native startup workspace | BG-09 | SF-18 | UC-15 | FR-03, FR-04, FR-14 | QAS-16 | DDR-01, DDR-06 | DD-22, DD-23, DD-25 | amended by R-53 |
| R-32 Korean text entry | BG-13 | SF-27 | UC-24 | FR-18, NFR-10 | QAS-25 | DDR-07 | DD-34 | current |
| R-33 Leave the workspace without saving | BG-09 | SF-28 | UC-25 | FR-19, NFR-10 | QAS-26 | DDR-07 | DD-39 | current |
| R-38 Discoverable workspace settings | BG-17 | SF-32 | UC-29 | FR-23, NFR-16 | QAS-30 | DDR-13 | DD-46 | current |
| R-39 Multiple workspaces | BG-17 | SF-33 | UC-30 | FR-24, NFR-16 | QAS-31 | DDR-13 | DD-47 | amended by R-149 |
| R-40 Named, registered workspaces | BG-17 | SF-34 | UC-31 | FR-25, NFR-16 | QAS-32 | DDR-13 | DD-48 | current |
| R-50 Workspace changes keep a reviewed plan | BG-17 | SF-42 | UC-41 | FR-35 | QAS-43 | DDR-13 | DD-58 | current |
| R-52 Shareable workspace settings (export/import) | BG-20 | SF-43 | UC-42 | FR-37, NFR-18 | QAS-44 | DDR-15 | DD-59 | current |
| R-58 Export/import includes slug archives | BG-21 | SF-48 | UC-47 | FR-43, NFR-19 | QAS-49 | DDR-16 | DD-64 | current |
| R-70 Interface language | BG-27 | SF-58 | UC-58 | FR-54, NFR-27 | QAS-63 | — | DD-76 | current |
| R-88 Reliable named workspace creation | BG-43 | SF-75 | UC-78 | FR-73 | QAS-84 | — | DD-96 | current |
| R-89 Delete an unwanted workspace | BG-17, BG-43, BG-44 | SF-33, SF-34, SF-75, SF-76 | UC-30, UC-31, UC-78, UC-79 | FR-24, FR-25, FR-73, FR-74, NFR-16 | QAS-31, QAS-32, QAS-84, QAS-85 | DDR-13 | DD-47, DD-48, DD-96, DD-97 | current |
| R-95 Abandon a blocked planning workspace | BG-50 | SF-82 | UC-85 | FR-80 | QAS-91 | — | DD-103 | amended by R-209 |
| R-98 Planning start and workspace naming feedback | BG-53 | SF-85 | UC-88 | FR-83 | QAS-94 | — | DD-106 | amended by R-209 |
| R-128 Selectable interface themes | BG-83 | SF-115 | UC-118 | FR-113, NFR-49 | QAS-124 | — | DD-137 | current |
| R-131 Readable and focused Settings | BG-86 | SF-118 | UC-121 | FR-116 | QAS-127 | — | DD-140 | amended by R-178 |
| R-139 Workspace and Agent Room feedback refinements | BG-94 | SF-126 | UC-129 | FR-124 | QAS-135 | — | DD-148 | current |
| R-143 Workspace transfer menu | BG-98 | SF-130 | UC-133 | FR-128 | QAS-139 | — | DD-152 | current |
| R-147 Retained completed rooms do not block workspace creation | BG-102 | SF-134 | UC-137 | FR-132 | QAS-143 | — | DD-156 | amended by R-149 |
| R-149 Active workspace navigation | BG-104 | SF-136 | UC-139 | FR-134 | QAS-145 | — | DD-158 | current |
| R-150 One project workspace with multiple goal records | BG-105 | SF-137 | UC-140 | FR-135 | QAS-146 | — | DD-159 | current |
| R-180 Screens show only information the operator can use | BG-135 | SF-168 | UC-171 | FR-168 | QAS-178 | — | DD-191 | current |
| R-185 A deleted default workspace stays deleted | BG-140 | SF-173 | UC-176 | FR-177 | QAS-183 | — | DD-197 | current |
| R-209 Deleting workspaces needs no replacement name | BG-164 | SF-197 | UC-201 | FR-203 | QAS-208 | — | DD-221 | current |
| R-210 One goal screen with a single six-step stepper | BG-165 | SF-198 | UC-202 | FR-204 | QAS-209 | — | DD-222 | current |

### Planning

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-04 Decompose a goal into scoped tasks (originally exactly three; supersed | BG-02 | SF-02 | UC-02 | FR-03, FR-04, FR-05, FR-06, NFR-05 | QAS-01 | DDR-02, DDR-06 | DD-01, DD-05 | current |
| R-25 Deterministic supervision and optional AI coordination | BG-11 | SF-21 | UC-18 | FR-10, NFR-04 | QAS-19 | DDR-05 | DD-28 | current |
| R-37 Plan without an LLM coordinator | BG-16 | SF-31 | UC-28 | FR-22, NFR-15 | QAS-29 | DDR-12 | DD-45 | amended by R-53 |
| R-42 Explain and resolve an unclean repository | BG-13 | SF-36 | UC-33 | FR-27 | QAS-34 | DDR-01 | DD-50 | current |
| R-44 Ignore operating-system files and continue | BG-13 | SF-36 | UC-36 | FR-29 | QAS-37 | DDR-01 | DD-52 | current |
| R-45 Visible planning progress | BG-12 | SF-38 | UC-37 | FR-30 | QAS-38 | DDR-07 | DD-53 | amended by R-83, R-135, R-196 |
| R-46 Human-readable task instructions | BG-13 | SF-39 | UC-38 | FR-31 | QAS-39 | DDR-07 | DD-54 | current |
| R-47 Agent role rules before planning | BG-19 | SF-40 | UC-39 | FR-32 | QAS-40 | DDR-12 | DD-55 | current |
| R-53 Plans with one to twelve tasks | BG-02 | SF-44 | UC-43 | FR-38 | QAS-45 | DDR-02 | DD-60 | current |
| R-54 Read an existing plan document | BG-19 | SF-44 | UC-43 | FR-39 | QAS-45 | DDR-12 | DD-60 | current |
| R-55 Goals become slugs with a work tree | BG-21 | SF-45 | UC-44 | FR-40, NFR-19 | QAS-46 | DDR-16 | DD-61 | amended by R-74, R-84, R-100 |
| R-56 Analysis first, then sequential phases | BG-21 | SF-46 | UC-45 | FR-41, NFR-19 | QAS-47 | DDR-16 | DD-62 | current |
| R-57 Progress tracking without tree mismatches | BG-21 | SF-47 | UC-46 | FR-42, NFR-19 | QAS-48 | DDR-16 | DD-63 | current |
| R-59 Editable plan rules | BG-21 | SF-49 | UC-48 | FR-44, NFR-19 | QAS-50 | DDR-16 | DD-65 | current |
| R-74 Slugs are named automatically | BG-12 | SF-46 | UC-64 | FR-59 | QAS-69 | — | DD-82 | amended by R-84 |
| R-77 Save new project folders before Git setup | BG-32 | SF-64 | UC-67 | FR-62, NFR-33 | QAS-72 | — | DD-85 | superseded by R-80, R-124 |
| R-79 Approved Git setup for new project planning | BG-34 | SF-66 | UC-69 | FR-64, NFR-35 | QAS-74 | — | DD-87 | superseded by R-80, R-124 |
| R-83 Planning start and progress are truthful | BG-38 | SF-70 | UC-73 | FR-68, NFR-39 | QAS-79 | — | DD-91 | current |
| R-84 English slug names | BG-39 | SF-71 | UC-74 | FR-69, NFR-40 | QAS-80 | — | DD-92 | current |
| R-100 Goal folders directly in the project folder | BG-55 | SF-87 | UC-90 | FR-85 | QAS-96 | — | DD-108 | current |
| R-112 Plan review and run completion are organised by phase | BG-67 | SF-99 | UC-102 | FR-97 | QAS-108 | — | DD-121 | current |
| R-116 Task cards in Review the plan fold to one line | BG-71 | SF-103 | UC-106 | FR-101 | QAS-112 | — | DD-124 | current |
| R-117 Review the whole phased plan up front | BG-72 | SF-104 | UC-107 | FR-102 | QAS-113 | — | DD-125 | current |
| R-129 Plans know whether the project uses Git, and verification commands tha | BG-84 | SF-116 | UC-119 | FR-114 | QAS-125 | — | DD-138 | current |
| R-135 Clear retry feedback and show coordinator activity | BG-90 | SF-122 | UC-125 | FR-120 | QAS-131 | — | DD-144 | current |
| R-146 Schema-constrained planning replies | BG-101 | SF-133 | UC-136 | FR-131 | QAS-142 | — | DD-155 | amended by GitHub #337 |
| R-152 Recover clearly from a slow plan retry | BG-107 | SF-139 | UC-142 | FR-137 | QAS-148 | — | DD-162 | current |
| R-177 The operator can improve a goal with the bundled prompt-optimizer skil | BG-132 | SF-165 | UC-168 | FR-163, FR-164, NFR-73 | QAS-174, QAS-175 | DDR-37 | DD-188 | current |
| R-190 The planning contract matches plan validation | BG-145 | SF-178 | UC-181 | FR-182 | QAS-188 | — | DD-202 | current |
| R-193 A task's facts line stays readable in the plan review | BG-148 | SF-181 | UC-184 | FR-185 | QAS-191 | — | DD-205 | current |
| R-195 The next phase can always be reviewed after a phase is done | BG-150 | SF-183 | UC-186 | FR-187 | QAS-193 | — | DD-207 | current |

### Execution, safety and recovery

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-05 Optional worktree separation and parallel execution | BG-02 | SF-03 | UC-02 | FR-03, FR-04, FR-05, FR-06, NFR-05 | QAS-06 | DDR-02, DDR-06 | DD-01, DD-05 | current |
| R-06 Collect results, diffs, and tests | BG-02 | SF-04 | UC-02 | FR-03, FR-04, FR-05, FR-06, NFR-05 | QAS-08 | DDR-02, DDR-06 | DD-01, DD-05 | current |
| R-08 Approve dangerous actions | BG-03 | SF-05, SF-06 | UC-03, UC-04 | FR-08, NFR-01, NFR-03 | QAS-03 | DDR-01, DDR-03 | DD-02, DD-06, DD-07, DD-14, DD-17, DD-18, DD-19 | current |
| R-09 Stop, resume, feedback, auditing | BG-03 | SF-05, SF-06 | UC-03, UC-04 | FR-09, NFR-01, NFR-03 | QAS-04 | DDR-01, DDR-03 | DD-02, DD-06, DD-07, DD-14, DD-17, DD-18, DD-19 | current |
| R-30 Live human tool decisions | BG-13 | SF-25 | UC-22 | FR-07, FR-17, NFR-01, NFR-13 | QAS-23 | DDR-01, DDR-09 | DD-32 | current |
| R-80 Work in ordinary project folders | BG-35 | SF-67 | UC-70 | FR-65, NFR-36 | QAS-75 | — | DD-88 | amended by R-124 |
| R-96 Agents work inside the workspace sandbox; the coordinator decides beyo | BG-51 | SF-83 | UC-86 | FR-81 | QAS-92 | — | DD-104 | amended by R-107, R-122 |
| R-106 Decide a live tool request from its window | BG-61 | SF-93 | UC-96 | FR-91 | QAS-102 | — | DD-114 | current |
| R-107 Coordinator decides routine requests and records each decision | BG-62 | SF-94 | UC-97 | FR-92 | QAS-103 | — | DD-115 | current |
| R-113 A blocking verification finding fails integration verification | BG-68 | SF-100 | UC-103 | FR-98 | QAS-109 | — | DD-120 | current |
| R-118 A run survives closing or losing the desktop app | BG-73 | SF-105 | UC-108 | FR-103 | QAS-114 | — | DD-127 | current |
| R-119 Interrupted runs recover and resume automatically | BG-74 | SF-106 | UC-109 | FR-104 | QAS-115 | — | DD-128 | current |
| R-120 Interruptions and silent workers are visible | BG-75 | SF-107 | UC-110 | FR-105 | QAS-116 | — | DD-129 | amended by R-159 |
| R-121 Reassign tasks that have not started while the run is working | BG-76 | SF-108 | UC-111 | FR-106 | QAS-117 | — | DD-130 | current |
| R-122 Agents run with the highest permission by default | BG-77 | SF-109 | UC-112 | FR-107 | QAS-118 | — | DD-131 | current |
| R-123 The coordinator resolves verification failures | BG-78 | SF-110 | UC-113 | FR-108 | QAS-119 | — | DD-132 | current |
| R-124 Execute in the selected project without adding Git metadata | BG-79 | SF-111 | UC-114 | FR-109 | QAS-120 | — | DD-133 | current |
| R-140 Runs keep going and stay visible | BG-95 | SF-127 | UC-130 | FR-125 | QAS-136 | — | DD-149 | current |
| R-153 Explain task timeouts and resume automatically | BG-108 | SF-140 | UC-143 | FR-138 | QAS-149 | — | DD-163 | current |
| R-157 Preserve task-state writes during transient SQLite contention | BG-112 | SF-144 | UC-147 | FR-142 | QAS-153 | DDR-18 | DD-167 | current |
| R-162 Recover from an out-of-scope integration failure through the app | BG-117 | SF-149 | UC-152 | FR-147, NFR-56 | QAS-158 | DDR-23 | DD-172 | current |
| R-163 The coordinator decides when a task stops without finishing | BG-118 | SF-150 | UC-153 | FR-148, NFR-57 | QAS-159 | DDR-24 | DD-173 | current |
| R-166 The coordinator guards the goal, the plan rules and the scope as one c | BG-121 | SF-153 | UC-156 | FR-151, NFR-60 | QAS-162 | DDR-27 | DD-176 | current |
| R-168 A run does not start when it cannot work | BG-121 | SF-155 | UC-158 | FR-153, NFR-62 | QAS-164 | DDR-27 | DD-178 | current |
| R-169 The coordinator amends the plan within limits, and every amendment can | BG-121 | SF-156 | UC-159 | FR-154, NFR-63 | QAS-165 | DDR-29 | DD-179 | current |
| R-170 A run keeps its charter, and the coordinator cannot drift from it sile | BG-121 | SF-157 | UC-160 | FR-155, NFR-64 | QAS-166 | DDR-30 | DD-180 | current |
| R-172 The operator can send feedback to the coordinator, and it is acted on | BG-124 | SF-159 | UC-162 | FR-157, NFR-66 | QAS-168 | DDR-32 | DD-182 | current |

### Review, approval and merge

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-07 Human review and approval before merge | BG-03 | SF-05, SF-06 | UC-03, UC-04 | FR-07, NFR-01, NFR-03 | QAS-02 | DDR-01, DDR-03 | DD-02, DD-06, DD-07, DD-14, DD-17, DD-18, DD-19 | current |
| R-105 Coordinator final step in the task list | BG-60 | SF-92 | UC-95 | FR-90 | QAS-101 | — | DD-113 | current |
| R-110 Failure cause and the next action on a failed run | BG-65 | SF-97 | UC-100 | FR-95 | QAS-106 | — | DD-118 | current |
| R-111 The next step is always shown | BG-66 | SF-98 | UC-101 | FR-96 | QAS-107 | — | DD-119 | current |
| R-156 Make final review, approval, merge, and return to workspace visibly co | BG-111 | SF-143 | UC-146 | FR-141 | QAS-152 | — | DD-166 | current |

### Run visibility and responsiveness

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-17 Functional lightweight intervention UI and README preview | BG-03 | SF-08 | UC-03, UC-04 | FR-14, NFR-10 | QAS-21 | DDR-07 | DD-10, DD-22, DD-23 | current |
| R-28 On-demand orchestration briefing | BG-12 | SF-22 | UC-19 | FR-13, NFR-09 | QAS-20 | DDR-07 | DD-30 | current |
| R-29 At-a-glance agent status lights | BG-12 | SF-23 | UC-20 | FR-13, NFR-10 | QAS-21 | DDR-07 | DD-31 | amended by R-83 |
| R-99 Run and task status at a glance | BG-54 | SF-86 | UC-89 | FR-84 | QAS-95 | — | DD-107 | current |
| R-104 Activity tab reads like the CLI | BG-59 | SF-91 | UC-94 | FR-89 | QAS-100 | — | DD-112 | current |
| R-108 Task list without account usage | BG-63 | SF-95 | UC-98 | FR-93 | QAS-104 | — | DD-116 | current |
| R-109 The coordinator is selectable and shows its work; one name | BG-64 | SF-96 | UC-99 | FR-94 | QAS-105 | — | DD-117 | current |
| R-114 The run's phase beside the workspace name | BG-69 | SF-101 | UC-104 | FR-99 | QAS-110 | — | DD-122 | current |
| R-115 Readable Agent Room text and agent icons sized to their names | BG-70 | SF-102 | UC-105 | FR-100 | QAS-111 | — | DD-123 | current |
| R-159 Show live agent activity, current tool, and time since last output | BG-114 | SF-146 | UC-149 | FR-144, NFR-54 | QAS-155 | DDR-20 | DD-169 | current |
| R-160 Keep the Agent Room focus on the work that is in progress | BG-115 | SF-147 | UC-150 | FR-145 | QAS-156 | DDR-21 | DD-170 | current |
| R-165 The task detail column can be folded and still never hides a decision | BG-120 | SF-152 | UC-155 | FR-150, NFR-59 | QAS-161 | DDR-26 | DD-175 | current |
| R-167 Every coordinator judgment is shown, whatever the level of trust | BG-122 | SF-154 | UC-157 | FR-152, NFR-61 | QAS-163 | DDR-28 | DD-177 | current |
| R-182 Request timeouts can be traced | BG-137 | SF-170 | UC-173 | FR-170 | QAS-180 | — | DD-193 | current |
| R-184 Idle polling does not report false request timeouts | BG-139 | SF-172 | UC-175 | FR-176, NFR-75 | QAS-182 | DDR-39 | DD-196 | current |
| R-191 Requests never wait on a slow step of another request | BG-146 | SF-179 | UC-182 | FR-183 | QAS-189 | — | DD-203 | current |
| R-194 The coordinator Activity tab shows the run's status, not a repeating l | BG-149 | SF-182 | UC-185 | FR-186 | QAS-192 | — | DD-206 | current |
| R-196 No step of planning or execution looks frozen | BG-151 | SF-184 | UC-187 | FR-188 | QAS-194 | — | DD-208 | current |
| R-198 Every wait of the workspace page counts its time | BG-153 | SF-186 | UC-189 | FR-190 | QAS-196 | — | DD-210 | current |
| R-199 Every press is answered on screen | BG-154 | SF-187 | UC-190 | FR-191 | QAS-197 | — | DD-211 | current |
| R-202 No request can be added whose wait shows nothing | BG-157 | SF-190 | UC-193 | FR-194 | QAS-200 | — | DD-214 | current |

### Coordinator chat

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-161 Chat with the coordinator from the workspace screen | BG-116 | SF-148 | UC-151 | FR-146, NFR-55 | QAS-157 | DDR-22 | DD-171 | amended by R-179, R-181, R-187, R-188, R-192 |
| R-179 The coordinator chat behaves like a CLI session | BG-134 | SF-167 | UC-170 | FR-166, FR-167 | QAS-177 | — | DD-190 | current |
| R-181 The chat keeps going when a reply is slow | BG-136 | SF-169 | UC-172 | FR-169 | QAS-179 | — | DD-192 | current |
| R-187 An unconfirmed chat send is never rolled back | BG-142 | SF-175 | UC-178 | FR-179 | QAS-185 | — | DD-199 | current |
| R-188 The chat coordinator knows the workspace and the latest run | BG-143 | SF-176 | UC-179 | FR-180 | QAS-186 | — | DD-200 | current |
| R-192 The coordinator can be asked while a plan is being made | BG-147 | SF-180 | UC-183 | FR-184 | QAS-190 | — | DD-204 | current |

### Memory, knowledge and coordination signals

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-10 Shared memory, lessons, progress reports | BG-04 | SF-07 | UC-05 | FR-11, NFR-04 | QAS-07 | DDR-05 | DD-08 | current |
| R-11 LLM-wiki knowledge assets | BG-04 | SF-07 | UC-05 | FR-11, NFR-04 | QAS-13 | DDR-05 | DD-21 | current |
| R-27 Cooperative steering through agentmemory | BG-11, BG-12 | SF-24 | UC-21 | FR-11, FR-12, C-05 | QAS-22 | DDR-05 | DD-29 | current |
| R-92 Selectable signal transport with Rust fallback | BG-47 | SF-79 | UC-82 | FR-77, NFR-48 | QAS-88 | — | DD-100 | current |
| R-141 Selected signal instructions | BG-96 | SF-128 | UC-131 | FR-126 | QAS-137 | — | DD-150 | current |
| R-164 Lessons learned at each step are shown and shared with the agents | BG-119 | SF-151 | UC-154 | FR-149, NFR-58 | QAS-160 | DDR-25 | DD-174 | current |

### External sources and messengers

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-43 Messenger integration (planned) | BG-18 | SF-37 | UC-34, UC-35 | FR-28, NFR-17 | QAS-35, QAS-36 | DDR-14 | DD-51 | implemented (phases 1–2) |
| R-62 Jira and Confluence goal imports | BG-23 | SF-52 | UC-51, UC-52 | FR-47, FR-48, NFR-22 | QAS-53, QAS-54, QAS-55 | — | DD-68 | current |
| R-90 Register MCP servers in Settings | BG-45 | SF-77 | UC-80 | FR-75 | QAS-86 | — | DD-98 | current |
| R-134 Coordinator-selected MCP sources and large documents | BG-89 | SF-121 | UC-124 | FR-119 | QAS-130 | — | DD-143 | current |

### Windows and macOS platform behavior

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-23 Cross-platform development continuity | BG-10 | SF-20 | UC-17 | FR-15, NFR-11 | QAS-14 | DDR-08 | DD-27, DD-35, DD-36, DD-37, DD-38 | current |
| R-76 Foreground pickers and console-free desktop execution | BG-31 | SF-63 | UC-66 | FR-61, NFR-32 | QAS-71 | — | DD-84 | current |
| R-82 Picked project paths are used exactly on Windows | BG-37 | SF-69 | UC-72 | FR-67, NFR-38 | QAS-78 | — | DD-90 | current |
| R-87 Readable Windows project paths | BG-42 | SF-74 | UC-77 | FR-72 | QAS-83 | — | DD-95 | merged into R-103 |
| R-103 Readable file paths everywhere | BG-58 | SF-90 | UC-93 | FR-88 | QAS-99 | — | DD-111 | current |
| R-142 responsive native path selection | BG-97 | SF-129 | UC-132 | FR-127 | QAS-138 | — | DD-151 | current |

### Release, update and documentation

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-24 Verified GitHub release artifacts | BG-10 | SF-20 | UC-17 | FR-15, NFR-12 | QAS-18 | DDR-08 | DD-27, DD-35, DD-36, DD-37, DD-38 | current |
| R-36 Product README and separate user manual | BG-15 | SF-30 | UC-27 | FR-21, NFR-14 | QAS-28 | DDR-11 | DD-42 | current |
| R-41 Release notes summarize changes | BG-10 | SF-35 | UC-32 | FR-26 | QAS-33 | DDR-08 | DD-49 | current |
| R-81 Install guide for companion agent tools | BG-36 | SF-68 | UC-71 | FR-66, NFR-37 | QAS-77 | — | DD-89 | current |
| R-93 Read the running app version | BG-48 | SF-80 | UC-83 | FR-78 | QAS-89 | — | DD-101 | current |
| R-125 Release-matched English and Korean manuals | BG-80 | SF-112 | UC-115 | FR-110 | QAS-121 | — | DD-134 | amended by R-144, R-197, R-200 |
| R-126 The app updates itself from GitHub releases | BG-81 | SF-113 | UC-116 | FR-111 | QAS-122 | — | DD-135 | current |
| R-133 Read actual release notes inside the update dialog | BG-88 | SF-120 | UC-123 | FR-118 | QAS-129 | — | DD-142 | current |
| R-137 Codex transport recovery | BG-92 | SF-124 | UC-127 | FR-122 | QAS-133 | — | DD-146 | current |
| R-138 in-app manual | BG-93 | SF-125 | UC-128 | FR-123 | QAS-134 | — | DD-147 | amended by R-178 |
| R-144 Readable offline HTML manuals | BG-99 | SF-131 | UC-134 | FR-129 | QAS-140 | — | DD-153 | current |
| R-155 Maintain a local company build label | BG-110 | SF-142 | UC-145 | FR-140, NFR-52 | QAS-151 | — | DD-165 | current |
| R-178 Guide and manual open from Settings | BG-133 | SF-166 | UC-169 | FR-165 | QAS-176 | — | DD-189 | current |
| R-197 The manual and README show the current release | BG-152 | SF-185 | UC-188 | FR-189 | QAS-195 | — | DD-209 | current |
| R-200 The release docs are checked by tests | BG-155 | SF-188 | UC-191 | FR-192 | QAS-198 | — | DD-212 | current |

### Engineering policy and resource budget

| Requirement | BG | SF | UC | FR / NFR / C | QAS | DDR | DD | Status |
|---|---|---|---|---|---|---|---|---|
| R-12 Rust and lightweight local architecture | BG-05 | SF-12 | UC-17 | FR-15, NFR-12, C-02, C-03, C-04 | QAS-14 | DDR-06, DDR-08 | DD-01, DD-09 | current |
| R-13 Development workflow and TDD | BG-05 | SF-12 | UC-17 | FR-15, NFR-12, C-02, C-03, C-04 | QAS-14 | DDR-06, DDR-08 | DD-01, DD-09 | current |
| R-14 Use development tools | BG-05 | SF-12 | UC-17 | FR-15, NFR-12, C-02, C-03, C-04 | QAS-14 | DDR-06, DDR-08 | DD-01, DD-09 | current |
| R-15 Review internal distribution licenses | BG-05 | SF-12 | UC-17 | FR-15, NFR-12, C-02, C-03, C-04 | QAS-14 | DDR-06, DDR-08 | DD-01, DD-09 | current |
| R-16 Documentation language and public README | BG-05 | SF-12 | UC-17 | FR-15, NFR-12, C-02, C-03, C-04 | QAS-14 | DDR-06, DDR-08 | DD-01, DD-09 | current |
| R-35 Maintainable source layout | BG-05 | SF-12 | — | NFR-07, C-04 | QAS-12 | DDR-04, DDR-06 | DD-41 | current |
| R-65 Idle footprint within the resource budget | BG-10 | — | — | NFR-05 | QAS-58 | DDR-06 | DD-71 | current |
| R-94 Bound and clear disposable caches | BG-49 | SF-81 | UC-84 | FR-79 | QAS-90 | — | DD-102 | current |
| R-151 Rust-only application and release toolchain | BG-106 | SF-138 | UC-141 | FR-136 | QAS-147 | — | DD-160 | in progress (not on main) |
| R-171 CI tests synchronize on observable readiness | BG-123 | SF-158 | UC-161 | FR-156, NFR-65 | QAS-167 | DDR-31 | DD-181 | current |
| R-174 New Rust code follows a short, evidence-based review policy | BG-129 | SF-162 | UC-165 | FR-160, NFR-70, C-06 | QAS-171 | DDR-34 | DD-185 | current |
| R-175 Every unsafe operation states why it is sound | BG-130 | SF-163 | UC-166 | FR-161, NFR-71 | QAS-172 | DDR-35 | DD-186 | current |
| R-176 Tests do not mutate the process environment unsoundly | BG-131 | SF-164 | UC-167 | FR-162, NFR-72 | QAS-173 | DDR-36 | DD-187 | current |

## Reserved and unused identifiers

<!-- reserved identifiers: not definitions, not references -->
Do not reuse these numbers. Retired when the Rust migration traces were renumbered to avoid
collisions with `main` (commit 9f951a57): R 173, BG 125–128, SF 160–161, UC 163–164, FR 158–159,
NFR 50 and 67–69, QAS 169–170, DDR 33, DD 183–184. Never assigned: BG 6, SF 13–17, UC 6–8 and
12–14 (early traces that cited them now cite the defined use cases), NFR 43–46 and 51, QAS 15.
Held by the Rust migration line (`codex/issue-9-native-ui-parity`, GitHub #9 and the issues split
from it) and not yet on `main`; use them only when that line merges, and start new `main` entries
above them: R 203–208, BG 158–163, SF 191–196, UC 194–200, FR 195–202, NFR 76–91, QAS 201–207,
DDR 40–57, DD 215–220. Next free on `main`: R 211, BG 166, SF 199, UC 203, FR 205, NFR 92,
QAS 210, DDR 58, DD 223. Its R 205, R 207 and R 208 port GitHub #323, #331 and #341, which `main`
traces as amendments of R-110, R-126 and R-183; when the line merges, keep both and mark the
amended `main` requirement with `Status: Amended by` the ported one.
<!-- end reserved identifiers -->

| R-211 | Goal row ownership and context | BG-166 | SF-199 | UC-203 | FR-205 / NFR-92 | QAS-210 | DD-223 |
