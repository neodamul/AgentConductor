# System features

| ID | Feature | Goal | MVP |
|---|---|---|---|
| SF-01 | Registered subprocess profiles and parsers | BG-01 | Included |
| SF-02 | Three-task plan and dependency scheduler | BG-02 | Included |
| SF-03 | Worktree lifecycle, ownership, integration candidate | BG-02/03 | Included |
| SF-04 | Logs, results, diffs, test evidence, resume | BG-03 | Included |
| SF-05 | Dangerous-action controls, human approval, merge | BG-03 | Included; full interception of arbitrary CLI tools excluded |
| SF-06 | SQLite state, audit, recovery | BG-03 | Included |
| SF-07 | agentmemory, local wiki, lessons | BG-04 | Minimal integration |
| SF-08 | CLI status, review, approval list | BG-03/05 | Included; Ratatui later |
| SF-09 | License/SBOM and TDD verification | BG-05 | Development/release procedures |

Automatic planning runs the coordinator CLI once to produce JSON, then validates it
in Rust. The coordinator reuses its login. Manual plans use the same validation path.
