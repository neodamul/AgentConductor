# Task verification verdicts

Source: [GitHub #1095](https://github.com/neodamul/AgentConductor/issues/1095), UC-95, DD-113.

A successful agent process records worker completion. The integration test independently verifies the combined result. The public Room snapshot sends the workflow verdict as `tests[].passed`, including when a plan expects its test to fail. In that case, a qualifying nonzero exit can pass directly. A zero exit stays unverified until the coordinator accepts evidence that the wrapper observed the expected failure.

The task row, task detail and coordinator board use that verdict for the corresponding task. Keep the recorded worker state intact so verification-stage detection and recovery can still use it.

The scheduler runs the remaining tests and verifies the candidate before routing ambiguous zero-exit tests to the existing coordinator review. The reviewer sees bounded excerpts with the end of verbose logs and must cite each ambiguous test. Acceptance binds the exact plan, candidate and log hashes and records a verification pass. The work still needs human merge approval.
