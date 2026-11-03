# Planning repair for overlapping phase scopes

When phase planning returns overlapping task scopes, AgentConductor keeps the safety validation and gives the coordinator one bounded correction attempt. The retry includes the validation error and asks for literal, disjoint repository-relative file or directory paths; category labels such as `score:` are not paths. A second invalid response still fails planning and keeps both attempts' process logs.

Regression evidence: `tests/planning_overlap_repair.rs` runs a deterministic coordinator that first returns duplicate `score:` scopes and then returns a valid plan. Related checks: phased planning, output schema, and MCP planner suites.

Trace: GitHub #52; Rust migration parity tracked by #9.


The exact-current full Rust regression passed 725 tests, 14 ignored, and 0 failed across 249 suites when Cargo debug symbols and incremental artifacts were disabled to keep the build within available disk. All-target Clippy with warnings denied and the optimized release CLI build also passed.
