# Architecture verification

Date: 2026-11-03. Issue: GitHub #349. Baseline: `main` at v0.1.58 (b747e392) plus this change.

This record checks the implementation against the consolidated architecture in
[README.md](README.md): for every requirement still in force (not superseded or merged), it names
the tests that exercise it and says whether the behavior is present on `main`.

## Method

1. **Trace integrity** — `tests/architecture_trace.rs` checks that every identifier is defined
   once in its own document, every cited identifier is defined, every requirement is in the README
   matrix once, no `Status:` line points to a superseded entry, and every issue-driven unit of
   the implementation plan traces to the architecture. A mutation run (a duplicated heading, an
   undefined reference, a status pointing to a superseded requirement and an untraced plan unit)
   made four of its six tests fail, so the checks are not vacuous.
2. **Evidence** — a test counts as evidence for a requirement when it cites the requirement, its
   use case, QAS, design decision or GitHub issue, or when the plan unit for that requirement
   names it; requirements that no test cites were matched to tests by reading the test names and
   their first assertions, and checked against the code.
3. **Result** — the whole suite (`cargo test --all-targets --locked --no-fail-fast`, which also
   runs the Node page tests) is run on macOS (792 passed, 0 failed, 15 ignored); "passes" means the named tests are
   part of that green run. `tests/agent_master_list.rs`, added afterwards, passes on its own and
   fails when `master::adopt` is disabled. Windows runs in CI on the pull request.
4. **Gaps** — requirements without an implementation or without a test, found by the evidence
   step and by plan units whose implementation boxes were still unchecked.

## Corrections made to the architecture

