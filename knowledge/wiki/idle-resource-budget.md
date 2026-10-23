# Idle resource budget

The macOS idle check uses physical footprint because shared framework pages inflate process RSS. QAS-58 amends the comparison metric: a visible idle release build should stay below 100 MB physical footprint and below 1% average CPU. This target excludes agent CLI/model and sidecar processes.

## Exact v0.1.36 Agent Room sample — 2026-10-23

The v0.1.36 macOS arm64 binary at source `f4388209` was observed for 15 seconds in a visible Agent Room with no running task. It was the only AgentConductor process and had no child process. Four five-second samples showed 71–75 MiB resident/physical memory and 0.0–0.4% CPU (0.275% average). `footprint` reported 71 MB physical footprint, below the 100 MB idle target. Its since-launch peak was 178 MB; this sample does not establish startup or active-work limits. See [measurement evidence](../../docs/evidence/issue9-f438820-idle-resources-20261023.json).

## 30-second follow-up — 2026-10-23

A longer sample of the same exact package and visible idle state satisfies the 30-second QAS-58 observation window. Seven five-second samples averaged 0.3286% CPU; current physical footprint was 73 MB. The first `top` rsize reference was 175 MiB and then settled to 73–75 MiB, reinforcing that shared-page RSS must not be used for the QAS-58 comparison. The since-launch peak was 180 MB and remains a separate startup/active investigation. See [30-second evidence](../../docs/evidence/issue9-f438820-idle-resources-30s-20261023.json).
