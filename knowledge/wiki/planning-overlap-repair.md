# Planning repair for overlapping phase scopes

When phase planning returns overlapping task scopes, AgentConductor keeps the safety validation and gives the coordinator one bounded correction attempt. The retry includes the validation error and asks for literal, disjoint repository-relative file or directory paths; category labels such as `score:` are not paths. A second invalid response still fails planning and keeps both attempts' process logs.

Regression evidence: `tests/planning_overlap_repair.rs` runs a deterministic coordinator that first returns duplicate `score:` scopes and then returns a valid plan. Related checks: phased planning, output schema and MCP planner suites.

Trace: GitHub #52.