| Problem found | Correction |
|---|---|
| One chain of identifiers had two meanings: "Deleting workspaces needs no replacement name" reused R-124, BG-79, SF-111, UC-114, FR-109, QAS-120 and DD-133 | Renumbered to R-209, BG-164, SF-197, UC-201, FR-203, QAS-208, DD-221 (docs, plan and `tests/workspace_delete_default.rs`) |
| R-143, R-145, R-146 and R-149 sections were titled by their DD name in every document, so 20 BG/SF/UC/FR/QAS identifiers were cited but never defined | Restored the identifier headings in all seven documents |
| BG-09 defined twice (session models for R-20, one local application for R-21) | Merged into one goal |
| FR-140 / NFR-52 body said `FR-138:` / `NFR 50:` (FR-138 is task timeouts) | Corrected |
| DD-126 cited by docs/05 and docs/07, DDR-19 and DDR-36 cited by chains, never defined | Defined from the plan and the QAS they serve |
| DD-67, DD-68, DD-141 and BG-22 amendments written as second headings with the same identifier; amendment paragraphs (R-43, R-61, R-70, R-86, BG-26) far from their entries | Moved into the entry they amend |
| DD-68 UI amendment copied into docs/01 and docs/04 with a 44 × 44 px target while docs/06 and the code (`#start-settings-button`, 32 × 32 px) agree on 32 × 32 | Deleted the stale copies |
| Early traces cited use cases 6 and 8, which were never defined | Pointed to the defined use cases |
| Compound headings put the second identifier mid-title (`R-137 … / R-138 …`) | `R-137 / R-138 …` form, so both are definitions |
| 24 requirements were changed by later ones without saying so (for example R-66 "new workspaces start without agents" against R-101 "master list"; R-69 reset time in the badge against R-130 tooltip; R-96 "permission is not chosen in the profile" against R-122 "highest permission by default, the operator can lower it"; R-77/R-79 Git required against R-80/R-124) | `Status:` lines on the requirement, its use case and its QAS |
| R-43 said "planned, not implemented" although phases 1 and 2 are in `src/messenger/` | Status corrected |
| R-151 (Rust-only) read as met, although `main` ships the JS pages | Status: in progress on the migration line |
| R-201 required the card to show the usage-query time; GitHub #321 removed it at the user's request | R-201, UC-192 and QAS-199 amended |
| GitHub #337 and #341 and 38 other issue units existed only in the plan | Each plan unit traces to the requirement it refines; #337 amends R-146, #341 amends R-183 |
| The Rust migration line (`codex/issue-9-native-ui-parity`, where #9 is being split into issues) already uses R 203–208, BG 158–163, SF 191–196, UC 194–200, FR 195–202, NFR 76–91, QAS 201–207, DDR 40–57 and DD 215–220, and its R 203 chain had exactly the numbers first chosen for the renumbered chain above | The renumbered chain starts above that range (R-209 … DD-221); the README lists the range as held by that line, with the next free numbers on `main` and how its ports of #323, #331 and #341 relate to the `main` amendments |
| README index stopped at R-59 / UC-48 / DD-65 and its matrix missed about 100 requirements | Rebuilt: real ranges, consolidation rules, a complete matrix grouped by area, retired numbers |

## Gaps

| Requirement | Kind | Detail |
|---|---|---|
| R-151 | in progress | Rust-only UI is on the migration line (#9); `main` ships the JS pages. |
| R-174 | policy | Applied in review and by the CI Clippy gates (docs/13); not a test. |

## Requirement evidence

### Agent CLIs and profiles

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-01 Reuse existing CLI authentication and subscriptions | current | `architecture_trace.rs` | passes |
| R-02 Control subprocess I/O | current | `architecture_trace.rs` | passes |
| R-03 Register three native CLIs and arbitrary CLIs | current | `architecture_trace.rs` | passes |
| R-20 Query model catalogs through existing authenticated CLI subp | current | `empty_default_agents.rs`, `profile_persistence.rs`, `startup.rs`, `workspace_navigation.rs` | passes |
| R-31 Planning with unverifiable company CLIs | current | `agent_tools_guide.rs`, `app_brand.rs`, `baseline_commit.rs`, `korean_input.rs` (+14) | passes |
| R-34 Recognizable agent icons | current | `icons.rs`, `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs` (+5) | passes |
| R-48 Agent registrations are saved immediately | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+14) | passes |
| R-49 Model lists prepared per agent | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+12) | passes |
| R-51 Compact agent list | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `icons.rs` (+14) | passes |
| R-66 New workspaces start without agents | superseded by R-101 | — (see the replacing requirement) | — |
| R-67 Agent preferences and consistent native identity | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+8) | passes |
| R-68 Provider choice defines the whole agent profile | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+8) | passes |
| R-71 Fallback model choices are always offered | current | `fallback_models.rs` | passes |
| R-72 Windows CLI metadata and provider visibility | current | `windows_icons.rs`, `windows_metadata.rs` | passes |
| R-73 Self-hosted CLI model catalogs without discovery | current | `model_catalogs.rs`, `test_custom_model_catalog.py` | passes |
| R-78 Nonblocking agent registration and metadata | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+7) | passes |
| R-91 Save an edited agent profile visibly | current | `profile_persistence.rs`, `test_profile_save_feedback.py` | passes |
| R-101 Agent CLI list is a master list | current | `agent_master_list.rs`, `workspace_registry.rs` | passes |
| R-102 Reorder agent cards | current | `agent_order_runtime.cjs` (added; the drag gesture itself is not driven) | passes |
| R-132 Custom CLIs that approve their own tools can run | current | `self_approving_cli.rs` | passes |
| R-154 Custom CLI behavior belongs to its agent profile | current | `protocol.rs`, `quota_custom.rs` | passes |
| R-158 Configure custom CLI models, execution options, and result d | current | `declarative_cli_profiles.rs` | passes |
| R-183 A standard, schema-based CLI interface | amended by GitHub #341 | `custom_cli_denials.rs`, `permission_denied_status_runtime.cjs`, `acio_runtime.rs`, `adapter_golden.rs`, `agent_settings.rs`, `cli_io.rs` (+8) | passes |
| R-186 Every agent prompt has one standard form and its reply is ch | current | `coordinator_format_retry.rs`, `prompt_form.rs`, `prompt_order.rs` | passes |
| R-189 A valid JSON block is found even after an unfinished one | current | `coordinator_document.rs` | passes |

