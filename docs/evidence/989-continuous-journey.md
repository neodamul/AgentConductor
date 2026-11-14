# Continuous operator journey (#989)

Source baseline: stable 0.2.11, 986a8cd8d3a42259bd943ee6d2f8e263ae5100eb.

## Plan and scope

Exercise one continuous operator journey across existing workspace, Goal, role, execution and approval use cases. Reuse the assembled production UI and existing deterministic native-bridge fixture. Preserve historical-result, deletion-safety and next-Phase checks; add intervention and restored-state assertions. No production implementation change or new dependency is required.

## Scenario

1. Create a workspace and save its first Goal.
2. Set and save the development role model and fallback agent/model.
3. Generate a delayed plan; verify execution stays unavailable while planning.
4. Start the run, stop it with a reason, reassign a task and model, then resume.
5. Reject a pending tool request using its exact request identity.
6. Inspect evidence, confirm its exact candidate, approve, then explicitly merge.
7. Review completion and historical results; open the Goal folder.
8. Start a second Goal with inherited team choices; delete the inactive historical Goal through its confirmation.
9. Continue a retained Phase without starting a duplicate planner.
10. Reopen from a restored backend snapshot; check workspace, Goal, fallback, history and absence of unintended mutations.

## Verification

- Initial ad hoc run: 32 continuous and six restored-state checks per cell, light/dark at 960 and 1280 pixels: 152 passed.
- Retained regression: 32 continuous and six restored-state checks in six light/dark × 840/960/1280 cells: 228 passed. The original 840-pixel coverage is preserved.
- Real Rust backend targets: cli_workflow 2/2, folder_workflow 8/8 and workflow_resume_failed 3/3 passed. These cover plan consent, approval/merge, scheduling/model selection, folder safety and retries after task/review/integration failures.
- No new product defect was confirmed in this continuous batch. Existing findings #985–#988 are not resolved by these passes.

## Limits

The UI bridge is deterministic; actual CLI/backend workflows are verified separately. Restored snapshots check rendering and side effects, not filesystem durability by themselves. No authenticated provider, live external service or physical Windows UI was exercised. Only generic fixture data was used.
