# Task verification verdicts

Source: [GitHub #1095](https://github.com/neodamul/AgentConductor/issues/1095), UC-95, DD-113.

A successful agent process records worker completion. The integration test independently verifies the combined result. The public Room snapshot sends the workflow verdict as `tests[].passed`, including when a plan expects its test to fail. In that case, an ordinary zero exit is a verification failure, and a qualifying nonzero exit is a pass.

The task row, task detail and coordinator board use that verdict for the corresponding task. Keep the recorded worker state intact so verification-stage detection and recovery can still use it.