### Account capacity and availability

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-18 Persist subscription/quota/transient restrictions, source ev | current | `architecture_trace.rs` | passes |
| R-19 Support multiple independent logical/provider sessions per p | current | `native_session.rs`, `ownership.rs` | passes |
| R-22 Remaining subscription quota | current | `agent_tools_guide.rs`, `app_brand.rs`, `korean_input.rs`, `live_tool_decision.rs` (+17) | passes |
| R-26 Per-agent availability fallback | current | `fallback.rs`, `fallback_models.rs` | passes |
| R-60 Collect Claude subscription quota without extra generation | amended by R-97, R-148, R-201 | `claude_quota.rs`, `claude_usage.rs` | passes |
| R-61 Inline agent quota — Show paid CLI remaining capacity beside | amended by R-64, R-108, R-130 | `quota_display.rs`, `startup_quota.rs` | passes |
| R-63 Agent-specific refresh and action icons | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+7) | passes |
| R-64 Compact quota windows | amended by R-130 | `quota_display.rs` | passes |
| R-69 Show when exhausted quota returns | superseded by R-130 | — (see the replacing requirement) | — |
| R-75 Enterprise quota budgets | current | `quota.rs`, `quota_custom.rs` | passes |
| R-85 Enterprise usage as a percentage | current | `quota.rs`, `quota_display.rs` | passes |
| R-86 Claude capacity on Windows reports why it is unavailable | current | `claude_usage.rs`, `support/terminal_quota_agent.rs`, `test_planning_feedback.py`, `windows_metadata.rs` | passes |
| R-97 Usage query accepts the Claude folder-trust prompt | current | `claude_prompt.rs`, `windows_metadata.rs` | passes |
| R-127 Durable quota resume and five-hour fallback | current | `availability_timer.rs`, `availability_watcher.rs`, `quota_reset_zones.rs` | passes |
| R-130 Compact quota cards and readable step explanations | current | `quota_display.rs` | passes |
| R-136 Approval readiness and Claude API usage limits | current | `claude_api_limit.rs`, `claude_api_workflow.rs` | passes |
| R-145 Saved-project Claude quota retry | current | `quota_refresh.rs`, `test_quota_project.py` | passes |
| R-148 Claude prompt-compatible metadata collection | current | `claude_prompt.rs` | passes |
| R-201 A slow Claude usage query is waited for and shown | amended by GitHub #321 | `claude_prompt.rs`, `claude_usage.rs`, `claude_wait.rs`, `progress_runtime.cjs` (+2) | passes |

### Workspaces, settings and interface

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-21 Native startup workspace | amended by R-53 | `baseline_commit.rs`, `empty_default_agents.rs`, `idle_footprint.rs`, `korean_input.rs` (+8) | passes |
| R-32 Korean text entry | current | `baseline_commit.rs`, `icons.rs`, `korean_input.rs`, `live_tool_decision.rs` (+8) | passes |
| R-33 Leave the workspace without saving | current | `empty_default_agents.rs`, `icons.rs`, `korean_input.rs`, `live_tool_decision.rs` (+9) | passes |
| R-38 Discoverable workspace settings | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+13) | passes |
| R-39 Multiple workspaces | amended by R-149 | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+12) | passes |
| R-40 Named, registered workspaces | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+12) | passes |
| R-50 Workspace changes keep a reviewed plan | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+12) | passes |
| R-52 Shareable workspace settings (export/import) | current | `workspace_share.rs` | passes |
| R-58 Export/import includes slug archives | current | `architecture_trace.rs`, `archive.rs`, `phase_link.rs`, `phased_planning.rs` (+7) | passes |
| R-70 Interface language | current | `coordinator.rs`, `fallback_models.rs`, `i18n_runtime.cjs`, `interface_language.rs` (+1) | passes |
| R-88 Reliable named workspace creation | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+7) | passes |
| R-89 Delete an unwanted workspace | current | `agent_tools_guide.rs`, `app_brand.rs`, `empty_default_agents.rs`, `macos_bundle.rs` (+12) | passes |
| R-95 Abandon a blocked planning workspace | amended by R-209 | `planning_stop.rs` | passes |
| R-98 Planning start and workspace naming feedback | amended by R-209 | `test_planning_feedback.py` | passes |
| R-128 Selectable interface themes | current | `agent_tools_guide.rs`, `app_brand.rs`, `interface_theme.rs`, `macos_bundle.rs` (+9) | passes |
| R-131 Readable and focused Settings | amended by R-178 | `agent_tools_guide.rs`, `test_signal_settings_ui.py` | passes |
| R-139 Workspace and Agent Room feedback refinements | current | `goal_sources_ui.rs`, `room_quota_selection.rs`, `started_phase_label.rs` | passes |
| R-143 Workspace transfer menu | current | `workspace_menu.rs`, `workspace_share.rs` | passes |
| R-147 Retained completed rooms do not block workspace creation | amended by R-149 | `workspace_after_run.rs` | passes |
| R-149 Active workspace navigation | current | `workspace_active.rs`, `workspace_navigation.rs` | passes |
| R-150 One project workspace with multiple goal records | current | `project_goals.rs` | passes |
| R-180 Screens show only information the operator can use | current | `static_labels.rs` | passes |
| R-185 A deleted default workspace stays deleted | current | `first_workspace_runtime.cjs`, `launch_workspace.rs`, `workspace_settings.rs` | passes |
| R-209 Deleting workspaces needs no replacement name | current | `workspace_delete_default.rs`, `workspace_registry.rs` | passes |

