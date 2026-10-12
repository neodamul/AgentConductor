# Slug work trees, phases, archives and the idle budget (DD-61 … DD-65, DD-71, DD-72)

Sources: `src/slugs.rs`, `src/slug_progress.rs`, `src/phases.rs`, `src/archive.rs`,
`src/plan_rules.rs`, `src/planner.rs`, `src/workflow/phase_link.rs`,
`src/ui/setup/{slug,share,planning,profiles}.rs`, `src/ui/{startup,room}.js`; tests
`slugs`, `slug_progress`, `phases`, `phased_planning`, `phase_link`, `archive`,
`slug_archives`, `plan_rules`, `idle_footprint`, `empty_default_agents`.

- **Slug = goal.** Goals accumulate as `[a-z0-9_]` slugs with `plan/ phase/ task/
  verification/ completed/` under `<repo>/.agentconductor/<slug>` (excluded through
  `.git/info/exclude`) or `<workspace>/<slug>`. Save never edits the slug list.
- **The tree is the truth.** Progress is computed from where task files are; the verifier
  reports duplicates, undeclared files/phases, missing tasks and out-of-order phases.
  `plan/plan.json` is written last so a half-written plan is never "planned".
- **Phases.** `plan … --phases <slug>` asks for analysis + 1–6 phases; each phase is
  validated like a plan with role rules; only the current phase can run
  (`run … --phase <slug> <id>`); tests passing → verification, merge → completed, by
  single-step atomic renames; a failed move after merge is reported, not rolled back.
- **Archives.** In-house deterministic store-only ZIP (CRC-32, UTF-8 names, fixed 1980
  timestamps, unsafe names refused, staged extraction). Export lists
  `{slug, goal, archive, progress}`; import stages every archive and refuses progress
  mismatches before replacing anything.
- **Plan rules.** `~/.agentconductor/plan-rules.md` created once with defaults, never
  overwritten; workspace text overrides; passed with `--rules`.
- **Idle budget (R-65).** Fixed 1 s polling plus icons in every snapshot cost ≈1.8 % CPU;
  adaptive refresh (1 s active / 5 s idle / paused when hidden) and on-demand icons give
  ≈0.1 % (main) and ≈0.5 % for the whole tree incl. WebKit helpers; RSS ≈107 + 85 MiB.
- **Empty default registry (R-66).** No agents until the operator adds them; tests add
  template agents first.
- **Lessons:** measure before claiming efficiency; never push before clippy passes;
  coordinate shared files with the other agent through signals instead of racing.

## Automatic naming (R-74 / DD-82)

No slug input. `slugs::suggest_at` lower-cases ASCII words; ≤ 48 chars on a word boundary; a goal with any non-English letter or no words → `goal_<yyyymmdd>` (UTC). DD-92 superseded romanization (user: "사다리게임을 만들자 그런데" became `sadari_geimeul_mandeulja_gerondae`, not English): the coordinator returns an English `slug` with the plan, the planner stores a valid one in `slug.txt`, and settle renames a date placeholder folder and its settings entry. `create_slug` skips names whose folder already exists on disk.

