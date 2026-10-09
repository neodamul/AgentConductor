# QAS: Quality Attribute Scenarios

These are targets, not measured results. Tests must provide verification evidence.

| ID / Attribute | Source and stimulus | Environment / Target | Response | Measure |
|---|---|---|---|---|
| QAS-01 Concurrency | User starts three ready tasks | Local scheduler | Three worktrees within limit | Overlapping mock intervals; processes ≤3 |
| QAS-02 Safety | Unapproved merge or changed candidate | Merge executor | Reject before execution | Zero protected-ref changes; audit recorded |
| QAS-03 Recovery | Supervisor killed | Running / SQLite and processes | Identify survivors, recover stale runs | No lost artifacts/duplicate runs; recovered display within 5s target |
| QAS-04 Stopping | User stop | Child and grandchild running | Interrupt, terminate, kill, reap | Tree gone within 15s target |
| QAS-05 Parser reliability | Split UTF-8, large stderr, unknown JSON | Process I/O | Preserve raw bytes, normalize, bound memory | No lost events/deadlock; diagnose malformed output |
| QAS-06 Change isolation | Out-of-scope edits | Worktree verifier | Block review | Detect new/deleted/renamed files and symlink escapes |
| QAS-07 Availability | agentmemory unavailable | Running / outbox | SQLite persistence, later sync | Work continues; no lost reports; suppress duplicate display |
| QAS-08 Resource efficiency | Three mocks stream for 10min | Specified host / Rust core | Disk spool, bounded queues | Core RSS <100MiB, idle CPU <1% targets; exclude CLI/model/sidecar |
| QAS-09 Extensibility | Register custom CLI | Profile/parser | Common execution/review | argv+JSONL fixture without scheduler changes |
| QAS-10 Knowledge integrity | Changed source/conflicting lesson | Wiki/index | Mark stale/conflicting | No promotion without evidence; detect hash changes |
| QAS-11 Distribution | Dependency/bundle change | Release verification | Policy, notices, SBOM | Zero unreviewed licenses/assets for completion |

Traceability: UC-02→QAS-01/05/06/08/09; UC-03→QAS-03/04/07;
UC-04→QAS-02/06; UC-05→QAS-07/10; UC-01→QAS-11.