### Planning

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-04 Decompose a goal into scoped tasks (originally exactly three | current | `agent_tools_guide.rs`, `app_brand.rs`, `idle_footprint.rs`, `korean_input.rs` (+14) | passes |
| R-25 Deterministic supervision and optional AI coordination | current | `gates.rs`, `workflow.rs` | passes |
| R-37 Plan without an LLM coordinator | amended by R-53 | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `manual_plan.rs` (+9) | passes |
| R-42 Explain and resolve an unclean repository | current | `baseline_commit.rs`, `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs` (+3) | passes |
| R-44 Ignore operating-system files and continue | current | `baseline_commit.rs`, `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs` (+3) | passes |
| R-45 Visible planning progress | amended by R-83, R-135, R-196 | `agent_tools_guide.rs`, `app_brand.rs`, `korean_input.rs`, `live_tool_decision.rs` (+18) | passes |
| R-46 Human-readable task instructions | current | `agent_tools_guide.rs`, `app_brand.rs`, `baseline_commit.rs`, `korean_input.rs` (+17) | passes |
| R-47 Agent role rules before planning | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `manual_plan.rs` (+10) | passes |
| R-53 Plans with one to twelve tasks | current | `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs`, `native_claude_workflow.rs` (+3) | passes |
| R-54 Read an existing plan document | current | `manual_plan.rs`, `plan_size.rs`, `role_rules.rs` | passes |
| R-55 Goals become slugs with a work tree | amended by R-74, R-84, R-100 | `architecture_trace.rs`, `archive.rs`, `phase_link.rs`, `phased_planning.rs` (+7) | passes |
| R-56 Analysis first, then sequential phases | current | `architecture_trace.rs`, `archive.rs`, `phase_dependencies.rs`, `phase_link.rs` (+9) | passes |
| R-57 Progress tracking without tree mismatches | current | `architecture_trace.rs`, `archive.rs`, `phase_link.rs`, `phased_planning.rs` (+7) | passes |
| R-59 Editable plan rules | current | `architecture_trace.rs`, `archive.rs`, `goal_controls_runtime.cjs`, `phase_link.rs` (+8) | passes |
| R-74 Slugs are named automatically | amended by R-84 | `archive.rs`, `phase_link.rs`, `phased_planning.rs`, `phases.rs` (+7) | passes |
| R-77 Save new project folders before Git setup | superseded by R-80, R-124 | — (see the replacing requirement) | — |
| R-79 Approved Git setup for new project planning | superseded by R-80, R-124 | — (see the replacing requirement) | — |
| R-83 Planning start and progress are truthful | current | `planning_progress.rs`, `test_planning_feedback.py` | passes |
| R-84 English slug names | current | `slugs.rs`, `startup_folder_plan.rs` | passes |
| R-100 Goal folders directly in the project folder | current | `slug_folder.rs` | passes |
| R-112 Plan review and run completion are organised by phase | current | `plan_phase_cards.rs` | passes |
| R-116 Task cards in Review the plan fold to one line | current | `plan_phase_cards.rs` | passes |
| R-117 Review the whole phased plan up front | current | `plan_phase_cards.rs` | passes |
| R-129 Plans know whether the project uses Git, and verification co | current | `folder_workflow.rs` | passes |
| R-135 Clear retry feedback and show coordinator activity | current | `planning_progress_runtime.cjs`, `test_planning_progress.py` | passes |
| R-146 Schema-constrained planning replies | amended by GitHub #337 | `planning_failure_detail.rs`, `planning_schema.rs` | passes |
| R-152 Recover clearly from a slow plan retry | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+7) | passes |
| R-177 The operator can improve a goal with the bundled prompt-opti | current | `optimize_runtime.cjs`, `optimize_ui.rs`, `prompt_optimizer.rs`, `prompt_optimizer_ask.rs` (+2) | passes |
| R-190 The planning contract matches plan validation | current | `plan.rs`, `planning_overlap_repair.rs`, `planning_schema.rs` | passes |
| R-193 A task's facts line stays readable in the plan review | current | `task_profile_layout.rs` | passes |
| R-195 The next phase can always be reviewed after a phase is done | current | `start_pending_runtime.cjs` | passes |

