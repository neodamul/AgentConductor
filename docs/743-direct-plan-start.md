# Direct planning and execution transitions (#743)

## Requirement and business goal

Reduce interruptions while preserving explicit plan approval, safe execution, and
reviewable evidence. The operator starts planning from the selected configuration
and starts execution by approving the displayed plan, without a second dialog.

## System features and use cases

- Generate or import a plan using the currently selected coordinator and model.
  Reject invalid selections and changes made while the saved configuration or
  repository baseline is being checked.
- Approve and run the displayed plan in one action. Expose its digest, repository
  base, mode, agent assignments, owned paths, fallback and test commands for review.
- Reject a stale plan, revision, or repository base. Keep risk approvals and final
  merge approval in their existing workflows.

## Functional and quality analysis

The reviewed evidence must remain accessible without obstructing the normal flow.
The execution request must bind to the digest and revision the operator reviewed;
the backend must validate them and record the digest and repository base. Planning
must retain the configuration snapshot across asynchronous save and baseline checks.
Repeated clicks must not start duplicate operations. Error states remain visible.

## Quality attribute scenarios and design drivers

- When configuration changes during planning preparation, reject the stale request
  and tell the operator to review it again (integrity and usability).
- When the plan or repository changes after review, refuse execution and preserve
  the audit trail (integrity and traceability).
- When the operator starts a valid plan, navigate directly to execution without
  another consent dialog (usability and responsiveness).

## Architecture and verification

The UI captures the planning selection before asynchronous work and sends the
reviewed plan identity to the existing Rust dispatch. Rust remains authoritative
for digest, revision and repository-base validation and audit records. Provide a
nonmodal evidence view on the plan screen. Verify direct transitions, stale inputs,
duplicate clicks, evidence visibility, and unchanged risk and merge gates in
focused local tests and the installed application.

## Workflow

- [x] Plan: record the owner's decision and trace existing validation.
- [ ] Task: add failing behavior tests and implement direct transitions.
- [ ] Verify: run local regression checks and inspect the installed application.
- [ ] Completed: update evidence, knowledge, issue and pull request.
