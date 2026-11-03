---
source: local Rust tests, root PR #22, and GitHub issue tracker
retrieved_at: 2026-11-03T12:30:00+09:00
revision: 397069af17d33a6296397ace706016907e065579
hash: git commit 397069af
---

# Source observation: open bug queue on the Rust issue #9 branch

The current open bug list has #189 (coordinator chat), #226 (deleted workspace restoration), and #310 (Windows schedule test run path). Inspection shows the Rust source fixes and regression coverage are already present on the root PR #22 branch. Local focused verification on `397069af`:

```text
cargo test --locked --offline --test launch_workspace --test workspace_delete_default --test workspace_registry --test windows_schedule_dispatch --test windows_schedule_marker --test stop_resume --test source_layout
15 passed across 7 suites
```

Issue #189 separately passed 11 tests across five suites, including the accessible-name rendering contract and Windows chat-driver script contract. None of these local tests executes a packaged Windows UI. Platform Validation run `41500210315` was still active; leave all three issues open until the exact package journeys run without skip and satisfy their persisted-state assertions.