### Execution, safety and recovery

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-05 Optional worktree separation and parallel execution | current | `agent_tools_guide.rs`, `app_brand.rs`, `idle_footprint.rs`, `korean_input.rs` (+13) | passes |
| R-06 Collect results, diffs, and tests | current | `agent_tools_guide.rs`, `app_brand.rs`, `idle_footprint.rs`, `korean_input.rs` (+13) | passes |
| R-08 Approve dangerous actions | current | `agent_tools_guide.rs`, `app_brand.rs`, `baseline_commit.rs`, `cooperative_handoff.rs` (+16) | passes |
| R-09 Stop, resume, feedback, auditing | current | `agent_tools_guide.rs`, `app_brand.rs`, `baseline_commit.rs`, `cooperative_handoff.rs` (+16) | passes |
| R-30 Live human tool decisions | current | `baseline_commit.rs`, `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs` (+4) | passes |
| R-80 Work in ordinary project folders | amended by R-124 | `folder_workflow.rs`, `snapshot_without_git.rs` | passes |
| R-96 Agents work inside the workspace sandbox; the coordinator de | amended by R-107, R-122 | `full_access_profiles.rs`, `guard_policy.rs` | passes |
| R-106 Decide a live tool request from its window | current | `native_ui_tool_dialog.rs`, `test_tool_approval_dialog.py` | passes |
| R-107 Coordinator decides routine requests and records each decisi | current | `coordinator_decisions.rs`, `native_guard_coordinator_requests.rs` | passes |
| R-113 A blocking verification finding fails integration verificati | current | `verification_blockers.rs` | passes |
| R-118 A run survives closing or losing the desktop app | current | `close_window.rs`, `close_dialog_runtime.cjs` (GitHub #377) | passes |
| R-119 Interrupted runs recover and resume automatically | current | `crash_recovery.rs`, `recovery.rs`, `stop_resume.rs` | passes |
| R-210 One goal screen with a single six-step stepper | current | `unified_stepper_runtime.cjs`, `rail_tabs_runtime.cjs`, `inspector_fold_runtime.cjs`, `goal_panel_runtime.cjs`, `result_popup_runtime.cjs`, `room_task_rows.rs` (GitHub #390, #392, #394, #396, #398, #400; steps 1-2 of the approved goal-screen design #387) | passes |
| R-120 Interruptions and silent workers are visible | amended by R-159 | `interruption_history.rs`, `thinking_collapse_runtime.cjs` (GitHub #385) | passes |
| R-121 Reassign tasks that have not started while the run is workin | current | `pending_edit.rs` | passes |
| R-122 Agents run with the highest permission by default | current | `coordinator.rs`, `coordinator_chat_check.rs`, `full_access_profiles.rs`, `progress_runtime.cjs` | passes |
| R-123 The coordinator resolves verification failures | current | `repair.rs`, `support/mock_agent.rs` | passes |
| R-124 Execute in the selected project without adding Git metadata | current | `folder_execution.rs`, `test_folder_baseline.py` | passes |
| R-140 Runs keep going and stay visible | current | `test_busy_buttons.py`, `transient_capacity.rs` | passes |
| R-153 Explain task timeouts and resume automatically | current | `process_timeout.rs`, `room_timeout_banner.rs` | passes |
| R-157 Preserve task-state writes during transient SQLite contentio | current | `workflow_load_readonly.rs` | passes |
| R-162 Recover from an out-of-scope integration failure through the | current | `scope_recovery.rs`, `scope_recovery_room.rs`, `scope_recovery_runtime.cjs`, `scope_recovery_ui.rs` | passes |
| R-163 The coordinator decides when a task stops without finishing | current | `repair.rs`, `rework.rs` | passes |
| R-166 The coordinator guards the goal, the plan rules and the scop | current | `amendments.rs`, `charter.rs`, `coordinator_decisions.rs`, `launch_gate.rs` (+1) | passes |
| R-168 A run does not start when it cannot work | current | `amendments.rs`, `charter.rs`, `coordinator_decisions.rs`, `launch_gate.rs` (+1) | passes |
| R-169 The coordinator amends the plan within limits, and every ame | current | `amendments.rs`, `charter.rs`, `coordinator_decisions.rs`, `launch_gate.rs` (+1) | passes |
| R-170 A run keeps its charter, and the coordinator cannot drift fr | current | `amendments.rs`, `charter.rs`, `coordinator_decisions.rs`, `launch_gate.rs` (+1) | passes |
| R-172 The operator can send feedback to the coordinator, and it is | current | `coordinator_request.rs`, `support/amend_agent.rs` | passes |

### Review, approval and merge

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-07 Human review and approval before merge | current | `agent_tools_guide.rs`, `app_brand.rs`, `baseline_commit.rs`, `cooperative_handoff.rs` (+15) | passes |
| R-105 Coordinator final step in the task list | current | `room_task_rows.rs` | passes |
| R-110 Failure cause and the next action on a failed run | current | `failure_output.rs`, `progress_runtime.cjs` | passes |
| R-111 The next step is always shown | current | `approval_feedback_runtime.cjs`, `test_approval_feedback.py` | passes |
| R-156 Make final review, approval, merge, and return to workspace  | current | `diff_review_runtime.cjs`, `ui_diff_review.rs` | passes |

### Run visibility and responsiveness

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-17 Functional lightweight intervention UI and README preview | current | `agent_tools_guide.rs`, `app_brand.rs`, `cooperative_handoff.rs`, `icons.rs` (+19) | passes |
| R-28 On-demand orchestration briefing | current | `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs`, `native_claude_workflow.rs` (+7) | passes |
| R-29 At-a-glance agent status lights | amended by R-83 | `icons.rs`, `korean_input.rs`, `live_tool_decision.rs`, `native_claude.rs` (+9) | passes |
| R-99 Run and task status at a glance | current | `room_task_rows.rs` | passes |
| R-104 Activity tab reads like the CLI | current | `room_activity_text.cjs`, `room_activity_text.rs` | passes |
| R-108 Task list without account usage | current | `room_quota_selection.rs` | passes |
| R-109 The coordinator is selectable and shows its work; one name | current | `coordinator_board.cjs` | passes |
| R-114 The run's phase beside the workspace name | current | `run_phase_title_runtime.cjs` (added) | passes |
| R-115 Readable Agent Room text and agent icons sized to their name | current | `room_task_rows.rs` | passes |
| R-159 Show live agent activity, current tool, and time since last  | current | `activity_record.rs`, `activity_tool.rs`, `room_activity_snapshot.rs`, `room_activity_text.cjs` | passes |
| R-160 Keep the Agent Room focus on the work that is in progress | current | `room_follow_focus.cjs`, `room_follow_focus.rs` | passes |
| R-165 The task detail column can be folded and still never hides a | current | `inspector_fold.rs`, `inspector_fold_runtime.cjs` | passes |
| R-167 Every coordinator judgment is shown, whatever the level of t | current | `amendments.rs`, `charter.rs`, `coordinator_decisions.rs`, `launch_gate.rs` (+1) | passes |
| R-182 Request timeouts can be traced | current | `request_diagnostics.rs` | passes |
| R-184 Idle polling does not report false request timeouts | current | `idle_poll_timeout_runtime.cjs`, `poll_failure_runtime.cjs`, `test_idle_poll_timeout.py` | passes |
| R-191 Requests never wait on a slow step of another request | current | `chat_runtime.cjs`, `git_head.rs`, `request_steps.rs`, `snapshot_without_git.rs` | passes |
| R-194 The coordinator Activity tab shows the run's status, not a r | current | `coordinator_board.cjs` | passes |
| R-196 No step of planning or execution looks frozen | current | `coordinator_request.rs`, `node_runtime.rs`, `optimize_runtime.cjs`, `progress_runtime.cjs` (+4) | passes |
| R-198 Every wait of the workspace page counts its time | current | `chat_runtime.cjs`, `mcp_bindings_runtime.cjs`, `optimize_runtime.cjs`, `progress_runtime.cjs` | passes |
| R-199 Every press is answered on screen | current | `idle_poll_timeout_runtime.cjs`, `progress_runtime.cjs`, `request_wait_runtime.cjs` | passes |
| R-202 No request can be added whose wait shows nothing | current | `request_kinds_runtime.cjs` | passes |

### Coordinator chat

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-161 Chat with the coordinator from the workspace screen | amended by R-179, R-181, R-187, R-188, R-192 | `chat_ask.rs`, `chat_prompt.rs`, `chat_runtime.cjs`, `chat_ui.rs` (+1) | passes |
| R-179 The coordinator chat behaves like a CLI session | current | `chat_history.rs`, `chat_prompt.rs`, `chat_runtime.cjs`, `chat_ui.rs` (+1) | passes |
| R-181 The chat keeps going when a reply is slow | current | `chat_async.rs`, `chat_runtime.cjs` | passes |
| R-187 An unconfirmed chat send is never rolled back | current | `chat_runtime.cjs` | passes |
| R-188 The chat coordinator knows the workspace and the latest run | current | `chat_async.rs`, `chat_context.rs` | passes |
| R-192 The coordinator can be asked while a plan is being made | current | `chat_running.rs`, `chat_runtime.cjs`, `chat_slot.rs` | passes |

### Memory, knowledge and coordination signals

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-10 Shared memory, lessons, progress reports | current | `memory.rs`, `memory_transport.rs` | passes |
| R-11 LLM-wiki knowledge assets | current | `knowledge.rs` | passes |
| R-27 Cooperative steering through agentmemory | current | `cooperative_handoff.rs`, `planning_progress.rs`, `slug_page.rs`, `slugs.rs` | passes |
| R-92 Selectable signal transport with Rust fallback | current | `cooperative.rs`, `memory.rs`, `signal_transport.rs`, `test_signal_settings_ui.py` | passes |
| R-141 Selected signal instructions | current | `signal_prompt.rs` | passes |
| R-164 Lessons learned at each step are shown and shared with the a | current | `step_lessons.rs` | passes |

### External sources and messengers

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-43 Messenger integration (planned) | implemented (phases 1–2) | `architecture_trace.rs`, `messenger.rs`, `messenger_inbound.rs`, `messenger_renotify.rs` (+1) | passes |
| R-62 Jira and Confluence goal imports | current | `messenger.rs`, `messenger_settings.rs` | passes |
| R-90 Register MCP servers in Settings | current | `mcp_config.rs`, `mcp_remote.rs` | passes |
| R-134 Coordinator-selected MCP sources and large documents | current | `mcp_auto_sources.rs`, `mcp_large_sources.rs` | passes |

### Windows and macOS platform behavior

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-23 Cross-platform development continuity | current | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+11) | passes |
| R-76 Foreground pickers and console-free desktop execution | current | `windows_console.rs`, `windows_desktop.rs` | passes |
| R-82 Picked project paths are used exactly on Windows | current | `picker_output.rs`, `windows_console.rs` | passes |
| R-87 Readable Windows project paths | merged into R-103 | — (see the replacing requirement) | — |
| R-103 Readable file paths everywhere | current | `i18n_runtime.cjs`, `mcp_bindings_runtime.cjs`, `ui_messages.py` | passes |
| R-142 responsive native path selection | current | `picker_output.rs` | passes |

### Release, update and documentation

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-24 Verified GitHub release artifacts | current | `agent_tools_guide.rs`, `app_brand.rs`, `icons.rs`, `macos_bundle.rs` (+13) | passes |
| R-36 Product README and separate user manual | current | `public_docs.rs` | passes |
| R-41 Release notes summarize changes | current | `workspace_registry.rs` | passes |
| R-81 Install guide for companion agent tools | current | `agent_tools_guide.rs` | passes |
| R-93 Read the running app version | current | `titlebar_version.rs` | passes |
| R-125 Release-matched English and Korean manuals | amended by R-144, R-197, R-200 | `public_docs.rs`, `release_docs.rs` | passes |
| R-126 The app updates itself from GitHub releases | current | `progress_runtime.cjs`, `update.rs`, `update_blockers.rs` | passes |
| R-133 Read actual release notes inside the update dialog | current | `update_notes.rs` | passes |
| R-137 Codex transport recovery | current | `codex_reconnect.rs` | passes |
| R-138 in-app manual | amended by R-178 | `manual_menu.rs` | passes |
| R-144 Readable offline HTML manuals | current | `manual_renderer.rs` | passes |
| R-155 Maintain a local company build label | current | `release_package.rs` | passes |
| R-178 Guide and manual open from Settings | current | `agent_tools_guide.rs` | passes |
| R-197 The manual and README show the current release | current | `manual_renderer.rs`, `ui_live_bridge.rs` | passes |
| R-200 The release docs are checked by tests | current | `claude_wait.rs`, `progress_runtime.cjs`, `release_docs.rs`, `release_notes.rs` (+3) | passes |

### Engineering policy and resource budget

| Requirement | Status | Test evidence | Result |
|---|---|---|---|
| R-12 Rust and lightweight local architecture | current | `agent_tools_guide.rs`, `app_brand.rs`, `icons.rs`, `idle_footprint.rs` (+11) | passes |
| R-13 Development workflow and TDD | current | `agent_tools_guide.rs`, `app_brand.rs`, `icons.rs`, `idle_footprint.rs` (+11) | passes |
| R-14 Use development tools | current | `agent_tools_guide.rs`, `app_brand.rs`, `icons.rs`, `idle_footprint.rs` (+11) | passes |
| R-15 Review internal distribution licenses | current | `agent_tools_guide.rs`, `app_brand.rs`, `icons.rs`, `idle_footprint.rs` (+11) | passes |
| R-16 Documentation language and public README | current | `agent_tools_guide.rs`, `app_brand.rs`, `icons.rs`, `idle_footprint.rs` (+11) | passes |
| R-35 Maintainable source layout | current | `architecture_trace.rs`, `idle_footprint.rs` | passes |
| R-65 Idle footprint within the resource budget | current | `agent_tools_guide.rs`, `app_brand.rs`, `idle_footprint.rs`, `macos_bundle.rs` (+10) | passes |
| R-94 Bound and clear disposable caches | current | `cache_cleanup.rs` | passes |
| R-151 Rust-only application and release toolchain | in progress (not on main) | `agent_tools_guide.rs`, `app_brand.rs`, `macos_bundle.rs`, `native_room_render.rs` (+7) | in progress |
| R-171 CI tests synchronize on observable readiness | current | `support/process_run.rs`, `ui_live_bridge.rs` | passes |
| R-174 New Rust code follows a short, evidence-based review policy | current | — | policy |
| R-175 Every unsafe operation states why it is sound | current | `icons.rs`, `leases.rs` | passes |
| R-176 Tests do not mutate the process environment unsoundly | current | `availability_watcher.rs`, `env_guard.rs`, `support/env.rs`, `windows_icons.rs` | passes |
